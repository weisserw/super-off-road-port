//! Native truck body frames and immutable images. Source pose words, banks and
//! strip layout are decoded here, at the asset/save boundary. Gameplay selects
//! a frame by body tilt and heading; presentation consumes its already loaded
//! image at the established NMI, without constructing upload commands.
use super::VideoMemory;
use crate::trucks::motion::{HEADING_DIRECTIONS, TILT_POSES, TILT_SELECTORS};
use anyhow::{Context, Result};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{collections::BTreeMap, sync::Arc};

pub const TILE_ROW_BYTES: usize = 64;
const POSE_FRAMES: usize = TILT_POSES * HEADING_DIRECTIONS;
const TILT_ROM: usize = 0xca2;
const SOURCE_WINDOW_BYTES: usize = 0x8000;
const FIRST_GRAPHICS: usize = 0x8000;
const SECOND_GRAPHICS: usize = 0x10000;
const SECOND_ROW_OFFSET: u16 = 0x200;
const SOURCE_OFFSET_MASK: u16 = 0x7fff;
const SOURCE_BANK_BIT: u16 = 0x8000;

/// Frame identity in the native five-by-five body-tilt/32-heading catalogue.
/// Source selector is immutable serialization provenance, not its lookup key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BodyFrame {
    key: usize,
    encoded: u16,
    working_heading: u8,
}
impl BodyFrame {
    pub fn encoded(self) -> u16 {
        self.encoded
    }
    pub fn working_heading(self) -> u8 {
        self.working_heading
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PoseRules {
    frames: Box<[BodyFrame; POSE_FRAMES]>,
}
impl PoseRules {
    pub fn load(rom: &[u8]) -> Result<Self> {
        let bytes = rom
            .get(TILT_ROM..TILT_ROM + TILT_POSES * 2)
            .context("truncated body pose table")?;
        Ok(Self::decode(std::array::from_fn(|i| {
            u16::from_le_bytes([bytes[i * 2], bytes[i * 2 + 1]])
        })))
    }
    /// Decode the original table's nonzero-tilt heading resolution and packed
    /// image reference once. Overridden table words retain their exact mapping.
    pub fn decode(tilts: [u16; TILT_POSES]) -> Self {
        Self {
            frames: Box::new(std::array::from_fn(|key| {
                let tilt = tilts[key / HEADING_DIRECTIONS];
                let heading = (key % HEADING_DIRECTIONS) as u8;
                let working_heading = if tilt == 0 { heading } else { heading >> 1 };
                BodyFrame {
                    key,
                    encoded: encode_source_pose(tilt | u16::from(working_heading)),
                    working_heading,
                }
            })),
        }
    }
    pub fn frame(&self, lateral: u8, longitudinal: u8, heading: u8) -> BodyFrame {
        let tilt = usize::from(longitudinal) * TILT_SELECTORS + usize::from(lateral);
        self.frames[tilt * HEADING_DIRECTIONS + usize::from(heading)]
    }
}

/// Original source-asset encoding. Not called during body selection or NMI.
pub fn encode_source_pose(pose: u16) -> u16 {
    const LOW_MASK: u16 = 7;
    const LOW_SHIFT: u32 = 6;
    ((pose & LOW_MASK) << LOW_SHIFT) | ((pose & !LOW_MASK) >> 1).swap_bytes()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PoseImage {
    encoded: u16,
    tile_rows: [[u8; TILE_ROW_BYTES]; 2],
}

/// Normal state is a native catalogue frame. A legacy save starts unresolved,
/// then becomes an immutable resource-bound image before live adoption. Only
/// its original word is serialized, so version2 saves/debug values stay stable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PoseSelection(Selection);
#[derive(Clone, Debug, PartialEq, Eq)]
enum Selection {
    Frame(BodyFrame),
    Legacy(u16),
    Restored(Arc<PoseImage>),
}
impl Default for PoseSelection {
    fn default() -> Self {
        Self::from_encoded(0)
    }
}
impl PoseSelection {
    pub fn frame(frame: BodyFrame) -> Self {
        Self(Selection::Frame(frame))
    }
    pub fn from_encoded(encoded: u16) -> Self {
        Self(Selection::Legacy(encoded))
    }
    pub fn encoded(&self) -> u16 {
        match &self.0 {
            Selection::Frame(frame) => frame.encoded,
            Selection::Legacy(encoded) => *encoded,
            Selection::Restored(image) => image.encoded,
        }
    }
}
impl Serialize for PoseSelection {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        self.encoded().serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for PoseSelection {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        Ok(Self::from_encoded(u16::deserialize(deserializer)?))
    }
}

/// Shared immutable body images; the 800 orientation/heading entries alias416
/// original frames. Two checked graphics views serve arbitrary old save words
/// without materializing65536 full images in every loaded game.
pub struct PoseAtlas {
    frames: Vec<Arc<PoseImage>>,
    source_images: BTreeMap<u16, Arc<PoseImage>>,
    legacy_graphics: [Vec<u8>; 2],
}
impl PoseAtlas {
    pub fn load(rom: &[u8], rules: &PoseRules) -> Result<Self> {
        // Legacy unaligned spans can cross the end of a source window. Preserve
        // the faithful native contiguous-ROM policy for those accepted words.
        let window = |base| -> Result<Vec<u8>> {
            Ok(rom
                .get(base..base + SOURCE_WINDOW_BYTES + TILE_ROW_BYTES - 1)
                .context("truncated truck pose graphics")?
                .to_vec())
        };
        let mut atlas = Self {
            frames: Vec::with_capacity(POSE_FRAMES),
            source_images: BTreeMap::new(),
            legacy_graphics: [window(FIRST_GRAPHICS)?, window(SECOND_GRAPHICS)?],
        };
        for frame in rules.frames.iter() {
            let image = if let Some(image) = atlas.source_images.get(&frame.encoded) {
                image.clone()
            } else {
                let image = Arc::new(atlas.decode_legacy(frame.encoded)?);
                atlas.source_images.insert(frame.encoded, image.clone());
                image
            };
            atlas.frames.push(image);
        }
        Ok(atlas)
    }
    /// Asset/legacy-save boundary only: bank is chosen before adding the second
    /// row offset, and that addition wraps without changing the selected bank.
    fn decode_legacy(&self, encoded: u16) -> Result<PoseImage> {
        let bank = usize::from(encoded & SOURCE_BANK_BIT != 0);
        let row = |offset: u16| -> Result<[u8; TILE_ROW_BYTES]> {
            let at = usize::from(offset & SOURCE_OFFSET_MASK);
            Ok(self.legacy_graphics[bank]
                .get(at..at + TILE_ROW_BYTES)
                .context("truncated truck pose row")?
                .try_into()?)
        };
        Ok(PoseImage {
            encoded,
            tile_rows: [row(encoded)?, row(encoded.wrapping_add(SECOND_ROW_OFFSET))?],
        })
    }
    /// Resolve a complete candidate atomically before restoring/cold-starting
    /// live state. Normal simulation only stages Frame selections after this.
    pub fn resolve(&self, poses: &mut [PoseSelection; 4]) -> Result<()> {
        let mut resolved = poses.clone();
        for pose in &mut resolved {
            if let Selection::Legacy(encoded) = pose.0 {
                let image = match self.source_images.get(&encoded) {
                    Some(image) => image.clone(),
                    None => Arc::new(self.decode_legacy(encoded)?),
                };
                pose.0 = Selection::Restored(image);
            }
        }
        *poses = resolved;
        Ok(())
    }
    fn image<'a>(&'a self, pose: &'a PoseSelection) -> Result<&'a PoseImage> {
        match &pose.0 {
            Selection::Frame(frame) => self
                .frames
                .get(frame.key)
                .map(Arc::as_ref)
                .context("invalid native body frame"),
            Selection::Restored(image) => Ok(image),
            Selection::Legacy(_) => anyhow::bail!("unresolved legacy body pose"),
        }
    }
    /// Native renderer boundary: install two tile rows per truck in the existing
    /// backend layout, retaining all holes and validating all inputs first.
    /// NMI inhibition/timing remains owned by the shared frame scheduler.
    pub fn present(&self, memory: &mut VideoMemory, poses: &[PoseSelection; 4]) -> Result<()> {
        const BODY_BASE: usize = 0xc000;
        const LOWER_ROW_OFFSET: usize = 0x200;
        let images = poses
            .iter()
            .map(|pose| self.image(pose))
            .collect::<Result<Vec<_>>>()?;
        for (slot, image) in images.iter().enumerate() {
            let upper = BODY_BASE + slot * TILE_ROW_BYTES;
            let lower = upper + LOWER_ROW_OFFSET;
            memory.vram[upper..upper + TILE_ROW_BYTES].copy_from_slice(&image.tile_rows[0]);
            memory.vram[lower..lower + TILE_ROW_BYTES].copy_from_slice(&image.tile_rows[1]);
        }
        Ok(())
    }
}
