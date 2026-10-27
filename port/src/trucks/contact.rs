//! Truck pair contact contracts. Source derivations and every conditional entry:
//! contact.trace.md. Pairs share real terrain and truck proposal ownership.
use super::{
    bearings::BearingTable,
    motion::{Truck, WorkingMotion},
};
use crate::world::terrain::{SamplingTables, Terrain};
use crate::world::tracks::TRUCKS;
use anyhow::{Result, ensure};

pub const CONTACT_DISTANCE_LIMIT: u16 = 40;
pub const DISTANT_PAIR: u16 = 41;
pub const CONTACT_COOLDOWN_TICKS: u8 = 6;
pub const IMPACT_SOUND_COMMAND: u16 = 0x0302;
pub fn coarse_distance(current: [u16; 2], other: [u16; 2]) -> u16 {
    const AXIS_CUTOFF: u16 = 0x0a00;
    const COARSE_AXIS_SHIFT: u32 = 6;
    let mut sum = 0;
    for axis in 0..2 {
        let distance = (current[axis].wrapping_sub(other[axis]) as i16).unsigned_abs();
        if distance >= AXIS_CUTOFF {
            return DISTANT_PAIR;
        }
        let coarse = distance >> COARSE_AXIS_SHIFT;
        sum += coarse * coarse;
    }
    sum
}
pub fn heading_sector(relative: u8) -> u8 {
    const SECTOR_BIAS: u8 = 0x20;
    const SECTOR_SHIFT: u32 = 6;
    relative.wrapping_add(SECTOR_BIAS) >> SECTOR_SHIFT
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContactAngles {
    pub normal: u8,
    pub relative_current: u8,
    pub relative_other: u8,
    pub packed_sectors: u8,
}
/// Position differences use committed records, while the current heading is the
/// current working heading. AI's shared vector quantization must not be replaced
/// by float atan2 or an independently different contact quantizer.
pub fn contact_angles(
    committed_current: &Truck,
    working: &WorkingMotion,
    other: &Truck,
    tables: &BearingTable,
) -> ContactAngles {
    const VECTOR_REDUCTION_SHIFT: u32 = 4;
    const OTHER_SECTOR_SHIFT: u32 = 4;
    let x = (committed_current.world[0].wrapping_sub(other.world[0]) >> VECTOR_REDUCTION_SHIFT)
        as u8 as i8;
    let y = ((committed_current.world[1].wrapping_sub(other.world[1]) >> VECTOR_REDUCTION_SHIFT)
        as u8)
        .wrapping_neg() as i8;
    let normal = tables.cells[usize::from(super::ai::quantize_vector(x, y))];
    let relative_current = (working.heading as u8).wrapping_sub(normal);
    let relative_other = (other.heading as u8).wrapping_sub(normal);
    ContactAngles {
        normal,
        relative_current,
        relative_other,
        packed_sectors: heading_sector(relative_current)
            | (heading_sector(relative_other) << OTHER_SECTOR_SHIFT),
    }
}
/// Mutate speed/heading only. Positional separation follows every case, including
/// quiet/attract responses. High bytes survive all low-byte stores.
pub fn respond_motion(
    working: &mut WorkingMotion,
    other: &mut Truck,
    packed_sectors: u8,
    animation: u8,
    active_humans: u8,
) -> Option<u16> {
    const REVERSE_CASE: u8 = 0x02;
    const PARALLEL_CASES: [u8; 2] = [0, 0x22];
    const OBLIQUE_CASES: [u8; 4] = [0x11, 0x13, 0x31, 0x33];
    const IMPACT_THRESHOLD: u8 = 0x20;
    const PARALLEL_TURN: u8 = 0x10;
    const OBLIQUE_TURN: u8 = 4;
    const OTHER_ASYMMETRIC_TURN: u8 = 8;
    let current_speed = working.speed as u8;
    let other_speed = other.speed as u8;
    let sound = if packed_sectors == REVERSE_CASE {
        let current = other_speed.wrapping_neg();
        let exchanged = current_speed.wrapping_neg();
        set_low(&mut working.speed, current);
        set_low(&mut other.speed, exchanged);
        (current as i8).unsigned_abs() >= IMPACT_THRESHOLD
            || (exchanged as i8).unsigned_abs() >= IMPACT_THRESHOLD
    } else if PARALLEL_CASES.contains(&packed_sectors) {
        set_low(&mut working.speed, other_speed);
        set_low(&mut other.speed, current_speed);
        turn(
            &mut other.heading,
            if animation & 1 == 0 {
                PARALLEL_TURN
            } else {
                PARALLEL_TURN.wrapping_neg()
            },
        );
        (other_speed.wrapping_sub(current_speed) as i8).unsigned_abs() >= IMPACT_THRESHOLD
    } else if OBLIQUE_CASES.contains(&packed_sectors) {
        turn(
            &mut working.heading,
            if packed_sectors & 3 == 1 {
                OBLIQUE_TURN.wrapping_neg()
            } else {
                OBLIQUE_TURN
            },
        );
        turn(
            &mut other.heading,
            if packed_sectors >> 4 == 3 {
                OBLIQUE_TURN.wrapping_neg()
            } else {
                OBLIQUE_TURN
            },
        );
        false
    } else if packed_sectors & 1 != 0 {
        turn(&mut working.heading, PARALLEL_TURN);
        other_speed >= IMPACT_THRESHOLD
    } else {
        if (current_speed as i8) >= 0 {
            set_low(&mut working.speed, current_speed >> 2);
        }
        turn(&mut other.heading, OTHER_ASYMMETRIC_TURN);
        working.speed as u8 >= IMPACT_THRESHOLD
    };
    (sound && active_humans != 0).then_some(IMPACT_SOUND_COMMAND)
}
/// Isolated original $83E1 entry has no established natural caller. Keeping its
/// conditional behavior does not route normal sector cases through it.
pub fn conditional_deflect_current(working: &mut WorkingMotion, animation: u8) {
    const SPEED_CUTOFF: u8 = 0x60;
    const SPEED_INCREMENT: u8 = 0x20;
    const HEADING_INCREMENT: u8 = 8;
    if (working.speed as u8) < SPEED_CUTOFF {
        let speed = (working.speed as u8).wrapping_add(SPEED_INCREMENT);
        set_low(&mut working.speed, speed);
    }
    turn(
        &mut working.heading,
        if animation & 1 == 0 {
            HEADING_INCREMENT
        } else {
            HEADING_INCREMENT.wrapping_neg()
        },
    );
}
pub struct ContactEnvironment<'a> {
    pub terrain: &'a Terrain,
    pub sampling: &'a SamplingTables,
    pub tables: &'a BearingTable,
    pub original_track: u8,
    pub animation: u8,
    pub active_humans: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Separation {
    pub proposed_current: [u16; 2],
    pub proposed_other: [u16; 2],
    pub accepted: [bool; 2],
}
/// The midpoint uses committed current position, not working coordinates. Two
/// actual terrain classifications update shared marker history even if blocked.
pub fn separate_pair(
    committed_current: &Truck,
    working: &mut WorkingMotion,
    other: &mut Truck,
    normal: u8,
    environment: &ContactEnvironment<'_>,
    terrain_history: &mut u8,
) -> Result<Separation> {
    use crate::world::terrain::{Surface, sample_at};
    const QUARTER_TURN: u8 = 0x40;
    const HALF_TURN: u8 = 0x80;
    let midpoint = std::array::from_fn::<_, 2, _>(|axis| {
        committed_current.world[axis].wrapping_add(other.world[axis]) >> 1
    });
    let candidate = |angle: u8| {
        let offset =
            |heading: u8| (i16::from(environment.sampling.sine[usize::from(heading)]) * 2) as u16;
        [
            midpoint[0].wrapping_add(offset(angle.wrapping_add(QUARTER_TURN))),
            midpoint[1].wrapping_add(offset(angle)),
        ]
    };
    let proposed_current = candidate(normal);
    let proposed_other = candidate(normal.wrapping_add(HALF_TURN));
    let current_sample = sample_at(
        environment.terrain,
        environment.sampling,
        proposed_current,
        environment.original_track,
        *terrain_history,
    )?;
    let other_sample = sample_at(
        environment.terrain,
        environment.sampling,
        proposed_other,
        environment.original_track,
        current_sample.control_history,
    )?;
    let accepted = [
        matches!(current_sample.surface, Surface::Height(_)),
        matches!(other_sample.surface, Surface::Height(_)),
    ];
    *terrain_history = other_sample.control_history;
    if accepted[0] {
        working.world = proposed_current;
    }
    if accepted[1] {
        other.world = proposed_other;
    }
    Ok(Separation {
        proposed_current,
        proposed_other,
        accepted,
    })
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContactVisit {
    pub other_slot: usize,
    pub angles: ContactAngles,
    pub separation: Separation,
    pub sound: Option<u16>,
}
/// Motion invokes this after projection/shadow and before committing working
/// state. A nonzero current cooldown decrements and skips the entire scan. A zero
/// cooldown scans other records in descending slot order, ignoring their cooldowns.
pub fn service_contacts(
    trucks: &mut [Truck; TRUCKS],
    current_slot: usize,
    working: &mut WorkingMotion,
    environment: &ContactEnvironment<'_>,
    terrain_history: &mut u8,
) -> Result<Vec<ContactVisit>> {
    ensure!(current_slot < TRUCKS, "invalid contact truck slot");
    if trucks[current_slot].contact_cooldown != 0 {
        trucks[current_slot].contact_cooldown -= 1;
        return Ok(Vec::new());
    }
    // Validate on an isolated proposal so a malformed file-backed terrain cannot
    // leave half of a pair applied. Later pairs still see earlier accepted peers.
    let mut next = trucks.clone();
    let mut proposal = *working;
    let mut history = *terrain_history;
    let committed = next[current_slot].clone();
    let mut visits = Vec::new();
    for other_slot in (0..TRUCKS).rev() {
        if other_slot == current_slot
            || coarse_distance(committed.world, next[other_slot].world) >= CONTACT_DISTANCE_LIMIT
        {
            continue;
        }
        let angles = contact_angles(&committed, &proposal, &next[other_slot], environment.tables);
        let sound = respond_motion(
            &mut proposal,
            &mut next[other_slot],
            angles.packed_sectors,
            environment.animation,
            environment.active_humans,
        );
        let separation = separate_pair(
            &committed,
            &mut proposal,
            &mut next[other_slot],
            angles.normal,
            environment,
            &mut history,
        )?;
        next[current_slot].contact_cooldown = CONTACT_COOLDOWN_TICKS;
        visits.push(ContactVisit {
            other_slot,
            angles,
            separation,
            sound,
        });
    }
    *trucks = next;
    *working = proposal;
    *terrain_history = history;
    Ok(visits)
}

fn set_low(word: &mut u16, low: u8) {
    *word = (*word & !u16::from(u8::MAX)) | u16::from(low);
}
fn turn(heading: &mut u16, delta: u8) {
    let low = (*heading as u8).wrapping_add(delta);
    set_low(heading, low);
}

#[cfg(test)]
mod tests {
    use super::super::bearings::BEARING_CELLS;
    use super::*;
    use crate::world::terrain::{CELL_BYTES, ROW_BYTES};
    use crate::world::tracks::TERRAIN_BYTES;
    const ORIGIN: [u16; 2] = [0x1080, 0x1080];
    const DIRECTION_ROM: std::ops::Range<usize> = 0x1cc7..0x1dc7;
    const SINE_ROM: std::ops::Range<usize> = 0x1dc7..0x1ec7;
    const PHASE_ROM: std::ops::Range<usize> = 0x1f83..0x1f87;
    const ROW_ROM: std::ops::Range<usize> = 0x1f87..0x1fdf;
    const SPEED_HIGH: u16 = 0xab00;
    const HEADING_HIGH: u16 = 0xcd00;
    fn rom() -> Vec<u8> {
        std::fs::read("../input/game.rom").unwrap()
    }
    // Fixture loading only; no response, bearing or separation oracle is here.
    fn sampling() -> SamplingTables {
        let b = rom();
        SamplingTables {
            sine: std::array::from_fn(|i| b[SINE_ROM.start + i] as i8),
            wheel_phases: b[PHASE_ROM].try_into().unwrap(),
            row_offsets: std::array::from_fn(|i| {
                u16::from_le_bytes(
                    b[ROW_ROM.start + i * 2..ROW_ROM.start + i * 2 + 2]
                        .try_into()
                        .unwrap(),
                )
            }),
        }
    }
    fn flat() -> Terrain {
        let mut cells = vec![40; TERRAIN_BYTES];
        for c in cells.chunks_exact_mut(CELL_BYTES) {
            c[2] = 0;
        }
        Terrain { cells }
    }
    fn truck() -> Truck {
        Truck {
            world: ORIGIN,
            ..Truck::default()
        }
    }
    fn working(speed: u8, heading: u8) -> WorkingMotion {
        WorkingMotion {
            world: [0x2000, 0x2000],
            speed: SPEED_HIGH | speed as u16,
            heading: HEADING_HIGH | u16::from(heading),
        }
    }
    fn environment<'a>(
        terrain: &'a Terrain,
        sampling: &'a SamplingTables,
        tables: &'a BearingTable,
    ) -> ContactEnvironment<'a> {
        ContactEnvironment {
            terrain,
            sampling,
            tables,
            original_track: 0,
            animation: 0,
            active_humans: 1,
        }
    }
    #[test]
    fn bearing_loader_preserves_all_rom_cells_and_checks_bounds() {
        let b = rom();
        let t = BearingTable::load(&b).unwrap();
        assert_eq!(t.cells.as_slice(), &b[DIRECTION_ROM.clone()]);
        assert_eq!(
            [t.cells[0], t.cells[0x11], t.cells[0xff]],
            [0xff, 0xe0, 0x5f]
        );
        assert!(BearingTable::load(&b[..DIRECTION_ROM.end - 1]).is_err());
    }
    #[test]
    fn coarse_distance_quantizes_each_absolute_axis_before_squaring() {
        assert_eq!(coarse_distance([0, 0], [63, 63]), 0);
        assert_eq!(coarse_distance([0, 0], [64, 128]), 5);
        assert_eq!(coarse_distance([0x1234, 0x2345], [0x11f4, 0x22c5]), 5);
        assert_eq!(coarse_distance([0xffff, 0], [0, 0]), 0);
    }
    #[test]
    fn axis_cutoff_is_inclusive_and_rejects_signed_minimum() {
        assert_eq!(coarse_distance([0, 0], [0x09ff, 0]), 1521);
        assert_eq!(coarse_distance([0, 0], [0x0a00, 0]), DISTANT_PAIR);
        assert_eq!(coarse_distance([0, 0], [0, 0x0a00]), DISTANT_PAIR);
        assert_eq!(coarse_distance([0, 0], [0x8000, 0]), DISTANT_PAIR);
    }
    #[test]
    fn relative_sector_boundaries_wrap_on_both_sides_of_zero() {
        for (a, s) in [
            (0, 0),
            (0x1f, 0),
            (0x20, 1),
            (0x5f, 1),
            (0x60, 2),
            (0x9f, 2),
            (0xa0, 3),
            (0xdf, 3),
            (0xe0, 0),
            (0xff, 0),
        ] {
            assert_eq!(heading_sector(a), s, "angle{a:02x}");
        }
    }
    #[test]
    fn contact_bearing_uses_committed_positions_but_working_current_heading() {
        let t = BearingTable {
            cells: rom()[DIRECTION_ROM.clone()].try_into().unwrap(),
        };
        let current = truck();
        let mut other = truck();
        other.world = [0x1070, 0x1090];
        other.heading = 0x20;
        // Difference (+16,-16)>>4 gives vector(+1,+1), cell$11=$E0.
        let a = contact_angles(&current, &working(0, 0x10), &other, &t);
        assert_eq!(
            a,
            ContactAngles {
                normal: 0xe0,
                relative_current: 0x30,
                relative_other: 0x40,
                packed_sectors: 0x11
            }
        );
    }
    #[test]
    fn bearing_signed_nibbles_and_joint_scaling_match_fixed_rom_cells() {
        let t = BearingTable {
            cells: rom()[DIRECTION_ROM.clone()].try_into().unwrap(),
        };
        let current = truck();
        for (world, normal) in [
            ([0x1090, 0x1070], 0x5f),
            ([0x0f80, 0x1080], 0xff),
            ([0x1180, 0x1080], 0x7f),
            ([0x1080, 0x1180], 0xbf),
            ([0x1080, 0x0f80], 0x40),
        ] {
            let other = Truck { world, ..truck() };
            assert_eq!(
                contact_angles(&current, &working(0, 0), &other, &t).normal,
                normal
            );
        }
    }
    #[test]
    fn all_sixteen_sector_pairs_follow_exact_dispatch_without_conditional_dead_entry() {
        // Initial current speed48/heading$10, other speed16/heading$70.
        // Fixed outcomes from the branch trace, ordered high(other),low(current).
        let expected = [
            (16, 48, 0x10, 0x80, true),
            (48, 16, 0x20, 0x70, false),
            (240, 208, 0x10, 0x70, true),
            (48, 16, 0x20, 0x70, false),
            (12, 16, 0x10, 0x78, false),
            (48, 16, 0x0c, 0x74, false),
            (12, 16, 0x10, 0x78, false),
            (48, 16, 0x14, 0x74, false),
            (12, 16, 0x10, 0x78, false),
            (48, 16, 0x20, 0x70, false),
            (16, 48, 0x10, 0x80, true),
            (48, 16, 0x20, 0x70, false),
            (12, 16, 0x10, 0x78, false),
            (48, 16, 0x0c, 0x6c, false),
            (12, 16, 0x10, 0x78, false),
            (48, 16, 0x14, 0x6c, false),
        ];
        for (i, (cs, os, ch, oh, sound)) in expected.into_iter().enumerate() {
            let packed = ((i / 4) as u8) * 16 + (i % 4) as u8;
            let mut w = working(48, 0x10);
            let mut o = Truck {
                speed: 0xef10,
                heading: 0x5670,
                ..truck()
            };
            assert_eq!(
                respond_motion(&mut w, &mut o, packed, 0, 1),
                sound.then_some(IMPACT_SOUND_COMMAND),
                "case{packed:02x}"
            );
            assert_eq!(
                (w.speed, o.speed, w.heading, o.heading),
                (SPEED_HIGH | cs, 0xef00 | os, HEADING_HIGH | ch, 0x5600 | oh),
                "case{packed:02x}"
            );
        }
    }
    #[test]
    fn reverse_case_exchanges_negated_speed_bytes_and_threshold_includes_32() {
        for (current, other, expected, sound) in [
            (31, 1, (255, 225), false),
            (32, 1, (255, 224), true),
            (0x80, 0, (0, 0x80), true),
            (0, 0, (0, 0), false),
        ] {
            let mut w = working(current, 0);
            let mut o = Truck {
                speed: 0xef00 | other as u16,
                ..truck()
            };
            assert_eq!(
                respond_motion(&mut w, &mut o, 0x02, 0, 1),
                sound.then_some(IMPACT_SOUND_COMMAND)
            );
            assert_eq!(
                (w.speed, o.speed),
                (SPEED_HIGH | expected.0 as u16, 0xef00 | expected.1 as u16)
            );
        }
    }
    #[test]
    fn parallel_exchange_heading_parity_and_wrapped_speed_difference() {
        for (animation, expected) in [(0, 0x08), (1, 0xe8)] {
            let mut w = working(0x80, 0);
            let mut o = Truck {
                speed: 0xef7f,
                heading: 0x56f8,
                ..truck()
            };
            // $7F-$80 wraps$FF, absolute byte magnitude1: quiet, despite large signed difference.
            assert_eq!(respond_motion(&mut w, &mut o, 0x22, animation, 1), None);
            assert_eq!(
                (w.speed, o.speed, o.heading),
                (0xab7f, 0xef80, 0x5600 | expected)
            );
        }
    }
    #[test]
    fn exchange_sound_threshold_is_absolute_byte_32_inclusive() {
        for (c, o, sound) in [(0, 31, false), (0, 32, true), (32, 0, true), (31, 0, false)] {
            let mut w = working(c, 0);
            let mut other = Truck {
                speed: o as u16,
                ..truck()
            };
            assert_eq!(
                respond_motion(&mut w, &mut other, 0, 0, 1),
                sound.then_some(IMPACT_SOUND_COMMAND)
            );
        }
    }
    #[test]
    fn odd_sector_deflections_wrap_low_heading_and_never_request_impact_sound() {
        for (case, current, other) in [
            (0x11, 0xfc, 4),
            (0x13, 4, 4),
            (0x31, 0xfc, 0xfc),
            (0x33, 4, 0xfc),
        ] {
            let mut w = working(0x7f, 0);
            let mut o = Truck {
                speed: 0xef7f,
                heading: 0x5600,
                ..truck()
            };
            assert_eq!(respond_motion(&mut w, &mut o, case, 1, 1), None);
            assert_eq!(
                (w.heading, o.heading),
                (HEADING_HIGH | current, 0x5600 | other)
            );
        }
    }
    #[test]
    fn asymmetric_even_current_quarters_only_positive_speed_and_uses_unsigned_sound_test() {
        for (speed, expected, sound) in [
            (0x7f, 31, false),
            (0x80, 0x80, true),
            (0xff, 0xff, true),
            (0, 0, false),
        ] {
            let mut w = working(speed, 0);
            let mut o = Truck {
                heading: 0x56fc,
                ..truck()
            };
            assert_eq!(
                respond_motion(&mut w, &mut o, 0x10, 0, 1),
                sound.then_some(IMPACT_SOUND_COMMAND)
            );
            assert_eq!((w.speed, o.heading), (SPEED_HIGH | expected, 0x5604));
        }
    }
    #[test]
    fn asymmetric_odd_current_turns16_and_tests_other_speed_unsigned() {
        for (speed, sound) in [(31, false), (32, true), (0xff, true)] {
            let mut w = working(0x60, 0xf8);
            let mut o = Truck {
                speed: speed as u16,
                ..truck()
            };
            assert_eq!(
                respond_motion(&mut w, &mut o, 0x01, 1, 1),
                sound.then_some(IMPACT_SOUND_COMMAND)
            );
            assert_eq!(
                (w.speed, w.heading, o.speed),
                (0xab60, 0xcd08, speed as u16)
            );
        }
    }
    #[test]
    fn impact_audio_is_gated_by_any_nonzero_human_mask_not_current_slot() {
        for (mask, expected) in [
            (0, None),
            (0x08, Some(IMPACT_SOUND_COMMAND)),
            (0x80, Some(IMPACT_SOUND_COMMAND)),
        ] {
            let mut w = working(0x60, 0);
            let mut o = Truck {
                speed: 0,
                ..truck()
            };
            assert_eq!(respond_motion(&mut w, &mut o, 0, 0, mask), expected);
        }
    }
    #[test]
    fn conditional_current_deflection_has_96_cutoff_wrapping_add_and_parity_turn() {
        // Conditional M8,binary,DB0/DP0 entry only; no invented natural caller.
        for (speed, animation, expected_speed, heading) in [
            (0x5f, 0, 0x7f, 4),
            (0x60, 1, 0x60, 0xf4),
            (0xff, 1, 0xff, 0xf4),
        ] {
            let mut w = working(speed, 0xfc);
            conditional_deflect_current(&mut w, animation);
            assert_eq!(
                (w.speed, w.heading),
                (SPEED_HIGH | expected_speed, HEADING_HIGH | heading)
            );
        }
    }
    #[test]
    fn separation_uses_committed_midpoint_and_signed_doubled_sine_for_all_octants() {
        let terrain = flat();
        let sine = sampling();
        let tables = BearingTable {
            cells: [0; BEARING_CELLS],
        };
        let env = environment(&terrain, &sine, &tables);
        for (normal, current, other) in [
            (0, [0x117e, 0x1080], [0x0f82, 0x1080]),
            (32, [0x1136, 0x1136], [0x0fca, 0x0fca]),
            (64, [0x1080, 0x117e], [0x1080, 0x0f82]),
            (96, [0x0fca, 0x1136], [0x1136, 0x0fca]),
            (128, [0x0f82, 0x1080], [0x117e, 0x1080]),
            (160, [0x0fca, 0x0fca], [0x1136, 0x1136]),
            (192, [0x1080, 0x0f82], [0x1080, 0x117e]),
            (224, [0x1136, 0x0fca], [0x0fca, 0x1136]),
        ] {
            let mut w = working(16, 0);
            let mut o = truck();
            let mut history = 0x80;
            let r = separate_pair(&truck(), &mut w, &mut o, normal, &env, &mut history).unwrap();
            assert_eq!(
                r,
                Separation {
                    proposed_current: current,
                    proposed_other: other,
                    accepted: [true, true]
                }
            );
            assert_eq!((w.world, o.world, history), (current, other, 0x20));
            assert_eq!((w.speed, w.heading), (0xab10, 0xcd00));
        }
    }
    #[test]
    fn separation_midpoint_addition_wraps_before_logical_halving() {
        let terrain = flat();
        let sine = sampling();
        let tables = BearingTable {
            cells: [0; BEARING_CELLS],
        };
        let env = environment(&terrain, &sine, &tables);
        // $F000+$F100 wraps$E100 then halves to$7080, not$F080.
        // Coordinate index byte wrapping still places both valid terrain views.
        let current = Truck {
            world: [0xf000, 0],
            ..truck()
        };
        let mut other = Truck {
            world: [0xf100, 0],
            ..truck()
        };
        let mut w = working(0, 0);
        let r = separate_pair(&current, &mut w, &mut other, 0, &env, &mut 0).unwrap();
        assert_eq!(
            (r.proposed_current, r.proposed_other),
            ([0x717e, 0], [0x6f82, 0])
        );
    }
    #[test]
    fn independently_blocked_separation_candidates_preserve_only_rejected_position() {
        let sine = sampling();
        let tables = BearingTable {
            cells: [0; BEARING_CELLS],
        };
        for (blocks, accepted) in [
            ([false, false], [true, true]),
            ([true, false], [false, true]),
            ([false, true], [true, false]),
            ([true, true], [false, false]),
        ] {
            let mut terrain = flat();
            if blocks[0] {
                terrain.cells[ROW_BYTES * 16 + CELL_BYTES * 17 + 2] = 0xaa;
            }
            if blocks[1] {
                terrain.cells[ROW_BYTES * 16 + CELL_BYTES * 15 + 2] = 0xaa;
            }
            let env = environment(&terrain, &sine, &tables);
            let mut w = working(0, 0);
            let old = w.world;
            let mut other = truck();
            let mut history = 0x80;
            let result =
                separate_pair(&truck(), &mut w, &mut other, 0, &env, &mut history).unwrap();
            assert_eq!(result.accepted, accepted);
            assert_eq!(w.world, if accepted[0] { [0x117e, 0x1080] } else { old });
            assert_eq!(
                other.world,
                if accepted[1] {
                    [0x0f82, 0x1080]
                } else {
                    ORIGIN
                }
            );
            // Every classification shifts history, even a carry-set blocked result.
            assert_eq!(history, 0x20);
        }
    }
    #[test]
    fn both_terrain_classifications_append_surface_markers_in_current_then_other_order() {
        let mut terrain = flat();
        let sine = sampling();
        let tables = BearingTable {
            cells: [0; BEARING_CELLS],
        };
        // Only the current candidate supplies marker1. Other marker0 then shifts it tobit6.
        terrain.cells[ROW_BYTES * 16 + CELL_BYTES * 17 + 2] = 0x55;
        let env = environment(&terrain, &sine, &tables);
        let mut w = working(0, 0);
        let mut other = truck();
        let mut history = 0;
        separate_pair(&truck(), &mut w, &mut other, 0, &env, &mut history).unwrap();
        assert_eq!(history, 0x40);
    }
    #[test]
    fn every_nonzero_cooldown_skips_scanning_and_decrements_once() {
        let terrain = flat();
        let sine = sampling();
        let tables = BearingTable {
            cells: [0; BEARING_CELLS],
        };
        let env = environment(&terrain, &sine, &tables);
        for cooldown in [1, 6, 255] {
            let mut trucks = std::array::from_fn(|_| truck());
            trucks[0].contact_cooldown = cooldown;
            let mut expected = trucks.clone();
            expected[0].contact_cooldown = cooldown - 1;
            let mut w = working(16, 0);
            let old = w;
            let mut history = 0x80;
            assert!(
                service_contacts(&mut trucks, 0, &mut w, &env, &mut history)
                    .unwrap()
                    .is_empty()
            );
            assert_eq!((trucks, w, history), (expected, old, 0x80));
        }
    }
    #[test]
    fn zero_cooldown_scan_visits_other_slots_descending_without_filtering_their_cooldowns() {
        let terrain = flat();
        let sine = sampling();
        let tables = BearingTable {
            cells: [0; BEARING_CELLS],
        };
        let env = environment(&terrain, &sine, &tables);
        let mut trucks = std::array::from_fn(|i| Truck {
            speed: (i * 10) as u16,
            contact_cooldown: if i == 0 { 0 } else { 255 },
            ..truck()
        });
        let mut w = working(1, 0);
        let mut history = 0x80;
        let visits = service_contacts(&mut trucks, 0, &mut w, &env, &mut history).unwrap();
        assert_eq!(
            visits.iter().map(|v| v.other_slot).collect::<Vec<_>>(),
            [3, 2, 1]
        );
        assert_eq!(trucks[0].contact_cooldown, CONTACT_COOLDOWN_TICKS);
        assert_eq!(
            [trucks[1].speed, trucks[2].speed, trucks[3].speed],
            [20, 30, 1]
        );
        assert_eq!(
            [
                trucks[1].contact_cooldown,
                trucks[2].contact_cooldown,
                trucks[3].contact_cooldown
            ],
            [255; 3]
        );
        assert_eq!(
            (trucks[0].world, trucks[0].speed, w.world, w.speed),
            (ORIGIN, 0, [0x117e, 0x1080], 0xab0a)
        );
        // Six classifications shift initialbit7 down tobit1.
        assert_eq!(history, 2);
    }
    #[test]
    fn contact_scan_rejects_exact_metric40_but_accepts37_and_leaves_far_records_unchanged() {
        let terrain = flat();
        let sine = sampling();
        let tables = BearingTable {
            cells: [0; BEARING_CELLS],
        };
        let env = environment(&terrain, &sine, &tables);
        for (offset_y, count) in [(0x80, 0), (0x40, 1)] {
            let mut trucks = std::array::from_fn(|_| Truck {
                world: [0x2000, 0x2000],
                ..truck()
            });
            trucks[0] = truck();
            trucks[1] = Truck {
                world: [ORIGIN[0] + 0x180, ORIGIN[1] + offset_y],
                ..truck()
            };
            let before = trucks.clone();
            let mut w = working(0, 0);
            let mut history = 0;
            let visits = service_contacts(&mut trucks, 0, &mut w, &env, &mut history).unwrap();
            assert_eq!(visits.len(), count);
            assert_eq!((&trucks[2], &trucks[3]), (&before[2], &before[3]));
            assert_eq!(trucks[0].contact_cooldown, if count == 0 { 0 } else { 6 });
            if count == 0 {
                assert_eq!(trucks, before);
            }
        }
    }
    #[test]
    fn current_slot_is_excluded_even_when_all_other_trucks_are_distant() {
        let terrain = flat();
        let sine = sampling();
        let tables = BearingTable {
            cells: [0; BEARING_CELLS],
        };
        let env = environment(&terrain, &sine, &tables);
        for slot in 0..TRUCKS {
            let mut trucks = std::array::from_fn(|i| Truck {
                world: [0x0800 + (i as u16) * 0x0800, 0x1000],
                ..truck()
            });
            let before = trucks.clone();
            let mut w = working(0, 0);
            assert!(
                service_contacts(&mut trucks, slot, &mut w, &env, &mut 0)
                    .unwrap()
                    .is_empty()
            );
            assert_eq!(trucks, before);
        }
    }
    #[test]
    fn checked_native_interfaces_reject_missing_terrain_and_invalid_slot() {
        let terrain = Terrain { cells: vec![] };
        let sine = sampling();
        let tables = BearingTable {
            cells: [0; BEARING_CELLS],
        };
        let env = environment(&terrain, &sine, &tables);
        let mut trucks = std::array::from_fn(|_| truck());
        let mut w = working(0, 0);
        assert!(service_contacts(&mut trucks, TRUCKS, &mut w, &env, &mut 0).is_err());
        assert!(separate_pair(&truck(), &mut w, &mut truck(), 0, &env, &mut 0).is_err());
    }
}
