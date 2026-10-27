//! Computer driver, route progress and ranking contracts. Independent derivations
//! and source-width/call-order evidence: ai.trace.md. Shared race progress owns all laps.
use super::motion::Truck;
use crate::{
    runtime::RandomState,
    world::tracks::{Route, TRUCKS},
};
use anyhow::{Context, Result, ensure};

pub const VECTOR_CELLS: usize = 256;
pub const AI_PARAMETERS: usize = 8;
pub const WAYPOINT_RESET_TICKS: u8 = 100;
pub const BIAS_DELAY_TICKS: u8 = 100;
pub const AI_ACCELERATION: u8 = 6;
pub const AI_THROTTLE_WORD: u16 = 0x8000;
pub const LAST_LAP: u8 = 4;
pub const WINNING_LAP: u8 = 5;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiTables {
    pub bearings: [u8; VECTOR_CELLS],
    pub turn_steps: [u8; TRUCKS],
    pub waypoint_timeouts: [u8; AI_PARAMETERS],
    pub parameter_thresholds: [u8; AI_PARAMETERS],
    pub rank_threshold_one: [u8; TRUCKS],
    pub rank_threshold_two: [u8; TRUCKS],
}
pub fn load_tables(rom: &[u8]) -> Result<AiTables> {
    const BEARING_ROM: usize = 0x1cc7;
    const TURN_ROM: usize = 0x989;
    const TIMEOUT_ROM: usize = 0xa64;
    const PARAMETER_ROM: usize = 0x2687;
    const RANK_ONE_ROM: usize = 0x268c;
    const RANK_TWO_ROM: usize = 0x2690;
    fn read<const N: usize>(rom: &[u8], at: usize) -> Result<[u8; N]> {
        Ok(rom
            .get(at..at + N)
            .context("truncated AI table")?
            .try_into()?)
    }
    Ok(AiTables {
        bearings: read(rom, BEARING_ROM)?,
        turn_steps: read(rom, TURN_ROM)?,
        waypoint_timeouts: read(rom, TIMEOUT_ROM)?,
        parameter_thresholds: read(rom, PARAMETER_ROM)?,
        rank_threshold_one: read(rom, RANK_ONE_ROM)?,
        rank_threshold_two: read(rom, RANK_TWO_ROM)?,
    })
}
/// Shared with contact::contact_angles: both unsigned magnitudes halve together;
/// output signs are four-bit two's-complement, never a floating-point angle.
pub fn quantize_vector(x: i8, y: i8) -> u8 {
    const MAGNITUDE_BOUND: u8 = 8;
    const NIBBLE_MASK: u8 = 0x0f;
    const Y_NIBBLE_SHIFT: u32 = 4;
    let (mut x_magnitude, mut y_magnitude) = (x.unsigned_abs(), y.unsigned_abs());
    while x_magnitude >= MAGNITUDE_BOUND || y_magnitude >= MAGNITUDE_BOUND {
        x_magnitude >>= 1;
        y_magnitude >>= 1;
    }
    let restore = |magnitude: u8, original: i8| {
        if original < 0 {
            magnitude.wrapping_neg() & NIBBLE_MASK
        } else {
            magnitude
        }
    };
    restore(x_magnitude, x) | (restore(y_magnitude, y) << Y_NIBBLE_SHIFT)
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Driver {
    pub checkpoint: u8,
    pub lap: u8,
    pub timeout: u8,
    pub bias_delay: u8,
    pub parameter: u8,
}
/// Race is the sole mutable authority for lap/checkpoint/banner/winner state.
pub use crate::race::Progress as RaceAi;
/// Setup's all-human scan writes one route-buffer tail byte outside all original
/// route windows. Surface that unused side effect without introducing a RAM map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SetupEffects {
    pub unused_route_tail_byte: Option<u8>,
}
pub fn setup_parameters(trucks: &mut [Truck; TRUCKS], race: &mut RaceAi) -> SetupEffects {
    const MINIMUM_INITIAL: u8 = 6;
    const RAISED_INCREMENT: u8 = 2;
    const PARAMETER_CAP: u8 = 7;
    const MINIMUM_SLOT: usize = 2;
    let mut maximum = 0;
    let mut minimum = MINIMUM_INITIAL;
    for (slot, truck) in trucks.iter_mut().enumerate() {
        if race.active_humans & (1 << slot) != 0 {
            maximum = maximum.max(truck.upgrades.speed);
            minimum = minimum.min(truck.upgrades.speed);
        } else {
            truck.upgrades.suspension = MINIMUM_INITIAL;
        }
    }
    for driver in &mut race.drivers {
        driver.parameter = maximum.saturating_sub(1);
    }
    race.drivers[MINIMUM_SLOT].parameter = minimum.saturating_sub(1);
    let raised = maximum.wrapping_add(RAISED_INCREMENT).min(PARAMETER_CAP);
    if let Some(slot) = (0..TRUCKS)
        .rev()
        .find(|slot| race.active_humans & (1 << slot) == 0)
    {
        race.drivers[slot].parameter = raised;
        SetupEffects {
            unused_route_tail_byte: None,
        }
    } else {
        SetupEffects {
            unused_route_tail_byte: Some(raised),
        }
    }
}
/// Uses current route record with arithmetic half-bias unless low extent is one.
/// Reduced Y is quantizer X and reduced X is quantizer Y, followed by +quarterturn.
pub fn desired_heading(truck: &Truck, record: [u16; 2], tables: &AiTables) -> u8 {
    const EXTENT_MASK: u16 = 0x1f;
    const CENTER_MASK: u16 = 0xbfe0;
    const BIAS_SCALE: i16 = 128;
    const VECTOR_SHIFT: u32 = 6;
    const QUARTER_TURN: u8 = 0x40;
    let bias = (i16::from(truck.ai_route_bias as i8) * BIAS_SCALE) as u16;
    let reduced: [i8; 2] = std::array::from_fn(|axis| {
        let target = if record[axis] & EXTENT_MASK == 1 {
            record[axis]
        } else {
            record[axis].wrapping_add(bias)
        } & CENTER_MASK;
        (target.wrapping_sub(truck.world[axis]) >> VECTOR_SHIFT) as u8 as i8
    });
    tables.bearings[usize::from(quantize_vector(reduced[1], reduced[0]))].wrapping_add(QUARTER_TURN)
}
pub fn steer_heading(truck: &mut Truck, driver: &mut Driver, desired: u8, step: u8) {
    const DEADBAND_BIAS: u8 = 4;
    const DEADBAND_WIDTH: u8 = 8;
    if driver.bias_delay != 0 {
        driver.bias_delay -= 1;
        if driver.bias_delay == 0 {
            truck.ai_route_bias = 0;
        }
    }
    if truck.steering_lock != 0 {
        truck.steering_lock -= 1;
        return;
    }
    let heading = truck.heading as u8;
    let difference = desired.wrapping_sub(heading);
    if difference.wrapping_add(DEADBAND_BIAS) < DEADBAND_WIDTH {
        return;
    }
    let new = if (difference as i8) < 0 {
        heading.wrapping_sub(step)
    } else {
        heading.wrapping_add(step)
    };
    truck.heading = (truck.heading & !u16::from(u8::MAX)) | u16::from(new);
}
pub fn tick_timeout(driver: &mut Driver, animation: u8, route_count: u8) {
    if animation & 1 == 0 {
        return;
    }
    driver.timeout = driver.timeout.wrapping_sub(1);
    if driver.timeout == 0 {
        driver.checkpoint = driver.checkpoint.wrapping_sub(1);
        if (driver.checkpoint as i8) < 0 {
            driver.checkpoint = route_count;
        }
        driver.timeout = WAYPOINT_RESET_TICKS;
    }
}
pub fn inside_waypoint(world: [u16; 2], record: [u16; 2]) -> bool {
    crate::race::inside_checkpoint(world, record)
}
pub fn ranking(
    trucks: &[Truck; TRUCKS],
    drivers: &[Driver; TRUCKS],
    route: &Route,
) -> Result<[usize; TRUCKS]> {
    const EXTENT_MASK: u16 = 0x1f;
    const RANK_CENTER_MASK: u16 = 0xffe0;
    for driver in drivers {
        ensure!(
            route
                .words
                .get(usize::from(driver.checkpoint) * 2)
                .is_some(),
            "ranking checkpoint beyond route"
        );
    }
    let distance = |slot: usize| -> Result<u16> {
        let at = usize::from(drivers[slot].checkpoint) * 2;
        let x = *route.words.get(at).context("missing ranking X")?;
        let axis = usize::from(x & EXTENT_MASK != 1);
        let packed = *route
            .words
            .get(at + axis)
            .context("missing ranking coordinate")?;
        Ok(
            ((packed & RANK_CENTER_MASK).wrapping_sub(trucks[slot].world[axis]) as i16)
                .unsigned_abs(),
        )
    };
    let mut order = [0; TRUCKS];
    let mut selected = [false; TRUCKS];
    for output in &mut order {
        let mut best: Option<usize> = None;
        for slot in (0..TRUCKS).rev() {
            if selected[slot] || (drivers[slot].lap.wrapping_add(1) as i8) < 0 {
                continue;
            }
            let better = if let Some(previous) = best {
                let progress = (
                    drivers[slot].lap.wrapping_add(1),
                    drivers[slot].checkpoint.wrapping_sub(1),
                );
                let old = (
                    drivers[previous].lap.wrapping_add(1),
                    drivers[previous].checkpoint.wrapping_sub(1),
                );
                progress > old || (progress == old && distance(slot)? <= distance(previous)?)
            } else {
                true
            };
            if better {
                best = Some(slot);
            }
        }
        let slot = best.context("no supported ranking candidate")?;
        *output = slot;
        selected[slot] = true;
    }
    Ok(order)
}
pub fn human_relative_rank(slot: usize, order: [usize; TRUCKS], active_humans: u8) -> Result<u8> {
    ensure!(slot < TRUCKS, "invalid ranked truck");
    let mut seen = [false; TRUCKS];
    for &candidate in &order {
        ensure!(
            candidate < TRUCKS && !seen[candidate],
            "invalid rank permutation"
        );
        seen[candidate] = true;
    }
    let mut humans = 0;
    for candidate in order {
        if candidate == slot {
            return Ok(humans);
        }
        if active_humans & (1 << candidate) != 0 {
            humans += 1;
        }
    }
    anyhow::bail!("truck missing from rank order")
}
/// One supplied rank is the ranker's output, not an alternate ranking oracle.
/// RandomState records the real shared source RNG call sequence.
pub fn adjust_rank(
    truck: &mut Truck,
    driver: &Driver,
    human_rank: u8,
    tables: &AiTables,
    random: &mut RandomState,
) {
    use crate::runtime::RandomResultWidth;
    const SECOND_PLACE_BIAS: u8 = 0x38;
    let rank = usize::from(human_rank);
    let draw = random.next(RandomResultWidth::Byte) as u8;
    if draw < tables.rank_threshold_one[rank] {
        if draw >= tables.rank_threshold_two[rank] {
            truck.upgrades.speed = truck.upgrades.speed.saturating_sub(1);
        }
        return;
    }
    match truck.upgrades.speed.cmp(&driver.parameter) {
        std::cmp::Ordering::Less => truck.upgrades.speed = truck.upgrades.speed.wrapping_add(1),
        std::cmp::Ordering::Greater => {
            truck.upgrades.speed = truck.upgrades.speed.saturating_sub(1)
        }
        std::cmp::Ordering::Equal => {
            if rank == 0 && driver.lap != LAST_LAP {
                return;
            }
            let index = if rank == 0 {
                driver.lap
            } else {
                truck.upgrades.speed
            };
            let mut threshold = tables.parameter_thresholds[usize::from(index)];
            if rank == 1 {
                threshold = threshold.wrapping_add(SECOND_PLACE_BIAS);
            }
            if (random.next(RandomResultWidth::Byte) as u8) < threshold {
                truck.controller.0 = u16::MAX;
            }
        }
    }
}
/// Advances the already-accepted checkpoint, including bias RNG before FD RNG.
pub fn next_target(
    truck: &mut Truck,
    driver: &mut Driver,
    route: &Route,
    random: &mut RandomState,
) -> Result<()> {
    use crate::runtime::RandomResultWidth;
    const EXTENT_MASK: u8 = 0x1f;
    const BIAS_MINIMUM_EXTENT: u8 = 4;
    const BIAS_RANDOM_MASK: u8 = 7;
    const BIAS_CENTER: u8 = 4;
    const CONDITIONAL_REDIRECT: u8 = 0xfd;
    const UNCONDITIONAL_REDIRECT: u8 = 0xfe;
    const REDIRECT_RANDOM_MASK: u8 = 3;
    let checkpoint = driver.checkpoint.wrapping_add(1);
    // ASL twice runs at byte width in this particular source consumer.
    let at = usize::from(checkpoint.wrapping_mul(4)) / 2;
    let first = *route.words.get(at).context("next target X beyond route")?;
    let second = *route
        .words
        .get(at + 1)
        .context("next target Y beyond route")?;
    truck.ai_route_bias = 0;
    driver.checkpoint = checkpoint;
    if ((first as u8 | second as u8) & EXTENT_MASK) >= BIAS_MINIMUM_EXTENT {
        truck.ai_route_bias = ((random.next(RandomResultWidth::Byte) as u8) & BIAS_RANDOM_MASK)
            .wrapping_sub(BIAS_CENTER);
        driver.bias_delay = BIAS_DELAY_TICKS;
    }
    match (first >> u8::BITS) as u8 {
        UNCONDITIONAL_REDIRECT => driver.checkpoint = second as u8,
        CONDITIONAL_REDIRECT => {
            if random.next(RandomResultWidth::Byte) as u8 & REDIRECT_RANDOM_MASK == 0 {
                driver.checkpoint = second as u8;
            } else {
                driver.checkpoint = driver.checkpoint.wrapping_add(1);
            }
        }
        _ => {}
    }
    Ok(())
}
/// Cached record is the pre-timeout record. Timeout rewinds must not cause the
/// already-selected rectangle to be reread from a new index during this call.
pub fn progress_waypoint(
    trucks: &mut [Truck; TRUCKS],
    race: &mut RaceAi,
    slot: usize,
    cached_record: [u16; 2],
    route: &Route,
    tables: &AiTables,
    random: &mut RandomState,
) -> Result<bool> {
    ensure!(slot < TRUCKS, "invalid computer slot");
    tick_timeout(
        &mut race.drivers[slot],
        race.animation_frame,
        route.checkpoint_count,
    );
    if !inside_waypoint(trucks[slot].world, cached_record) {
        return Ok(false);
    }
    let order = ranking(trucks, &race.drivers, route)?;
    let human_rank = human_relative_rank(slot, order, race.active_humans)?;
    adjust_rank(
        &mut trucks[slot],
        &race.drivers[slot],
        human_rank,
        tables,
        random,
    );
    race.drivers[slot].timeout = *tables
        .waypoint_timeouts
        .get(usize::from(race.drivers[slot].parameter))
        .context("invalid AI parameter")?;
    if race.drivers[slot].checkpoint == 0 {
        race.drivers[slot].lap = race.drivers[slot].lap.wrapping_add(1);
        crate::race::request_last_lap(race, slot)?;
        if race.drivers[slot].lap == WINNING_LAP {
            race.winner_plus_one = slot as u8 + 1;
        }
    } else if race.drivers[slot].checkpoint == route.checkpoint_count {
        race.drivers[slot].checkpoint = 0;
        return Ok(true);
    }
    next_target(&mut trucks[slot], &mut race.drivers[slot], route, random)?;
    Ok(true)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiPhase {
    SetAcceleration(usize),
    SetThrottle(usize),
    Steer(usize),
    Waypoint(usize),
}
/// Natural after-HUD service visits computer slots 3..0 and never replaces human
/// control words or reruns movement in the current frame.
pub fn service_computers(
    trucks: &mut [Truck; TRUCKS],
    race: &mut RaceAi,
    route: &Route,
    tables: &AiTables,
    random: &mut RandomState,
) -> Result<Vec<AiPhase>> {
    let mut phases = Vec::new();
    for slot in (0..TRUCKS).rev() {
        if race.active_humans & (1 << slot) != 0 {
            continue;
        }
        let at = usize::from(race.drivers[slot].checkpoint) * 2;
        let record = [
            *route.words.get(at).context("AI target X beyond route")?,
            *route
                .words
                .get(at + 1)
                .context("AI target Y beyond route")?,
        ];
        phases.push(AiPhase::SetAcceleration(slot));
        trucks[slot].upgrades.acceleration = AI_ACCELERATION;
        phases.push(AiPhase::SetThrottle(slot));
        trucks[slot].controller.0 = AI_THROTTLE_WORD;
        let desired = desired_heading(&trucks[slot], record, tables);
        phases.push(AiPhase::Steer(slot));
        steer_heading(
            &mut trucks[slot],
            &mut race.drivers[slot],
            desired,
            tables.turn_steps[slot],
        );
        phases.push(AiPhase::Waypoint(slot));
        progress_waypoint(trucks, race, slot, record, route, tables, random)?;
    }
    Ok(phases)
}

#[cfg(test)]
mod tests {
    use super::*;
    const BEARINGS: std::ops::Range<usize> = 0x1cc7..0x1dc7;
    const TURN_STEPS: std::ops::Range<usize> = 0x989..0x98d;
    const TIMEOUTS: std::ops::Range<usize> = 0xa64..0xa6c;
    const PARAM_THRESHOLDS: std::ops::Range<usize> = 0x2687..0x268f;
    const RANK_ONE: std::ops::Range<usize> = 0x268c..0x2690;
    const RANK_TWO: std::ops::Range<usize> = 0x2690..0x2694;
    const ROM_TABLE_END: usize = 0x2694;
    const ORIGIN: [u16; 2] = [0x2000, 0x2000];
    const HEADING_HIGH: u16 = 0xab00;
    const PRESERVED_CONTROLLER: u16 = 0x1234;
    fn rom() -> Vec<u8> {
        std::fs::read("../input/game.rom").unwrap()
    }
    // Fixture loading only; no quantizer/rank/AI oracle is implemented here.
    fn tables() -> AiTables {
        let b = rom();
        AiTables {
            bearings: b[BEARINGS].try_into().unwrap(),
            turn_steps: b[TURN_STEPS].try_into().unwrap(),
            waypoint_timeouts: b[TIMEOUTS].try_into().unwrap(),
            parameter_thresholds: b[PARAM_THRESHOLDS].try_into().unwrap(),
            rank_threshold_one: b[RANK_ONE].try_into().unwrap(),
            rank_threshold_two: b[RANK_TWO].try_into().unwrap(),
        }
    }
    fn trucks() -> [Truck; TRUCKS] {
        std::array::from_fn(|_| Truck {
            world: ORIGIN,
            controller: crate::controls::ControllerWord(PRESERVED_CONTROLLER),
            ..Truck::default()
        })
    }
    fn race() -> RaceAi {
        RaceAi {
            drivers: [Driver {
                lap: 1,
                timeout: 200,
                ..Driver::default()
            }; TRUCKS],
            active_humans: 1,
            animation_frame: 0,
            winner_plus_one: 0,
            banner_state: 0,
        }
    }
    fn route() -> Route {
        Route {
            words: vec![0x2002, 0x2002, 0x3002, 0x3002, 0x4002, 0x4002, 0xff02],
            checkpoint_count: 2,
        }
    }
    // Runtime trace gives first DF then6A, retained state after first draw fixed here.
    fn random() -> RandomState {
        RandomState([0x12, 0x34, 0x56, 0x78, 0x9b])
    }
    fn after_one() -> RandomState {
        RandomState([0xdf, 0x12, 0x34, 0x56, 0x78])
    }
    // A zero-s3 input with s4 bit0 zero and only s1/s2 nibble data selects first byte.
    fn draw_zero() -> RandomState {
        RandomState([0x12, 0, 0, 0, 0])
    }
    #[test]
    fn rom_tables_preserve_overlapping_threshold_windows_and_exact_timeout_extent() {
        let b = rom();
        let t = load_tables(&b).unwrap();
        assert_eq!(t.bearings.as_slice(), &b[BEARINGS]);
        assert_eq!(t.turn_steps, [3, 4, 4, 4]);
        assert_eq!(
            t.waypoint_timeouts,
            [255, 255, 255, 200, 200, 200, 150, 150]
        );
        assert_eq!(t.parameter_thresholds, [24, 48, 56, 24, 80, 128, 128, 128]);
        assert_eq!(t.rank_threshold_one, [128; 4]);
        assert_eq!(t.rank_threshold_two, [128; 4]);
        assert!(load_tables(&b[..ROM_TABLE_END - 1]).is_err());
    }
    #[test]
    fn quantizer_joint_halving_handles_signed_extremes_without_negative_eight_cells() {
        let cases = [
            ((0, 0), 0x00),
            ((7, 7), 0x77),
            ((8, 7), 0x34),
            ((7, 8), 0x43),
            ((127, 1), 0x07),
            ((-128, 0), 0x0c),
            ((0, -128), 0xc0),
            ((-128, 127), 0x3c),
            ((127, -128), 0xc3),
            ((-7, -7), 0x99),
            ((-1, 1), 0x1f),
            ((1, -1), 0xf1),
        ];
        for ((x, y), expected) in cases {
            assert_eq!(quantize_vector(x, y), expected, "{x},{y}");
        }
    }
    #[test]
    fn quantizer_preserves_all_small_signed_quadrants_and_zero_axes() {
        // Literal complete -1..1 grid: no sign/magnitude behavior is an oracle.
        for ((x, y), v) in [
            ((-1, -1), 0xff),
            ((0, -1), 0xf0),
            ((1, -1), 0xf1),
            ((-1, 0), 0x0f),
            ((0, 0), 0),
            ((1, 0), 1),
            ((-1, 1), 0x1f),
            ((0, 1), 0x10),
            ((1, 1), 0x11),
        ] {
            assert_eq!(quantize_vector(x, y), v);
        }
    }
    #[test]
    fn setup_uses_human_max_min_and_last_computer_override_not_fixed_slot_three() {
        let mut t = trucks();
        let mut r = race();
        r.active_humans = 0b1001;
        t[0].upgrades.speed = 2;
        t[3].upgrades.speed = 6;
        t[0].upgrades.suspension = 1;
        t[3].upgrades.suspension = 2;
        let effect = setup_parameters(&mut t, &mut r);
        assert_eq!(r.drivers.map(|d| d.parameter), [5, 5, 7, 5]);
        assert_eq!(t.map(|t| t.upgrades.suspension), [1, 6, 6, 2]);
        assert_eq!(effect.unused_route_tail_byte, None);
    }
    #[test]
    fn setup_zero_human_upgrade_and_attract_no_human_have_distinct_minimum_defaults() {
        let mut t = trucks();
        let mut r = race();
        setup_parameters(&mut t, &mut r);
        assert_eq!(r.drivers.map(|d| d.parameter), [0, 0, 0, 2]);
        r.active_humans = 0;
        setup_parameters(&mut t, &mut r);
        assert_eq!(r.drivers.map(|d| d.parameter), [0, 0, 5, 2]);
    }
    #[test]
    fn all_human_setup_preserves_parameters_and_reports_unused_route_tail_write() {
        let mut t = trucks();
        let mut r = race();
        r.active_humans = 0x0f;
        for (truck, speed) in t.iter_mut().zip([0, 2, 4, 6]) {
            truck.upgrades.speed = speed;
            truck.upgrades.suspension = 1;
        }
        assert_eq!(
            setup_parameters(&mut t, &mut r).unused_route_tail_byte,
            Some(7)
        );
        assert_eq!(r.drivers.map(|d| d.parameter), [5, 5, 0, 5]);
        assert!(t.iter().all(|t| t.upgrades.suspension == 1));
    }
    #[test]
    fn desired_heading_swaps_reduced_axes_and_adds_quarter_turn_with_wrapping() {
        let t = trucks();
        let table = tables();
        // dx=+64,dy=0 -> vector(0,+1)->lookup10=BF, +40=FF.
        assert_eq!(desired_heading(&t[0], [0x2041, 0x2001], &table), 0xff);
        // dx=0,dy=+64 -> vector(+1,0)->lookup01=FF, +40=3F.
        assert_eq!(desired_heading(&t[0], [0x2001, 0x2041], &table), 0x3f);
        assert_eq!(desired_heading(&t[0], [0x1fc1, 0x2001], &table), 0x80);
        assert_eq!(desired_heading(&t[0], [0x2001, 0x1fc1], &table), 0xbf);
    }
    #[test]
    fn route_bias_is_signed_half_world_cell_and_extent_one_suppresses_each_axis() {
        let mut t = trucks();
        let table = tables();
        t[0].ai_route_bias = 0xfc;
        // -4 gives signed-16 -512; both components -8 quantize(-4,-4)=CC.
        assert_eq!(desired_heading(&t[0], [0x2002, 0x2002], &table), 0x9f);
        assert_eq!(desired_heading(&t[0], [0x2001, 0x2001], &table), 0x3f);
    }
    #[test]
    fn heading_deadband_is_negative_four_through_positive_three_and_high_byte_survives() {
        for (desired, expected) in [
            (0x7c, 0x80),
            (0x83, 0x80),
            (0x7b, 0x7d),
            (0x84, 0x83),
            (0, 0x7d),
        ] {
            let mut t = Truck {
                heading: HEADING_HIGH | 0x80,
                ..Truck::default()
            };
            let mut d = Driver::default();
            steer_heading(&mut t, &mut d, desired, 3);
            assert_eq!(t.heading, HEADING_HIGH | expected);
        }
    }
    #[test]
    fn heading_delay_clears_bias_on_expiry_before_lock_decrement_and_lock_blocks_turn() {
        let mut t = Truck {
            heading: HEADING_HIGH | 0xfe,
            ai_route_bias: 3,
            steering_lock: 1,
            ..Truck::default()
        };
        let mut d = Driver {
            bias_delay: 1,
            ..Driver::default()
        };
        steer_heading(&mut t, &mut d, 0x20, 4);
        assert_eq!(
            (t.heading, t.ai_route_bias, t.steering_lock, d.bias_delay),
            (HEADING_HIGH | 0xfe, 0, 0, 0)
        );
        steer_heading(&mut t, &mut d, 0x20, 4);
        assert_eq!(t.heading, HEADING_HIGH | 2);
    }
    #[test]
    fn nonexpired_bias_delay_decrements_without_preventing_turn_or_clearing_bias() {
        let mut t = Truck {
            heading: 0,
            ai_route_bias: 0xff,
            ..Truck::default()
        };
        let mut d = Driver {
            bias_delay: 2,
            ..Driver::default()
        };
        steer_heading(&mut t, &mut d, 0x40, 3);
        assert_eq!((t.heading, t.ai_route_bias, d.bias_delay), (3, 0xff, 1));
    }
    #[test]
    fn timeout_ticks_only_odd_animation_frames_wraps_zero_and_rewinds_at_one() {
        let mut d = Driver {
            checkpoint: 2,
            timeout: 1,
            ..Driver::default()
        };
        tick_timeout(&mut d, 0, 4);
        assert_eq!((d.checkpoint, d.timeout), (2, 1));
        tick_timeout(&mut d, 1, 4);
        assert_eq!((d.checkpoint, d.timeout), (1, 100));
        d.checkpoint = 0;
        d.timeout = 1;
        tick_timeout(&mut d, 255, 4);
        assert_eq!((d.checkpoint, d.timeout), (4, 100));
        d.timeout = 0;
        tick_timeout(&mut d, 3, 4);
        assert_eq!(d.timeout, 255);
    }
    #[test]
    fn waypoint_rectangle_lower_edge_excluded_upper_included_zero_extent_empty() {
        let rec = [0x2002, 0x3002];
        for (p, v) in [
            ([0x1f00, 0x2f00], false),
            ([0x20ff, 0x30ff], true),
            ([0x2100, 0x3000], true),
            ([0x2000, 0x3100], true),
            ([0x1eff, 0x3000], false),
        ] {
            assert_eq!(inside_waypoint(p, rec), v);
        }
        assert!(!inside_waypoint([0x2000, 0x3000], [0x2000, 0x3000]));
        assert!(inside_waypoint([0xfff0, 0], [0x0001, 0x0001]));
    }
    #[test]
    fn ai_rectangles_clear_bit_fourteen_where_ranking_distance_keeps_it() {
        assert!(inside_waypoint([0, 0], [0x4001, 0x4001]));
    }
    #[test]
    fn ranking_orders_laps_then_checkpoint_minus_one_in_unsigned_byte_order() {
        let t = trucks();
        let mut d = [Driver {
            lap: 1,
            checkpoint: 2,
            ..Driver::default()
        }; TRUCKS];
        d[1].lap = 2;
        d[2].checkpoint = 0;
        d[3].checkpoint = 3;
        assert_eq!(ranking(&t, &d, &route()).unwrap(), [1, 2, 3, 0]);
    }
    #[test]
    fn ranking_x_extent_one_ties_choose_nearer_then_lower_slot_on_equal_distance() {
        let mut t = trucks();
        let d = [Driver {
            lap: 1,
            checkpoint: 0,
            ..Driver::default()
        }; TRUCKS];
        let r = Route {
            words: vec![0x2001, 0x3002, 0xff00],
            checkpoint_count: 0,
        };
        for (truck, x) in t.iter_mut().zip([0x2100, 0x1f80, 0x2000, 0x2200]) {
            truck.world = [x, 0];
        }
        assert_eq!(ranking(&t, &d, &r).unwrap(), [2, 1, 0, 3]);
        t[1].world[0] = 0x2000;
        assert_eq!(ranking(&t, &d, &r).unwrap(), [1, 2, 0, 3]);
    }
    #[test]
    fn ranking_y_ties_use_signed_wrapping_word_absolute_and_bit_fourteen_center() {
        let mut t = trucks();
        let d = [Driver {
            lap: 1,
            checkpoint: 0,
            ..Driver::default()
        }; TRUCKS];
        let r = Route {
            words: vec![0x2002, 0x7002, 0xff00],
            checkpoint_count: 0,
        };
        for (truck, y) in t.iter_mut().zip([0x7100, 0x6f80, 0x7000, 0x7200]) {
            truck.world = [0, y];
        }
        assert_eq!(ranking(&t, &d, &r).unwrap(), [2, 1, 0, 3]);
        // 8000 is its own two's complement and compares as unsigned distance32768.
        t[3].world[1] = 0xf000;
        assert_eq!(ranking(&t, &d, &r).unwrap(), [2, 1, 0, 3]);
    }
    #[test]
    fn ranking_rejects_missing_route_records_before_mutating_any_domain_state() {
        let t = trucks();
        let mut d = [Driver::default(); TRUCKS];
        d[0].checkpoint = 9;
        assert!(ranking(&t, &d, &route()).is_err());
    }
    #[test]
    fn human_relative_rank_counts_preceding_humans_only_with_slot_masks() {
        for (slot, v) in [(3, 0), (1, 0), (2, 1), (0, 2)] {
            assert_eq!(human_relative_rank(slot, [3, 1, 2, 0], 0b0110).unwrap(), v);
        }
        assert!(human_relative_rank(4, [3, 1, 2, 0], 1).is_err());
    }
    #[test]
    fn rank_adjustment_original_equal_threshold_windows_make_low_half_noop() {
        let mut t = trucks();
        t[0].upgrades.speed = 4;
        let d = Driver {
            parameter: 2,
            ..Driver::default()
        };
        let mut rng = draw_zero();
        adjust_rank(&mut t[0], &d, 2, &tables(), &mut rng);
        assert_eq!(t[0].upgrades.speed, 4);
        assert_eq!(rng, RandomState([0, 0x12, 0, 0, 0]));
    }
    #[test]
    fn rank_adjustment_raises_or_lowers_toward_parameter_with_one_draw() {
        for (speed, expected) in [(1, 2), (5, 4)] {
            let mut t = trucks();
            t[0].upgrades.speed = speed;
            let d = Driver {
                parameter: 3,
                ..Driver::default()
            };
            let mut rng = random();
            adjust_rank(&mut t[0], &d, 1, &tables(), &mut rng);
            assert_eq!(t[0].upgrades.speed, expected);
            assert_eq!(rng, after_one());
            assert_eq!(t[0].controller.0, PRESERVED_CONTROLLER);
        }
    }
    #[test]
    fn conditional_second_threshold_decreases_but_zero_speed_saturates() {
        let mut table = tables();
        table.rank_threshold_one[2] = 240;
        table.rank_threshold_two[2] = 128;
        for (speed, expected) in [(3, 2), (0, 0)] {
            let mut t = trucks();
            t[0].upgrades.speed = speed;
            let mut rng = random();
            adjust_rank(&mut t[0], &Driver::default(), 2, &table, &mut rng);
            assert_eq!(t[0].upgrades.speed, expected);
            assert_eq!(rng, after_one());
        }
    }
    #[test]
    fn equal_speed_leader_not_lap_four_skips_second_random_draw() {
        let mut t = trucks();
        t[0].upgrades.speed = 3;
        let mut rng = random();
        adjust_rank(
            &mut t[0],
            &Driver {
                parameter: 3,
                lap: 3,
                ..Driver::default()
            },
            0,
            &tables(),
            &mut rng,
        );
        assert_eq!(rng, after_one());
        assert_eq!(t[0].controller.0, PRESERVED_CONTROLLER);
    }
    #[test]
    fn equal_speed_rank_one_adds_fifty_six_to_parameter_threshold_and_overrides_word() {
        let mut t = trucks();
        t[0].upgrades.speed = 4;
        let mut rng = random();
        // DF selects equal branch, next6A<50+38=88, so both bytes becomeFF.
        adjust_rank(
            &mut t[0],
            &Driver {
                parameter: 4,
                lap: 2,
                ..Driver::default()
            },
            1,
            &tables(),
            &mut rng,
        );
        assert_eq!(t[0].controller.0, 0xffff);
        assert_eq!(rng, RandomState([0x6a, 0xdf, 0x12, 0x34, 0x56]));
    }
    #[test]
    fn equal_speed_leader_lap_four_indexes_threshold_by_lap_not_parameter() {
        let mut t = trucks();
        t[0].upgrades.speed = 1;
        let mut rng = random();
        // Original threshold50 rejects6A; a distinct synthetic table value70
        // makes the source lap-index choice observable versus parameter1=30.
        let mut table = tables();
        table.parameter_thresholds[4] = 0x70;
        adjust_rank(
            &mut t[0],
            &Driver {
                parameter: 1,
                lap: 4,
                ..Driver::default()
            },
            0,
            &table,
            &mut rng,
        );
        assert_eq!(t[0].controller.0, 0xffff);
    }
    #[test]
    fn next_target_width_less_four_clears_bias_without_rng_or_delay_reset() {
        let mut t = trucks();
        t[0].ai_route_bias = 3;
        let mut d = Driver {
            checkpoint: 0,
            bias_delay: 7,
            ..Driver::default()
        };
        let mut rng = random();
        next_target(&mut t[0], &mut d, &route(), &mut rng).unwrap();
        assert_eq!((d.checkpoint, t[0].ai_route_bias, d.bias_delay), (1, 0, 7));
        assert_eq!(rng, random());
    }
    #[test]
    fn next_target_wide_extent_draws_signed_bias_and_sets_hundred_tick_delay() {
        let mut t = trucks();
        let mut d = Driver::default();
        let mut r = route();
        r.words[2] = 0x3004;
        let mut rng = random();
        next_target(&mut t[0], &mut d, &r, &mut rng).unwrap();
        assert_eq!(
            (d.checkpoint, t[0].ai_route_bias, d.bias_delay),
            (1, 3, 100)
        );
        assert_eq!(rng, after_one());
    }
    #[test]
    fn route_fe_unconditionally_redirects_fd_zero_redirects_other_outcomes_skip() {
        for (control, seed, expected) in [
            (0xfe, random(), 0),
            (0xfd, draw_zero(), 0),
            (0xfd, random(), 2),
        ] {
            let mut t = trucks();
            let mut d = Driver::default();
            let r = Route {
                words: vec![
                    0x2002,
                    0x2002,
                    u16::from_le_bytes([0, control]),
                    0,
                    0x3002,
                    0x3002,
                    0xff02,
                ],
                checkpoint_count: 2,
            };
            let mut rng = seed;
            next_target(&mut t[0], &mut d, &r, &mut rng).unwrap();
            assert_eq!(d.checkpoint, expected);
            if control == 0xfe {
                assert_eq!(rng, seed);
            } else {
                assert_ne!(rng, seed);
            }
        }
    }
    #[test]
    fn wide_fd_draws_bias_before_redirect_lottery_and_preserves_rng_call_count() {
        let mut t = trucks();
        let mut d = Driver::default();
        let r = Route {
            words: vec![0x2002, 0x2002, 0xfd04, 0, 0x3002, 0x3002, 0xff02],
            checkpoint_count: 2,
        };
        let mut rng = random();
        next_target(&mut t[0], &mut d, &r, &mut rng).unwrap();
        assert_eq!(
            (d.checkpoint, t[0].ai_route_bias, d.bias_delay),
            (2, 3, 100)
        );
        assert_eq!(rng, RandomState([0x6a, 0xdf, 0x12, 0x34, 0x56]));
    }
    #[test]
    fn waypoint_rejection_keeps_progress_but_timeout_rewind_uses_cached_record() {
        let mut t = trucks();
        let mut r = race();
        r.drivers[1].checkpoint = 1;
        r.drivers[1].timeout = 1;
        r.animation_frame = 1;
        let mut rng = random();
        assert!(
            !progress_waypoint(
                &mut t,
                &mut r,
                1,
                [0x7002, 0x7002],
                &route(),
                &tables(),
                &mut rng
            )
            .unwrap()
        );
        assert_eq!((r.drivers[1].checkpoint, r.drivers[1].timeout), (0, 100));
        assert_eq!(rng, random());
    }
    #[test]
    fn accepted_cached_record_after_timeout_rewind_increments_lap_and_reloads_parameter_timeout() {
        let mut t = trucks();
        let mut r = race();
        r.drivers[1] = Driver {
            checkpoint: 1,
            lap: 2,
            timeout: 1,
            parameter: 6,
            ..Driver::default()
        };
        r.animation_frame = 1;
        let mut rng = draw_zero();
        assert!(
            progress_waypoint(
                &mut t,
                &mut r,
                1,
                [0x2002, 0x2002],
                &route(),
                &tables(),
                &mut rng
            )
            .unwrap()
        );
        assert_eq!(
            (
                r.drivers[1].lap,
                r.drivers[1].checkpoint,
                r.drivers[1].timeout
            ),
            (3, 1, 150)
        );
    }
    #[test]
    fn fifth_lap_records_slot_plus_one_and_banner_before_advancing_checkpoint() {
        let mut t = trucks();
        let mut r = race();
        r.drivers[2] = Driver {
            lap: 4,
            parameter: 0,
            ..Driver::default()
        };
        let mut rng = draw_zero();
        progress_waypoint(
            &mut t,
            &mut r,
            2,
            [0x2002, 0x2002],
            &route(),
            &tables(),
            &mut rng,
        )
        .unwrap();
        assert_eq!(
            (
                r.drivers[2].lap,
                r.drivers[2].checkpoint,
                r.winner_plus_one,
                r.banner_state
            ),
            (5, 1, 3, 30)
        );
    }
    #[test]
    fn terminal_count_restarts_without_lap_and_existing_banner_is_not_restarted() {
        let mut t = trucks();
        let mut r = race();
        r.drivers[1].checkpoint = 2;
        r.drivers[1].lap = 4;
        r.banner_state = 19;
        let mut rng = draw_zero();
        progress_waypoint(
            &mut t,
            &mut r,
            1,
            [0x2002, 0x2002],
            &route(),
            &tables(),
            &mut rng,
        )
        .unwrap();
        assert_eq!(
            (
                r.drivers[1].checkpoint,
                r.drivers[1].lap,
                r.banner_state,
                r.winner_plus_one
            ),
            (0, 4, 19, 0)
        );
    }
    #[test]
    fn full_driver_service_descends_ai_slots_sets_acceleration_and_preserves_human_state() {
        let mut t = trucks();
        let mut r = race();
        r.active_humans = 0b0101;
        let preserved = [t[0].clone(), t[2].clone()];
        let mut rng = random();
        // Route is distant, so no waypoint RNG/rank adjustment; only target steering.
        let far = Route {
            words: vec![0x7002, 0x7002, 0xff00],
            checkpoint_count: 0,
        };
        let phases = service_computers(&mut t, &mut r, &far, &tables(), &mut rng).unwrap();
        assert_eq!(
            phases,
            vec![
                AiPhase::SetAcceleration(3),
                AiPhase::SetThrottle(3),
                AiPhase::Steer(3),
                AiPhase::Waypoint(3),
                AiPhase::SetAcceleration(1),
                AiPhase::SetThrottle(1),
                AiPhase::Steer(1),
                AiPhase::Waypoint(1)
            ]
        );
        assert_eq!([t[0].clone(), t[2].clone()], preserved);
        assert_eq!(t[3].upgrades.acceleration, 6);
        assert_eq!(t[1].controller.0, AI_THROTTLE_WORD);
        assert_eq!((t[1].heading, t[3].heading), (4, 4));
        assert!(t.iter().all(|truck| truck.world == ORIGIN));
        assert_eq!(rng, random());
    }
    #[test]
    fn all_human_service_has_no_rng_or_driver_mutation_and_high_mask_bits_are_ignored() {
        for mask in [0x0f, 0xff] {
            let mut t = trucks();
            let old = t.clone();
            let mut r = race();
            r.active_humans = mask;
            let before = r.clone();
            let mut rng = random();
            assert!(
                service_computers(&mut t, &mut r, &route(), &tables(), &mut rng)
                    .unwrap()
                    .is_empty()
            );
            assert_eq!(t, old);
            assert_eq!(r, before);
            assert_eq!(rng, random());
        }
    }
    #[test]
    fn fourth_lap_banner_respects_existing_counter_and_sixth_lap_is_not_new_winner() {
        for (old_lap, banner, expected_lap, expected_banner) in [(3, 19, 4, 19), (5, 0, 6, 30)] {
            let mut t = trucks();
            let mut r = race();
            r.drivers[1].lap = old_lap;
            r.banner_state = banner;
            let mut rng = draw_zero();
            progress_waypoint(
                &mut t,
                &mut r,
                1,
                [0x2002, 0x2002],
                &route(),
                &tables(),
                &mut rng,
            )
            .unwrap();
            assert_eq!(
                (r.drivers[1].lap, r.banner_state, r.winner_plus_one),
                (expected_lap, expected_banner, 0)
            );
        }
    }
    #[test]
    fn next_target_missing_record_is_reported_without_fabricating_route_coordinates() {
        let mut t = trucks();
        let mut d = Driver {
            checkpoint: 7,
            ..Driver::default()
        };
        let mut rng = random();
        assert!(next_target(&mut t[0], &mut d, &route(), &mut rng).is_err());
    }
}
