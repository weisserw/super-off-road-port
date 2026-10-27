//! Race lifetime, shared progress and source HUD/banner rules. Independent
//! branch/width/cross-owner derivations and contract coverage: race.trace.md.
//! All behavioral interfaces remain visibly pending for stage 4.
pub mod assets;
use crate::{
    controls::ControllerWord,
    runtime::RandomState,
    trucks::{
        ai::{AiTables, Driver},
        motion::{MotionEnvironment, Truck},
    },
    world::tracks::{Grid, Route, Selection, TRUCKS},
};
use anyhow::{Context, Result, ensure};
use assets::RaceAssets;

pub const ORIGINAL_BANNER_TRACKS: usize = 16;
pub const PRIMARY_BANNER_TILES: usize = 35;
pub const SECONDARY_BANNER_TILES: usize = 6;
pub const LAST_LAP_THRESHOLD: u8 = 4;
pub const WINNING_LAP: u8 = 5;
pub const LAST_LAP_BANNER: u16 = 30;
pub const PRIMARY_DECAY_THRESHOLD: u16 = 19;
pub const SECONDARY_TRIGGER: u16 = 15;
pub const INITIAL_AI_TIMEOUT: u8 = 200;
pub const PRIMARY_SPRITE_ATTRIBUTES: u16 = 0x3584;
pub const HIDDEN_SPRITE_COORDINATES: u16 = 0xf0f0;
pub const TIMER_DIGIT_TILE_BASE: u8 = 0x15;
pub const ORIGINAL_ORDER_MASK: u8 = 63;

/// Single race authority shared by human and computer checkpoint consumers.
/// AI retains `RaceAi` as an alias to this type, never a second copied owner.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Progress {
    #[serde(with = "crate::snapshot::array")]
    pub drivers: [Driver; TRUCKS],
    pub active_humans: u8,
    pub animation_frame: u8,
    pub winner_plus_one: u8,
    /// Source word; the last-lap helper stores only its low byte.
    pub banner_state: u16,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RaceTimer {
    pub fraction_bcd: u8,
    pub middle_bcd: u8,
    pub high_bcd: u8,
}
/// Increment is source encoded decimal16, not claimed wall-clock milliseconds.
/// Winner/prepared guards do not exist inside this HUD-time operation.
pub fn advance_timer(timer: &mut RaceTimer) -> [u8; 3] {
    const FRACTION_STEP: u8 = 0x16;
    const MAX_HIGH: u8 = 0x09;
    const MAX_MIDDLE: u8 = 0x99;
    const NIBBLE_MASK: u8 = 0x0f;
    fn decimal_add(value: u8, increment: u8, carry: bool) -> (u8, bool) {
        let mut low =
            u16::from(value & NIBBLE_MASK) + u16::from(increment & NIBBLE_MASK) + u16::from(carry);
        let low_carry = low > 9;
        if low_carry {
            low -= 10;
        }
        let mut high = u16::from(value >> 4) + u16::from(increment >> 4) + u16::from(low_carry);
        let carry = high > 9;
        if carry {
            high -= 10;
        }
        (((high << 4) | low) as u8, carry)
    }
    if timer.high_bcd < MAX_HIGH || timer.middle_bcd < MAX_MIDDLE {
        let (fraction, carry) = decimal_add(timer.fraction_bcd, FRACTION_STEP, false);
        let (middle, carry) = decimal_add(timer.middle_bcd, 0, carry);
        let (high, _) = decimal_add(timer.high_bcd, 0, carry);
        timer.fraction_bcd = fraction;
        timer.middle_bcd = middle;
        timer.high_bcd = high;
    }
    [
        timer.high_bcd & NIBBLE_MASK,
        timer.middle_bcd >> 4,
        timer.middle_bcd & NIBBLE_MASK,
    ]
    .map(|digit| digit.wrapping_add(TIMER_DIGIT_TILE_BASE))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BannerSprite {
    pub coordinates: u16,
    pub tile_attributes: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BannerDisplay {
    pub primary: BannerSprite,
    pub secondary: BannerSprite,
    pub secondary_state: u16,
    /// Two track-eight fixed sprites; other tracks do not overwrite these slots.
    #[serde(with = "crate::snapshot::array")]
    pub fixed_pair: [BannerSprite; 2],
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BannerTables {
    pub coordinates: [u16; ORIGINAL_BANNER_TRACKS],
    pub primary_tiles: [u16; PRIMARY_BANNER_TILES],
    pub secondary_tiles: [u16; SECONDARY_BANNER_TILES],
}
pub fn load_banner_tables(rom: &[u8]) -> Result<BannerTables> {
    const COORDINATES_ROM: usize = 0xf38;
    const PRIMARY_ROM: usize = 0xfec;
    const SECONDARY_ROM: usize = 0xfe0;
    fn words<const N: usize>(rom: &[u8], at: usize) -> Result<[u16; N]> {
        let bytes = rom.get(at..at + N * 2).context("truncated banner table")?;
        Ok(std::array::from_fn(|index| {
            u16::from_le_bytes([bytes[index * 2], bytes[index * 2 + 1]])
        }))
    }
    Ok(BannerTables {
        coordinates: words(rom, COORDINATES_ROM)?,
        primary_tiles: words(rom, PRIMARY_ROM)?,
        secondary_tiles: words(rom, SECONDARY_ROM)?,
    })
}
/// Added named tracks can provide their own validated layout. The original
/// sixteen-entry lookup bounds only the ROM table, not the native catalogue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BannerLayout {
    pub coordinates: u16,
    pub install_fixed_pair: bool,
}
pub fn original_banner_layout(track: u8, tables: &BannerTables) -> Result<BannerLayout> {
    const FIXED_PAIR_TRACK: u8 = 8;
    Ok(BannerLayout {
        coordinates: *tables
            .coordinates
            .get(usize::from(track))
            .context("invalid original banner track")?,
        install_fixed_pair: track == FIXED_PAIR_TRACK,
    })
}
pub fn initialize_banner(
    progress: &mut Progress,
    display: &mut BannerDisplay,
    layout: BannerLayout,
) {
    const FIXED_FIRST_COORDINATES: u16 = 0x2760;
    const FIXED_SECOND_COORDINATES: u16 = 0x2770;
    const FIXED_FIRST_TILES: u16 = 0x3180;
    const FIXED_SECOND_TILES: u16 = 0x3182;
    progress.banner_state = 0;
    display.secondary_state = 0;
    display.primary = BannerSprite {
        coordinates: layout.coordinates,
        tile_attributes: PRIMARY_SPRITE_ATTRIBUTES,
    };
    if layout.install_fixed_pair {
        display.fixed_pair = [
            BannerSprite {
                coordinates: FIXED_FIRST_COORDINATES,
                tile_attributes: FIXED_FIRST_TILES,
            },
            BannerSprite {
                coordinates: FIXED_SECOND_COORDINATES,
                tile_attributes: FIXED_SECOND_TILES,
            },
        ];
    }
}
pub fn request_last_lap(progress: &mut Progress, slot: usize) -> Result<()> {
    let driver = progress
        .drivers
        .get(slot)
        .ok_or_else(|| anyhow::anyhow!("invalid lap truck slot"))?;
    if driver.lap >= LAST_LAP_THRESHOLD
        && (progress.banner_state as u8) < PRIMARY_DECAY_THRESHOLD as u8
    {
        progress.banner_state = (progress.banner_state & !u16::from(u8::MAX)) | LAST_LAP_BANNER;
    }
    Ok(())
}
pub fn animate_banner(
    progress: &mut Progress,
    display: &mut BannerDisplay,
    tables: &BannerTables,
) -> Result<()> {
    const PHASE_MASK: u8 = 3;
    const SECONDARY_START: u16 = 5;
    const SECONDARY_X_OFFSET: u16 = 4;
    const SECONDARY_Y_STEP: u16 = 0x100;
    ensure!(
        usize::from(progress.banner_state) < tables.primary_tiles.len()
            && usize::from(display.secondary_state) < tables.secondary_tiles.len(),
        "invalid banner selector"
    );
    let phase = progress.animation_frame & PHASE_MASK;
    if progress.banner_state >= PRIMARY_DECAY_THRESHOLD
        && progress.winner_plus_one == 0
        && phase == 0
    {
        progress.banner_state -= 1;
    }
    display.primary.tile_attributes = tables.primary_tiles[usize::from(progress.banner_state)];
    if display.secondary_state == 0 {
        display.secondary.coordinates = HIDDEN_SPRITE_COORDINATES;
        if progress.banner_state != SECONDARY_TRIGGER {
            return Ok(());
        }
        display.secondary.coordinates =
            display.primary.coordinates.wrapping_add(SECONDARY_X_OFFSET);
        display.secondary_state = SECONDARY_START;
    }
    if phase == 0 {
        display.secondary_state -= 1;
    }
    if phase & 1 != 0 {
        display.secondary.coordinates =
            display.secondary.coordinates.wrapping_sub(SECONDARY_Y_STEP);
    }
    display.secondary.tile_attributes =
        tables.secondary_tiles[usize::from(display.secondary_state)];
    Ok(())
}
/// Both race consumers share the same center-world packed rectangle helper.
/// Wrapped unsigned acceptance excludes lower world edge and includes upper.
pub fn inside_checkpoint(world: [u16; 2], record: [u16; 2]) -> bool {
    const HALF_EXTENT_MASK: u16 = 0x1f;
    const CENTER_MASK: u16 = 0xbfe0;
    const EXTENT_SHIFT: u32 = 7;
    (0..2).all(|axis| {
        let half = (record[axis] & HALF_EXTENT_MASK) << EXTENT_SHIFT;
        (record[axis] & CENTER_MASK)
            .wrapping_sub(world[axis])
            .wrapping_add(half)
            < half * 2
    })
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HumanRouteVisits {
    pub tested_slots: Vec<usize>,
    pub accepted_slots: Vec<usize>,
    pub lap_slots: Vec<usize>,
}
/// Copy enabled automatic input first, then human slots3..0. FD/FE/bit14 records
/// skip without redirects/RNG; FF wraps/reinterprets immediately. Missing/cyclic
/// extension records return a native validation error rather than a host hang.
pub fn service_human_routes(
    trucks: &mut [Truck; TRUCKS],
    progress: &mut Progress,
    route: &Route,
    automatic: [ControllerWord; TRUCKS],
) -> Result<HumanRouteVisits> {
    const CONDITIONAL_MARKER: u8 = 0xfd;
    const REDIRECT_MARKER: u8 = 0xfe;
    const TERMINATOR: u8 = 0xff;
    const SKIP_BIT: u8 = 0x40;
    let mut next_trucks = trucks.clone();
    let mut next = progress.clone();
    let mut retained = std::array::from_fn(|slot| next_trucks[slot].controller);
    crate::controls::copy_human_words(&mut retained, next.active_humans, automatic);
    for slot in 0..TRUCKS {
        next_trucks[slot].controller = retained[slot];
    }
    let mut visits = HumanRouteVisits::default();
    for slot in (0..TRUCKS).rev() {
        if next.active_humans & (1 << slot) == 0 {
            continue;
        }
        let mut seen = [false; u8::MAX as usize + 1];
        let record = loop {
            let checkpoint = next.drivers[slot].checkpoint;
            ensure!(!seen[usize::from(checkpoint)], "control-only route cycle");
            seen[usize::from(checkpoint)] = true;
            let at = usize::from(checkpoint.wrapping_mul(4)) / 2;
            let x = *route.words.get(at).context("missing human route record")?;
            let high = (x >> u8::BITS) as u8;
            if high == TERMINATOR {
                next.drivers[slot].checkpoint = 0;
                continue;
            }
            if matches!(high, CONDITIONAL_MARKER | REDIRECT_MARKER) || high & SKIP_BIT != 0 {
                next.drivers[slot].checkpoint = checkpoint.wrapping_add(1);
                continue;
            }
            break [
                x,
                *route.words.get(at + 1).context("missing human route Y")?,
            ];
        };
        visits.tested_slots.push(slot);
        if !inside_checkpoint(next_trucks[slot].world, record) {
            continue;
        }
        visits.accepted_slots.push(slot);
        next.drivers[slot].checkpoint = next.drivers[slot].checkpoint.wrapping_add(1);
        if next.drivers[slot].checkpoint == 1 {
            next.drivers[slot].lap = next.drivers[slot].lap.wrapping_add(1);
            visits.lap_slots.push(slot);
            request_last_lap(&mut next, slot)?;
            if next.drivers[slot].lap == WINNING_LAP {
                next.winner_plus_one = slot as u8 + 1;
            }
        } else if next.drivers[slot].checkpoint >= route.checkpoint_count {
            next.drivers[slot].checkpoint = 0;
        }
    }
    *trucks = next_trucks;
    *progress = next;
    Ok(visits)
}
/// Explicit initialization writes before AI/grid/presentation preparation. Most
/// retained truck fields (upgrades/fractions/locks/nitro) are not zeroed here.
pub fn initialize_race_records(trucks: &mut [Truck; TRUCKS], progress: &mut Progress) {
    const INITIAL_ATTRIBUTES: u16 = 0x2000;
    const ATTRIBUTE_STEP: u16 = 0x202;
    const INITIAL_HEADING: u16 = 0x40;
    for (slot, truck) in trucks.iter_mut().enumerate() {
        truck.tile_attributes = INITIAL_ATTRIBUTES + ATTRIBUTE_STEP * slot as u16;
        truck.heading = INITIAL_HEADING;
        truck.speed = 0;
        progress.drivers[slot].checkpoint = 0;
        progress.drivers[slot].lap = 1;
    }
}
/// Reset follows AI-parameter setup; never reset its other retained state.
pub fn reset_route_timeouts(progress: &mut Progress) {
    for driver in &mut progress.drivers {
        driver.timeout = INITIAL_AI_TIMEOUT;
    }
}
/// Route loader changes checkpoint0→1 after setup. Grid applies the already
/// traced positions/headings; neither helper resets retained powertrain history.
pub fn apply_route_grid(trucks: &mut [Truck; TRUCKS], progress: &mut Progress, grid: Grid) {
    for (slot, truck) in trucks.iter_mut().enumerate() {
        truck.world = [grid.x[slot], grid.y[slot]];
        truck.heading = (truck.heading & !u16::from(u8::MAX)) | u16::from(grid.heading);
        progress.drivers[slot].checkpoint = 1;
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameExit {
    PreparedReturn,
    Continue,
    HumanPostrace,
    AttractRestart,
}
pub fn frame_exit(prepared: u8, active_humans: u8, winner_plus_one: u8) -> FrameExit {
    if prepared != 0 {
        FrameExit::PreparedReturn
    } else if winner_plus_one == 0 {
        FrameExit::Continue
    } else if active_humans == 0 {
        FrameExit::AttractRestart
    } else {
        FrameExit::HumanPostrace
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputDispatch {
    HumanPauseService,
    ContinueAttract,
    FrontendRestart,
}
/// Attract Start unwinds this race call rather than returning to update services.
/// The caller resets frontend active-mask/order as part of its own restart.
pub fn poll_race_start(
    active_humans: u8,
    automatic: [ControllerWord; TRUCKS],
    frontend_reentry: &mut u8,
) -> InputDispatch {
    if active_humans != 0 {
        return InputDispatch::HumanPauseService;
    }
    if automatic
        .iter()
        .any(|controller| controller.0 & crate::controls::BUTTON_START != 0)
    {
        *frontend_reentry = frontend_reentry.wrapping_add(1);
        InputDispatch::FrontendRestart
    } else {
        InputDispatch::ContinueAttract
    }
}
/// Called after the ready transition: always wait one boundary, send nothing in
/// attract, otherwise host command01 with selector(order&7)+6.
pub fn race_music_command(active_humans: u8, order_slot: u8) -> Option<u16> {
    const MUSIC_ORDER_MASK: u8 = 7;
    const FIRST_SELECTOR: u8 = 6;
    const SONG_COMMAND: u8 = 1;
    (active_humans != 0).then(|| {
        u16::from_le_bytes([
            SONG_COMMAND,
            (order_slot & MUSIC_ORDER_MASK) + FIRST_SELECTOR,
        ])
    })
}
/// Invoke only after results returns normally. Results can replace the order
/// slot, so advance the returned value, never a pre-results cached selection.
pub fn advance_original_order(
    postresults_slot: u8,
    order: &[Selection],
) -> Result<(u8, Selection)> {
    let slot = postresults_slot.wrapping_add(1) & ORIGINAL_ORDER_MASK;
    Ok((slot, crate::world::tracks::select_order(order, slot)?))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServicePhase {
    Motion,
    Wheels,
    Suspension,
    Poses,
    SpriteSort,
    SpritePack,
    SpriteAnimation,
    HumanRoutes,
    TrackObjects,
    Banner,
    Hud,
    Computers,
    Exit,
}
/// Every cursor has a single domain owner: trucks own nitro and contact-effect
/// cursors, world owns surface cursors, and these four remaining slots retain the
/// source's otherwise unselected middle group. The 16-slot renderer gets a view.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RaceVideo {
    pub memory: crate::presentation::VideoMemory,
    pub display: crate::presentation::Display,
    pub sprites: crate::presentation::sprites::SpriteArrays,
    #[serde(with = "crate::snapshot::array")]
    pub poses: [u16; TRUCKS],
    #[serde(with = "crate::snapshot::array")]
    pub middle_cursors: [u8; TRUCKS],
}
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RaceEntry {
    #[default]
    Running,
    FadeIn(crate::presentation::transitions::Fade),
    Ready,
    WaitForMusic,
}
/// Mutable race owners consumed by the shared frame driver.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RaceState {
    /// Byte at source route-buffer offset F7. All-human AI setup writes here;
    /// a sufficiently long later route copy replaces it. It is not an AI parameter.
    pub route_tail_byte: u8,
    pub world: crate::world::objects::WorldObjects,
    pub video: RaceVideo,
    pub sounds: Vec<u16>,
    pub entry: RaceEntry,
    pub working_pose: u8,
    #[serde(with = "crate::snapshot::array")]
    pub trucks: [Truck; TRUCKS],
    pub progress: Progress,
    pub timer: RaceTimer,
    pub banner: BannerDisplay,
    pub prepared: u8,
    pub terrain_history: u8,
    pub render_mode: u8,
    pub video_inhibit: u8,
    pub enabled_layers: u8,
}
pub struct FrameEnvironment<'a> {
    pub assets: &'a RaceAssets,
    pub motion: MotionEnvironment<'a>,
    pub route: &'a Route,
    pub ai: &'a AiTables,
    pub banners: &'a BannerTables,
    pub automatic: [ControllerWord; TRUCKS],
}
/// Real owner orchestration must update domain state as well as phase evidence.
/// Prepared service skips wait/input-dispatch and exits only AFTER all12 calls.
pub fn update_frame_services(
    race: &mut RaceState,
    environment: &FrameEnvironment<'_>,
    random: &mut RandomState,
) -> Result<(FrameExit, Vec<ServicePhase>)> {
    use crate::{
        presentation::{self, sprites},
        trucks::motion,
        world::objects,
    };
    let mut phases = Vec::new();
    race.render_mode = 2;
    phases.push(ServicePhase::Motion);
    let frame = motion::update_trucks(
        &mut race.trucks,
        &environment.motion,
        &mut race.terrain_history,
        &mut race.world.contacts,
        &environment.assets.contacts,
    )?;
    stage_motion_frame(race, frame);
    phases.push(ServicePhase::Wheels);
    motion::integrate_wheels(&mut race.trucks);
    phases.push(ServicePhase::Suspension);
    for truck in &mut race.trucks {
        motion::correct_suspension_speed(truck);
    }
    phases.push(ServicePhase::Poses);
    for slot in (0..TRUCKS).rev() {
        let pose = motion::body_pose(&race.trucks[slot], environment.motion.tables);
        race.video.poses[slot] = pose.source;
        race.working_pose = (race.trucks[slot].heading as u8) >> 3;
        if environment.motion.tables.tilt_poses
            [usize::from(pose.longitudinal) * 5 + usize::from(pose.lateral)]
            != 0
        {
            race.working_pose >>= 1;
        }
        // Only the strip source changes here; truck tile words are per-truck
        // tile/palette/priority assignments, not the pose-strip selector.
    }
    phases.push(ServicePhase::SpriteSort);
    sprites::sort_by_y(&mut race.video.sprites);
    phases.push(ServicePhase::SpritePack);
    sprites::pack_sorted(&race.video.sprites, &mut race.video.memory.staged_oam)?;
    phases.push(ServicePhase::SpriteAnimation);
    let mut cursors = cursor_view(race);
    sprites::animate(
        &mut race.video.memory,
        &mut cursors,
        race.progress.animation_frame,
        &environment.assets.animations,
    )?;
    apply_cursor_view(race, cursors);
    phases.push(ServicePhase::HumanRoutes);
    service_human_routes(
        &mut race.trucks,
        &mut race.progress,
        environment.route,
        environment.automatic,
    )?;
    phases.push(ServicePhase::TrackObjects);
    let (_, sounds) = objects::service_world(
        &mut race.world,
        &environment.assets.objects,
        &mut race.trucks,
        race.progress.animation_frame,
        race.progress.active_humans,
        random,
        environment.motion.projection,
        &environment.assets.object_tables,
    )?;
    race.sounds.extend(sounds);
    stage_world(
        race,
        environment.motion.projection,
        &environment.assets.objects,
    );
    phases.push(ServicePhase::Banner);
    animate_banner(&mut race.progress, &mut race.banner, environment.banners)?;
    stage_banner(race);
    phases.push(ServicePhase::Hud);
    presentation::update_hud_digits(
        &mut race.video.memory,
        race.progress.drivers.map(|driver| driver.lap),
        race.trucks.each_ref().map(|truck| truck.nitro_stock_bcd),
        &mut race.timer,
    );
    phases.push(ServicePhase::Computers);
    crate::trucks::ai::service_computers(
        &mut race.trucks,
        &mut race.progress,
        environment.route,
        environment.ai,
        random,
    )?;
    phases.push(ServicePhase::Exit);
    Ok((
        frame_exit(
            race.prepared,
            race.progress.active_humans,
            race.progress.winner_plus_one,
        ),
        phases,
    ))
}

/// Preparation runs these calls in order; waits/fade/ready are resumed through
/// the transitions owner rather than blocking the host thread.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreparationPhase {
    BlankAndConfigureLayers,
    FontAndClearBackground,
    TruckRecords,
    AiParameters,
    ResetTimeouts,
    ClearWheels,
    ClearSpriteAnimations,
    LoadRoute,
    LoadTrackAssets,
    StartGrid,
    SharedSpritePalette,
    SharedSpriteTiles,
    RaceOam,
    PackSpriteArrays,
    HudAndTimer,
    TrackObjects,
    Banner,
    EnableFrameBoundary,
    PreparedServices,
    ClearPrepared,
    FadeIn,
    ReadyTransition,
    WaitAndMusic,
}
/// Composition of the actual declared owners: loads/prepares native state,
/// performs one service call, then enters resumable fade/ready. The environment
/// carries real domain data, never callbacks that implement test expectations.
pub fn prepare_selected_race(
    race: &mut RaceState,
    environment: &FrameEnvironment<'_>,
    grid: Grid,
    random: &mut RandomState,
) -> Result<Vec<PreparationPhase>> {
    use crate::{
        presentation::{self, memory, sprites},
        world::{
            objects,
            terrain::{self, Surface},
        },
    };
    let mut phases = Vec::new();
    macro_rules! phase {
        ($name:ident) => {
            phases.push(PreparationPhase::$name);
        };
    }
    phase!(BlankAndConfigureLayers);
    race.video_inhibit = 0x80;
    race.video.display.force_blank = true;
    presentation::configure_race(&mut race.video.display);
    phase!(FontAndClearBackground);
    memory::upload_fonts(&mut race.video.memory, &environment.assets.graphics, false)?;
    phase!(TruckRecords);
    initialize_race_records(&mut race.trucks, &mut race.progress);
    phase!(AiParameters);
    if let Some(byte) = crate::trucks::ai::setup_parameters(&mut race.trucks, &mut race.progress)
        .unused_route_tail_byte
    {
        race.route_tail_byte = byte;
    }
    phase!(ResetTimeouts);
    reset_route_timeouts(&mut race.progress);
    phase!(ClearWheels);
    for truck in &mut race.trucks {
        for wheel in &mut truck.wheels {
            wheel.sampled = 0;
            wheel.height = 0;
            wheel.velocity = 0;
        }
    }
    phase!(ClearSpriteAnimations);
    apply_cursor_view(race, [0; crate::presentation::SORTED_SPRITES]);
    phase!(LoadRoute);
    if let Some(word) = environment.route.words.get(0xf7 / 2) {
        race.route_tail_byte = word.to_le_bytes()[1];
    }
    for driver in &mut race.progress.drivers {
        driver.checkpoint = 1;
    }
    phase!(LoadTrackAssets);
    let palette: [u16; presentation::PALETTE_COLORS] = environment
        .assets
        .graphics
        .race_palette
        .clone()
        .try_into()
        .map_err(|_| anyhow::anyhow!("invalid race palette"))?;
    presentation::install_track_video(&mut race.video.memory, &environment.assets.track, &palette)?;
    phase!(StartGrid);
    apply_route_grid(&mut race.trucks, &mut race.progress, grid);
    let probe = terrain::wheel_probe(
        environment.motion.sampling,
        grid.initial_surface_origin,
        race.working_pose,
        0,
    );
    let sample = terrain::sample_at(
        environment.motion.terrain,
        environment.motion.sampling,
        probe,
        environment.motion.original_track,
        race.terrain_history,
    )?;
    race.terrain_history = sample.control_history;
    // The source grid consumes the returned byte even if the probe reports a
    // block direction; no invented zero height replaces that conditional result.
    let height = match sample.surface {
        Surface::Height(height) => height,
        Surface::Blocked(direction) => direction.0,
    };
    for truck in &mut race.trucks {
        truck.surface_summary = height;
        for wheel in &mut truck.wheels {
            wheel.height = height;
        }
    }
    phase!(SharedSpritePalette);
    memory::upload_sprite_palette(&mut race.video.memory, &environment.assets.graphics)?;
    phase!(SharedSpriteTiles);
    memory::upload_shared_sprites(&mut race.video.memory, &environment.assets.graphics)?;
    phase!(RaceOam);
    presentation::initialize_race_oam(&mut race.video.memory, &mut race.video.display);
    phase!(PackSpriteArrays);
    for slot in 0..TRUCKS {
        race.video.sprites.tile_attributes[slot] = race.trucks[slot].tile_attributes;
    }
    sprites::initialize_arrays(&mut race.video.sprites);
    for slot in 0..TRUCKS {
        race.trucks[slot].screen_x = race.video.sprites.x[slot];
        race.trucks[slot].screen_y = race.video.sprites.y[slot];
    }
    sprites::pack_sorted(&race.video.sprites, &mut race.video.memory.staged_oam)?;
    phase!(HudAndTimer);
    presentation::initialize_hud(
        &mut race.video.memory,
        &environment.assets.rom,
        &environment.assets.presentation,
        &environment.assets.hud,
        environment.assets.reversed,
        &mut race.timer,
    )?;
    phase!(TrackObjects);
    seed_retained_sprite_views(race);
    race.sounds.extend(objects::initialize_world(
        &mut race.world,
        &environment.assets.objects,
        &mut race.trucks,
        race.progress.animation_frame,
        race.progress.active_humans,
        random,
        environment.motion.projection,
        &environment.assets.object_tables,
    )?);
    stage_world(
        race,
        environment.motion.projection,
        &environment.assets.objects,
    );
    phase!(Banner);
    initialize_banner(
        &mut race.progress,
        &mut race.banner,
        environment.assets.banner,
    );
    stage_banner(race);
    phase!(EnableFrameBoundary);
    race.prepared = 1;
    race.progress.winner_plus_one = 0;
    race.video_inhibit = 0;
    race.enabled_layers = 0x1f;
    race.video.display.main_layers = race.enabled_layers;
    phase!(PreparedServices);
    update_frame_services(race, environment, random)?;
    phase!(ClearPrepared);
    race.prepared = race.prepared.wrapping_sub(1);
    phase!(FadeIn);
    race.entry = RaceEntry::FadeIn(presentation::transitions::Fade {
        direction: presentation::transitions::FadeDirection::In,
        primed: false,
        complete: false,
        animation_counter: race.progress.animation_frame,
        enabled_layers: race.enabled_layers,
    });
    // These are scheduled continuation phases. The Game frame driver advances
    // the real fade, ready window and final NMI/music wait; preparation never
    // collapses their frame boundaries into a synchronous host call.
    phase!(ReadyTransition);
    phase!(WaitAndMusic);
    Ok(phases)
}
fn cursor_view(race: &RaceState) -> [u8; crate::presentation::SORTED_SPRITES] {
    std::array::from_fn(|slot| match slot / TRUCKS {
        0 => race.trucks[slot].nitro_animation,
        1 => race.world.contacts[slot - TRUCKS].cursor,
        2 => race.video.middle_cursors[slot - TRUCKS * 2],
        _ => race.trucks[slot - TRUCKS * 3].effect_cooldown,
    })
}
fn apply_cursor_view(race: &mut RaceState, cursors: [u8; crate::presentation::SORTED_SPRITES]) {
    for slot in 0..TRUCKS {
        race.trucks[slot].nitro_animation = cursors[slot];
        race.world.contacts[slot].cursor = cursors[slot + TRUCKS];
        race.video.middle_cursors[slot] = cursors[slot + TRUCKS * 2];
        race.trucks[slot].effect_cooldown = cursors[slot + TRUCKS * 3];
    }
}
fn seed_retained_sprite_views(race: &mut RaceState) {
    use crate::world::objects::ObjectSprite;
    let read = |offset: usize| ObjectSprite {
        coordinates: u16::from_le_bytes(
            race.video.memory.staged_oam[offset..offset + 2]
                .try_into()
                .unwrap(),
        ),
        tile_attributes: u16::from_le_bytes(
            race.video.memory.staged_oam[offset + 2..offset + 4]
                .try_into()
                .unwrap(),
        ),
    };
    for part in 0..2 {
        race.world.pickup.sprites[part] = read(0x190 + part * 4);
    }
    for (part, object) in race.world.dynamic.iter_mut().enumerate() {
        object.sprite = read(0x198 + part * 4);
    }
    for (slot, contact) in race.world.contacts.iter_mut().enumerate() {
        contact.sprite = read(0x160 + slot * 4);
    }
    let banner = |offset| {
        let sprite = read(offset);
        BannerSprite {
            coordinates: sprite.coordinates,
            tile_attributes: sprite.tile_attributes,
        }
    };
    race.banner.secondary = banner(0x84);
    race.banner.fixed_pair = [banner(0x88), banner(0x8c)];
}
fn stage_sprite(
    memory: &mut crate::presentation::VideoMemory,
    offset: usize,
    coordinates: u16,
    attributes: u16,
) {
    memory.staged_oam[offset..offset + 2].copy_from_slice(&coordinates.to_le_bytes());
    memory.staged_oam[offset + 2..offset + 4].copy_from_slice(&attributes.to_le_bytes());
}
fn stage_motion_frame(race: &mut RaceState, frame: crate::trucks::motion::FleetFrame) {
    use crate::trucks::motion::MotionEffect;
    const SHADOW_BASE: usize = 0x140;
    const SHADOW_HIGH: usize = 0x214;
    const NITRO_BASE: usize = 0x150;
    const SURFACE_BASE: usize = 0x160;
    const CONTACT_BASE: usize = 0x180;
    for slot in 0..TRUCKS {
        let truck = &race.trucks[slot];
        race.video.sprites.x[slot] = truck.screen_x;
        race.video.sprites.y[slot] = truck.screen_y;
        race.video.sprites.tile_attributes[slot] = truck.tile_attributes;
        for wheel in 0..4 {
            race.video.sprites.wheel_sample_heights[slot * 4 + wheel] = truck.wheels[wheel].sampled;
        }
        let shadow = frame.shadows[slot];
        stage_sprite(
            &mut race.video.memory,
            SHADOW_BASE + slot * 4,
            u16::from_le_bytes([shadow.x, shadow.y]),
            shadow.tile_attributes,
        );
        race.video.memory.staged_oam[SHADOW_HIGH] = shadow.packed_high;
    }
    for (slot, sprite) in frame.surface_sprites {
        let at = SURFACE_BASE + slot * 4;
        race.video.memory.staged_oam[at..at + 2].copy_from_slice(&sprite.coordinates.to_le_bytes());
    }
    for (slot, effect) in frame.effects {
        let (offset, sprite) = match effect {
            MotionEffect::Sound(command) => {
                race.sounds.push(command);
                continue;
            }
            MotionEffect::NitroSprite(sprite) => (NITRO_BASE, sprite),
            MotionEffect::ContactSprite(sprite) => (CONTACT_BASE, sprite),
        };
        let at = offset + slot * 4;
        race.video.memory.staged_oam[at] = sprite.x;
        race.video.memory.staged_oam[at + 1] = sprite.y;
    }
    for (_, visit) in frame.contacts {
        if let Some(sound) = visit.sound {
            race.sounds.push(sound);
        }
    }
}
fn stage_banner(race: &mut RaceState) {
    const BANNER_BASE: usize = 0x80;
    stage_sprite(
        &mut race.video.memory,
        BANNER_BASE,
        race.banner.primary.coordinates,
        race.banner.primary.tile_attributes,
    );
    stage_sprite(
        &mut race.video.memory,
        BANNER_BASE + 4,
        race.banner.secondary.coordinates,
        race.banner.secondary.tile_attributes,
    );
    for (part, sprite) in race.banner.fixed_pair.iter().enumerate() {
        stage_sprite(
            &mut race.video.memory,
            BANNER_BASE + 8 + part * 4,
            sprite.coordinates,
            sprite.tile_attributes,
        );
    }
}
fn stage_world(
    race: &mut RaceState,
    projection: crate::world::terrain::Projection,
    content: &crate::world::objects::ObjectContent,
) {
    const PICKUP_BASE: usize = 0x190;
    const DYNAMIC_BASE: usize = 0x198;
    const COMPOSITE_BASE: usize = 0x40;
    for (part, sprite) in race.world.pickup.sprites.iter().enumerate() {
        stage_sprite(
            &mut race.video.memory,
            PICKUP_BASE + part * 4,
            sprite.coordinates,
            sprite.tile_attributes,
        );
    }
    for (part, object) in race.world.dynamic.iter().enumerate() {
        // Inactive slots are source-retained sprite state, not implicitly hidden.
        stage_sprite(
            &mut race.video.memory,
            DYNAMIC_BASE + part * 4,
            object.sprite.coordinates,
            object.sprite.tile_attributes,
        );
    }
    if content.composites.is_some()
        && let Some(composites) = &race.world.composites
    {
        for (index, triplet) in composites.sprites.iter().enumerate() {
            for (part, sprite) in triplet.iter().enumerate() {
                stage_sprite(
                    &mut race.video.memory,
                    COMPOSITE_BASE + index * 12 + part * 4,
                    sprite.coordinates,
                    sprite.tile_attributes,
                );
            }
        }
    }
    if content.track_three_sprites {
        for part in 0..3 {
            let sprite = race.world.special_staging[part];
            race.video.sprites.x[10 + part] = u16::from(sprite.coordinates as u8);
            race.video.sprites.y[10 + part] = (sprite.coordinates >> 8) as u8;
            race.video.sprites.tile_attributes[10 + part] = sprite.tile_attributes;
            let upper = race.world.special_upper[part];
            stage_sprite(
                &mut race.video.memory,
                COMPOSITE_BASE + part * 4,
                upper.coordinates,
                upper.tile_attributes,
            );
        }
    }
    for (part, object) in race.world.placements.iter().enumerate() {
        let slot = part + TRUCKS;
        if slot >= crate::presentation::SORTED_SPRITES {
            break;
        }
        let position = crate::world::terrain::project(
            projection,
            object.placement.world,
            object.placement.height as u8,
        );
        race.video.sprites.x[slot] = position.x.wrapping_sub(0x40);
        race.video.sprites.y[slot] = (object.sprite.coordinates >> 8) as u8;
        race.video.sprites.tile_attributes[slot] = object.sprite.tile_attributes;
    }
}

/// Source transition calls use the same live banner and cursor owners as racing.
pub fn service_transition_visuals(
    race: &mut RaceState,
    assets: &RaceAssets,
    banners: &BannerTables,
    banner_calls: u32,
    animation_calls: u32,
) -> Result<()> {
    for _ in 0..banner_calls {
        animate_banner(&mut race.progress, &mut race.banner, banners)?;
        stage_banner(race);
    }
    for _ in 0..animation_calls {
        let mut cursors = cursor_view(race);
        crate::presentation::sprites::animate(
            &mut race.video.memory,
            &mut cursors,
            race.progress.animation_frame,
            &assets.animations,
        )?;
        apply_cursor_view(race, cursors);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        controls::{BUTTON_B, BUTTON_START},
        trucks::motion::{MotionTables, Wheel},
        world::{
            terrain::{CELL_BYTES, Projection, ROW_BYTES, SamplingTables, Terrain},
            tracks::{Direction, TERRAIN_BYTES},
        },
    };
    const CHECKPOINT_CENTER: [u16; 2] = [0x2000, 0x3000];
    const CHECKPOINT_RECORD: [u16; 2] = [0x2002, 0x3002];
    const TIMER_STEP: u8 = 0x16;
    const PRESERVED_UPPER_BYTE: u16 = 0xab00;
    const ALL_HUMANS: u8 = 0x0f;
    const NO_INPUT: [ControllerWord; TRUCKS] = [ControllerWord(0); TRUCKS];
    fn progress() -> Progress {
        Progress {
            drivers: [Driver {
                checkpoint: 1,
                lap: 1,
                timeout: 200,
                bias_delay: 7,
                parameter: 3,
            }; TRUCKS],
            active_humans: ALL_HUMANS,
            animation_frame: 0,
            winner_plus_one: 0,
            banner_state: 0,
        }
    }
    fn trucks() -> [Truck; TRUCKS] {
        std::array::from_fn(|_| Truck {
            world: CHECKPOINT_CENTER,
            ..Truck::default()
        })
    }
    fn route() -> Route {
        Route {
            words: vec![0x2002, 0x3002, 0x2002, 0x3002, 0x2002, 0x3002, 0xff03],
            checkpoint_count: 3,
        }
    }
    fn display() -> BannerDisplay {
        BannerDisplay {
            primary: BannerSprite {
                coordinates: 0x842c,
                tile_attributes: 0x3584,
            },
            secondary: BannerSprite {
                coordinates: 0x1234,
                tile_attributes: 0x789a,
            },
            secondary_state: 0,
            fixed_pair: [BannerSprite {
                coordinates: 0x1111,
                tile_attributes: 0x2222,
            }; 2],
        }
    }
    // Parse fixture words only; expectations below are independent literals.
    // Named physical ranges are loader metadata, not executable game constants.
    const BANNER_COORDINATES_ROM: std::ops::Range<usize> = 0xf38..0xf58;
    const PRIMARY_TILES_ROM: std::ops::Range<usize> = 0xfec..0x1032;
    const SECONDARY_TILES_ROM: std::ops::Range<usize> = 0xfe0..0xfec;
    fn source_tables() -> BannerTables {
        let rom = std::fs::read("../input/game.rom").unwrap();
        fn words<const N: usize>(bytes: &[u8]) -> [u16; N] {
            std::array::from_fn(|i| u16::from_le_bytes([bytes[i * 2], bytes[i * 2 + 1]]))
        }
        BannerTables {
            coordinates: words(&rom[BANNER_COORDINATES_ROM]),
            primary_tiles: words(&rom[PRIMARY_TILES_ROM]),
            secondary_tiles: words(&rom[SECONDARY_TILES_ROM]),
        }
    }
    #[test]
    fn banner_rom_loader_preserves_all_track_and_word_tile_entries_and_bounds() {
        let rom = std::fs::read("../input/game.rom").unwrap();
        let tables = load_banner_tables(&rom).unwrap();
        assert_eq!(
            tables.coordinates,
            [
                0x842c, 0xb87c, 0xac2c, 0xb768, 0xac5e, 0xb768, 0xb87c, 0xbc5e, 0xbc5e, 0xbc1e,
                0xb466, 0xb47e, 0xb44e, 0xbc56, 0xb426, 0xb456
            ]
        );
        assert_eq!(
            (
                tables.primary_tiles[0],
                tables.primary_tiles[15],
                tables.primary_tiles[18],
                tables.primary_tiles[34]
            ),
            (0x3584, 0x35a4, 0x3584, 0x35ce)
        );
        assert_eq!(
            tables.secondary_tiles,
            [0x3b09, 0x3b08, 0x3b07, 0x3b06, 0x3b05, 0x3b04]
        );
        assert!(load_banner_tables(&rom[..PRIMARY_TILES_ROM.end - 1]).is_err());
        assert!(original_banner_layout(16, &tables).is_err());
    }
    #[test]
    fn original_banner_layout_covers_all_tracks_and_only_eight_installs_extra_pair() {
        let tables = source_tables();
        for track in 0..ORIGINAL_BANNER_TRACKS {
            assert_eq!(
                original_banner_layout(track as u8, &tables).unwrap(),
                BannerLayout {
                    coordinates: tables.coordinates[track],
                    install_fixed_pair: track == 8
                }
            );
        }
    }
    #[test]
    fn initial_banner_clears_word_states_but_retains_unwritten_secondary_and_fixed_slots() {
        let mut p = progress();
        p.banner_state = 0xabcd;
        let mut d = display();
        d.secondary_state = 0x4567;
        let previous = d.clone();
        initialize_banner(
            &mut p,
            &mut d,
            BannerLayout {
                coordinates: 0xb768,
                install_fixed_pair: false,
            },
        );
        assert_eq!(p.banner_state, 0);
        assert_eq!(
            d.primary,
            BannerSprite {
                coordinates: 0xb768,
                tile_attributes: 0x3584
            }
        );
        assert_eq!(d.secondary_state, 0);
        assert_eq!(d.secondary, previous.secondary);
        assert_eq!(d.fixed_pair, previous.fixed_pair);
    }
    #[test]
    fn track_eight_banner_extra_pair_has_sixteen_pixel_separation_and_source_tilewords() {
        let mut p = progress();
        let mut d = display();
        initialize_banner(
            &mut p,
            &mut d,
            BannerLayout {
                coordinates: 0xbc5e,
                install_fixed_pair: true,
            },
        );
        assert_eq!(
            d.fixed_pair,
            [
                BannerSprite {
                    coordinates: 0x2760,
                    tile_attributes: 0x3180
                },
                BannerSprite {
                    coordinates: 0x2770,
                    tile_attributes: 0x3182
                }
            ]
        );
        assert_eq!(d.primary.coordinates, 0xbc5e);
    }
    #[test]
    fn last_lap_request_thresholds_preserve_word_high_byte_and_do_not_overwrite_active_banner() {
        for (lap, state, expected) in [
            (3, 0, 0),
            (4, 18, 30),
            (5, 18, 30),
            (4, 19, 19),
            (4, 30, 30),
            (4, 0xab02, 0xab1e),
        ] {
            let mut p = progress();
            p.drivers[2].lap = lap;
            p.banner_state = state;
            request_last_lap(&mut p, 2).unwrap();
            assert_eq!(p.banner_state, expected);
        }
    }
    #[test]
    fn last_lap_invalid_slot_is_rejected_without_mutating_shared_progress() {
        let mut p = progress();
        let old = p.clone();
        assert!(request_last_lap(&mut p, TRUCKS).is_err());
        assert_eq!(p, old);
    }
    #[test]
    fn primary_banner_decays_only_at_four_frame_phase_and_only_at_or_above_nineteen() {
        let tables = source_tables();
        for (state, frame, expected, tile) in [
            (30, 0, 29, 0x35a8),
            (30, 1, 30, 0x35a6),
            (19, 0, 18, 0x3584),
            (18, 0, 18, 0x3584),
            (0, 0, 0, 0x3584),
        ] {
            let mut p = progress();
            p.banner_state = state;
            p.animation_frame = frame;
            let mut d = display();
            animate_banner(&mut p, &mut d, &tables).unwrap();
            assert_eq!(
                (p.banner_state, d.primary.tile_attributes),
                (expected, tile)
            );
            assert_eq!(d.secondary.coordinates, HIDDEN_SPRITE_COORDINATES);
        }
    }
    #[test]
    fn winner_freezes_primary_decay_but_still_updates_tiles_and_secondary_animation() {
        let tables = source_tables();
        let mut p = progress();
        p.banner_state = 30;
        p.winner_plus_one = 2;
        let mut d = display();
        d.secondary_state = 3;
        animate_banner(&mut p, &mut d, &tables).unwrap();
        assert_eq!(p.banner_state, 30);
        assert_eq!(d.primary.tile_attributes, 0x35a6);
        assert_eq!(
            (d.secondary_state, d.secondary.tile_attributes),
            (2, 0x3b07)
        );
    }
    #[test]
    fn secondary_trigger_falls_through_to_same_call_decrement_and_tile_lookup() {
        let tables = source_tables();
        let mut p = progress();
        p.banner_state = SECONDARY_TRIGGER;
        let mut d = display();
        animate_banner(&mut p, &mut d, &tables).unwrap();
        assert_eq!(
            (
                d.secondary.coordinates,
                d.secondary_state,
                d.secondary.tile_attributes
            ),
            (0x8430, 4, 0x3b05)
        );
        assert_eq!(p.banner_state, 15);
    }
    #[test]
    fn secondary_moves_on_odd_parity_without_decrement_and_wraps_packed_y() {
        let tables = source_tables();
        let mut p = progress();
        p.banner_state = 20;
        p.animation_frame = 3;
        let mut d = display();
        d.secondary.coordinates = 0x0002;
        d.secondary_state = 5;
        animate_banner(&mut p, &mut d, &tables).unwrap();
        assert_eq!(
            (
                d.secondary.coordinates,
                d.secondary_state,
                d.secondary.tile_attributes
            ),
            (0xff02, 5, 0x3b04)
        );
    }
    #[test]
    fn secondary_final_tile_is_visible_for_one_call_before_zero_state_hides_it() {
        let tables = source_tables();
        let mut p = progress();
        p.banner_state = 20;
        let mut d = display();
        d.secondary_state = 1;
        animate_banner(&mut p, &mut d, &tables).unwrap();
        assert_eq!(
            (d.secondary_state, d.secondary.tile_attributes),
            (0, 0x3b09)
        );
        p.animation_frame = 1;
        animate_banner(&mut p, &mut d, &tables).unwrap();
        assert_eq!(d.secondary.coordinates, HIDDEN_SPRITE_COORDINATES);
    }
    #[test]
    fn invalid_banner_selector_returns_error_without_partial_sprite_or_counter_mutation() {
        let tables = source_tables();
        let mut p = progress();
        p.banner_state = 35;
        let mut d = display();
        let before = (p.clone(), d.clone());
        assert!(animate_banner(&mut p, &mut d, &tables).is_err());
        assert_eq!((p, d), before);
    }
    #[test]
    fn packed_rectangle_excludes_lower_world_edge_includes_upper_and_rejects_zero_extent() {
        for (world, expected) in [
            ([0x1f00, 0x3000], false),
            ([0x1f01, 0x3000], true),
            ([0x2100, 0x3000], true),
            ([0x2101, 0x3000], false),
            ([0x2000, 0x2f00], false),
            ([0x2000, 0x3100], true),
        ] {
            assert_eq!(inside_checkpoint(world, CHECKPOINT_RECORD), expected);
        }
        assert!(!inside_checkpoint(CHECKPOINT_CENTER, [0x2000, 0x3002]));
        assert!(!inside_checkpoint(CHECKPOINT_CENTER, [0x2002, 0x3000]));
    }
    #[test]
    fn rectangle_masks_control_bit_fourteen_preserves_bit_fifteen_and_wraps_world_subtraction() {
        assert!(inside_checkpoint([0, 0], [0x4001, 0x4001]));
        assert!(inside_checkpoint([0x8000, 0x8000], [0x8001, 0x8001]));
        assert!(!inside_checkpoint([0, 0], [0x8001, 0x8001]));
        assert!(inside_checkpoint([0xfff0, 0x0080], [0x0001, 0x0001]));
        assert!(!inside_checkpoint([0xff80, 0], [0x0001, 0x0001]));
    }
    #[test]
    fn enabled_humans_scan_descending_copy_new_full_words_and_preserve_disabled_ai_words() {
        let mut t = trucks();
        let mut p = progress();
        p.active_humans = 0b0101;
        for (slot, truck) in t.iter_mut().enumerate() {
            truck.controller = ControllerWord(0x6000 + slot as u16);
        }
        let automatic = [
            ControllerWord(0xffff),
            ControllerWord(0xaaaa),
            ControllerWord(0x1234),
            ControllerWord(0xbbbb),
        ];
        let visits = service_human_routes(&mut t, &mut p, &route(), automatic).unwrap();
        assert_eq!(visits.tested_slots, vec![2, 0]);
        assert_eq!(visits.accepted_slots, vec![2, 0]);
        assert_eq!(
            t.each_ref().map(|v| v.controller),
            [
                automatic[0],
                ControllerWord(0x6001),
                automatic[2],
                ControllerWord(0x6003)
            ]
        );
        assert_eq!(p.drivers.map(|d| d.checkpoint), [2, 1, 2, 1]);
    }
    #[test]
    fn no_human_low_bits_neither_change_progress_nor_retained_inputs() {
        for mask in [0, 0xf0] {
            let mut t = trucks();
            let mut p = progress();
            p.active_humans = mask;
            let old = (t.clone(), p.clone());
            let visits =
                service_human_routes(&mut t, &mut p, &route(), [ControllerWord(0xffff); TRUCKS])
                    .unwrap();
            assert_eq!(visits, HumanRouteVisits::default());
            assert_eq!((t, p), old);
        }
    }
    #[test]
    fn human_fd_fe_and_bit_fourteen_records_skip_without_reading_redirect_targets() {
        let r = Route {
            words: vec![
                0xfd00, 0xffff, 0xfe00, 0xfffe, 0x4002, 0, 0x2002, 0x3002, 0xff04,
            ],
            checkpoint_count: 4,
        };
        let mut t = trucks();
        let mut p = progress();
        p.active_humans = 1;
        p.drivers[0].checkpoint = 0;
        let visits = service_human_routes(&mut t, &mut p, &r, NO_INPUT).unwrap();
        assert_eq!(visits.tested_slots, vec![0]);
        assert_eq!(visits.accepted_slots, vec![0]);
        assert_eq!(p.drivers[0].checkpoint, 0);
        assert_eq!(p.drivers[0].lap, 1);
    }
    #[test]
    fn human_terminator_wraps_immediately_then_start_acceptance_increments_lap() {
        let mut t = trucks();
        let mut p = progress();
        p.active_humans = 1;
        p.drivers[0].checkpoint = 3;
        p.drivers[0].lap = 2;
        let visits = service_human_routes(&mut t, &mut p, &route(), NO_INPUT).unwrap();
        assert_eq!(visits.lap_slots, vec![0]);
        assert_eq!((p.drivers[0].checkpoint, p.drivers[0].lap), (1, 3));
    }
    #[test]
    fn rejected_human_checkpoint_keeps_index_but_preceding_control_skips_still_persist() {
        let r = Route {
            words: vec![0x4001, 0, 0x2002, 0x3002, 0xff02],
            checkpoint_count: 2,
        };
        let mut t = trucks();
        t[0].world = [0, 0];
        let mut p = progress();
        p.active_humans = 1;
        p.drivers[0].checkpoint = 0;
        let visits = service_human_routes(&mut t, &mut p, &r, NO_INPUT).unwrap();
        assert!(visits.accepted_slots.is_empty());
        assert_eq!((p.drivers[0].checkpoint, p.drivers[0].lap), (1, 1));
    }
    #[test]
    fn accepted_nonstart_checkpoint_at_limit_wraps_without_incrementing_lap() {
        let mut t = trucks();
        let mut p = progress();
        p.active_humans = 1;
        p.drivers[0].checkpoint = 2;
        service_human_routes(&mut t, &mut p, &route(), NO_INPUT).unwrap();
        assert_eq!((p.drivers[0].checkpoint, p.drivers[0].lap), (0, 1));
    }
    #[test]
    fn start_acceptance_bypasses_checkpoint_limit_test_even_when_limit_is_one() {
        let r = Route {
            words: vec![0x2002, 0x3002, 0xff01],
            checkpoint_count: 1,
        };
        let mut t = trucks();
        let mut p = progress();
        p.active_humans = 1;
        p.drivers[0].checkpoint = 0;
        service_human_routes(&mut t, &mut p, &r, NO_INPUT).unwrap();
        assert_eq!((p.drivers[0].checkpoint, p.drivers[0].lap), (1, 2));
    }
    #[test]
    fn simultaneous_fifth_lap_humans_overwrite_winner_in_descending_scan_and_raise_banner() {
        let mut t = trucks();
        let mut p = progress();
        p.drivers.iter_mut().for_each(|d| {
            d.checkpoint = 0;
            d.lap = 4;
        });
        let visits = service_human_routes(&mut t, &mut p, &route(), NO_INPUT).unwrap();
        assert_eq!(visits.lap_slots, vec![3, 2, 1, 0]);
        assert_eq!(p.winner_plus_one, 1);
        assert_eq!(p.banner_state, 30);
        assert_eq!(p.drivers.map(|d| d.lap), [5; TRUCKS]);
    }
    #[test]
    fn lap_winner_requires_exactly_five_and_byte_lap_overflow_does_not_finish() {
        for (lap, expected) in [(3, 4), (5, 6), (255, 0)] {
            let mut t = trucks();
            let mut p = progress();
            p.active_humans = 1;
            p.drivers[0].checkpoint = 0;
            p.drivers[0].lap = lap;
            service_human_routes(&mut t, &mut p, &route(), NO_INPUT).unwrap();
            assert_eq!(p.drivers[0].lap, expected);
            assert_eq!(p.winner_plus_one, 0);
        }
    }
    #[test]
    fn missing_or_control_only_cyclic_extension_route_returns_error_without_partial_copy_or_progress()
     {
        for r in [
            Route {
                words: vec![0xff00],
                checkpoint_count: 0,
            },
            Route {
                words: vec![0xfe00, 0, 0xff01],
                checkpoint_count: 1,
            },
        ] {
            let mut t = trucks();
            let mut p = progress();
            p.active_humans = 1;
            p.drivers[0].checkpoint = 0;
            let old = (t.clone(), p.clone());
            assert!(
                service_human_routes(&mut t, &mut p, &r, [ControllerWord(0xffff); TRUCKS]).is_err()
            );
            assert_eq!((t, p), old);
        }
    }
    #[test]
    fn timer_adds_decimal_sixteen_with_fraction_and_middle_carry() {
        for (initial, expected, tiles) in [
            ([0, 0, 0], [0x16, 0, 0], [0x15, 0x15, 0x15]),
            ([0x84, 0x09, 0], [0, 0x10, 0], [0x15, 0x16, 0x15]),
            ([0x99, 0x99, 8], [0x15, 0, 9], [0x1e, 0x15, 0x15]),
        ] {
            let mut timer = RaceTimer {
                fraction_bcd: initial[0],
                middle_bcd: initial[1],
                high_bcd: initial[2],
            };
            assert_eq!(advance_timer(&mut timer), tiles);
            assert_eq!(
                [timer.fraction_bcd, timer.middle_bcd, timer.high_bcd],
                expected
            );
        }
    }
    #[test]
    fn timer_freezes_only_when_both_high_and_middle_thresholds_reached_retaining_fraction() {
        for fraction in [0x00, 0x17, 0x99] {
            let mut timer = RaceTimer {
                fraction_bcd: fraction,
                middle_bcd: 0x99,
                high_bcd: 9,
            };
            let old = timer;
            assert_eq!(advance_timer(&mut timer), [0x1e, 0x1e, 0x1e]);
            assert_eq!(timer, old);
        }
        let mut timer = RaceTimer {
            fraction_bcd: 0,
            middle_bcd: 0x98,
            high_bcd: 9,
        };
        advance_timer(&mut timer);
        assert_eq!(timer.fraction_bcd, TIMER_STEP);
    }
    #[test]
    fn timer_crossing_saturation_reaches_first_middle_99_then_freezes_on_following_call() {
        let mut timer = RaceTimer {
            fraction_bcd: 0x90,
            middle_bcd: 0x98,
            high_bcd: 9,
        };
        advance_timer(&mut timer);
        assert_eq!(
            timer,
            RaceTimer {
                fraction_bcd: 0x06,
                middle_bcd: 0x99,
                high_bcd: 9
            }
        );
        advance_timer(&mut timer);
        assert_eq!(timer.fraction_bcd, 0x06);
    }
    #[test]
    fn race_record_initialization_writes_only_explicit_truck_fields_and_preserves_powertrain_history()
     {
        let mut t = trucks();
        let mut p = progress();
        for truck in &mut t {
            truck.heading = 0xabcd;
            truck.speed = 0xef12;
            truck.tile_attributes = 0xffff;
            truck.nitro_stock_bcd = 0x42;
            truck.acceleration_fraction = 0x1234;
            truck.steering_fraction = 99;
            truck.steering_lock = 8;
            truck.upgrades.speed = 3;
        }
        p.winner_plus_one = 4;
        p.banner_state = 12;
        p.drivers.iter_mut().for_each(|d| d.timeout = 17);
        initialize_race_records(&mut t, &mut p);
        assert_eq!(
            t.each_ref().map(|v| v.tile_attributes),
            [0x2000, 0x2202, 0x2404, 0x2606]
        );
        assert!(t.iter().all(|v| v.heading == 0x0040
            && v.speed == 0
            && v.nitro_stock_bcd == 0x42
            && v.acceleration_fraction == 0x1234
            && v.steering_fraction == 99
            && v.steering_lock == 8
            && v.upgrades.speed == 3));
        assert_eq!(p.drivers.map(|d| d.checkpoint), [0; TRUCKS]);
        assert_eq!(p.drivers.map(|d| d.lap), [1; TRUCKS]);
        // Timeouts are written only after the separate AI-parameter setup call.
        assert_eq!(p.drivers.map(|d| d.timeout), [17; TRUCKS]);
        // Winner and banner are cleared by later preparation, not these early writes.
        assert_eq!((p.winner_plus_one, p.banner_state), (4, 12));
    }
    #[test]
    fn route_grid_application_sets_checkpoint_one_and_source_byte_headings_preserving_unwritten_high_bytes()
     {
        let grid = Grid {
            x: [0x1000, 0x2000, 0x3000, 0x4000],
            y: [0x5000, 0x6000, 0x7000, 0x8000],
            heading: 0x80,
            initial_surface_origin: [0x9000, 0xa000],
        };
        let mut t = trucks();
        let mut p = progress();
        t[0].heading = PRESERVED_UPPER_BYTE;
        apply_route_grid(&mut t, &mut p, grid);
        assert_eq!(p.drivers.map(|d| d.checkpoint), [1; TRUCKS]);
        for (slot, truck) in t.iter().enumerate() {
            assert_eq!(truck.world, [grid.x[slot], grid.y[slot]]);
            assert_eq!(
                truck.heading,
                if slot == 0 {
                    PRESERVED_UPPER_BYTE | 0x80
                } else {
                    0x0080
                }
            );
        }
    }
    #[test]
    fn prepared_return_has_priority_over_human_attract_and_winner_dispatch() {
        for humans in [0, 1, 0xf0] {
            for winner in [0, 1, 4] {
                assert_eq!(frame_exit(1, humans, winner), FrameExit::PreparedReturn);
            }
        }
        assert_eq!(frame_exit(0, 0, 0), FrameExit::Continue);
        assert_eq!(frame_exit(0, 1, 0), FrameExit::Continue);
        assert_eq!(frame_exit(0, 0, 1), FrameExit::AttractRestart);
        assert_eq!(frame_exit(0, 0xf0, 1), FrameExit::HumanPostrace);
    }
    #[test]
    fn race_music_selectors_are_five_through_twelve_after_spc_decrement_and_human_gate() {
        for slot in 0..64 {
            assert_eq!(
                race_music_command(1, slot),
                Some((u16::from((slot & 7) + 6) << 8) | 1)
            );
            assert_eq!(race_music_command(0, slot), None);
        }
        assert_eq!(race_music_command(0x80, 255), Some(0x0d01));
    }
    #[test]
    fn attract_start_checks_all_four_words_and_increments_reentry_byte_without_running_human_pause()
    {
        for slot in 0..TRUCKS {
            let mut automatic = NO_INPUT;
            automatic[slot] = ControllerWord(BUTTON_START);
            let mut reentry = 0xff;
            assert_eq!(
                poll_race_start(0, automatic, &mut reentry),
                InputDispatch::FrontendRestart
            );
            assert_eq!(reentry, 0);
        }
        let mut reentry = 7;
        assert_eq!(
            poll_race_start(1, [ControllerWord(BUTTON_START); TRUCKS], &mut reentry),
            InputDispatch::HumanPauseService
        );
        assert_eq!(reentry, 7);
        assert_eq!(
            poll_race_start(0, [ControllerWord(BUTTON_B); TRUCKS], &mut reentry),
            InputDispatch::ContinueAttract
        );
        assert_eq!(reentry, 7);
    }
    #[test]
    fn original_progression_masks_after_increment_and_uses_order_table_not_low_nibble_track() {
        let order: Vec<_> = (0..64)
            .map(|i| Selection {
                original_track: ((i + 4) % 16) as u8,
                direction: if i & 1 == 0 {
                    Direction::Forward
                } else {
                    Direction::Reversed
                },
            })
            .collect();
        assert_eq!(
            advance_original_order(0, &order).unwrap(),
            (
                1,
                Selection {
                    original_track: 5,
                    direction: Direction::Reversed
                }
            )
        );
        assert_eq!(advance_original_order(63, &order).unwrap(), (0, order[0]));
        assert_eq!(advance_original_order(255, &order).unwrap(), (0, order[0]));
        assert!(advance_original_order(1, &[]).is_err());
    }
    // Real synthetic terrain, tables and truck state exercise the composed domain
    // service; phase evidence alone cannot satisfy these state/input/time assertions.
    fn frame_fixture() -> (
        RaceState,
        MotionTables,
        Terrain,
        SamplingTables,
        AiTables,
        BannerTables,
        Route,
    ) {
        let mut cells = vec![20; TERRAIN_BYTES];
        for row in 0..44 {
            for column in 0..44 {
                let offset = row * ROW_BYTES + column * CELL_BYTES;
                if offset + 2 < cells.len() {
                    cells[offset + 2] = 0;
                }
            }
        }
        let terrain = Terrain { cells };
        let sampling = SamplingTables {
            sine: [0; 256],
            wheel_phases: [0; 4],
            row_offsets: std::array::from_fn(|i| (i * ROW_BYTES) as u16),
        };
        let motion = MotionTables {
            speed_limits: [0; 9],
            acceleration_steps: [0; 7],
            steering_rates: [0; 7],
            x_vectors: [0; 32],
            y_vectors: [0; 32],
            tilt_poses: [0; 25],
            shadow_masks: [0; 8],
            blocked_coordinate_by_quadrant: [0; 4],
        };
        let ai = AiTables {
            bearings: [0; 256],
            turn_steps: [0; 4],
            waypoint_timeouts: [100; 8],
            parameter_thresholds: [0; 8],
            rank_threshold_one: [128; 4],
            rank_threshold_two: [128; 4],
        };
        let mut t = trucks();
        for (slot, truck) in t.iter_mut().enumerate() {
            truck.world = [0x1080 + slot as u16 * 0x400, 0x1080];
            truck.wheels = [Wheel {
                height: 20,
                sampled: 20,
                ..Wheel::default()
            }; 4];
            truck.surface_summary = 20;
            truck.contact_cooldown = 2;
        }
        let r = Route {
            words: vec![0x9001, 0x9001, 0x9001, 0x9001, 0xff02],
            checkpoint_count: 2,
        };
        (
            RaceState {
                route_tail_byte: 0,
                world: crate::world::objects::WorldObjects::default(),
                video: RaceVideo::default(),
                sounds: Vec::new(),
                entry: RaceEntry::Running,
                working_pose: 0,
                trucks: t,
                progress: progress(),
                timer: RaceTimer::default(),
                banner: display(),
                prepared: 1,
                terrain_history: 0,
                render_mode: 0,
                video_inhibit: 0x80,
                enabled_layers: 0,
            },
            motion,
            terrain,
            sampling,
            ai,
            source_tables(),
            r,
        )
    }
    #[test]
    fn prepared_frame_executes_all_owners_in_source_order_and_increments_timer_before_return() {
        let (mut r, motion, terrain, sampling, ai, banners, route) = frame_fixture();
        r.progress.winner_plus_one = 3;
        r.trucks[0].effect_cooldown = 24;
        let resources =
            RaceAssets::from_original(&std::fs::read("../input/game.rom").unwrap(), 0, 0, false)
                .unwrap();
        let env = FrameEnvironment {
            assets: &resources,
            motion: MotionEnvironment {
                tables: &motion,
                terrain: &terrain,
                sampling: &sampling,
                projection: Projection {
                    offset: [0; 2],
                    scale: [103, 67],
                },
                original_track: 0,
                active_humans: ALL_HUMANS,
                prepared: true,
                animation: 0,
            },
            route: &route,
            ai: &ai,
            banners: &banners,
            automatic: [ControllerWord(BUTTON_B); TRUCKS],
        };
        let mut random = RandomState([0; 5]);
        let (exit, phases) = update_frame_services(&mut r, &env, &mut random).unwrap();
        assert_eq!(exit, FrameExit::PreparedReturn);
        assert_eq!(
            phases,
            vec![
                ServicePhase::Motion,
                ServicePhase::Wheels,
                ServicePhase::Suspension,
                ServicePhase::Poses,
                ServicePhase::SpriteSort,
                ServicePhase::SpritePack,
                ServicePhase::SpriteAnimation,
                ServicePhase::HumanRoutes,
                ServicePhase::TrackObjects,
                ServicePhase::Banner,
                ServicePhase::Hud,
                ServicePhase::Computers,
                ServicePhase::Exit
            ]
        );
        assert_eq!(r.render_mode, 2);
        assert_eq!(r.trucks[0].effect_cooldown, 25);
        assert_eq!(
            r.video.memory.staged_oam[0x182],
            resources.animations.tiles[24]
        );
        assert_eq!(r.video.sprites.x[0], r.trucks[0].screen_x);
        assert_eq!(r.timer.fraction_bcd, TIMER_STEP);
        assert!(r.trucks.iter().all(|t| t.speed == 0
            && t.controller == ControllerWord(BUTTON_B)
            && t.contact_cooldown == 1));
        assert_eq!(random, RandomState([0; 5]));
    }
    #[test]
    fn winner_frame_still_advances_hud_timer_and_copies_input_before_postrace_dispatch() {
        let (mut r, motion, terrain, sampling, ai, banners, route) = frame_fixture();
        r.prepared = 0;
        r.progress.winner_plus_one = 2;
        let resources =
            RaceAssets::from_original(&std::fs::read("../input/game.rom").unwrap(), 0, 0, false)
                .unwrap();
        let env = FrameEnvironment {
            assets: &resources,
            motion: MotionEnvironment {
                tables: &motion,
                terrain: &terrain,
                sampling: &sampling,
                projection: Projection {
                    offset: [0; 2],
                    scale: [103, 67],
                },
                original_track: 0,
                active_humans: ALL_HUMANS,
                prepared: false,
                animation: 0,
            },
            route: &route,
            ai: &ai,
            banners: &banners,
            automatic: [ControllerWord(BUTTON_START); TRUCKS],
        };
        let mut random = RandomState([0; 5]);
        let (exit, _) = update_frame_services(&mut r, &env, &mut random).unwrap();
        assert_eq!(exit, FrameExit::HumanPostrace);
        assert_eq!(r.timer.fraction_bcd, TIMER_STEP);
        assert!(
            r.trucks
                .iter()
                .all(|t| t.controller == ControllerWord(BUTTON_START))
        );
    }
    #[test]
    fn selected_race_preparation_orders_real_owner_setup_and_one_service_before_ready() {
        let (mut r, motion, terrain, sampling, ai, banners, route) = frame_fixture();
        r.timer = RaceTimer {
            fraction_bcd: 0x99,
            middle_bcd: 0x99,
            high_bcd: 9,
        };
        r.progress.winner_plus_one = 4;
        r.progress.banner_state = 30;
        r.progress.drivers[0].lap = 5;
        let grid = Grid {
            x: [0x1080, 0x1480, 0x1880, 0x1c80],
            y: [0x1080; TRUCKS],
            heading: 0x80,
            initial_surface_origin: [0x1080, 0x1080],
        };
        let resources =
            RaceAssets::from_original(&std::fs::read("../input/game.rom").unwrap(), 0, 0, false)
                .unwrap();
        let env = FrameEnvironment {
            assets: &resources,
            motion: MotionEnvironment {
                tables: &motion,
                terrain: &terrain,
                sampling: &sampling,
                projection: Projection {
                    offset: [0; 2],
                    scale: [103, 67],
                },
                original_track: 0,
                active_humans: ALL_HUMANS,
                prepared: true,
                animation: 0,
            },
            route: &route,
            ai: &ai,
            banners: &banners,
            automatic: NO_INPUT,
        };
        let mut random = RandomState([0; 5]);
        let phases = prepare_selected_race(&mut r, &env, grid, &mut random).unwrap();
        assert_eq!(
            phases,
            vec![
                PreparationPhase::BlankAndConfigureLayers,
                PreparationPhase::FontAndClearBackground,
                PreparationPhase::TruckRecords,
                PreparationPhase::AiParameters,
                PreparationPhase::ResetTimeouts,
                PreparationPhase::ClearWheels,
                PreparationPhase::ClearSpriteAnimations,
                PreparationPhase::LoadRoute,
                PreparationPhase::LoadTrackAssets,
                PreparationPhase::StartGrid,
                PreparationPhase::SharedSpritePalette,
                PreparationPhase::SharedSpriteTiles,
                PreparationPhase::RaceOam,
                PreparationPhase::PackSpriteArrays,
                PreparationPhase::HudAndTimer,
                PreparationPhase::TrackObjects,
                PreparationPhase::Banner,
                PreparationPhase::EnableFrameBoundary,
                PreparationPhase::PreparedServices,
                PreparationPhase::ClearPrepared,
                PreparationPhase::FadeIn,
                PreparationPhase::ReadyTransition,
                PreparationPhase::WaitAndMusic
            ]
        );
        assert_eq!(
            (r.prepared, r.video_inhibit, r.enabled_layers),
            (0, 0, 0x1f)
        );
        assert!(matches!(r.entry, RaceEntry::FadeIn(_)));
        assert_eq!(
            r.world.placements.len(),
            resources.objects.placements.prefix.len()
        );
        assert!(r.video.memory.vram.iter().any(|byte| *byte != 0));
        assert_eq!(r.progress.winner_plus_one, 0);
        assert_eq!(r.progress.drivers.map(|d| d.lap), [1; TRUCKS]);
        assert_eq!(r.progress.drivers.map(|d| d.checkpoint), [1; TRUCKS]);
        assert_eq!(
            r.timer,
            RaceTimer {
                fraction_bcd: TIMER_STEP,
                middle_bcd: 0,
                high_bcd: 0
            }
        );
        assert!(r.trucks.iter().all(|t| t.heading == 0x0080 && t.speed == 0));
    }
    #[test]
    fn timeout_reset_follows_ai_setup_and_preserves_all_other_driver_state() {
        let mut p = progress();
        p.drivers.iter_mut().for_each(|d| d.timeout = 17);
        let before = p.clone();
        reset_route_timeouts(&mut p);
        assert_eq!(p.drivers.map(|d| d.timeout), [INITIAL_AI_TIMEOUT; TRUCKS]);
        for (new, old) in p.drivers.iter().zip(before.drivers) {
            assert_eq!(
                (new.checkpoint, new.lap, new.parameter, new.bias_delay),
                (old.checkpoint, old.lap, old.parameter, old.bias_delay)
            );
        }
    }
    #[test]
    fn conditional_human_record_index_multiplies_at_byte_width_before_route_access() {
        // Original streams never reach64 records. This supplied conditional state
        // locks the source ASL/ASL byte-width behavior for native extension callers.
        let mut words = vec![0x0000; 134];
        words[0] = 0x2002;
        words[1] = 0x3002;
        words.push(0xff42);
        let route = Route {
            words,
            checkpoint_count: 66,
        };
        let mut t = trucks();
        let mut p = progress();
        p.active_humans = 1;
        p.drivers[0].checkpoint = 64;
        let visits = service_human_routes(&mut t, &mut p, &route, NO_INPUT).unwrap();
        assert_eq!(visits.accepted_slots, vec![0]);
        assert_eq!((p.drivers[0].checkpoint, p.drivers[0].lap), (65, 1));
    }
}
