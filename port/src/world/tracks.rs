//! Track catalogue, packed route loading and source start-grid contracts.
//! Source derivations, units and cross-owner effects: tracks.trace.md.
use anyhow::{Context, Result, ensure};
use romatlas_port::assets::MemoryAssets;
use romatlas_port::assets::{Asset, Origin};

pub const ORIGINAL_TRACK_COUNT: usize = 16;
pub const ORDER_SLOTS: usize = 64;
pub const TRUCKS: usize = 4;
pub const WHEELS: usize = 16;
pub const TERRAIN_BYTES: usize = 0x16b1;
pub const BACKGROUND_BYTES: usize = 0x600;
pub const GRAPHICS_BYTES: usize = 0x8000;

/// CPU cartridge pointer. Conversion belongs to loading, never game rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CpuSource(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Direction {
    Forward,
    Reversed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Selection {
    pub original_track: u8,
    pub direction: Direction,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Descriptor {
    pub projection_offset: [u16; 2],
    pub projection_scale: [u8; 2],
    pub terrain: CpuSource,
    pub backgrounds: [CpuSource; 2],
    pub graphics: CpuSource,
}
/// Route words retain source packed rectangle/control encoding; race/AI decode
/// their own consumers. The terminal word is retained, not a coordinate pair.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Route {
    pub words: Vec<u16>,
    pub checkpoint_count: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TrackContent {
    pub projection_offset: [u16; 2],
    pub projection_scale: [u8; 2],
    pub terrain: Vec<u8>,
    pub backgrounds: [Vec<u8>; 2],
    pub graphics: Vec<u8>,
    pub routes: [Route; 2],
    pub headings: [u8; 2],
    /// Added/replaced named tracks carry validated object definitions. Original
    /// None resolves the ROM full-order group through objects::load_original_content.
    pub objects: Option<super::objects::ObjectContent>,
    /// Original None resolves source title/HUD/overlay layout; named extensions
    /// require explicit validated presentation, without an original track index.
    pub presentation: Option<crate::presentation::TrackPresentation>,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NamedTrack {
    pub id: String,
    pub content: TrackContent,
}
pub struct Catalogue {
    pub tracks: Vec<NamedTrack>,
    pub order: Vec<String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grid {
    pub x: [u16; TRUCKS],
    pub y: [u16; TRUCKS],
    pub heading: u8,
    /// Position queried before any column/row reassignment.
    pub initial_surface_origin: [u16; 2],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WheelPreparation {
    pub sampled: [u8; WHEELS],
    pub integrated: [u8; WHEELS],
    pub velocity: [u8; WHEELS],
    pub truck_surface: [u8; TRUCKS],
}

/// Named original table locations, all physical ROM offsets.
pub const ORDER_ROM: usize = 0x216;
pub const PARAMETERS_ROM: usize = 0x123a;
pub const PARAMETERS_BYTES: usize = 0xf0;
pub const GRAPHICS_POINTERS_ROM: usize = 0x13ef;
pub const ROUTE_POINTERS_ROM: usize = 0x57fa;
pub const HEADINGS_ROM: usize = 0x68b;
const CPU_WINDOW: usize = 0x8000;
const REVERSED_BIT: u8 = 0x80;
const ORIGINAL_ID_MASK: u8 = 0x7f;
const ROUTE_TERMINAL: u16 = 0xff00;
const ROUTE_TERMINAL_MASK: u16 = 0xff00;
const GRID_X_MASK: u16 = 0xbfe0;
const GRID_Y_MASK: u16 = 0xffe0;
const REVERSE_GRID_STEP: u16 = 0xfc00;
const GRID_COLUMN_STEP: u16 = 0x300;
const GRID_TOP_STEP: u16 = 0x100;
const GRID_ROW_STEP: u16 = 0x1e0;

fn word(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}
/// Checked LoROM bus conversion. Work RAM banks are never cartridge resources.
pub fn physical(source: CpuSource) -> Result<usize> {
    let bank = (source.0 >> 16) as u8;
    let address = (source.0 & u32::from(u16::MAX)) as usize;
    ensure!(
        source.0 <= 0xffffff && !matches!(bank, 0x7e | 0x7f) && address >= CPU_WINDOW,
        "unmapped cartridge source"
    );
    Ok(usize::from(bank & 0x7f) * CPU_WINDOW + address - CPU_WINDOW)
}
fn window(rom: &[u8], source: CpuSource, length: usize) -> Result<Vec<u8>> {
    let at = physical(source)?;
    Ok(rom
        .get(at..at.checked_add(length).context("asset range overflow")?)
        .context("truncated cartridge resource")?
        .to_vec())
}
pub fn decode_order(bytes: &[u8]) -> Result<Vec<Selection>> {
    ensure!(
        bytes.len() == ORDER_SLOTS,
        "original order must contain 64 entries"
    );
    bytes
        .iter()
        .map(|&encoded| {
            let original_track = encoded & ORIGINAL_ID_MASK;
            ensure!(
                usize::from(original_track) < ORIGINAL_TRACK_COUNT,
                "invalid original track"
            );
            Ok(Selection {
                original_track,
                direction: if encoded & REVERSED_BIT == 0 {
                    Direction::Forward
                } else {
                    Direction::Reversed
                },
            })
        })
        .collect()
}
pub fn select_order(order: &[Selection], slot: u8) -> Result<Selection> {
    order
        .get(usize::from(slot))
        .copied()
        .context("order slot out of bounds")
}
pub fn decode_descriptors(parameters: &[u8], graphics: &[u8]) -> Result<Vec<Descriptor>> {
    const Y_OFFSET: usize = 0x20;
    const Y_SCALE: usize = 0x40;
    const X_SCALE: usize = 0x50;
    const BG1_WORD: usize = 0x60;
    const BG1_BANK: usize = 0x80;
    const BG2_WORD: usize = 0x90;
    const BG2_BANK: usize = 0xb0;
    const TERRAIN_BANK: usize = 0xc0;
    const TERRAIN_WORD: usize = 0xd0;
    ensure!(
        parameters.len() == PARAMETERS_BYTES && graphics.len() == ORIGINAL_TRACK_COUNT * 3,
        "invalid parallel track arrays"
    );
    Ok((0..ORIGINAL_TRACK_COUNT)
        .map(|track| {
            let pointer = |low, bank| {
                CpuSource(
                    u32::from(word(parameters, low + track * 2))
                        | (u32::from(parameters[bank + track]) << 16),
                )
            };
            let at = track * 3;
            Descriptor {
                projection_offset: [
                    word(parameters, track * 2),
                    word(parameters, Y_OFFSET + track * 2),
                ],
                projection_scale: [parameters[X_SCALE + track], parameters[Y_SCALE + track]],
                terrain: pointer(TERRAIN_WORD, TERRAIN_BANK),
                backgrounds: [pointer(BG1_WORD, BG1_BANK), pointer(BG2_WORD, BG2_BANK)],
                graphics: CpuSource(
                    u32::from(word(graphics, at)) | (u32::from(graphics[at + 2]) << 16),
                ),
            }
        })
        .collect())
}
pub fn load_route(rom: &[u8], track: u8, direction: Direction) -> Result<Route> {
    ensure!(
        usize::from(track) < ORIGINAL_TRACK_COUNT,
        "invalid original route track"
    );
    let view = usize::from(track)
        + if direction == Direction::Reversed {
            ORIGINAL_TRACK_COUNT
        } else {
            0
        };
    let at = ROUTE_POINTERS_ROM + view * 2;
    let pointer = rom
        .get(at..at + 2)
        .context("truncated route pointer table")?;
    let start = physical(CpuSource(u32::from(word(pointer, 0))))?;
    decode_route(rom.get(start..).context("route pointer beyond ROM")?)
}
pub fn decode_route(bytes: &[u8]) -> Result<Route> {
    let mut words = Vec::new();
    for pair in bytes.chunks_exact(2) {
        let value = word(pair, 0);
        words.push(value);
        if value & ROUTE_TERMINAL_MASK == ROUTE_TERMINAL {
            return Ok(Route {
                words,
                checkpoint_count: value as u8,
            });
        }
    }
    anyhow::bail!("route has no complete terminal word")
}
pub fn decode_headings(bytes: &[u8]) -> Result<Vec<[u8; 2]>> {
    ensure!(
        bytes.len() == ORIGINAL_TRACK_COUNT * 4,
        "invalid heading array"
    );
    Ok((0..ORIGINAL_TRACK_COUNT)
        .map(|track| {
            [
                bytes[track * 2],
                bytes[ORIGINAL_TRACK_COUNT * 2 + track * 2],
            ]
        })
        .collect())
}
pub fn prepare_grid(route_start: [u16; 2], heading: u8, order_slot: u8) -> Grid {
    let displacement = if heading == 0 {
        REVERSE_GRID_STEP
    } else {
        u16::from(heading)
    };
    let base_x = (route_start[0] & GRID_X_MASK).wrapping_add(displacement);
    let base_y = route_start[1] & GRID_Y_MASK;
    let next_x = base_x.wrapping_add(GRID_COLUMN_STEP);
    let top = base_y.wrapping_sub(GRID_TOP_STEP);
    let bottom = top.wrapping_add(GRID_ROW_STEP);
    let mut x = [base_x, base_x, next_x, next_x];
    if displacement == REVERSE_GRID_STEP {
        x.rotate_left(2);
    }
    let mut y = [top, bottom, top, bottom];
    if order_slot & 1 != 0 {
        y.swap(0, 1);
        y.swap(2, 3);
    }
    Grid {
        x,
        y,
        heading,
        initial_surface_origin: [base_x, base_y],
    }
}
pub fn clear_wheels(state: &mut WheelPreparation) {
    state.sampled.fill(0);
    state.integrated.fill(0);
    state.velocity.fill(0);
}
pub fn apply_initial_surface(state: &mut WheelPreparation, height: u8) {
    state.integrated.fill(height);
    state.truck_surface.fill(height);
}
pub fn load_content(rom: &[u8], descriptor: &Descriptor, track: u8) -> Result<TrackContent> {
    let headings = decode_headings(
        rom.get(HEADINGS_ROM..HEADINGS_ROM + ORIGINAL_TRACK_COUNT * 4)
            .context("truncated heading table")?,
    )?;
    let heading = *headings
        .get(usize::from(track))
        .context("invalid original track")?;
    Ok(TrackContent {
        projection_offset: descriptor.projection_offset,
        projection_scale: descriptor.projection_scale,
        terrain: window(rom, descriptor.terrain, TERRAIN_BYTES)?,
        backgrounds: [
            window(rom, descriptor.backgrounds[0], BACKGROUND_BYTES)?,
            window(rom, descriptor.backgrounds[1], BACKGROUND_BYTES)?,
        ],
        graphics: window(rom, descriptor.graphics, GRAPHICS_BYTES)?,
        routes: [
            load_route(rom, track, Direction::Forward)?,
            load_route(rom, track, Direction::Reversed)?,
        ],
        headings: heading,
        objects: None,
        presentation: None,
    })
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct TrackFile {
    version: u32,
    projection_offset: [u16; 2],
    projection_scale: [u8; 2],
    terrain: Vec<u8>,
    backgrounds: [Vec<u8>; 2],
    graphics: Vec<u8>,
    routes: [Vec<u16>; 2],
    headings: [u8; 2],
    objects: Option<super::objects::ObjectContent>,
    presentation: Option<crate::presentation::TrackPresentation>,
}
impl TrackFile {
    fn from_content(content: TrackContent) -> Self {
        Self {
            version: 1,
            projection_offset: content.projection_offset,
            projection_scale: content.projection_scale,
            terrain: content.terrain,
            backgrounds: content.backgrounds,
            graphics: content.graphics,
            routes: content.routes.map(|route| route.words),
            headings: content.headings,
            objects: content.objects,
            presentation: content.presentation,
        }
    }
    fn decode(self, file_backed: bool) -> Result<TrackContent> {
        ensure!(self.version == 1, "unsupported track version");
        ensure!(
            self.terrain.len() == TERRAIN_BYTES
                && self
                    .backgrounds
                    .iter()
                    .all(|bg| bg.len() == BACKGROUND_BYTES)
                && self.graphics.len() == GRAPHICS_BYTES,
            "invalid track resource length"
        );
        let [first, second] = self.routes;
        let route = |words: Vec<u16>| -> Result<Route> {
            let bytes: Vec<u8> = words.iter().flat_map(|word| word.to_le_bytes()).collect();
            let decoded = decode_route(&bytes)?;
            ensure!(
                decoded.words.len() == words.len()
                    && decoded.checkpoint_count != 0
                    && words.len() >= 3,
                "invalid track route terminal/count"
            );
            Ok(decoded)
        };
        if file_backed {
            super::objects::validate_content(
                self.objects
                    .as_ref()
                    .context("file track requires objects")?,
            )?;
            crate::presentation::validate_track_presentation(
                self.presentation
                    .as_ref()
                    .context("file track requires presentation")?,
            )?;
        }
        let routes = [route(first)?, route(second)?];
        if file_backed {
            for route in &routes {
                validate_playable_route(route)?;
            }
        }
        Ok(TrackContent {
            projection_offset: self.projection_offset,
            projection_scale: self.projection_scale,
            terrain: self.terrain,
            backgrounds: self.backgrounds,
            graphics: self.graphics,
            routes,
            headings: self.headings,
            objects: self.objects,
            presentation: self.presentation,
        })
    }
}
/// File tracks must include the last AI checkpoint, not just a terminal word
/// carrying its index. Original variable-length source views remain decoded as-is.
pub fn validate_playable_route(route: &Route) -> Result<()> {
    let records = route.words.len().saturating_sub(1) / 2;
    ensure!(
        route.words.len() % 2 == 1
            && usize::from(route.checkpoint_count) < records
            && route.checkpoint_count != 0,
        "file route lacks full last checkpoint record"
    );
    for record in route.words[..route.words.len() - 1].chunks_exact(2) {
        if matches!((record[0] >> 8) as u8, 0xfd | 0xfe) {
            ensure!(
                usize::from(record[1] as u8) < records,
                "file route redirect target missing"
            );
        }
    }
    ensure!(route.words[..route.words.len()-1].chunks_exact(2).any(|record| !matches!((record[0]>>8) as u8,0xfd|0xfe) && record[0]&0x4000==0),"file route contains no human checkpoint");
    Ok(())
}
/// Both frontends merge replacements and additions through the same collection
/// path. Validation is complete before any selected content becomes live.
pub fn load_catalogue(original: Vec<NamedTrack>, overrides: &MemoryAssets) -> Result<Catalogue> {
    let defaults = original
        .into_iter()
        .map(|track| {
            Ok(Asset {
                id: track.id,
                bytes: serde_json::to_vec(&TrackFile::from_content(track.content))?,
                origin: Origin::DerivedFromRom {
                    recipe: "decoded-original-track".into(),
                },
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let collection = overrides.collection("tracks.json", defaults, |asset| {
        let file: TrackFile = serde_json::from_slice(&asset.bytes)?;
        file.decode(matches!(asset.origin, Origin::File(_)))?;
        Ok(())
    })?;
    let order = collection
        .assets
        .iter()
        .map(|asset| asset.id.clone())
        .collect();
    let tracks = collection
        .assets
        .into_iter()
        .map(|asset| {
            let file: TrackFile = serde_json::from_slice(&asset.bytes)?;
            Ok(NamedTrack {
                id: asset.id,
                content: file.decode(matches!(asset.origin, Origin::File(_)))?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Catalogue { tracks, order })
}

#[cfg(test)]
mod tests {
    use super::*;
    use romatlas_port::assets::{FileAsset, Manifest};
    // Physical table boundaries are loader definitions, not executable game addresses.
    const ORDER: std::ops::Range<usize> = 0x216..0x256;
    const PARAMETERS: std::ops::Range<usize> = 0x123a..0x132a;
    const GRAPHICS_POINTERS: std::ops::Range<usize> = 0x13ef..0x141f;
    const HEADINGS: std::ops::Range<usize> = 0x68b..0x6cb;
    fn rom() -> Vec<u8> {
        std::fs::read("../input/game.rom").unwrap()
    }

    #[test]
    fn all_order_slots_decode_low_track_bits_and_independent_reverse_flag() {
        let b = rom();
        let order = decode_order(&b[ORDER]).unwrap();
        assert_eq!(order.len(), ORDER_SLOTS);
        // Independently inspected source table metadata, not implementation output.
        let expected = [
            4, 6, 2, 1, 138, 134, 130, 128, 5, 1, 3, 0, 132, 7, 5, 139, 135, 10, 3, 140, 8, 2, 9,
            11, 0, 14, 12, 134, 13, 135, 129, 136, 9, 15, 131, 14, 4, 10, 11, 15, 133, 128, 8, 13,
            7, 12, 142, 6, 8, 130, 143, 139, 132, 129, 141, 131, 136, 138, 133, 0, 142, 141, 140,
            143,
        ];
        for (actual, encoded) in order.iter().zip(expected) {
            assert_eq!(
                *actual,
                Selection {
                    original_track: encoded & 0x7f,
                    direction: if encoded & 0x80 == 0 {
                        Direction::Forward
                    } else {
                        Direction::Reversed
                    }
                }
            );
        }
        assert_eq!(
            select_order(&order, 63).unwrap(),
            Selection {
                original_track: 15,
                direction: Direction::Reversed
            }
        );
        // Selection itself does not mask its index. Race owns (slot+1)&63.
        assert!(select_order(&order, 64).is_err());
    }
    #[test]
    fn order_validation_rejects_wrong_size_or_non_original_descriptor() {
        assert!(decode_order(&[0; ORDER_SLOTS - 1]).is_err());
        let mut invalid = [0; ORDER_SLOTS];
        invalid[0] = 16;
        assert!(decode_order(&invalid).is_err());
        invalid[0] = 0x90;
        assert!(decode_order(&invalid).is_err());
    }
    #[test]
    fn all_parallel_descriptors_preserve_signed_offsets_and_byte_word_bank_strides() {
        let b = rom();
        let d = decode_descriptors(&b[PARAMETERS], &b[GRAPHICS_POINTERS]).unwrap();
        let expected = [
            ([0, 0], [103, 67], 0x048000, [0x068600, 0x068000], 0x038000),
            ([0, 0], [103, 68], 0x0496b0, [0x069200, 0x068c00], 0x038000),
            (
                [0xffff, 0],
                [103, 67],
                0x04ad60,
                [0x069e00, 0x069800],
                0x038000,
            ),
            (
                [0xfffe, 0x150],
                [103, 71],
                0x04c410,
                [0x06aa00, 0x06a400],
                0x038000,
            ),
            (
                [0xfffe, 0x20],
                [103, 69],
                0x058000,
                [0x06b600, 0x06b000],
                0x038000,
            ),
            (
                [0, 0x20],
                [103, 67],
                0x0596b0,
                [0x06c200, 0x06bc00],
                0x038000,
            ),
            (
                [0, 0x10],
                [101, 70],
                0x05ad60,
                [0x06ce00, 0x06c800],
                0x038000,
            ),
            (
                [0, 0x18],
                [103, 68],
                0x05c410,
                [0x0de622, 0x0de022],
                0x0c8000,
            ),
            (
                [0xfffe, 0x100],
                [103, 69],
                0x0a8000,
                [0x0d8600, 0x0d8000],
                0x0c8000,
            ),
            ([3, 0], [104, 70], 0x0a96aa, [0x0d9212, 0x0d8c12], 0x0c8000),
            (
                [0, 0x60],
                [103, 68],
                0x0aad5a,
                [0x0d9e12, 0x0d9812],
                0x0c8000,
            ),
            (
                [0, 0x70],
                [103, 68],
                0x0ac40a,
                [0x0daa12, 0x0da412],
                0x0c8000,
            ),
            (
                [0, 0x65],
                [103, 68],
                0x0b8000,
                [0x0db612, 0x0db012],
                0x0c8000,
            ),
            ([0, 0], [103, 70], 0x0b96b0, [0x0dc212, 0x0dbc12], 0x0c8000),
            (
                [3, 0x70],
                [103, 68],
                0x0bad60,
                [0x0dce12, 0x0dc812],
                0x0c8000,
            ),
            (
                [0, 0x70],
                [103, 68],
                0x0bc410,
                [0x0dda22, 0x0dd422],
                0x0c8000,
            ),
        ];
        assert_eq!(d.len(), ORIGINAL_TRACK_COUNT);
        for (a, (offset, scale, terrain, bg, graphics)) in d.iter().zip(expected) {
            assert_eq!(
                *a,
                Descriptor {
                    projection_offset: offset,
                    projection_scale: scale,
                    terrain: CpuSource(terrain),
                    backgrounds: bg.map(CpuSource),
                    graphics: CpuSource(graphics)
                }
            );
        }
    }
    #[test]
    fn descriptor_decoder_rejects_partial_arrays_before_consuming_them() {
        let b = rom();
        assert!(
            decode_descriptors(
                &b[PARAMETERS.start..PARAMETERS.end - 1],
                &b[GRAPHICS_POINTERS]
            )
            .is_err()
        );
        assert!(
            decode_descriptors(
                &b[PARAMETERS],
                &b[GRAPHICS_POINTERS.start..GRAPHICS_POINTERS.end - 1]
            )
            .is_err()
        );
    }
    #[test]
    fn all_thirty_two_route_views_keep_terminal_and_checkpoint_count() {
        let b = rom();
        let word_counts = [
            35, 29, 29, 29, 41, 35, 31, 25, 61, 15, 25, 27, 21, 47, 25, 19, 35, 29, 29, 29, 41, 35,
            31, 25, 63, 15, 25, 27, 21, 55, 21, 19,
        ];
        let checkpoints = [
            16, 13, 13, 13, 19, 16, 14, 11, 15, 6, 11, 12, 9, 13, 11, 8, 16, 13, 13, 13, 19, 16,
            14, 11, 15, 6, 11, 12, 9, 13, 9, 8,
        ];
        for view in 0..32 {
            let direction = if view < 16 {
                Direction::Forward
            } else {
                Direction::Reversed
            };
            let r = load_route(&b, (view % 16) as u8, direction).unwrap();
            assert_eq!(r.words.len(), word_counts[view], "route view {view}");
            assert_eq!(r.checkpoint_count, checkpoints[view]);
            assert_eq!(
                *r.words.last().unwrap(),
                0xff00 | u16::from(checkpoints[view])
            );
        }
    }
    #[test]
    fn route_high_ff_termination_is_not_low_ff_or_a_fixed_pair_count() {
        let route = decode_route(&[0xff, 0x12, 0x00, 0x80, 0x03, 0xff, 0xaa]).unwrap();
        assert_eq!(
            route,
            Route {
                words: vec![0x12ff, 0x8000, 0xff03],
                checkpoint_count: 3
            }
        );
        assert!(decode_route(&[0, 0, 1]).is_err());
        assert!(decode_route(&[]).is_err());
        assert!(load_route(&rom(), 16, Direction::Forward).is_err());
    }
    #[test]
    fn reverse_nine_and_overlapping_views_are_loaded_independently() {
        let b = rom();
        let nine = load_route(&b, 9, Direction::Reversed).unwrap();
        assert_eq!(&nine.words[..2], &[0x0dc1, 0x03ca]);
        assert_eq!(nine.checkpoint_count, 6);
        let forward = load_route(&b, 8, Direction::Forward).unwrap();
        let reverse = load_route(&b, 8, Direction::Reversed).unwrap();
        assert_eq!(forward.words.len(), 61);
        assert_eq!(reverse.words.len(), 63);
        assert_ne!(forward.words, reverse.words);
        assert!(load_route(&b[..0x5800], 0, Direction::Forward).is_err());
    }
    #[test]
    fn heading_lookup_uses_even_byte_stride_and_reverse_half_not_interleaved_byte() {
        let b = rom();
        let h = decode_headings(&b[HEADINGS]).unwrap();
        let forward = [
            128, 128, 128, 0, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
        ];
        let reverse = [0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 0];
        assert_eq!(h.len(), ORIGINAL_TRACK_COUNT);
        for t in 0..ORIGINAL_TRACK_COUNT {
            assert_eq!(h[t], [forward[t], reverse[t]]);
        }
        let mut synthetic = [0xff; 64];
        for index in (0..64).step_by(2) {
            synthetic[index] = 0;
        }
        assert_eq!(decode_headings(&synthetic).unwrap(), vec![[0, 0]; 16]);
        assert!(decode_headings(&synthetic[..63]).is_err());
    }
    #[test]
    fn nonzero_heading_offsets_x_and_even_slot_retains_grid_rows() {
        assert_eq!(
            prepare_grid([0x17e1, 0x04c8], 0x80, 0),
            Grid {
                x: [0x1860, 0x1860, 0x1b60, 0x1b60],
                y: [0x03c0, 0x05a0, 0x03c0, 0x05a0],
                heading: 0x80,
                initial_surface_origin: [0x1860, 0x04c0]
            }
        );
    }
    #[test]
    fn zero_heading_selects_negative_step_and_exchanges_x_columns() {
        assert_eq!(
            prepare_grid([0x17e1, 0x04c8], 0, 0),
            Grid {
                x: [0x16e0, 0x16e0, 0x13e0, 0x13e0],
                y: [0x03c0, 0x05a0, 0x03c0, 0x05a0],
                heading: 0,
                initial_surface_origin: [0x13e0, 0x04c0]
            }
        );
    }
    #[test]
    fn odd_order_independently_swaps_rows_and_arithmetic_wraps() {
        assert_eq!(
            prepare_grid([0x401f, 0x000f], 0, 63),
            Grid {
                x: [0xff00, 0xff00, 0xfc00, 0xfc00],
                y: [0x00e0, 0xff00, 0x00e0, 0xff00],
                heading: 0,
                initial_surface_origin: [0xfc00, 0]
            }
        );
        // Heading need not be the original table's $80: source tests zero only.
        assert_eq!(
            prepare_grid([0xbfff, 0xffff], 1, 2).x,
            [0xbfe1, 0xbfe1, 0xc2e1, 0xc2e1]
        );
    }
    #[test]
    fn wheel_clear_and_initial_surface_have_distinct_preservation_boundaries() {
        let mut state = WheelPreparation {
            sampled: [7; 16],
            integrated: [8; 16],
            velocity: [9; 16],
            truck_surface: [10; 4],
        };
        clear_wheels(&mut state);
        assert_eq!(
            state,
            WheelPreparation {
                sampled: [0; 16],
                integrated: [0; 16],
                velocity: [0; 16],
                truck_surface: [10; 4]
            }
        );
        apply_initial_surface(&mut state, 0xe1);
        assert_eq!(
            state,
            WheelPreparation {
                sampled: [0; 16],
                integrated: [0xe1; 16],
                velocity: [0; 16],
                truck_surface: [0xe1; 4]
            }
        );
    }
    #[test]
    fn content_loads_inclusive_terrain_and_both_map_sizes_with_correct_lorom_banks() {
        let b = rom();
        let descriptors = decode_descriptors(&b[PARAMETERS], &b[GRAPHICS_POINTERS]).unwrap();
        // Independent physical windows translated from source CPU pointers.
        let physical = [
            (0x20000, 0x30600, 0x30000, 0x18000),
            (0x296b0, 0x34200, 0x33c00, 0x18000),
            (0x516aa, 0x69212, 0x68c12, 0x60000),
        ];
        for (track, (terrain, bg1, bg2, gfx)) in [0, 5, 9].into_iter().zip(physical) {
            let c = load_content(&b, &descriptors[track], track as u8).unwrap();
            assert_eq!(c.terrain, b[terrain..terrain + TERRAIN_BYTES]);
            assert_eq!(c.backgrounds[0], b[bg1..bg1 + BACKGROUND_BYTES]);
            assert_eq!(c.backgrounds[1], b[bg2..bg2 + BACKGROUND_BYTES]);
            assert_eq!(c.graphics, b[gfx..gfx + GRAPHICS_BYTES]);
        }
    }
    #[test]
    fn content_loader_rejects_truncated_or_non_cartridge_pointers() {
        let b = rom();
        let mut d = decode_descriptors(&b[PARAMETERS], &b[GRAPHICS_POINTERS])
            .unwrap()
            .remove(0);
        assert!(load_content(&b[..0x20000], &d, 0).is_err());
        d.terrain = CpuSource(0x7e8000);
        assert!(load_content(&b, &d, 0).is_err());
    }
    fn content() -> TrackContent {
        TrackContent {
            projection_offset: [0, 0],
            projection_scale: [103, 67],
            terrain: vec![0; TERRAIN_BYTES],
            backgrounds: [vec![0; BACKGROUND_BYTES], vec![0; BACKGROUND_BYTES]],
            graphics: vec![0; GRAPHICS_BYTES],
            routes: [
                Route {
                    words: vec![0x0101, 0x0201, 0x0101, 0x0201, 0xff01],
                    checkpoint_count: 1,
                },
                Route {
                    words: vec![0x0101, 0x0201, 0x0101, 0x0201, 0xff01],
                    checkpoint_count: 1,
                },
            ],
            headings: [0x80, 0],
            presentation: Some(crate::presentation::TrackPresentation {
                countdown: crate::presentation::transitions::CountdownLayout {
                    center: [91, 139],
                    destination_word: 0x4900,
                },
                title: crate::presentation::MapImage {
                    rectangle: crate::presentation::Rectangle {
                        width_flags: 2,
                        rows: 1,
                        palette_remap: [0, 1, 2, 3, 4, 5, 6, 7],
                    },
                    words: vec![0x2001, 0x2002],
                    destination_word: 0x4900,
                },
                banner_destination_word: 0x4880,
                hud_anchor: [7, 9],
                overlays: vec![],
            }),
            objects: Some(super::super::objects::ObjectContent {
                placements: super::super::objects::PlacementList {
                    declared_count: 1,
                    prefix: vec![super::super::objects::Placement {
                        world: [4096, 8192],
                        height: 0,
                    }],
                },
                pickups: [super::super::objects::Placement {
                    world: [4096, 8192],
                    height: 0,
                }; 4],
                dynamic: [super::super::objects::Placement::default(); 6],
                composites: None,
                track_three_sprites: false,
            }),
        }
    }
    fn custom_json(terrain_size: usize) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({"version":1,"projection_offset":[0,0],"projection_scale":[103,67],"terrain":vec![0;terrain_size],"backgrounds":[vec![0;BACKGROUND_BYTES],vec![0;BACKGROUND_BYTES]],"graphics":vec![0;GRAPHICS_BYTES],"routes":[[257,513,257,513,65281],[257,513,257,513,65281]],"headings":[128,0],"presentation":{"countdown":{"center":[91,139],"destination_word":18688},"title":{"rectangle":{"width_flags":2,"rows":1,"palette_remap":[0,1,2,3,4,5,6,7]},"words":[8193,8194],"destination_word":18688},"banner_destination_word":18560,"hud_anchor":[7,9],"overlays":[]},"objects":{"placements":{"declared_count":1,"prefix":[{"world":[4096,8192],"height":0}]},"pickups":vec![serde_json::json!({"world":[4096,8192],"height":0});4],"dynamic":vec![serde_json::json!({"world":[0,0],"height":0});6],"composites":null,"track_three_sprites":false}})).unwrap()
    }
    fn extensions(terrain_size: usize) -> MemoryAssets {
        let mut m = MemoryAssets::default();
        m.manifests.insert(
            "tracks.json".into(),
            Manifest {
                version: 1,
                entries: vec![FileAsset {
                    id: "added".into(),
                    file: "added.json".into(),
                }],
                order: vec!["original".into(), "added".into()],
            },
        );
        m.files
            .insert("added.json".into(), custom_json(terrain_size));
        m
    }
    #[test]
    fn named_extension_catalogue_combines_original_and_added_tracks_in_manifest_order() {
        let c = load_catalogue(
            vec![NamedTrack {
                id: "original".into(),
                content: content(),
            }],
            &extensions(TERRAIN_BYTES),
        )
        .unwrap();
        assert_eq!(c.order, ["original", "added"]);
        assert_eq!(c.tracks.len(), 2);
        assert_eq!(
            c.tracks.iter().find(|x| x.id == "added").unwrap().content,
            content()
        );
    }
    #[test]
    fn extension_validation_rejects_bad_semantics_and_missing_order_ids() {
        assert!(load_catalogue(vec![], &extensions(TERRAIN_BYTES - 1)).is_err());
        let mut m = extensions(TERRAIN_BYTES);
        m.manifests
            .get_mut("tracks.json")
            .unwrap()
            .order
            .push("missing".into());
        assert!(
            load_catalogue(
                vec![NamedTrack {
                    id: "original".into(),
                    content: content()
                }],
                &m
            )
            .is_err()
        );
        let mut m = extensions(TERRAIN_BYTES);
        let mut value: serde_json::Value = serde_json::from_slice(&m.files["added.json"]).unwrap();
        value["routes"][0] = serde_json::json!([257, 513]);
        m.files
            .insert("added.json".into(), serde_json::to_vec(&value).unwrap());
        assert!(
            load_catalogue(
                vec![NamedTrack {
                    id: "original".into(),
                    content: content()
                }],
                &m
            )
            .is_err()
        );
    }
    #[test]
    fn added_track_requires_explicit_validated_object_definitions() {
        let mut memory = extensions(TERRAIN_BYTES);
        let mut value: serde_json::Value =
            serde_json::from_slice(&memory.files["added.json"]).unwrap();
        value.as_object_mut().unwrap().remove("objects");
        memory
            .files
            .insert("added.json".into(), serde_json::to_vec(&value).unwrap());
        assert!(load_catalogue(vec![], &memory).is_err());
        value["objects"] = serde_json::json!({"placements":{"declared_count":0,"prefix":[]}});
        memory
            .files
            .insert("added.json".into(), serde_json::to_vec(&value).unwrap());
        assert!(load_catalogue(vec![], &memory).is_err());
    }
    #[test]
    fn added_track_requires_explicit_validated_title_hud_and_overlay_layout() {
        let mut memory = extensions(TERRAIN_BYTES);
        let mut value: serde_json::Value =
            serde_json::from_slice(&memory.files["added.json"]).unwrap();
        value.as_object_mut().unwrap().remove("presentation");
        memory
            .files
            .insert("added.json".into(), serde_json::to_vec(&value).unwrap());
        assert!(load_catalogue(vec![], &memory).is_err());
        value["presentation"] = serde_json::json!({"title":{"rectangle":{"width_flags":2,"rows":1,"palette_remap":[0,1,2,3,4,5,6,7]},"words":[8193],"destination_word":18688},"banner_destination_word":18560,"hud_anchor":[7,9],"overlays":[]});
        memory
            .files
            .insert("added.json".into(), serde_json::to_vec(&value).unwrap());
        assert!(load_catalogue(vec![], &memory).is_err());
    }
    #[test]
    fn catalogue_has_no_sixteen_track_ceiling_and_replacements_keep_named_identity() {
        let originals = (0..17)
            .map(|slot| NamedTrack {
                id: format!("original-{slot}"),
                content: content(),
            })
            .collect();
        let catalogue = load_catalogue(originals, &MemoryAssets::default()).unwrap();
        assert_eq!(catalogue.tracks.len(), 17);
        let mut memory = extensions(TERRAIN_BYTES);
        let manifest = memory.manifests.get_mut("tracks.json").unwrap();
        manifest.entries[0].id = "original".into();
        manifest.order = vec!["original".into()];
        let mut old = content();
        old.headings = [0, 0];
        let catalogue = load_catalogue(
            vec![NamedTrack {
                id: "original".into(),
                content: old,
            }],
            &memory,
        )
        .unwrap();
        assert_eq!(catalogue.tracks.len(), 1);
        assert_eq!(catalogue.tracks[0].content, content());
        assert_eq!(catalogue.order, ["original"]);
    }
}
