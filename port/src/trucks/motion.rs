//! Four-truck integer motion, wheels, body pose and contact-response contracts.
//! Independent original-source derivations and cross-owner effects: motion.trace.md.
//! Native integer rules and actual sequential fleet service; no emulated RAM.
use crate::controls::ControllerWord;
use crate::world::terrain::{BlockDirection, Projection, SamplingTables, Terrain};
use crate::world::tracks::{TRUCKS, WHEELS};
use anyhow::{Context, Result};

mod powertrain;

pub const WHEELS_PER_TRUCK: usize = WHEELS / TRUCKS;
pub const UPGRADE_LEVELS: usize = 7;
pub const SPEED_LIMITS: usize = 9;
pub const HEADING_DIRECTIONS: usize = 32;
pub const TILT_SELECTORS: usize = 5;
pub const TILT_POSES: usize = TILT_SELECTORS * TILT_SELECTORS;
pub const NITRO_SPEED: u8 = 0x7c;
#[cfg(test)]
const CONTACT_EFFECT_ENTRY: u8 = 0x18;
pub const BLOCKED_RECOVERY_TICKS: u8 = 0x10;
pub const AI_RECOVERY_SPEED: u8 = 0x81;
pub const AI_RECOVERY_LOCK: u8 = 0x0f;
pub const ORDINARY_BLOCKED_LOCK: u8 = 2;
pub const WHEEL_GRAVITY_FRACTION: u8 = 0x90;
pub const NITRO_SOUND_COMMAND: u16 = 0x0202;

/// Original finite upgrade selectors; source limits differ from added track count.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Upgrades {
    pub steering: u8,
    pub speed: u8,
    pub acceleration: u8,
    pub suspension: u8,
}
/// Heights and velocities retain source byte wrapping. Velocity is interpreted as
/// signed only at the explicit impulse/gravity branch; fraction carries are unsigned.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Wheel {
    pub sampled: u8,
    pub height: u8,
    pub velocity: u8,
    pub fraction: u8,
}
/// Source speed/heading arrays contain words even where updates store only bytes.
/// Preserving upper bytes matters for direct state/restore contracts and avoids
/// silently changing byte-width mutation into a signed-word update.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Truck {
    #[serde(with = "crate::snapshot::array")]
    pub world: [u16; 2],
    pub speed: u16,
    pub heading: u16,
    pub controller: ControllerWord,
    pub upgrades: Upgrades,
    pub acceleration_fraction: u16,
    pub steering_fraction: u8,
    #[serde(rename = "nitro_stock_bcd")]
    pub nitro: super::nitro::NitroStock,
    pub nitro_button_history: u8,
    #[serde(with = "crate::snapshot::array")]
    pub wheels: [Wheel; WHEELS_PER_TRUCK],
    pub surface_summary: u8,
    pub contact_marker: u8,
    pub contact_transition: u8,
    pub contact_cooldown: u8,
    #[serde(rename = "effect_cooldown")]
    pub contact_effect_player: crate::presentation::effects::AnimationPlayer,
    pub blocked_frames: u8,
    pub steering_lock: u8,
    pub ai_route_bias: u8,
    pub screen_x: u16,
    pub screen_y: u8,
    pub tile_attributes: u16,
    #[serde(rename = "nitro_animation")]
    pub nitro_effect: crate::presentation::effects::AnimationPlayer,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MotionTables {
    pub speed_limits: [u8; SPEED_LIMITS],
    pub acceleration_steps: [u16; UPGRADE_LEVELS],
    pub steering_rates: [u8; UPGRADE_LEVELS],
    pub x_vectors: [i16; HEADING_DIRECTIONS],
    pub y_vectors: [i16; HEADING_DIRECTIONS],
    pub body_poses: crate::presentation::poses::PoseRules,
    /// Full eight bytes retained; natural truck selectors consume only even cells.
    pub shadow_masks: [u8; TRUCKS * 2],
    pub blocked_coordinate_by_quadrant: [u8; 4],
}
pub fn load_motion_tables(rom: &[u8]) -> Result<MotionTables> {
    const SPEED_ROM: usize = 0x1504;
    const ACCELERATION_ROM: usize = 0x150d;
    const STEERING_ROM: usize = 0x165c;
    const X_VECTOR_ROM: usize = 0x15dc;
    const Y_VECTOR_ROM: usize = 0x161c;
    const SHADOW_MASK_ROM: usize = 0xaeb;
    const BLOCK_AXIS_ROM: usize = 0xe7d;
    fn bytes<const N: usize>(rom: &[u8], at: usize) -> Result<[u8; N]> {
        Ok(rom
            .get(at..at + N)
            .context("truncated motion table")?
            .try_into()?)
    }
    fn words<const N: usize>(rom: &[u8], at: usize) -> Result<[u16; N]> {
        let bytes = rom.get(at..at + N * 2).context("truncated motion words")?;
        Ok(std::array::from_fn(|i| {
            u16::from_le_bytes([bytes[i * 2], bytes[i * 2 + 1]])
        }))
    }
    Ok(MotionTables {
        speed_limits: bytes(rom, SPEED_ROM)?,
        acceleration_steps: words(rom, ACCELERATION_ROM)?,
        steering_rates: bytes(rom, STEERING_ROM)?,
        x_vectors: words::<HEADING_DIRECTIONS>(rom, X_VECTOR_ROM)?.map(|v| v as i16),
        y_vectors: words::<HEADING_DIRECTIONS>(rom, Y_VECTOR_ROM)?.map(|v| v as i16),
        body_poses: crate::presentation::poses::PoseRules::load(rom)?,
        shadow_masks: bytes(rom, SHADOW_MASK_ROM)?,
        blocked_coordinate_by_quadrant: bytes(rom, BLOCK_AXIS_ROM)?,
    })
}
pub fn steer(truck: &mut Truck, tables: &MotionTables) {
    let input = powertrain::PowertrainInput::from(&*truck);
    let mut state = powertrain::PowertrainState::from(&*truck);
    state.steer(&input, tables);
    state.commit_all(truck);
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EffectSprite {
    pub x: u8,
    pub y: u8,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MotionEffect {
    NitroSprite(EffectSprite),
    Sound(u16),
    ContactSprite(EffectSprite),
}
pub fn use_nitro(truck: &mut Truck, prepared: bool, active_humans: u8) -> Vec<MotionEffect> {
    let input = powertrain::PowertrainInput::from(&*truck);
    let mut state = powertrain::PowertrainState::from(&*truck);
    let effects = state.use_nitro(&input, prepared, active_humans);
    state.commit_all(truck);
    effects
}
/// Accumulation occurs on every call, including no throttle/airborne calls.
/// Steering and nitro are separate owner operations run before this function.
pub fn accelerate(truck: &mut Truck, tables: &MotionTables, human: bool, animation: u8) {
    let input = powertrain::PowertrainInput::from(&*truck);
    let mut state = powertrain::PowertrainState::from(&*truck);
    state.accelerate(&input, tables, human, animation);
    state.commit_all(truck);
}
pub fn update_powertrain(
    truck: &mut Truck,
    tables: &MotionTables,
    human: bool,
    active_humans: u8,
    prepared: bool,
    animation: u8,
) -> Vec<MotionEffect> {
    let outcome = powertrain::propose(truck, tables, human, active_humans, prepared, animation);
    outcome.state.commit_all(truck);
    outcome.effects
}
pub fn integrate_position(
    world: [u16; 2],
    heading: u16,
    speed: u16,
    tables: &MotionTables,
) -> [u16; 2] {
    const HEADING_QUANTIZATION: u32 = 3;
    let index = usize::from(heading as u8 >> HEADING_QUANTIZATION);
    let displacement =
        |vector: i16| ((i32::from(vector) * i32::from(speed as u8 as i8)) >> u8::BITS) as u16;
    [
        world[0].wrapping_add(displacement(tables.x_vectors[index])),
        world[1].wrapping_add(displacement(tables.y_vectors[index])),
    ]
}
pub fn integrate_wheels(trucks: &mut [Truck; TRUCKS]) {
    const MAX_SUSPENSION_IMPULSE: u8 = 7;
    const CONTACT_MARKER: u8 = 0xff;
    for truck in trucks.iter_mut().rev() {
        truck.contact_marker = 0;
        for wheel in truck.wheels.iter_mut().rev() {
            let (delta, below) = wheel.sampled.overflowing_sub(wheel.height);
            if (wheel.velocity as i8) < 0 && !below {
                let cap = MAX_SUSPENSION_IMPULSE.wrapping_sub(truck.upgrades.suspension >> 1);
                wheel.velocity = if delta == 1 {
                    wheel.height = wheel.sampled;
                    u8::MAX
                } else {
                    delta.min(cap)
                };
                wheel.fraction = u8::MAX;
                truck.contact_marker = CONTACT_MARKER;
            } else {
                let (fraction, borrow) = wheel.fraction.overflowing_sub(WHEEL_GRAVITY_FRACTION);
                wheel.fraction = fraction;
                if borrow {
                    wheel.velocity = wheel.velocity.wrapping_sub(1);
                }
            }
            wheel.height = wheel.height.wrapping_add(wheel.velocity);
        }
    }
}
pub fn average_surface(truck: &mut Truck) {
    let total: u16 = truck
        .wheels
        .iter()
        .map(|wheel| u16::from(wheel.height))
        .sum();
    truck.surface_summary = (total / WHEELS_PER_TRUCK as u16) as u8;
}
pub fn correct_suspension_speed(truck: &mut Truck) {
    const SLOPE_QUANTIZATION: u32 = 3;
    const SLOPE_MASK: u8 = 3;
    if truck.contact_marker == 0 {
        return;
    }
    let heights = truck.wheels.map(|wheel| wheel.height);
    let front = heights[0].wrapping_add(heights[1]);
    let rear = heights[2].wrapping_add(heights[3]);
    let slope = rear.wrapping_sub(front) as i8;
    let magnitude = (slope.unsigned_abs() >> SLOPE_QUANTIZATION) & SLOPE_MASK;
    let correction = if slope < 0 {
        (magnitude as i8).wrapping_neg()
    } else {
        magnitude as i8
    };
    let (speed, overflow) = (truck.speed as u8 as i8).overflowing_sub(correction);
    if !overflow {
        set_low(&mut truck.speed, speed as u8);
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BodyPose {
    pub lateral: u8,
    pub longitudinal: u8,
    pub frame: crate::presentation::poses::BodyFrame,
}
pub fn body_pose(truck: &Truck, tables: &MotionTables) -> BodyPose {
    const TILT_BIAS: u8 = 0x28;
    const TILT_MAX: u8 = 0x4f;
    const TILT_QUANTIZATION: u32 = 4;
    const HEADING_QUANTIZATION: u32 = 3;
    let h = truck.wheels.map(|wheel| wheel.height);
    let selector = |first: u8, second: u8| {
        let biased = first.wrapping_sub(second).wrapping_add(TILT_BIAS);
        (if (biased as i8) < 0 {
            0
        } else {
            biased.min(TILT_MAX)
        }) >> TILT_QUANTIZATION
    };
    let lateral = selector(h[0].wrapping_add(h[2]), h[1].wrapping_add(h[3]));
    let longitudinal = selector(h[0].wrapping_add(h[1]), h[2].wrapping_add(h[3]));
    BodyPose {
        lateral,
        longitudinal,
        frame: tables.body_poses.frame(
            lateral,
            longitudinal,
            truck.heading as u8 >> HEADING_QUANTIZATION,
        ),
    }
}
/// Despite its old name, original BIT/BPL reverses the sign of halved magnitude.
pub fn reverse_halved_speed(speed: u16) -> u16 {
    let old = speed as u8 as i8;
    let half = old.unsigned_abs() >> 1;
    let reversed = if old < 0 { half } else { half.wrapping_neg() };
    (speed & !u16::from(u8::MAX)) | u16::from(reversed)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkingMotion {
    pub world: [u16; 2],
    pub speed: u16,
    pub heading: u16,
}
pub fn count_blocked(truck: &mut Truck, working: &mut WorkingMotion, human: bool) {
    truck.steering_lock = ORDINARY_BLOCKED_LOCK;
    truck.ai_route_bias = 0;
    truck.blocked_frames = truck.blocked_frames.wrapping_add(1);
    if truck.blocked_frames >= BLOCKED_RECOVERY_TICKS {
        truck.blocked_frames = 0;
        if !human {
            truck.steering_lock = AI_RECOVERY_LOCK;
            set_low(&mut working.speed, AI_RECOVERY_SPEED);
            set_low(&mut truck.speed, AI_RECOVERY_SPEED);
        }
    }
}
pub fn contact_effect(truck: &mut Truck) -> Option<EffectSprite> {
    const CONTACT_X_OFFSET: u8 = 4;
    const CONTACT_Y_OFFSET: u8 = 6;
    if !truck.contact_effect_player.is_idle() || truck.screen_x > u16::from(u8::MAX) {
        return None;
    }
    let (x, carry) = (truck.screen_x as u8).overflowing_add(CONTACT_X_OFFSET);
    if carry {
        return None;
    }
    truck.contact_effect_player.start_contact();
    Some(EffectSprite {
        x,
        y: truck.screen_y.wrapping_add(CONTACT_Y_OFFSET),
    })
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockedResponse {
    pub blocked: [Option<BlockDirection>; WHEELS_PER_TRUCK],
    pub effects: Vec<MotionEffect>,
}
/// Called only after terrain/object side effects. Two last-wheel blocks take the
/// complete rejection path; otherwise contact-entry and single-axis steering run.
pub fn resolve_blocked_motion(
    truck: &mut Truck,
    working: &mut WorkingMotion,
    mut blocked: [Option<BlockDirection>; WHEELS_PER_TRUCK],
    human: bool,
) -> BlockedResponse {
    const AXIS_TURN: u16 = 4;
    const QUADRANT_SHIFT: u32 = 6;
    const NEGATIVE_SPEED_BIT: u16 = 0x80;
    let mut effects = Vec::new();
    let spawn = |truck: &mut Truck, effects: &mut Vec<MotionEffect>| {
        if let Some(sprite) = contact_effect(truck) {
            effects.push(MotionEffect::ContactSprite(sprite));
        }
    };
    if blocked[2].is_some() && blocked[3].is_some() {
        working.world = truck.world;
        working.speed = reverse_halved_speed(truck.speed);
        count_blocked(truck, working, human);
        spawn(truck, &mut effects);
        return BlockedResponse { blocked, effects };
    }
    if truck.contact_transition == 0 {
        if truck.contact_marker == 0 {
            truck.contact_transition = 1;
        }
    } else if truck.contact_marker != 0 {
        truck.contact_transition = 0;
        spawn(truck, &mut effects);
    }
    if working.speed & NEGATIVE_SPEED_BIT != 0 {
        blocked[2] = blocked[1];
        blocked[3] = blocked[0];
    }
    if blocked[2].is_none() && blocked[3].is_none() {
        truck.blocked_frames = 0;
    } else {
        // The source table alternates Y/X for negative turns; positive turns
        // invert it. Select from the old quadrant before rotating the heading.
        let quadrant = (working.heading as u8 >> QUADRANT_SHIFT) & 3;
        let negative_turn = blocked[2].is_some();
        let restore_x = (quadrant & 1 != 0) == negative_turn;
        let axis = if restore_x { 0 } else { 1 };
        working.world[axis] = truck.world[axis];
        working.heading = u16::from(working.heading.wrapping_add(if negative_turn {
            AXIS_TURN.wrapping_neg()
        } else {
            AXIS_TURN
        }) as u8);
        spawn(truck, &mut effects);
        count_blocked(truck, working, human);
    }
    BlockedResponse { blocked, effects }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShadowSprite {
    pub x: u8,
    pub y: u8,
    pub tile_attributes: u16,
    pub packed_high: u8,
}
pub fn shadow_sprite(
    truck: &Truck,
    slot: usize,
    projection_vertical_term: u16,
    packed_high: u8,
    tables: &MotionTables,
) -> ShadowSprite {
    const BASE_Y: u16 = 0x2c00;
    const HEIGHT_SHIFT: u32 = 3;
    const SCREEN_SHIFT: u32 = 4;
    const SHADOW_Y_OFFSET: u16 = 0x2a;
    const ATTRIBUTE_MASK: u16 = 0x31ff;
    const SHADOW_ATTRIBUTE: u16 = 0x0800;
    const LARGE_SPRITES: u8 = 0xaa;
    let samples = truck.wheels.map(|wheel| wheel.sampled);
    let average = (u16::from(samples[0].wrapping_add(samples[1]))
        + u16::from(samples[2].wrapping_add(samples[3])))
        / WHEELS_PER_TRUCK as u16;
    let y = ((BASE_Y
        .wrapping_sub(projection_vertical_term)
        .wrapping_sub(average << HEIGHT_SHIFT)
        >> SCREEN_SHIFT)
        .wrapping_add(SHADOW_Y_OFFSET)) as u8;
    let mask = tables.shadow_masks[slot * 2];
    let mut high = (packed_high & !mask) | LARGE_SPRITES;
    if truck.screen_x > u16::from(u8::MAX) {
        high |= mask;
    }
    ShadowSprite {
        x: truck.screen_x as u8,
        y,
        tile_attributes: (truck.tile_attributes & ATTRIBUTE_MASK) | SHADOW_ATTRIBUTE,
        packed_high: high,
    }
}
/// Stable domain phase observations for ordering contracts, not CPU instructions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotionPhase {
    Powertrain(usize),
    Displacement(usize),
    SampleTerrain(usize),
    SpecialSurface(usize),
    ContactResponse(usize),
    SurfaceAverage(usize),
    Project(usize),
    Shadow(usize),
    PairContact(usize),
    Cooldown(usize),
    Commit(usize),
}
pub struct MotionEnvironment<'a> {
    pub tables: &'a MotionTables,
    pub terrain: &'a Terrain,
    pub sampling: &'a SamplingTables,
    pub projection: Projection,
    pub original_track: u8,
    pub active_humans: u8,
    pub prepared: bool,
    pub animation: u8,
}
/// Whole four-truck service, before global wheel integration/pose update. Each
/// commit is visible to later truck collision checks. Object/contact implementations
/// remain their own owners; phase observations preserve cross-call ordering.
#[derive(Clone, Debug)]
pub struct FleetFrame {
    pub phases: Vec<MotionPhase>,
    pub effects: Vec<(usize, MotionEffect)>,
    pub shadows: Vec<ShadowSprite>,
    pub contacts: Vec<(usize, super::contact::ContactVisit)>,
    pub surface_sprites: Vec<(usize, crate::world::objects::ObjectSprite)>,
}
/// Actual source service. The contact-animation array is borrowed from the world
/// owner and the bearing table is immutable; neither is replaced with an effect
/// callback or a second mutable authority. Consumers stage returned sprites/audio.
pub fn update_trucks(
    trucks: &mut [Truck; TRUCKS],
    environment: &MotionEnvironment<'_>,
    terrain_history: &mut u8,
    contact_animations: &mut [crate::world::objects::ContactAnimation; TRUCKS],
    contact_tables: &super::bearings::BearingTable,
) -> Result<FleetFrame> {
    use crate::world::terrain::{project, sample_wheels};
    let mut frame = FleetFrame {
        phases: Vec::new(),
        effects: Vec::new(),
        shadows: Vec::new(),
        contacts: Vec::new(),
        surface_sprites: Vec::new(),
    };
    let mut packed_shadow_high = 0;
    for slot in 0..TRUCKS {
        let human = environment.active_humans & (1 << slot) != 0;
        frame.phases.push(MotionPhase::Powertrain(slot));
        // Retained powertrain outputs commit now. Proposed speed/heading stay
        // separate until terrain rejection and sequential pair contacts finish.
        let outcome = powertrain::propose(
            &trucks[slot],
            environment.tables,
            human,
            environment.active_humans,
            environment.prepared,
            environment.animation,
        );
        outcome.state.commit_retained(&mut trucks[slot]);
        frame
            .effects
            .extend(outcome.effects.into_iter().map(|effect| (slot, effect)));
        frame.phases.push(MotionPhase::Displacement(slot));
        let mut working = WorkingMotion {
            world: integrate_position(
                trucks[slot].world,
                outcome.state.heading,
                outcome.state.speed,
                environment.tables,
            ),
            speed: outcome.state.speed,
            heading: outcome.state.heading,
        };
        frame.phases.push(MotionPhase::SampleTerrain(slot));
        let heights = std::array::from_fn(|index| {
            trucks[index / WHEELS_PER_TRUCK].wheels[index % WHEELS_PER_TRUCK].sampled
        });
        let summaries = std::array::from_fn(|index| trucks[index].surface_summary);
        let sampled = sample_wheels(
            environment.terrain,
            environment.sampling,
            working.world,
            working.heading as u8,
            slot,
            heights,
            summaries,
            *terrain_history,
            environment.original_track,
        )?;
        *terrain_history = sampled.control_history;
        for wheel in 0..WHEELS_PER_TRUCK {
            trucks[slot].wheels[wheel].sampled = sampled.heights[slot * WHEELS_PER_TRUCK + wheel];
        }
        frame.phases.push(MotionPhase::SpecialSurface(slot));
        let previous_cursor = contact_animations[slot].player;
        frame.effects.extend(
            crate::world::objects::special_surface(
                &trucks[slot],
                &mut working,
                *terrain_history,
                &mut contact_animations[slot],
                environment.active_humans,
            )
            .into_iter()
            .map(|sound| (slot, MotionEffect::Sound(sound))),
        );
        if previous_cursor != contact_animations[slot].player {
            frame
                .surface_sprites
                .push((slot, contact_animations[slot].sprite));
        }
        frame.phases.push(MotionPhase::ContactResponse(slot));
        let both_blocked = sampled.blocked[2].is_some() && sampled.blocked[3].is_some();
        let response =
            resolve_blocked_motion(&mut trucks[slot], &mut working, sampled.blocked, human);
        frame
            .effects
            .extend(response.effects.into_iter().map(|effect| (slot, effect)));
        if !both_blocked {
            frame.phases.push(MotionPhase::SurfaceAverage(slot));
            average_surface(&mut trucks[slot]);
        }
        frame.phases.push(MotionPhase::Project(slot));
        // The source working surface remains the pre-service summary even after
        // the integrated-height average writes its retained truck summary.
        let projected = project(
            environment.projection,
            working.world,
            sampled.working_surface_height,
        );
        let (x, y, attributes) = crate::presentation::sprites::truck_projection(
            environment.original_track,
            trucks[slot].world[1],
            trucks[slot].surface_summary,
            [projected.x, u16::from(projected.y)],
            trucks[slot].tile_attributes,
        );
        trucks[slot].screen_x = x;
        trucks[slot].screen_y = y;
        trucks[slot].tile_attributes = attributes;
        frame.phases.push(MotionPhase::Shadow(slot));
        let vertical = (((i32::from(working.world[1] as i16)
            * i32::from(environment.projection.scale[1] as i8))
            >> u8::BITS) as u16)
            .wrapping_add(environment.projection.offset[1]);
        let shadow = shadow_sprite(
            &trucks[slot],
            slot,
            vertical,
            packed_shadow_high,
            environment.tables,
        );
        packed_shadow_high = shadow.packed_high;
        frame.shadows.push(shadow);
        frame.phases.push(if trucks[slot].contact_cooldown == 0 {
            MotionPhase::PairContact(slot)
        } else {
            MotionPhase::Cooldown(slot)
        });
        let contacts = super::contact::service_contacts(
            trucks,
            slot,
            &mut working,
            &super::contact::ContactEnvironment {
                terrain: environment.terrain,
                sampling: environment.sampling,
                tables: contact_tables,
                original_track: environment.original_track,
                animation: environment.animation,
                active_humans: environment.active_humans,
            },
            terrain_history,
        )?;
        frame
            .contacts
            .extend(contacts.into_iter().map(|visit| (slot, visit)));
        frame.phases.push(MotionPhase::Commit(slot));
        trucks[slot].world = working.world;
        trucks[slot].speed = working.speed;
        trucks[slot].heading = working.heading;
    }
    Ok(frame)
}

/// Source byte-width writes preserve the unused half of a native word field.
fn set_low(word: &mut u16, value: u8) {
    *word = (*word & !u16::from(u8::MAX)) | u16::from(value);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controls::{BUTTON_B, BUTTON_L, BUTTON_LEFT, BUTTON_R, BUTTON_RIGHT, BUTTON_Y};
    use crate::world::terrain::{CELL_BYTES, ROW_BYTES};
    use crate::world::tracks::TERRAIN_BYTES;
    const TABLE_ROM_END: usize = 0x1663;
    const CONTACT: u8 = 0xff;
    const BASE_HEIGHT: u8 = 40;
    const START_WORLD: [u16; 2] = [0x1080, 0x1080];
    // Independent fixture reads only; no behavior is reimplemented as an oracle.
    fn tables() -> MotionTables {
        let rom = std::fs::read("../input/game.rom").unwrap();
        let words = |start, count| -> Vec<u16> {
            rom[start..start + count * 2]
                .chunks_exact(2)
                .map(|b| u16::from_le_bytes([b[0], b[1]]))
                .collect()
        };
        MotionTables {
            speed_limits: rom[0x1504..0x150d].try_into().unwrap(),
            acceleration_steps: words(0x150d, UPGRADE_LEVELS).try_into().unwrap(),
            steering_rates: rom[0x165c..TABLE_ROM_END].try_into().unwrap(),
            x_vectors: words(0x15dc, HEADING_DIRECTIONS)
                .into_iter()
                .map(|v| v as i16)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            y_vectors: words(0x161c, HEADING_DIRECTIONS)
                .into_iter()
                .map(|v| v as i16)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            body_poses: crate::presentation::poses::PoseRules::decode(
                words(0xca2, TILT_POSES).try_into().unwrap(),
            ),
            shadow_masks: rom[0xaeb..0xaf3].try_into().unwrap(),
            blocked_coordinate_by_quadrant: rom[0xe7d..0xe81].try_into().unwrap(),
        }
    }
    fn truck() -> Truck {
        Truck {
            world: START_WORLD,
            contact_marker: CONTACT,
            screen_x: 100,
            screen_y: 90,
            wheels: [Wheel {
                sampled: BASE_HEIGHT,
                height: BASE_HEIGHT,
                velocity: 0,
                fraction: 0xff,
            }; WHEELS_PER_TRUCK],
            ..Truck::default()
        }
    }
    fn working(t: &Truck) -> WorkingMotion {
        WorkingMotion {
            world: t.world,
            speed: t.speed,
            heading: t.heading,
        }
    }
    fn resolve_legacy_blocked_motion(
        truck: &mut Truck,
        working: &mut WorkingMotion,
        flags: [u8; WHEELS_PER_TRUCK],
        human: bool,
    ) -> BlockedResponse {
        resolve_blocked_motion(
            truck,
            working,
            flags.map(|value| {
                if value == 0 {
                    None
                } else {
                    Some(BlockDirection(value.wrapping_sub(1)))
                }
            }),
            human,
        )
    }
    #[test]
    fn named_tables_load_all_finite_domains_and_reject_truncated_rom() {
        let rom = std::fs::read("../input/game.rom").unwrap();
        let t = load_motion_tables(&rom).unwrap();
        assert_eq!(t, tables());
        assert_eq!(t.speed_limits, [60, 64, 68, 72, 76, 80, 84, 101, 48]);
        assert_eq!(
            t.acceleration_steps,
            [0x180, 0x1c0, 0x200, 0x240, 0x280, 0x2c0, 0x300]
        );
        assert_eq!(t.steering_rates, [56, 60, 64, 68, 72, 76, 80]);
        assert!(load_motion_tables(&rom[..TABLE_ROM_END - 1]).is_err());
    }
    #[test]
    fn steering_accumulates_each_upgrade_and_turns_only_on_byte_carry() {
        let rates = [56, 60, 64, 68, 72, 76, 80];
        for (level, rate) in rates.into_iter().enumerate() {
            let mut t = truck();
            t.upgrades.steering = level as u8;
            t.controller = ControllerWord(BUTTON_LEFT);
            t.heading = 0;
            t.steering_fraction = 255 - rate;
            steer(&mut t, &tables());
            assert_eq!((t.steering_fraction, t.heading), (255, 0));
            steer(&mut t, &tables());
            assert_eq!((t.steering_fraction, t.heading), (rate - 1, 8));
        }
    }
    #[test]
    fn steering_combines_all_four_bits_in_source_order_and_wraps_heading() {
        for (buttons, start, expected) in [
            (BUTTON_RIGHT, 0, 248),
            (BUTTON_R, 0, 252),
            (BUTTON_LEFT, 252, 4),
            (BUTTON_L, 254, 2),
            (BUTTON_RIGHT | BUTTON_R, 4, 248),
            (BUTTON_RIGHT | BUTTON_R | BUTTON_LEFT | BUTTON_L, 4, 4),
        ] {
            let mut t = truck();
            t.steering_fraction = 255;
            t.heading = start;
            t.controller = ControllerWord(buttons);
            steer(&mut t, &tables());
            assert_eq!(t.heading, expected);
        }
    }
    #[test]
    fn acceleration_fraction_updates_even_without_throttle_or_ground_contact() {
        let mut t = truck();
        t.acceleration_fraction = 0x00f0;
        t.speed = 0xab05;
        accelerate(&mut t, &tables(), true, 0);
        assert_eq!((t.acceleration_fraction, t.speed), (0x0070, 0xab04));
        t.controller = ControllerWord(BUTTON_B);
        t.contact_marker = 0;
        accelerate(&mut t, &tables(), true, 0);
        assert_eq!((t.acceleration_fraction, t.speed), (0x00f0, 0xab03));
    }
    #[test]
    fn each_acceleration_upgrade_consumes_integral_step_and_retains_low_fraction() {
        for (level, expected_speed, expected_fraction) in [
            (0, 12, 0x70),
            (1, 12, 0xb0),
            (2, 12, 0xf0),
            (3, 13, 0x30),
            (4, 13, 0x70),
            (5, 13, 0xb0),
            (6, 13, 0xf0),
        ] {
            let mut t = truck();
            t.controller = ControllerWord(BUTTON_B);
            t.speed = 10;
            t.acceleration_fraction = 0xf0;
            t.upgrades.acceleration = level;
            accelerate(&mut t, &tables(), true, 0);
            assert_eq!(
                (t.speed, t.acceleration_fraction),
                (expected_speed, expected_fraction)
            );
        }
    }
    #[test]
    fn positive_limit_or_crossing_uses_old_speed_and_alternating_frame_drag() {
        for (old, frame, expected) in [
            (58, 0, 59),
            (59, 0, 59),
            (59, 1, 58),
            (60, 0, 60),
            (60, 1, 59),
            (80, 1, 79),
        ] {
            let mut t = truck();
            t.controller = ControllerWord(BUTTON_B);
            t.speed = old;
            accelerate(&mut t, &tables(), true, frame);
            assert_eq!(t.speed, expected);
        }
    }
    #[test]
    fn zero_upgrade_ai_uses_special_limit_and_nonzero_ai_uses_ordinary_selector() {
        for (human, upgrade, old, expected) in [
            (true, 0, 49, 50),
            (false, 0, 49, 49),
            (false, 1, 60, 61),
            (false, 7, 100, 100),
        ] {
            let mut t = truck();
            t.controller = ControllerWord(BUTTON_B);
            t.speed = old;
            t.upgrades.speed = upgrade;
            accelerate(&mut t, &tables(), human, 0);
            assert_eq!(t.speed, expected);
        }
    }
    #[test]
    fn negative_speed_accelerates_as_byte_and_drag_moves_both_signs_toward_zero() {
        let mut t = truck();
        t.speed = 0xabc1;
        t.controller = ControllerWord(BUTTON_B);
        accelerate(&mut t, &tables(), true, 0);
        assert_eq!(t.speed, 0xabc2);
        for (old, expected) in [
            (0xabff, 0xab00),
            (0xab81, 0xab82),
            (0xab01, 0xab00),
            (0xab00, 0xab00),
        ] {
            t.speed = old;
            t.controller = ControllerWord(0);
            accelerate(&mut t, &tables(), true, 0);
            assert_eq!(t.speed, expected);
        }
    }
    #[test]
    fn positive_acceleration_signed_overflow_rejects_step_without_drag() {
        let mut data = tables();
        data.speed_limits[0] = 127;
        data.acceleration_steps[0] = 0x300;
        let mut t = truck();
        t.speed = 126;
        t.controller = ControllerWord(BUTTON_B);
        accelerate(&mut t, &data, true, 1);
        assert_eq!(t.speed, 126);
    }
    #[test]
    fn nitro_prepared_frame_preserves_edge_history_stock_speed_and_animation() {
        let mut t = truck();
        t.nitro_button_history = 0x91;
        t.nitro = crate::trucks::nitro::NitroStock::from_encoded(0x10);
        t.speed = 0xab22;
        t.controller = ControllerWord(BUTTON_Y);
        let before = t.clone();
        assert!(use_nitro(&mut t, true, 1).is_empty());
        assert_eq!(t, before);
    }
    #[test]
    fn nitro_press_decimal_borrow_hold_release_and_repress_have_distinct_effects() {
        let mut t = truck();
        t.nitro = crate::trucks::nitro::NitroStock::from_encoded(0x10);
        t.controller = ControllerWord(BUTTON_Y);
        t.speed = 0xab22;
        assert_eq!(
            use_nitro(&mut t, false, 1),
            vec![
                MotionEffect::NitroSprite(EffectSprite { x: 100, y: 90 }),
                MotionEffect::Sound(NITRO_SOUND_COMMAND)
            ]
        );
        assert_eq!(
            (
                t.nitro.encoded(),
                t.speed,
                t.nitro_button_history,
                t.nitro_effect.encoded()
            ),
            (9, 0xab7c, 0x80, 1)
        );
        assert!(use_nitro(&mut t, false, 1).is_empty());
        assert_eq!((t.nitro.encoded(), t.nitro_button_history), (9, 0xc0));
        t.controller = ControllerWord(0);
        assert!(use_nitro(&mut t, false, 1).is_empty());
        assert_eq!(t.nitro_button_history, 0x60);
        t.controller = ControllerWord(BUTTON_Y);
        use_nitro(&mut t, false, 1);
        assert_eq!((t.nitro.encoded(), t.nitro_button_history), (8, 0xb0));
    }
    #[test]
    fn nitro_empty_still_latches_edge_and_attract_still_boosts_without_audio() {
        let mut t = truck();
        t.controller = ControllerWord(BUTTON_Y);
        assert!(use_nitro(&mut t, false, 1).is_empty());
        assert_eq!(t.nitro_button_history, 0x80);
        t.nitro = crate::trucks::nitro::NitroStock::from_encoded(1);
        assert!(use_nitro(&mut t, false, 1).is_empty());
        assert_eq!(t.nitro.encoded(), 1);
        t.nitro_button_history = 0;
        assert_eq!(
            use_nitro(&mut t, false, 0),
            vec![MotionEffect::NitroSprite(EffectSprite { x: 100, y: 90 })]
        );
        assert_eq!((t.nitro.encoded(), t.speed), (0, NITRO_SPEED.into()));
    }
    #[test]
    fn nitro_offscreen_skips_sprite_but_global_other_human_enables_sound() {
        let mut t = truck();
        t.controller = ControllerWord(BUTTON_Y);
        t.nitro = crate::trucks::nitro::NitroStock::from_encoded(1);
        t.screen_x = 0x100;
        t.nitro_effect = 7.into();
        assert_eq!(
            use_nitro(&mut t, false, 8),
            vec![MotionEffect::Sound(NITRO_SOUND_COMMAND)]
        );
        assert_eq!(t.nitro_effect.encoded(), 7);
    }
    #[test]
    fn powertrain_steers_then_boosts_then_applies_normal_drag_or_speed_limit() {
        for (buttons, frame, expected) in [
            (BUTTON_Y, 0, 123),
            (BUTTON_Y | BUTTON_B, 0, 124),
            (BUTTON_Y | BUTTON_B, 1, 123),
        ] {
            let mut t = truck();
            t.controller = ControllerWord(buttons | BUTTON_RIGHT);
            t.steering_fraction = 255;
            t.nitro = crate::trucks::nitro::NitroStock::from_encoded(1);
            update_powertrain(&mut t, &tables(), true, 1, false, frame);
            assert_eq!(
                (t.heading, t.speed, t.acceleration_fraction),
                (248, expected, 128)
            );
        }
    }
    #[test]
    fn iteration_three_powertrain_retained_fields_and_proposal_widths_are_explicit() {
        let mut t = truck();
        t.speed = 0xab07;
        t.heading = 0xcd20;
        t.acceleration_fraction = 0x12ff;
        t.steering_fraction = 255;
        t.nitro_button_history = 1;
        t.nitro = crate::trucks::nitro::NitroStock::new(1);
        t.controller = ControllerWord(BUTTON_Y | BUTTON_RIGHT);
        let mut expected = t.clone();
        // Source steering masks the heading word only on a turn; boost and drag
        // preserve speed's high byte. Acceleration always retains just its low
        // fraction, even without throttle. No wheel/contact/projection is written.
        expected.heading = 0x18;
        expected.speed = 0xab7b;
        expected.acceleration_fraction = 0x7f;
        expected.steering_fraction = 55;
        expected.nitro_button_history = 0x80;
        expected.nitro = crate::trucks::nitro::NitroStock::new(0);
        expected.nitro_effect = 1.into();
        let effects = update_powertrain(&mut t, &tables(), true, 1, false, 0);
        assert_eq!(t, expected);
        assert_eq!(
            effects,
            [
                MotionEffect::NitroSprite(EffectSprite { x: 100, y: 90 }),
                MotionEffect::Sound(NITRO_SOUND_COMMAND),
            ]
        );
    }
    #[test]
    fn prepared_powertrain_still_steers_and_accumulates_but_does_not_latch_nitro() {
        let mut t = truck();
        t.controller = ControllerWord(BUTTON_Y | BUTTON_LEFT | BUTTON_B);
        t.steering_fraction = 255;
        t.nitro = crate::trucks::nitro::NitroStock::from_encoded(0x10);
        t.nitro_button_history = 0x31;
        assert!(update_powertrain(&mut t, &tables(), true, 1, true, 0).is_empty());
        assert_eq!((t.heading, t.speed, t.acceleration_fraction), (8, 1, 128));
        assert_eq!((t.nitro.encoded(), t.nitro_button_history), (0x10, 0x31));
    }
    #[test]
    fn negative_acceleration_can_cross_zero_without_positive_limit_branch() {
        let mut t = truck();
        t.speed = 0xabff;
        t.controller = ControllerWord(BUTTON_B);
        t.upgrades.acceleration = 6;
        accelerate(&mut t, &tables(), true, 1);
        assert_eq!(t.speed, 0xab02);
    }
    #[test]
    fn displacement_uses_signed_low_speed_ignores_high_byte_and_truncates_negative_products() {
        let data = tables();
        for (heading, speed, expected) in [
            (0, 0xab40, [0x1040, 0x1000]),
            (64, 64, [0x1000, 0x1040]),
            (128, 64, [0x0fc0, 0x1000]),
            (192, 64, [0x1000, 0x0fc0]),
            (8, 1, [0x1000, 0x1000]),
            (8, 0xffff, [0x0fff, 0x0fff]),
        ] {
            assert_eq!(
                integrate_position([0x1000; 2], heading, speed, &data),
                expected
            );
        }
        assert_eq!(integrate_position([0xfff0, 0], 0, 64, &data), [0x0030, 0]);
        assert_eq!(integrate_position([0, 0], 255, 8, &data), [7, 0xffff]);
    }
    #[test]
    fn wheel_gravity_uses_fraction_borrow_wrap_and_clears_stale_markers() {
        let mut fleet = std::array::from_fn(|_| truck());
        fleet[0].wheels = [
            Wheel {
                sampled: 40,
                height: 40,
                velocity: 0,
                fraction: 0x90,
            },
            Wheel {
                sampled: 40,
                height: 40,
                velocity: 0,
                fraction: 0x8f,
            },
            Wheel {
                sampled: 40,
                height: 0,
                velocity: 0,
                fraction: 0,
            },
            Wheel {
                sampled: 0,
                height: 0,
                velocity: 0x80,
                fraction: 0,
            },
        ];
        integrate_wheels(&mut fleet);
        assert_eq!(
            fleet[0].wheels,
            [
                Wheel {
                    sampled: 40,
                    height: 40,
                    velocity: 0,
                    fraction: 0
                },
                Wheel {
                    sampled: 40,
                    height: 39,
                    velocity: 0xff,
                    fraction: 0xff
                },
                Wheel {
                    sampled: 40,
                    height: 255,
                    velocity: 0xff,
                    fraction: 0x70
                },
                Wheel {
                    sampled: 0,
                    height: 0,
                    velocity: 0,
                    fraction: 255
                }
            ]
        );
        assert_eq!(fleet[0].contact_marker, CONTACT);
        assert_eq!(
            [
                fleet[1].contact_marker,
                fleet[2].contact_marker,
                fleet[3].contact_marker
            ],
            [0; 3]
        );
    }
    #[test]
    fn negative_wheel_velocity_recontacts_only_when_target_not_below_height() {
        let mut fleet = std::array::from_fn(|_| truck());
        fleet[0].wheels = [
            Wheel {
                sampled: 41,
                height: 40,
                velocity: 0xff,
                fraction: 0,
            },
            Wheel {
                sampled: 40,
                height: 40,
                velocity: 0xff,
                fraction: 0,
            },
            Wheel {
                sampled: 45,
                height: 40,
                velocity: 0xff,
                fraction: 0,
            },
            Wheel {
                sampled: 39,
                height: 40,
                velocity: 0xff,
                fraction: 0,
            },
        ];
        integrate_wheels(&mut fleet);
        assert_eq!(
            fleet[0].wheels,
            [
                Wheel {
                    sampled: 41,
                    height: 40,
                    velocity: 0xff,
                    fraction: 255
                },
                Wheel {
                    sampled: 40,
                    height: 40,
                    velocity: 0,
                    fraction: 255
                },
                Wheel {
                    sampled: 45,
                    height: 45,
                    velocity: 5,
                    fraction: 255
                },
                Wheel {
                    sampled: 39,
                    height: 38,
                    velocity: 0xfe,
                    fraction: 0x70
                }
            ]
        );
    }
    #[test]
    fn suspension_upgrade_caps_upward_impulse_at_seven_minus_half_upgrade() {
        for (upgrade, expected) in [(0, 7), (1, 7), (2, 6), (3, 6), (4, 5), (5, 5), (6, 4)] {
            let mut fleet = std::array::from_fn(|_| truck());
            fleet[0].upgrades.suspension = upgrade;
            fleet[0].wheels[0] = Wheel {
                sampled: 100,
                height: 40,
                velocity: 0xff,
                fraction: 0,
            };
            integrate_wheels(&mut fleet);
            let w = fleet[0].wheels[0];
            assert_eq!(
                (w.velocity, w.height, w.fraction),
                (expected, 40 + expected, 255)
            );
        }
    }
    #[test]
    fn upward_impulse_threshold_is_inclusive_cap_plus_one_and_height_addition_wraps() {
        for (delta, expected) in [(7, 7), (8, 7)] {
            let mut fleet = std::array::from_fn(|_| truck());
            fleet[0].wheels[0] = Wheel {
                sampled: 243 + delta,
                height: 243,
                velocity: 255,
                fraction: 0,
            };
            integrate_wheels(&mut fleet);
            assert_eq!(
                (fleet[0].wheels[0].velocity, fleet[0].wheels[0].height),
                (expected, 250)
            );
        }
        // Gravity adds positive velocity without clamping the source byte height.
        let mut fleet = std::array::from_fn(|_| truck());
        fleet[0].wheels[0] = Wheel {
            sampled: 0,
            height: 254,
            velocity: 4,
            fraction: 255,
        };
        integrate_wheels(&mut fleet);
        assert_eq!(
            (fleet[0].wheels[0].height, fleet[0].wheels[0].velocity),
            (2, 4)
        );
    }
    #[test]
    fn surface_average_reads_integrated_heights_as_words_not_sample_targets() {
        let mut t = truck();
        for (w, h) in t.wheels.iter_mut().zip([200, 201, 202, 203]) {
            w.height = h;
            w.sampled = 0;
        }
        average_surface(&mut t);
        assert_eq!(t.surface_summary, 201);
    }
    #[test]
    fn suspension_speed_delta_is_wrapping_byte_slope_quantization_and_guarded_subtraction() {
        for (heights, speed, expected) in [
            ([40, 40, 48, 48], 0xab20, 0xab1e),
            ([48, 48, 40, 40], 0xab20, 0xab22),
            ([0, 0, 20, 20], 0xab20, 0xab1f),
            ([0, 0, 8, 8], 0xab80, 0xab80),
            ([8, 8, 0, 0], 0xab7f, 0xab7f),
        ] {
            let mut t = truck();
            t.speed = speed;
            for (w, h) in t.wheels.iter_mut().zip(heights) {
                w.height = h;
            }
            correct_suspension_speed(&mut t);
            assert_eq!(t.speed, expected);
        }
        let mut t = truck();
        t.contact_marker = 0;
        t.speed = 20;
        t.wheels[2].height = 100;
        correct_suspension_speed(&mut t);
        assert_eq!(t.speed, 20);
    }
    #[test]
    fn pose_flat_body_uses_full_heading_and_nonzero_tilt_halves_heading() {
        let mut t = truck();
        t.heading = 255;
        let flat = body_pose(&t, &tables());
        assert_eq!(
            (
                flat.lateral,
                flat.longitudinal,
                flat.frame.working_heading(),
                flat.frame.encoded()
            ),
            (2, 2, 31, 0x0dc0)
        );
        for (w, h) in t.wheels.iter_mut().zip([60, 40, 60, 40]) {
            w.height = h;
        }
        let tilted = body_pose(&t, &tables());
        assert_eq!(
            (
                tilted.lateral,
                tilted.longitudinal,
                tilted.frame.working_heading(),
                tilted.frame.encoded()
            ),
            (4, 2, 15, 0x2dc0)
        );
    }
    #[test]
    fn pose_each_tilt_axis_clamps_signed_byte_before_five_by_five_index() {
        for (heights, lateral, longitudinal, pose) in [
            ([100, 0, 100, 0], 0, 2, 0x1000),
            ([0, 40, 0, 40], 0, 2, 0x1000),
            ([0, 0, 40, 40], 2, 0, 0x5800),
            ([40, 40, 0, 0], 2, 4, 0x8000),
        ] {
            let mut t = truck();
            for (w, h) in t.wheels.iter_mut().zip(heights) {
                w.height = h;
            }
            let p = body_pose(&t, &tables());
            assert_eq!(
                (p.lateral, p.longitudinal, p.frame.encoded()),
                (lateral, longitudinal, pose)
            );
        }
    }
    #[test]
    fn pose_packing_rearranges_strip_bits_at_all_bank_boundary_values() {
        for (pose, packed) in [
            (0, 0),
            (7, 0x1c0),
            (8, 0x400),
            (255, 0x7dc0),
            (256, 0x8000),
            (415, 0xcdc0),
        ] {
            assert_eq!(crate::presentation::poses::encode_source_pose(pose), packed);
        }
    }
    #[test]
    fn source_halving_helper_reverses_sign_instead_of_preserving_old_comment() {
        for (speed, expected) in [
            (0xab3f, 0xabe1),
            (0xabc1, 0xab1f),
            (0xab80, 0xab40),
            (0xab01, 0xab00),
            (0xabff, 0xab00),
            (0xab00, 0xab00),
        ] {
            assert_eq!(reverse_halved_speed(speed), expected);
        }
    }
    #[test]
    fn blocked_counter_sets_short_lock_resets_bias_and_only_recovers_computer_on_sixteen() {
        for human in [false, true] {
            let mut t = truck();
            t.blocked_frames = 14;
            t.ai_route_bias = 5;
            t.steering_lock = 9;
            t.speed = 0xab20;
            let mut w = working(&t);
            count_blocked(&mut t, &mut w, human);
            assert_eq!(
                (t.blocked_frames, t.steering_lock, t.ai_route_bias, w.speed),
                (15, ORDINARY_BLOCKED_LOCK, 0, 0xab20)
            );
            count_blocked(&mut t, &mut w, human);
            assert_eq!(t.blocked_frames, 0);
            if human {
                assert_eq!(
                    (t.steering_lock, w.speed, t.speed),
                    (ORDINARY_BLOCKED_LOCK, 0xab20, 0xab20)
                );
            } else {
                assert_eq!(
                    (t.steering_lock, w.speed, t.speed),
                    (AI_RECOVERY_LOCK, 0xab81, 0xab81)
                );
            }
        }
    }
    #[test]
    fn contact_effect_checks_existing_projection_byte_and_x_plus_four_carry() {
        for (screen_x, cooldown, expected) in [
            (0x100, 0, None),
            (252, 0, None),
            (251, 1, None),
            (251, 0, Some(EffectSprite { x: 255, y: 0 })),
        ] {
            let mut t = truck();
            t.screen_x = screen_x;
            t.screen_y = 250;
            t.contact_effect_player = cooldown.into();
            assert_eq!(contact_effect(&mut t), expected);
            assert_eq!(
                t.contact_effect_player.encoded(),
                if expected.is_some() {
                    CONTACT_EFFECT_ENTRY
                } else {
                    cooldown
                }
            );
        }
    }
    #[test]
    fn both_blocked_rejects_position_and_speed_but_keeps_new_heading_samples_and_history_boundary()
    {
        let mut t = truck();
        t.speed = 0xab40;
        t.surface_summary = 19;
        t.wheels[0].sampled = 99;
        let mut w = WorkingMotion {
            world: [0x1200, 0x1300],
            speed: 0xab7c,
            heading: 8,
        };
        resolve_legacy_blocked_motion(&mut t, &mut w, [0, 0, 1, 1], true);
        assert_eq!(
            w,
            WorkingMotion {
                world: START_WORLD,
                speed: 0xabe0,
                heading: 8
            }
        );
        assert_eq!(
            (t.surface_summary, t.wheels[0].sampled, t.blocked_frames),
            (19, 99, 1)
        );
    }
    #[test]
    fn one_blocked_axis_rotates_heading_and_restores_coordinate_by_original_quadrant() {
        for (heading, flags, expected_heading, expected_world) in [
            (0, [0, 0, 1, 0], 252, [0x1200, START_WORLD[1]]),
            (64, [0, 0, 1, 0], 60, [START_WORLD[0], 0x1300]),
            (0, [0, 0, 0, 1], 4, [START_WORLD[0], 0x1300]),
            (64, [0, 0, 0, 1], 68, [0x1200, START_WORLD[1]]),
        ] {
            let mut t = truck();
            let mut w = WorkingMotion {
                world: [0x1200, 0x1300],
                speed: 20,
                heading,
            };
            resolve_legacy_blocked_motion(&mut t, &mut w, flags, true);
            assert_eq!(
                (w.heading, w.world, t.blocked_frames),
                (expected_heading, expected_world, 1)
            );
        }
    }
    #[test]
    fn reverse_motion_rotates_front_flags_into_back_pair_before_axis_response() {
        let mut t = truck();
        let mut w = WorkingMotion {
            world: [0x1200, 0x1300],
            speed: 0x81,
            heading: 0,
        };
        let r = resolve_legacy_blocked_motion(&mut t, &mut w, [1, 0, 0, 0], true);
        assert_eq!(
            r.blocked
                .map(|value| value.map_or(0, |direction| direction.0.wrapping_add(1))),
            [1, 0, 0, 1]
        );
        assert_eq!((w.heading, w.world), (4, [START_WORLD[0], 0x1300]));
    }
    #[test]
    fn airborne_to_ground_transition_spawns_once_and_unblocked_clears_stuck_count() {
        let mut t = truck();
        t.contact_marker = 0;
        t.blocked_frames = 9;
        let mut w = working(&t);
        let r = resolve_legacy_blocked_motion(&mut t, &mut w, [0; 4], true);
        assert!(r.effects.is_empty());
        assert_eq!((t.contact_transition, t.blocked_frames), (1, 0));
        t.contact_marker = CONTACT;
        let r = resolve_legacy_blocked_motion(&mut t, &mut w, [0; 4], true);
        assert_eq!(
            r.effects,
            vec![MotionEffect::ContactSprite(EffectSprite { x: 104, y: 96 })]
        );
        assert_eq!(
            (t.contact_transition, t.contact_effect_player.encoded()),
            (0, CONTACT_EFFECT_ENTRY)
        );
        assert!(
            resolve_legacy_blocked_motion(&mut t, &mut w, [0; 4], true)
                .effects
                .is_empty()
        );
    }
    #[test]
    fn shadow_uses_wrapping_pair_sums_and_preserves_other_truck_x_bits_and_sizes() {
        let data = tables();
        let mut t = truck();
        t.tile_attributes = 0xffff;
        for w in &mut t.wheels {
            w.sampled = 200;
        }
        assert_eq!(
            shadow_sprite(&t, 0, 0, 0x55, &data),
            ShadowSprite {
                x: 100,
                y: 198,
                tile_attributes: 0x39ff,
                packed_high: 0xfe
            }
        );
        t.screen_x = 0x164;
        assert_eq!(shadow_sprite(&t, 0, 0, 0x54, &data).packed_high, 0xff);
        for (slot, mask) in [(0, 1), (1, 4), (2, 16), (3, 64)] {
            assert_eq!(
                shadow_sprite(&t, slot, 0, 0, &data).packed_high,
                0xaa | mask
            );
        }
    }
    #[test]
    fn shadow_projection_term_and_low_byte_y_wrap_are_explicit() {
        let mut t = truck();
        t.tile_attributes = 0x4200;
        t.screen_x = 0xff;
        assert_eq!(
            shadow_sprite(&t, 1, 0x2c00, 0, &tables()),
            ShadowSprite {
                x: 255,
                y: 22,
                tile_attributes: 0x0800,
                packed_high: 0xaa
            }
        );
    }
    fn flat_world() -> (Terrain, SamplingTables) {
        let mut cells = vec![BASE_HEIGHT; TERRAIN_BYTES];
        for row in cells.chunks_mut(ROW_BYTES) {
            for cell in row.chunks_exact_mut(CELL_BYTES) {
                cell[2] = 0;
            }
        }
        let bytes = std::fs::read("../input/game.rom").unwrap();
        let sine = std::array::from_fn(|i| bytes[0x1dc7 + i] as i8);
        let phases = bytes[0x196a..0x196e].try_into().unwrap();
        let row_offsets = std::array::from_fn(|i| {
            u16::from_le_bytes(bytes[0x1f87 + i * 2..0x1f89 + i * 2].try_into().unwrap())
        });
        (
            Terrain::from_fixture_bytes(cells),
            SamplingTables::from_source_tables(sine, phases, row_offsets),
        )
    }
    #[test]
    fn four_truck_service_preserves_proposal_object_projection_pair_commit_order_and_input_latency()
    {
        let data = tables();
        let (terrain, sampling) = flat_world();
        let mut fleet = std::array::from_fn(|slot| {
            let mut t = truck();
            t.world[0] += slot as u16 * 0x400;
            t.speed = 0;
            t.contact_cooldown = 1;
            t
        });
        let env = MotionEnvironment {
            tables: &data,
            terrain: &terrain,
            sampling: &sampling,
            projection: Projection {
                offset: [0; 2],
                scale: [103, 67],
            },
            original_track: 0,
            active_humans: 15,
            prepared: false,
            animation: 0,
        };
        let mut history = 0;
        let mut contacts =
            std::array::from_fn(|_| crate::world::objects::ContactAnimation::default());
        let bearings = super::super::bearings::BearingTable::load(
            &std::fs::read("../input/game.rom").unwrap(),
        )
        .unwrap();
        let phases = update_trucks(&mut fleet, &env, &mut history, &mut contacts, &bearings)
            .unwrap()
            .phases;
        let expected = (0..TRUCKS)
            .flat_map(|s| {
                [
                    MotionPhase::Powertrain(s),
                    MotionPhase::Displacement(s),
                    MotionPhase::SampleTerrain(s),
                    MotionPhase::SpecialSurface(s),
                    MotionPhase::ContactResponse(s),
                    MotionPhase::SurfaceAverage(s),
                    MotionPhase::Project(s),
                    MotionPhase::Shadow(s),
                    MotionPhase::Cooldown(s),
                    MotionPhase::Commit(s),
                ]
            })
            .collect::<Vec<_>>();
        assert_eq!(phases, expected);
        assert!(
            fleet
                .iter()
                .all(|t| t.contact_cooldown == 0 && t.speed == 0)
        );
        assert_eq!(history, 0);
        // Source input copy/AI run later; this service consumes retained words.
        assert!(fleet.iter().all(|t| t.controller == ControllerWord(0)));
        let phases = update_trucks(&mut fleet, &env, &mut history, &mut contacts, &bearings)
            .unwrap()
            .phases;
        for slot in 0..TRUCKS {
            assert!(phases.contains(&MotionPhase::PairContact(slot)));
            assert!(!phases.contains(&MotionPhase::Cooldown(slot)));
        }
    }
}
