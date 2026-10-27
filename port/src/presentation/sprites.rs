//! Four pose uploads and sixteen sorted/animated native sprite records.
use super::{CONTACT_FRAMES, OAM_BYTES, SORTED_SPRITES, VideoMemory};
use anyhow::{Context, Result, ensure};
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpriteArrays {
    #[serde(with = "crate::snapshot::array")]
    pub order: [u8; SORTED_SPRITES],
    #[serde(with = "crate::snapshot::array")]
    pub x: [u16; SORTED_SPRITES],
    #[serde(with = "crate::snapshot::array")]
    pub y: [u8; SORTED_SPRITES],
    #[serde(with = "crate::snapshot::array")]
    pub tile_attributes: [u16; SORTED_SPRITES],
    #[serde(with = "crate::snapshot::array")]
    pub wheel_sample_heights: [u8; SORTED_SPRITES],
}
pub fn initialize_arrays(sprites: &mut SpriteArrays) {
    sprites.order = std::array::from_fn(|index| index as u8);
    sprites.x.fill(0);
    sprites.y.fill(super::HIDDEN_Y);
    sprites.wheel_sample_heights.fill(0);
}
pub fn sort_by_y(sprites: &mut SpriteArrays) {
    // Stable sort changes only IDs, preserving tie order and retained staging.
    sprites
        .order
        .sort_by_key(|&index| std::cmp::Reverse(sprites.y[usize::from(index)]));
}
pub fn pack_sorted(sprites: &SpriteArrays, oam: &mut [u8; OAM_BYTES]) -> Result<()> {
    const SORTED_BASE: usize = 0x100;
    const HIGH_BASE: usize = 0x210;
    const BYTES_PER_OBJECT: usize = 4;
    const OBJECTS_PER_HIGH_BYTE: usize = 4;
    const LARGE_FLAG: u8 = 2;
    ensure!(
        sprites
            .order
            .iter()
            .all(|&index| usize::from(index) < SORTED_SPRITES),
        "invalid sorted sprite ID"
    );
    let mut high = [0; 4];
    for (position, &index) in sprites.order.iter().enumerate() {
        let slot = usize::from(index);
        let at = SORTED_BASE + position * BYTES_PER_OBJECT;
        let [tile, attributes] = sprites.tile_attributes[slot].to_le_bytes();
        oam[at..at + BYTES_PER_OBJECT].copy_from_slice(&[
            sprites.x[slot] as u8,
            sprites.y[slot],
            tile,
            attributes,
        ]);
        let flags = ((sprites.x[slot] >> u8::BITS) as u8 & 1) | LARGE_FLAG;
        high[position / OBJECTS_PER_HIGH_BYTE] |= flags << (position % OBJECTS_PER_HIGH_BYTE * 2);
    }
    oam[HIGH_BASE..HIGH_BASE + high.len()].copy_from_slice(&high);
    Ok(())
}
pub fn hide_frontend(sprites: &mut SpriteArrays, memory: &mut VideoMemory) -> Result<()> {
    const LOW_BYTES: usize = 512;
    const LARGE_PAIRS: u8 = 0xaa;
    ensure!(
        sprites
            .order
            .iter()
            .all(|&index| usize::from(index) < SORTED_SPRITES),
        "invalid sorted sprite ID"
    );
    for record in memory.staged_oam[..LOW_BYTES].chunks_exact_mut(4) {
        record[0] = super::HIDDEN_Y;
        record[1] = super::HIDDEN_Y;
    }
    sprites.y.fill(super::HIDDEN_Y);
    memory.staged_oam[LOW_BYTES..].fill(LARGE_PAIRS);
    pack_sorted(sprites, &mut memory.staged_oam)
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimationTables {
    pub tiles: [u8; CONTACT_FRAMES],
    pub attributes: [u8; CONTACT_FRAMES],
}
pub fn load_animations(rom: &[u8]) -> Result<AnimationTables> {
    const TILE_ROM: usize = 0x804;
    const ATTRIBUTES_ROM: usize = 0x823;
    Ok(AnimationTables {
        tiles: rom
            .get(TILE_ROM..TILE_ROM + CONTACT_FRAMES)
            .context("truncated animation tiles")?
            .try_into()?,
        attributes: rom
            .get(ATTRIBUTES_ROM..ATTRIBUTES_ROM + CONTACT_FRAMES)
            .context("truncated animation attributes")?
            .try_into()?,
    })
}
pub fn clear_cursors(cursors: &mut [u8; SORTED_SPRITES]) {
    cursors.fill(0);
}
pub fn animate(
    memory: &mut VideoMemory,
    cursors: &mut [u8; SORTED_SPRITES],
    frame: u8,
    tables: &AnimationTables,
) -> Result<()> {
    const SKIP: u8 = 0xfe;
    const TERMINATE: u8 = 0xff;
    const ANIMATION_BASE: usize = 0x150;
    ensure!(
        cursors
            .iter()
            .all(|&cursor| usize::from(cursor) < CONTACT_FRAMES),
        "invalid animation cursor"
    );
    for slot in (0..SORTED_SPRITES).rev() {
        let cursor = usize::from(cursors[slot]);
        let attributes = tables.attributes[cursor];
        let tile = tables.tiles[cursor];
        let at = ANIMATION_BASE + slot * 4;
        match tile {
            SKIP => continue,
            TERMINATE => {
                memory.staged_oam[at + 1] = super::HIDDEN_Y;
                cursors[slot] = 0;
            }
            _ => {
                memory.staged_oam[at + 2] = tile;
                memory.staged_oam[at + 3] = attributes;
                if frame & 3 == 0 {
                    cursors[slot] = cursors[slot].wrapping_add(1);
                }
            }
        }
    }
    Ok(())
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PoseStrip {
    pub source_cpu: u32,
    pub destination_byte: u16,
    pub bytes: usize,
}
pub fn pose_strips(selectors: [u16; 4]) -> [PoseStrip; 8] {
    const SOURCE_HIGH_BIT: u16 = 0x8000;
    const SECOND_SOURCE_STEP: u16 = 0x200;
    const FIRST_DESTINATION: u16 = 0xc000;
    const SECOND_DESTINATION_STEP: u16 = 0x200;
    const DESTINATION_SLOT_STEP: u16 = 0x40;
    const STRIP_BYTES: usize = 0x40;
    std::array::from_fn(|index| {
        let slot = index / 2;
        let second = index % 2 != 0;
        let selector = selectors[slot];
        let bank = if selector & SOURCE_HIGH_BIT == 0 {
            1u32
        } else {
            2
        };
        PoseStrip {
            source_cpu: (bank << 16)
                | u32::from(
                    selector.wrapping_add(if second { SECOND_SOURCE_STEP } else { 0 })
                        | SOURCE_HIGH_BIT,
                ),
            destination_byte: FIRST_DESTINATION
                + slot as u16 * DESTINATION_SLOT_STEP
                + if second { SECOND_DESTINATION_STEP } else { 0 },
            bytes: STRIP_BYTES,
        }
    })
}
pub fn upload_poses(memory: &mut VideoMemory, rom: &[u8], selectors: [u16; 4]) -> Result<()> {
    let strips = pose_strips(selectors);
    let spans = strips
        .iter()
        .map(|strip| {
            let physical =
                crate::world::tracks::physical(crate::world::tracks::CpuSource(strip.source_cpu))?;
            rom.get(physical..physical + strip.bytes)
                .context("truncated truck pose strip")
        })
        .collect::<Result<Vec<_>>>()?;
    for (strip, bytes) in strips.iter().zip(spans) {
        let at = usize::from(strip.destination_byte);
        memory.vram[at..at + strip.bytes].copy_from_slice(bytes);
    }
    Ok(())
}
pub fn present_boundary(
    memory: &mut VideoMemory,
    rom: &[u8],
    selectors: [u16; 4],
    inhibit: u8,
) -> Result<()> {
    if inhibit != 0 {
        return Ok(());
    }
    upload_poses(memory, rom, selectors)?;
    memory.visible_oam = memory.staged_oam;
    Ok(())
}
/// Domain projection view shared by motion (before pair separation) and sprite
/// staging. Track8 priority deliberately observes committed pre-contact worldY.
pub fn truck_projection(
    track: u8,
    world_y: u16,
    surface: u8,
    projected: [u16; 2],
    mut attributes: u16,
) -> (u16, u8, u16) {
    const SPECIAL_TRACK: u8 = 8;
    const BAND_START: u16 = 0x1d80;
    const BAND_LENGTH: u16 = 0x800;
    const ELEVATED_HEIGHT: u8 = 0x2e;
    const ELEVATED_PRIORITY: u16 = 0x3000;
    const LOWER_PRIORITY_BIT: u16 = 0x1000;
    const X_OFFSET: u16 = 0x40;
    const Y_OFFSET: u8 = 0x28;
    if track & 0x0f == SPECIAL_TRACK {
        if world_y.wrapping_sub(BAND_START) < BAND_LENGTH && surface >= ELEVATED_HEIGHT {
            attributes |= ELEVATED_PRIORITY;
        } else {
            attributes &= !LOWER_PRIORITY_BIT;
        }
    }
    (
        projected[0].wrapping_sub(X_OFFSET),
        (projected[1] as u8).wrapping_add(Y_OFFSET),
        attributes,
    )
}
pub fn project_truck(
    sprites: &mut SpriteArrays,
    slot: usize,
    track: u8,
    world_y: u16,
    surface: u8,
    projected: [u16; 2],
) -> Result<()> {
    anyhow::ensure!(slot < 4, "invalid truck sprite slot");
    let (x, y, attributes) = truck_projection(
        track,
        world_y,
        surface,
        projected,
        sprites.tile_attributes[slot],
    );
    sprites.x[slot] = x;
    sprites.y[slot] = y;
    sprites.tile_attributes[slot] = attributes;
    Ok(())
}

impl Default for SpriteArrays {
    fn default() -> Self {
        Self {
            order: std::array::from_fn(|i| i as u8),
            x: [0; SORTED_SPRITES],
            y: [super::HIDDEN_Y; SORTED_SPRITES],
            tile_attributes: [0; SORTED_SPRITES],
            wheel_sample_heights: [0; SORTED_SPRITES],
        }
    }
}
