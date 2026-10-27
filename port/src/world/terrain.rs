//! Source terrain neighborhoods, wheel probes and world-to-screen projection.
//! Independent derivations and source ranges are in
//! terrain.trace.md. Coordinates retain source word units, not invented metres.
use super::tracks::{TRUCKS, WHEELS};
use anyhow::{Context, Result, ensure};

pub const WHEELS_PER_TRUCK: usize = 4;
pub const SINE_SAMPLES: usize = 256;
pub const TERRAIN_ROWS: usize = 44;
pub const CELL_BYTES: usize = 3;
pub const ROW_BYTES: usize = TERRAIN_ROWS * CELL_BYTES;

/// Immutable source sine/row tables. Physical addresses remain loader details.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SamplingTables {
    pub sine: [i8; SINE_SAMPLES],
    pub wheel_phases: [u8; WHEELS_PER_TRUCK],
    pub row_offsets: [u16; TERRAIN_ROWS],
}
pub fn load_sampling_tables(rom: &[u8]) -> Result<SamplingTables> {
    const SINE_ROM: usize = 0x1dc7;
    const PHASE_ROM: usize = 0x196a;
    const ROW_ROM: usize = 0x1f87;
    let sine = rom
        .get(SINE_ROM..SINE_ROM + SINE_SAMPLES)
        .context("truncated sine table")?;
    let phases = rom
        .get(PHASE_ROM..PHASE_ROM + WHEELS_PER_TRUCK)
        .context("truncated wheel phases")?;
    let rows = rom
        .get(ROW_ROM..ROW_ROM + TERRAIN_ROWS * 2)
        .context("truncated terrain rows")?;
    Ok(SamplingTables {
        sine: std::array::from_fn(|index| sine[index] as i8),
        wheel_phases: phases.try_into()?,
        row_offsets: std::array::from_fn(|index| {
            u16::from_le_bytes([rows[index * 2], rows[index * 2 + 1]])
        }),
    })
}
/// Five height bytes surrounding a selected packed control cell. Heights are
/// unsigned endpoints; their differences are signed in interpolation products.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Neighborhood {
    pub corners: [u8; 4], // a,b,c,d in original unreflected order
    pub center: u8,
    pub control: u8,
}
/// Controls of adjacent cells, accessed only on the fully blocked path. Explicit
/// inputs avoid inventing a RAM image or zero-filled out-of-window neighbors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdjacentControls {
    pub lower: u8,
    pub left: u8,
    pub upper: u8,
    pub right: u8,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Terrain {
    pub cells: Vec<u8>,
}
pub fn decode_terrain(bytes: &[u8]) -> Result<Terrain> {
    ensure!(
        bytes.len() == super::tracks::TERRAIN_BYTES,
        "invalid terrain copy window"
    );
    Ok(Terrain {
        cells: bytes.to_vec(),
    })
}
/// Reproduce source byte-width row/column indexing before checked native reads.
pub fn neighborhood(
    terrain: &Terrain,
    tables: &SamplingTables,
    position: [u16; 2],
) -> Result<Neighborhood> {
    let index = cell_index(tables, position)?;
    let read = |offset| {
        terrain
            .cells
            .get(index + offset)
            .copied()
            .context("probe beyond loaded terrain")
    };
    Ok(Neighborhood {
        corners: [
            read(0)?,
            read(CELL_BYTES)?,
            read(ROW_BYTES)?,
            read(ROW_BYTES + CELL_BYTES)?,
        ],
        center: read(1)?,
        control: read(2)?,
    })
}
/// Source directional code, not an invented compass/angle classification.
/// Values are retained because motion compares and transforms the encoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockDirection(pub u8);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Surface {
    Height(u8),
    Blocked(BlockDirection),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sample {
    pub surface: Surface,
    pub control_history: u8,
}
pub fn interpolate(patch: Neighborhood, fractions: [u8; 2], original_track: u8) -> u8 {
    const SIGN_BIT: u8 = 0x80;
    const HEIGHT_BIAS_TRACK: u8 = 8;
    const HEIGHT_BIAS: u8 = 0x20;
    let mut corners = patch.corners;
    let [mut x, mut y] = fractions;
    // Reflections use one's complement, including the FF->0 endpoint, while
    // swapping the corresponding corner geometry before selecting a triangle.
    if x & SIGN_BIT != 0 {
        x = !x;
        corners.swap(0, 1);
        corners.swap(2, 3);
    }
    if y & SIGN_BIT != 0 {
        y = !y;
        corners.swap(0, 2);
        corners.swap(1, 3);
    }
    let [a, b, c, _] = corners.map(i32::from);
    let middle = i32::from(patch.center);
    let sum = if y < x {
        (2 * middle - a - b) * i32::from(y) + (b - a) * i32::from(x)
    } else {
        (2 * middle - a - c) * i32::from(x) + (c - a) * i32::from(y)
    } + (a << u8::BITS);
    let height = ((sum as u16) >> u8::BITS) as u8;
    if original_track == HEIGHT_BIAS_TRACK {
        height.wrapping_sub(HEIGHT_BIAS)
    } else {
        height
    }
}
pub fn classify(
    patch: Neighborhood,
    adjacent: AdjacentControls,
    fractions: [u8; 2],
    original_track: u8,
    history: u8,
) -> Sample {
    const BLOCK_PAIR_MASK: u8 = 0xaa;
    const MARKER_TO_HIGH: u32 = 7;
    const BLOCK_FLAG: u8 = 2;
    const FALLBACK: u8 = 0x08;
    const DIAGONAL_A: u8 = 0x22;
    const DIAGONAL_B: u8 = 0x0a;
    const DIAGONAL_C: u8 = 0x88;
    const DIAGONAL_D: u8 = 0xa0;
    const RETURN_A: u8 = 0xa0;
    const RETURN_B: u8 = 0xe0;
    const RETURN_C: u8 = 0x20;
    const RETURN_D: u8 = 0x60;
    let [x, y] = fractions;
    let quadrant =
        2 * u8::from(y >= x) + u8::from(u16::from(x) + u16::from(y) > u16::from(u8::MAX));
    let pair = patch.control >> (quadrant * 2);
    let control_history = (history >> 1) | ((pair & 1) << MARKER_TO_HIGH);
    if pair & BLOCK_FLAG == 0 {
        return Sample {
            surface: Surface::Height(interpolate(patch, fractions, original_track)),
            control_history,
        };
    }
    let pattern = patch.control & BLOCK_PAIR_MASK;
    let direction = if pattern != BLOCK_PAIR_MASK {
        match pattern {
            DIAGONAL_A => RETURN_A,
            DIAGONAL_B => RETURN_B,
            DIAGONAL_C => RETURN_C,
            DIAGONAL_D => RETURN_D,
            _ => FALLBACK,
        }
    } else {
        let AdjacentControls {
            lower,
            left,
            upper,
            right,
        } = adjacent;
        let [lower, left, upper, right] =
            [lower, left, upper, right].map(|value| value & BLOCK_PAIR_MASK);
        // Cardinal openings take precedence; diagonal fallbacks then follow
        // the original left/right/upper/lower branch order, not nearest angle.
        if lower == 0 {
            0xc0
        } else if left == 0 {
            0
        } else if upper == 0 {
            0x40
        } else if right == 0 {
            0x80
        } else if left == DIAGONAL_B {
            RETURN_B
        } else if left == DIAGONAL_C {
            RETURN_C
        } else if right == DIAGONAL_D {
            RETURN_D
        } else if right == DIAGONAL_A {
            RETURN_A
        } else if upper == DIAGONAL_C {
            RETURN_C
        } else if upper == DIAGONAL_D {
            RETURN_D
        } else if lower == DIAGONAL_B {
            RETURN_B
        } else if lower == DIAGONAL_A {
            RETURN_A
        } else {
            FALLBACK
        }
    };
    Sample {
        surface: Surface::Blocked(BlockDirection(direction)),
        control_history,
    }
}
pub fn wheel_probe(
    tables: &SamplingTables,
    origin: [u16; 2],
    pose_angle: u8,
    wheel_index: u8,
) -> [u16; 2] {
    const QUARTER_TURN: u8 = 0x40;
    const WHEEL_SELECTOR_MASK: u8 = 3;
    let angle = pose_angle
        .wrapping_add(tables.wheel_phases[usize::from(wheel_index & WHEEL_SELECTOR_MASK)]);
    let offset = |index: u8| (i16::from(tables.sine[usize::from(index)]) * 2) as u16;
    [
        origin[0].wrapping_add(offset(angle.wrapping_add(QUARTER_TURN))),
        origin[1].wrapping_add(offset(angle)),
    ]
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WheelSamples {
    pub heights: [u8; WHEELS],
    pub block_flags: [u8; WHEELS_PER_TRUCK],
    pub control_history: u8,
    pub working_surface_height: u8,
}
/// Wheel group is a native logical truck slot. The source's descending wheel
/// scan (last index to first) must remain observable in control-history order.
// Explicit disjoint owner borrows keep mutation authority visible at this boundary.
#[allow(clippy::too_many_arguments)]
pub fn sample_wheels(
    terrain: &Terrain,
    tables: &SamplingTables,
    origin: [u16; 2],
    pose_angle: u8,
    truck_slot: usize,
    heights: [u8; WHEELS],
    surface_summaries: [u8; TRUCKS],
    history: u8,
    original_track: u8,
) -> Result<WheelSamples> {
    ensure!(truck_slot < TRUCKS, "invalid truck probe slot");
    let mut sampled = WheelSamples {
        heights,
        block_flags: [0; WHEELS_PER_TRUCK],
        control_history: history,
        working_surface_height: surface_summaries[truck_slot],
    };
    for wheel in (0..WHEELS_PER_TRUCK).rev() {
        let position = wheel_probe(tables, origin, pose_angle, wheel as u8);
        let sample = sample_at(
            terrain,
            tables,
            position,
            original_track,
            sampled.control_history,
        )?;
        sampled.control_history = sample.control_history;
        match sample.surface {
            Surface::Height(height) => {
                sampled.heights[truck_slot * WHEELS_PER_TRUCK + wheel] = height
            }
            Surface::Blocked(direction) => sampled.block_flags[wheel] = direction.0.wrapping_add(1),
        }
    }
    Ok(sampled)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Projection {
    pub offset: [u16; 2],
    pub scale: [u8; 2],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenPosition {
    pub x: u16,
    pub y: u8,
}
pub fn project(projection: Projection, world: [u16; 2], height: u8) -> ScreenPosition {
    const ISOMETRIC_X_SCALE: u8 = 0x22;
    const SCREEN_Y_BASE: u16 = 0x2c00;
    const PROJECTION_SHIFT: u32 = 4;
    const HEIGHT_SCALE: u16 = 8;
    // The source reads MPYM independently for each signed product. Truncating
    // after addition would incorrectly retain discarded fractional carries.
    let product = |coordinate: u16, scale: u8| {
        ((i32::from(coordinate as i16) * i32::from(scale as i8)) >> u8::BITS) as u16
    };
    let x = (product(world[1], ISOMETRIC_X_SCALE)
        .wrapping_add(product(world[0], projection.scale[0]))
        >> PROJECTION_SHIFT)
        .wrapping_add(projection.offset[0]);
    let y_partial = product(world[1], projection.scale[1]).wrapping_add(projection.offset[1]);
    let y = (SCREEN_Y_BASE
        .wrapping_sub(y_partial)
        .wrapping_sub(u16::from(height) * HEIGHT_SCALE)
        >> PROJECTION_SHIFT) as u8;
    ScreenPosition { x, y }
}

fn cell_index(tables: &SamplingTables, position: [u16; 2]) -> Result<usize> {
    let column = (position[0] >> u8::BITS) as u8;
    let row_byte = ((position[1] >> u8::BITS) as u8).wrapping_mul(2);
    let row = *tables
        .row_offsets
        .get(usize::from(row_byte / 2))
        .context("unsupported terrain row")?;
    Ok(usize::from(row.wrapping_add(u16::from(
        column.wrapping_mul(CELL_BYTES as u8),
    ))))
}
pub fn sample_at(
    terrain: &Terrain,
    tables: &SamplingTables,
    position: [u16; 2],
    original_track: u8,
    history: u8,
) -> Result<Sample> {
    const BLOCK_PAIR_MASK: u8 = 0xaa;
    const CONTROL_OFFSET: usize = 2;
    let patch = neighborhood(terrain, tables, position)?;
    let adjacent = if patch.control & BLOCK_PAIR_MASK == BLOCK_PAIR_MASK {
        let index = cell_index(tables, position)?;
        let read = |delta: isize| -> Result<u8> {
            let relative = index as isize + delta;
            if (2 - ROW_BYTES as isize..0).contains(&relative) {
                return Ok(crate::config::UNCOPIED_TERRAIN_CONTROL);
            }
            let at = usize::try_from(relative)
                .context("terrain neighbor precedes supported WRAM controls")?;
            terrain
                .cells
                .get(at)
                .copied()
                .context("terrain neighbor beyond loaded window")
        };
        // The source returns at the first open cardinal neighbor. Later reads
        // may lie outside the copied terrain at an edge and must not be made
        // when an earlier opening already determines the blocking direction.
        let mut controls = [BLOCK_PAIR_MASK; 4];
        for (slot, delta) in [
            (ROW_BYTES + CONTROL_OFFSET) as isize,
            -1,
            CONTROL_OFFSET as isize - ROW_BYTES as isize,
            (CELL_BYTES + CONTROL_OFFSET) as isize,
        ]
        .into_iter()
        .enumerate()
        {
            controls[slot] = read(delta)?;
            if controls[slot] & BLOCK_PAIR_MASK == 0 {
                break;
            }
        }
        let [lower, left, upper, right] = controls;
        AdjacentControls {
            lower,
            left,
            upper,
            right,
        }
    } else {
        AdjacentControls {
            lower: 0,
            left: 0,
            upper: 0,
            right: 0,
        }
    };
    Ok(classify(
        patch,
        adjacent,
        position.map(|coordinate| coordinate as u8),
        original_track,
        history,
    ))
}

#[cfg(test)]
mod tests {
    use super::super::tracks::TERRAIN_BYTES;
    use super::*;
    const SINE_ROM: std::ops::Range<usize> = 0x1dc7..0x1ec7;
    const PHASE_ROM: std::ops::Range<usize> = 0x196a..0x196e;
    const ROW_ROM: std::ops::Range<usize> = 0x1f87..0x1fdf;
    const INTERIOR_ORIGIN: [u16; 2] = [0x1080, 0x1080];
    const FLAT_HEIGHT: u8 = 40;
    const TRACK_WITH_HEIGHT_BIAS: u8 = 8;
    const ORDINARY_TRACK: u8 = 0;
    const FIRST_TRUCK: usize = 0;
    const THIRD_TRUCK: usize = 2;
    const PRIOR_SAMPLE: u8 = 77;
    const ALL_BLOCK_PAIRS: u8 = 0xaa;
    const ALL_SURFACE_MARKERS: u8 = 0x55;
    const UNCHANGED_HISTORY: u8 = 0x5a;
    const DEFAULT_ADJACENT: AdjacentControls = AdjacentControls {
        lower: ALL_BLOCK_PAIRS,
        left: ALL_BLOCK_PAIRS,
        upper: ALL_BLOCK_PAIRS,
        right: ALL_BLOCK_PAIRS,
    };
    fn rom() -> Vec<u8> {
        std::fs::read("../input/game.rom").unwrap()
    }
    // Independent ROM fixture parsing only: no native behavior as an oracle.
    fn tables() -> SamplingTables {
        let bytes = rom();
        let sine = std::array::from_fn(|i| bytes[SINE_ROM.start + i] as i8);
        let wheel_phases = bytes[PHASE_ROM].try_into().unwrap();
        let row_offsets = std::array::from_fn(|i| {
            u16::from_le_bytes(
                bytes[ROW_ROM.start + i * 2..ROW_ROM.start + i * 2 + 2]
                    .try_into()
                    .unwrap(),
            )
        });
        SamplingTables {
            sine,
            wheel_phases,
            row_offsets,
        }
    }
    fn patch(corners: [u8; 4], center: u8, control: u8) -> Neighborhood {
        Neighborhood {
            corners,
            center,
            control,
        }
    }
    fn flat(control: u8) -> Terrain {
        let mut cells = vec![FLAT_HEIGHT; TERRAIN_BYTES];
        for record in cells.chunks_exact_mut(CELL_BYTES) {
            record[2] = control;
        }
        Terrain { cells }
    }
    #[test]
    fn rom_tables_load_full_signed_sine_four_phases_and_all_row_offsets() {
        let t = load_sampling_tables(&rom()).unwrap();
        assert_eq!(t.wheel_phases, [0x60, 0xa0, 0x20, 0xe0]);
        assert_eq!(
            [t.sine[0], t.sine[64], t.sine[128], t.sine[192]],
            [0, 127, 0, -127]
        );
        assert_eq!(
            [t.sine[32], t.sine[96], t.sine[160], t.sine[224]],
            [91, 91, -91, -91]
        );
        assert_eq!(
            [t.row_offsets[0], t.row_offsets[1], t.row_offsets[43]],
            [0, 132, 5676]
        );
        assert!(load_sampling_tables(&rom()[..ROW_ROM.end - 1]).is_err());
    }
    #[test]
    fn terrain_validation_retains_final_inclusive_byte_and_rejects_truncated_windows() {
        let mut cells = flat(0).cells;
        cells[TERRAIN_BYTES - 1] = 0xe7;
        assert_eq!(decode_terrain(&cells).unwrap().cells, cells);
        assert!(decode_terrain(&cells[..TERRAIN_BYTES - 1]).is_err());
        assert!(decode_terrain(&[]).is_err());
    }
    #[test]
    fn all_sixteen_rom_terrain_views_preserve_shared_union_and_overlapping_last_byte() {
        const TERRAIN_STARTS: [usize; 16] = [
            0x20000, 0x216b0, 0x22d60, 0x24410, 0x28000, 0x296b0, 0x2ad60, 0x2c410, 0x50000,
            0x516aa, 0x52d5a, 0x5440a, 0x58000, 0x596b0, 0x5ad60, 0x5c410,
        ];
        let bytes = rom();
        let expected_first = [
            128, 128, 85, 21, 155, 128, 168, 64, 64, 64, 64, 64, 64, 64, 64, 64,
        ];
        let expected_last = [
            128, 85, 21, 35, 128, 168, 64, 0, 64, 64, 64, 32, 64, 64, 64, 255,
        ];
        for (i, start) in TERRAIN_STARTS.into_iter().enumerate() {
            let data = decode_terrain(&bytes[start..start + TERRAIN_BYTES]).unwrap();
            assert_eq!(
                [data.cells[0], data.cells[TERRAIN_BYTES - 1]],
                [expected_first[i], expected_last[i]]
            );
            assert_eq!(data.cells, &bytes[start..start + TERRAIN_BYTES]);
        }
    }
    #[test]
    fn first_open_neighbor_at_top_edge_does_not_read_unneeded_upper_cell() {
        let mut terrain = flat(0);
        terrain.cells[2] = ALL_BLOCK_PAIRS;
        // The lower neighbor is open. Source returns C0 before attempting the
        // negative upper offset, which is outside this loaded field.
        assert_eq!(
            sample_at(&terrain, &tables(), [0, 0], ORDINARY_TRACK, 0)
                .unwrap()
                .surface,
            Surface::Blocked(BlockDirection(0xc0))
        );
    }

    #[test]
    fn top_edge_uses_explicit_uncopied_wram_control_policy() {
        let mut terrain = flat(ALL_BLOCK_PAIRS);
        terrain.cells[2] = ALL_BLOCK_PAIRS;
        assert_eq!(
            sample_at(&terrain, &tables(), [0x0100, 0], ORDINARY_TRACK, 0)
                .unwrap()
                .surface,
            Surface::Blocked(BlockDirection(0x40))
        );
    }

    #[test]
    fn neighborhood_uses_three_byte_cell_stride_and_distinct_center_and_lower_corners() {
        let mut terrain = flat(0);
        let start = ROW_BYTES * 2 + CELL_BYTES * 31;
        for (offset, value) in [
            (0, 10),
            (3, 20),
            (1, 30),
            (ROW_BYTES, 40),
            (ROW_BYTES + 3, 50),
            (2, 0xa5),
        ] {
            terrain.cells[start + offset] = value;
        }
        assert_eq!(
            neighborhood(&terrain, &tables(), [0x1f43, 0x027f]).unwrap(),
            patch([10, 20, 40, 50], 30, 0xa5)
        );
    }
    #[test]
    fn neighborhood_reproduces_byte_width_coordinate_index_wrap_before_word_addition() {
        let mut terrain = flat(0);
        terrain.cells[0] = 11;
        terrain.cells[CELL_BYTES] = 22;
        // 3*$56=$102 truncates to$02, and 2*$80 truncates to0.
        // This addresses byte2 rather than a fictitious column86 or row128.
        terrain.cells[2] = 33;
        terrain.cells[5] = 44;
        terrain.cells[3] = 55;
        terrain.cells[ROW_BYTES + 2] = 66;
        terrain.cells[ROW_BYTES + 5] = 77;
        terrain.cells[4] = 88;
        assert_eq!(
            neighborhood(&terrain, &tables(), [0x5600, 0x8000]).unwrap(),
            patch([33, 44, 66, 77], 55, 88)
        );
    }
    #[test]
    fn checked_neighborhood_does_not_invent_missing_bottom_row_or_short_field_bytes() {
        assert!(neighborhood(&Terrain { cells: vec![0; 2] }, &tables(), [0, 0]).is_err());
        assert!(neighborhood(&flat(0), &tables(), [0x2b00, 0x2b00]).is_err());
    }
    #[test]
    fn horizontal_triangle_combines_both_signed_slopes_and_fractional_carry() {
        let p = patch([40, 100, 60, 80], 80, 0);
        // 40*256+(100-40)*64+(160-40-100)*32=14720; high byte57.
        assert_eq!(interpolate(p, [64, 32], ORDINARY_TRACK), 57);
        assert_eq!(interpolate(p, [0, 0], ORDINARY_TRACK), 40);
        assert_eq!(
            interpolate(patch([10, 11, 10, 10], 10, 0), [127, 126], ORDINARY_TRACK),
            10
        );
    }
    #[test]
    fn alternate_triangle_uses_center_a_c_slope_and_y_slope_including_equal_diagonal() {
        let p = patch([40, 100, 60, 80], 80, 0);
        // (160-40-60)*32+(60-40)*64+40*256=13440 =>52.
        assert_eq!(interpolate(p, [32, 64], ORDINARY_TRACK), 52);
        assert_eq!(interpolate(p, [64, 64], ORDINARY_TRACK), 60);
    }
    #[test]
    fn negative_fractions_reflect_by_ones_complement_and_swap_corresponding_corners() {
        let p = patch([40, 100, 60, 80], 80, 0);
        // x=$C0 reflects to63; swapped a100,b40,c80,d60: 25600-3780+640=22460 =>87.
        assert_eq!(interpolate(p, [0xc0, 32], ORDINARY_TRACK), 87);
        // y=$BF reflects to64, swapped a60,b80,c40,d100:15360+1920-1280 =>62.
        assert_eq!(interpolate(p, [32, 0xbf], ORDINARY_TRACK), 62);
        // both: a80,b60,c100,d40, x63,y64 =>20480-1260+1280=20500 =>80.
        assert_eq!(interpolate(p, [0xc0, 0xbf], ORDINARY_TRACK), 80);
        // $FF ->0, not1: exact opposite corner at both negative endpoints.
        assert_eq!(interpolate(p, [0xff, 0xff], ORDINARY_TRACK), 80);
    }
    #[test]
    fn endpoint_differences_are_signed_words_and_height_result_wraps_as_byte() {
        let p = patch([250, 0, 0, 0], 0, 0);
        assert_eq!(interpolate(p, [64, 32], ORDINARY_TRACK), 156);
        assert_eq!(
            interpolate(patch([0, 255, 255, 0], 255, 0), [127, 126], ORDINARY_TRACK),
            252
        );
    }
    #[test]
    fn track_eight_subtracts_height_bias_for_both_triangles_with_byte_underflow() {
        let p = patch([40, 100, 60, 80], 80, 0);
        assert_eq!(interpolate(p, [64, 32], TRACK_WITH_HEIGHT_BIAS), 25);
        assert_eq!(interpolate(p, [32, 64], TRACK_WITH_HEIGHT_BIAS), 20);
        assert_eq!(
            interpolate(patch([0; 4], 0, 0), [0, 0], TRACK_WITH_HEIGHT_BIAS),
            224
        );
    }
    #[test]
    fn four_quadrants_follow_compare_and_sum_carry_and_shift_selected_marker() {
        // Only selected pair has flag1 and block0; all other pairs block.
        for (fractions, control) in [
            ([64, 32], 0xa9),
            ([224, 160], 0xa6),
            ([32, 64], 0x9a),
            ([160, 224], 0x6a),
        ] {
            let r = classify(
                patch([40; 4], 40, control),
                DEFAULT_ADJACENT,
                fractions,
                ORDINARY_TRACK,
                UNCHANGED_HISTORY,
            );
            assert_eq!(
                r,
                Sample {
                    surface: Surface::Height(40),
                    control_history: 0xad
                }
            );
        }
    }
    #[test]
    fn quadrant_equal_and_sum_overflow_boundaries_choose_source_pairs() {
        // Equal fractions select pair2; exactly256 also sets second index bit.
        let equal = classify(
            patch([40; 4], 40, 0x10),
            DEFAULT_ADJACENT,
            [64, 64],
            ORDINARY_TRACK,
            0,
        );
        assert_eq!(equal.control_history, 0x80);
        let sum = classify(
            patch([40; 4], 40, 0x40),
            DEFAULT_ADJACENT,
            [128, 128],
            ORDINARY_TRACK,
            0,
        );
        assert_eq!(sum.control_history, 0x80);
    }
    #[test]
    fn blocked_local_diagonal_patterns_return_embedded_lda_directions_and_fallback() {
        for (control, direction) in [
            (0x22, 0xa0),
            (0x0a, 0xe0),
            (0x88, 0x20),
            (0xa0, 0x60),
            (0x02, 0x08),
        ] {
            // Select a pair whose block bit is set without altering the pattern.
            let fractions = match control {
                0x88 | 0xa0 => [160, 224],
                _ => [64, 32],
            };
            let r = classify(
                patch([40; 4], 40, control),
                DEFAULT_ADJACENT,
                fractions,
                ORDINARY_TRACK,
                0x80,
            );
            assert_eq!(r.surface, Surface::Blocked(BlockDirection(direction)));
            assert_eq!(r.control_history, 0x40);
        }
    }
    #[test]
    fn fully_blocked_cell_selects_first_open_cardinal_neighbor_in_fixed_priority() {
        for (a, direction) in [
            (
                AdjacentControls {
                    lower: 0,
                    left: 0,
                    upper: 0,
                    right: 0,
                },
                0xc0,
            ),
            (
                AdjacentControls {
                    left: 0,
                    upper: 0,
                    right: 0,
                    ..DEFAULT_ADJACENT
                },
                0,
            ),
            (
                AdjacentControls {
                    upper: 0,
                    right: 0,
                    ..DEFAULT_ADJACENT
                },
                0x40,
            ),
            (
                AdjacentControls {
                    right: 0,
                    ..DEFAULT_ADJACENT
                },
                0x80,
            ),
        ] {
            assert_eq!(
                classify(patch([0; 4], 0, 0xaa), a, [64, 32], 0, 0).surface,
                Surface::Blocked(BlockDirection(direction))
            );
        }
    }
    #[test]
    fn fully_blocked_cell_searches_all_neighbor_diagonal_cases_then_falls_back() {
        for (a, direction) in [
            (
                AdjacentControls {
                    left: 0x0a,
                    ..DEFAULT_ADJACENT
                },
                0xe0,
            ),
            (
                AdjacentControls {
                    left: 0x88,
                    ..DEFAULT_ADJACENT
                },
                0x20,
            ),
            (
                AdjacentControls {
                    right: 0xa0,
                    ..DEFAULT_ADJACENT
                },
                0x60,
            ),
            (
                AdjacentControls {
                    right: 0x22,
                    ..DEFAULT_ADJACENT
                },
                0xa0,
            ),
            (
                AdjacentControls {
                    upper: 0x88,
                    ..DEFAULT_ADJACENT
                },
                0x20,
            ),
            (
                AdjacentControls {
                    upper: 0xa0,
                    ..DEFAULT_ADJACENT
                },
                0x60,
            ),
            (
                AdjacentControls {
                    lower: 0x0a,
                    ..DEFAULT_ADJACENT
                },
                0xe0,
            ),
            (
                AdjacentControls {
                    lower: 0x22,
                    ..DEFAULT_ADJACENT
                },
                0xa0,
            ),
            (DEFAULT_ADJACENT, 0x08),
        ] {
            assert_eq!(
                classify(patch([0; 4], 0, 0xaa), a, [64, 32], 0, 0).surface,
                Surface::Blocked(BlockDirection(direction))
            );
        }
        let priority = AdjacentControls {
            left: 0x0a,
            right: 0xa0,
            upper: 0x88,
            lower: 0x22,
        };
        assert_eq!(
            classify(patch([0; 4], 0, 0xaa), priority, [64, 32], 0, 0).surface,
            Surface::Blocked(BlockDirection(0xe0))
        );
    }
    #[test]
    fn classification_shifts_marker_even_for_blocking_return_and_masks_other_flag_bits() {
        let r = classify(patch([0; 4], 0, 0xab), DEFAULT_ADJACENT, [64, 32], 0, 0x03);
        assert_eq!(
            r,
            Sample {
                surface: Surface::Blocked(BlockDirection(0x08)),
                control_history: 0x81
            }
        );
    }
    #[test]
    fn wheel_positions_use_signed_doubled_sine_and_source_four_phases() {
        let t = tables();
        assert_eq!(wheel_probe(&t, [0x1000, 0x1000], 0, 0), [0x0f4a, 0x10b6]);
        assert_eq!(wheel_probe(&t, [0x1000, 0x1000], 0, 1), [0x0f4a, 0x0f4a]);
        assert_eq!(wheel_probe(&t, [0x1000, 0x1000], 0, 2), [0x10b6, 0x10b6]);
        assert_eq!(wheel_probe(&t, [0x1000, 0x1000], 0, 3), [0x10b6, 0x0f4a]);
        assert_eq!(wheel_probe(&t, [0x1000, 0x1000], 0, 0xff), [0x10b6, 0x0f4a]);
    }
    #[test]
    fn wheel_angle_and_world_addition_wrap_and_initial_grid_uses_inherited_pose() {
        let t = tables();
        assert_eq!(wheel_probe(&t, [0, 0], 0, 0), [0xff4a, 0x00b6]);
        // Angle$A0 plus wheel0 phase$60 wraps0, giving X+254,Y+0.
        assert_eq!(wheel_probe(&t, [0xff80, 0], 0xa0, 0), [0x007e, 0]);
        // Source grid's Y=$0600 is truncated by SEP30, selecting wheel0;
        // working pose is inherited and is NOT overwritten with grid heading.
        assert_eq!(wheel_probe(&t, [0x1000, 0x1000], 0xa0, 0), [0x10fe, 0x1000]);
    }
    #[test]
    fn four_wheel_scan_updates_only_selected_truck_and_returns_prior_surface_summary() {
        let r = sample_wheels(
            &flat(ALL_SURFACE_MARKERS),
            &tables(),
            INTERIOR_ORIGIN,
            0,
            THIRD_TRUCK,
            [PRIOR_SAMPLE; WHEELS],
            [10, 20, 30, 40],
            UNCHANGED_HISTORY,
            0,
        )
        .unwrap();
        let mut expected = [PRIOR_SAMPLE; WHEELS];
        expected[8..12].fill(FLAT_HEIGHT);
        assert_eq!(r.heights, expected);
        assert_eq!(r.block_flags, [0; 4]);
        assert_eq!(r.control_history, 0xf5);
        assert_eq!(r.working_surface_height, 30);
    }
    #[test]
    fn blocked_wheel_keeps_prior_height_sets_direction_plus_one_and_preserves_scan_order() {
        let mut terrain = flat(0);
        // Wheel2 is at($1136,$1136): cell17,17. Its lower control remains0,
        // so all-blocked local cell returns cardinal$C0 and records$C1.
        let cell = ROW_BYTES * 17 + CELL_BYTES * 17;
        terrain.cells[cell + 2] = ALL_BLOCK_PAIRS;
        // Wheel3 at cell17,15 supplies marker1; only first scan contributes.
        terrain.cells[ROW_BYTES * 15 + CELL_BYTES * 17 + 2] = ALL_SURFACE_MARKERS;
        let r = sample_wheels(
            &terrain,
            &tables(),
            INTERIOR_ORIGIN,
            0,
            FIRST_TRUCK,
            [PRIOR_SAMPLE; WHEELS],
            [10, 20, 30, 40],
            0,
            0,
        )
        .unwrap();
        assert_eq!(&r.heights[..4], &[40, 40, 77, 40]);
        assert_eq!(r.block_flags, [0, 0, 0xc1, 0]);
        assert_eq!(r.control_history, 0x10);
        assert_eq!(r.working_surface_height, 10);
    }
    #[test]
    fn wheel_scan_rejects_invalid_native_slot_or_missing_probe_data_transactionally() {
        assert!(
            sample_wheels(
                &flat(0),
                &tables(),
                INTERIOR_ORIGIN,
                0,
                TRUCKS,
                [0; WHEELS],
                [0; TRUCKS],
                0,
                0
            )
            .is_err()
        );
        assert!(
            sample_wheels(
                &Terrain { cells: vec![0; 2] },
                &tables(),
                INTERIOR_ORIGIN,
                0,
                0,
                [0; WHEELS],
                [0; TRUCKS],
                0,
                0
            )
            .is_err()
        );
    }
    #[test]
    fn projection_uses_independent_signed_products_offsets_height_and_low_y_byte() {
        let p = Projection {
            offset: [0, 0],
            scale: [103, 67],
        };
        // X=(1648+1088)>>4=171; Y=(11264-2144-320)>>4=550 ->38.
        assert_eq!(
            project(p, [0x1000, 0x2000], 40),
            ScreenPosition { x: 171, y: 38 }
        );
        assert_eq!(
            project(
                Projection {
                    offset: [0xffff, 16],
                    ..p
                },
                [0x1000, 0x2000],
                40
            ),
            ScreenPosition { x: 170, y: 37 }
        );
        assert_eq!(
            project(p, [0xff00, 0xff00], 0),
            ScreenPosition { x: 4087, y: 196 }
        );
    }
    #[test]
    fn projection_scale_bytes_are_signed_mode7_factors_even_for_added_content() {
        assert_eq!(
            project(
                Projection {
                    offset: [0, 0],
                    scale: [255, 255]
                },
                [0x1000, 0x1000],
                0
            ),
            ScreenPosition { x: 33, y: 193 }
        );
        // With Y zero, signed X contribution=-16 then logical word shift gives4095.
        assert_eq!(
            project(
                Projection {
                    offset: [0, 0],
                    scale: [255, 255]
                },
                [0x1000, 0],
                0
            ),
            ScreenPosition { x: 4095, y: 192 }
        );
    }
    #[test]
    fn projection_truncates_each_product_before_addition_and_uses_logical_word_shift() {
        let p = Projection {
            offset: [0, 0],
            scale: [103, 67],
        };
        assert_eq!(project(p, [1, 7], 0).x, 0); //103/256 and238/256 truncate separately
        assert_eq!(project(p, [4, 0xffff], 0), ScreenPosition { x: 0, y: 192 });
        assert_eq!(project(p, [0, 0], 255), ScreenPosition { x: 0, y: 64 });
        assert_eq!(
            project(
                Projection {
                    offset: [0xffff, 0xffff],
                    scale: [0, 0]
                },
                [0, 0],
                0
            ),
            ScreenPosition { x: 0xffff, y: 192 }
        );
    }
}
