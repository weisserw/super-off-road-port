//! Resumable source fades, double-buffered windows, countdowns and pause.
//! Source derivations and boundary scheduling: transitions.trace.md.
use super::memory;
use super::{Display, MapImage, VideoMemory};
use crate::controls::BUTTON_START;
use crate::controls::{CONTROLLER_COUNT, ControllerWord};
use crate::world::tracks::{CpuSource, physical};
use anyhow::{Context, Result, ensure};

pub const SHAPE_ROWS: usize = 128;
pub const WINDOW_ROWS: usize = 260;
pub const SHAPE_COUNT: usize = 3;
pub const ORIGINAL_TRACKS: usize = 16;
pub const COUNTDOWN_MAPS: usize = 34;
pub const CLOSED_EDGE: [u8; 2] = [129, 128];
pub const FULL_BRIGHTNESS: u8 = 15;
pub const FRONTEND_TARGET: u16 = 0x400;
pub const RACE_TARGET: u16 = 0x70;
pub const PAUSE_TARGET: u16 = 0x60;
pub const READY_AUDIO: u16 = 0x0e01;
pub const POSTRACE_AUDIO: u16 = 0x0f01;
pub const READY_DELAY: u16 = 6;
pub const COUNTDOWN_RELOAD: u16 = 5;
pub const POSTRACE_FIRST: u16 = 18;
pub const POSTRACE_END: u16 = 34;
pub const POSTRACE_HOLD: u16 = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ShapeId {
    Frontend,
    Race,
    Pause,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shape {
    pub edges: [[i8; 2]; SHAPE_ROWS],
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransitionAssets {
    pub shapes: [Shape; SHAPE_COUNT],
    pub centers: [[u8; 2]; ORIGINAL_TRACKS],
    pub destinations: [u16; ORIGINAL_TRACKS],
    pub countdown: Vec<MapImage>,
    pub pause: MapImage,
}
pub fn load_assets(rom: &[u8]) -> Result<TransitionAssets> {
    let shapes = decode_shapes(
        rom.get(0x3616..0x361f).context("missing shape directory")?,
        rom,
    )?;
    let centers = std::array::from_fn(|i| {
        [
            rom.get(0x55ee + i).copied().unwrap_or(0),
            rom.get(0x55fe + i).copied().unwrap_or(0),
        ]
    });
    let destination_bytes = rom
        .get(0x560e..0x562e)
        .context("missing countdown destinations")?;
    let destinations = std::array::from_fn(|i| {
        u16::from_le_bytes(destination_bytes[i * 2..i * 2 + 2].try_into().unwrap())
    });
    let descriptor = memory::decode_rectangle(
        rom.get(0x56cd..0x56e1)
            .context("missing countdown descriptor")?,
    )?;
    let directory = rom
        .get(0x5667..0x56cd)
        .context("missing countdown sources")?;
    let mut countdown = Vec::new();
    for record in directory.chunks_exact(3) {
        let cpu =
            u32::from(u16::from_le_bytes([record[0], record[1]])) | (u32::from(record[2]) << 16);
        countdown.push(MapImage {
            rectangle: descriptor.clone(),
            words: memory::read_rom_rectangle(rom, cpu, &descriptor)?,
            destination_word: 0,
        });
    }
    let pause_rectangle = memory::decode_rectangle(
        rom.get(0x5780..0x5794)
            .context("missing pause descriptor")?,
    )?;
    let pause = MapImage {
        words: memory::read_rom_rectangle(rom, 0x07dab4, &pause_rectangle)?,
        rectangle: pause_rectangle,
        destination_word: 0,
    };
    Ok(TransitionAssets {
        shapes,
        centers,
        destinations,
        countdown,
        pause,
    })
}
/// Three source bank/address records and three full 256-byte shapes; no
/// resemblance-based interpretation of neighboring unowned tilewords.
pub fn decode_shapes(directory: &[u8], rom: &[u8]) -> Result<[Shape; SHAPE_COUNT]> {
    ensure!(
        directory.len() == SHAPE_COUNT * 3,
        "invalid shape directory"
    );
    let mut shapes = Vec::new();
    for record in directory.chunks_exact(3) {
        let cpu =
            (u32::from(record[0]) << 16) | u32::from(u16::from_le_bytes([record[1], record[2]]));
        let start = physical(CpuSource(cpu))?;
        let bytes = rom
            .get(start..start + SHAPE_ROWS * 2)
            .context("truncated shape")?;
        shapes.push(Shape {
            edges: std::array::from_fn(|i| [bytes[i * 2] as i8, bytes[i * 2 + 1] as i8]),
        });
    }
    Ok(shapes.try_into().unwrap())
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Window {
    #[serde(with = "crate::snapshot::array")]
    pub center: [u8; 2],
    pub extent: u16,
    pub shape: ShapeId,
    /// Retained source configuration bytes, distinct from active Display masks.
    #[serde(with = "crate::snapshot::array")]
    pub masks: [u8; 3],
    /// Source selector points to the buffer displayed, opposite the new build.
    pub selector: u16,
    #[serde(with = "crate::snapshot::array")]
    pub buffers: [Vec<[u8; 2]>; 2],
    pub hdma_enabled: bool,
}
/// MPYM is only product bits8..15, then byte add/sub carry clamps each edge.
pub fn scale_edges(center: u8, extent: u16, samples: [i8; 2]) -> [u8; 2] {
    std::array::from_fn(|side| {
        let sample = samples[side];
        let magnitude = if sample < 0 {
            sample.wrapping_neg()
        } else {
            sample
        };
        let product = ((i32::from(extent as i16) * i32::from(magnitude)) >> 8) as u8;
        if sample < 0 {
            center
                .checked_sub(product)
                .unwrap_or(if side == 0 { 1 } else { 0 })
        } else {
            center.saturating_add(product)
        }
    })
}
pub fn build_window(window: &mut Window, shape: &Shape) -> Result<()> {
    ensure!(
        window
            .buffers
            .iter()
            .all(|buffer| buffer.len() == WINDOW_ROWS),
        "invalid window buffer capacity"
    );
    window.selector = window.selector.wrapping_add(1) & 1;
    let target = usize::from(window.selector ^ 1);
    let buffer = &mut window.buffers[target];
    buffer[..256].fill(CLOSED_EDGE);
    let mut last_row = 0;
    for (row, &samples) in shape.edges.iter().enumerate() {
        let distance = row.abs_diff(64) as i32;
        let offset = ((i32::from(window.extent as i16) * distance) >> 8) as u8;
        let y = if row < 64 {
            window.center[1].checked_sub(offset)
        } else {
            window.center[1].checked_add(offset)
        };
        if let Some(y) = y {
            last_row = usize::from(y);
            let edges = scale_edges(window.center[0], window.extent, samples);
            buffer[last_row..last_row + 4].fill(edges);
        }
    }
    buffer[last_row + 1..last_row + 4].fill(CLOSED_EDGE);
    Ok(())
}
/// Setup takes two NMI waits, builds twice between them, then enables masks.
pub fn prime_window(window: &mut Window, shape: &Shape, display: &mut Display) -> Result<u16> {
    ensure!(
        window
            .buffers
            .iter()
            .all(|buffer| buffer.len() == WINDOW_ROWS),
        "invalid window buffer capacity"
    );
    window.selector = 0;
    build_window(window, shape)?;
    build_window(window, shape)?;
    enable_window(window, display);
    Ok(2)
}
pub fn disable_masks(window: &mut Window, display: &mut Display) {
    let _ = window;
    display.masked_layers = 0;
    display.window_inverted_layers = 0;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum FadeDirection {
    In,
    Out,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fade {
    pub direction: FadeDirection,
    pub primed: bool,
    pub complete: bool,
    pub animation_counter: u8,
    pub enabled_layers: u8,
}
/// One observed NMI boundary; leading wait resets the animation counter,
/// subsequent boundaries use its previous captured value before increment.
pub fn advance_fade(fade: &mut Fade, display: &mut Display) {
    if fade.complete {
        return;
    }
    if !fade.primed {
        fade.primed = true;
        fade.animation_counter = 0;
        if fade.direction == FadeDirection::In {
            display.main_layers = fade.enabled_layers;
            display.main_window_layers = fade.enabled_layers;
        }
        return;
    }
    let previous = fade.animation_counter;
    fade.animation_counter = previous.wrapping_add(1);
    display.force_blank = false;
    display.brightness = match fade.direction {
        FadeDirection::Out => (previous & 15) ^ 15,
        FadeDirection::In => (previous >> 1) & 15,
    };
    fade.complete = match fade.direction {
        FadeDirection::Out => display.brightness == 0,
        FadeDirection::In => display.brightness == 15,
    };
    if fade.complete && fade.direction == FadeDirection::Out {
        display.force_blank = true;
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CountedWaitWidth {
    Byte,
    Word,
}
pub fn counted_wait_boundaries(initial: u16, width: CountedWaitWidth) -> u32 {
    let count = match width {
        CountedWaitWidth::Byte => u32::from(initial as u8),
        CountedWaitWidth::Word => u32::from(initial),
    };
    if count == 0 {
        match width {
            CountedWaitWidth::Byte => 256,
            CountedWaitWidth::Word => 65536,
        }
    } else {
        count
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum FrontendKind {
    Scrolling,
    Expanding,
    Contracting,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrontendWindow {
    /// Two synchronized setup waits before ordinary geometry frames.
    pub setup_waits: u8,
    pub kind: FrontendKind,
    pub abort_enabled: u8,
    pub velocity: i8,
    pub wait_timer: u16,
    pub complete: bool,
    pub aborted: bool,
}
pub fn begin_frontend(
    kind: FrontendKind,
    window: &mut Window,
    display: &mut Display,
) -> FrontendWindow {
    match kind {
        FrontendKind::Scrolling => {
            window.center = [224, 128];
            window.extent = 0;
            window.masks = [3, 0, 0];
            display.layers[0].scroll[1] = 0;
            display.brightness = 15;
            display.force_blank = false;
        }
        FrontendKind::Expanding => {
            window.center = [128, 176];
            window.extent = 0;
            window.masks = [0x22; 3];
            window.shape = ShapeId::Frontend;
        }
        FrontendKind::Contracting => {
            window.center = [128, 112];
            window.extent = 1024;
            window.masks = [3, 3, 0];
            window.shape = ShapeId::Frontend;
        }
    }
    window.selector = 0;
    window.hdma_enabled = false;
    FrontendWindow {
        setup_waits: 2,
        kind,
        abort_enabled: 0,
        velocity: -2,
        wait_timer: 100,
        complete: false,
        aborted: false,
    }
}
/// Actual geometry, BG1 scroll, display blanking and mask effects are mutated.
/// Start-abort returns without the normal completion cleanup in all three modes.
pub fn advance_frontend(
    state: &mut FrontendWindow,
    window: &mut Window,
    shape: &Shape,
    display: &mut Display,
    automatic: [ControllerWord; CONTROLLER_COUNT],
) -> Result<()> {
    if state.complete {
        return Ok(());
    }
    if state.setup_waits != 0 {
        if state.setup_waits == 2 {
            window.selector = 0;
            build_window(window, shape)?;
            build_window(window, shape)?;
        }
        state.setup_waits -= 1;
        if state.setup_waits == 0 {
            enable_window(window, display);
        }
        return Ok(());
    }
    if state.kind == FrontendKind::Scrolling {
        display.layers[0].scroll = [
            u16::from(window.center[0].wrapping_add(0x7c).wrapping_neg()),
            u16::from(window.center[1].wrapping_add(0x78).wrapping_neg()),
        ];
    }
    if state.kind == FrontendKind::Expanding {
        build_window(window, shape)?;
    }
    if state.abort_enabled != 0 && automatic[..2].iter().any(|word| word.0 & BUTTON_START != 0) {
        state.complete = true;
        state.aborted = true;
        return Ok(());
    }
    if state.kind != FrontendKind::Expanding {
        build_window(window, shape)?;
    }
    if window.hdma_enabled {
        show_window(window, display);
    }
    let step = (window.extent >> 4).max(1);
    match state.kind {
        FrontendKind::Contracting => {
            window.extent = window.extent.wrapping_sub(step);
            state.complete = (window.extent as i16) < 0;
        }
        FrontendKind::Scrolling => {
            window.extent = window.extent.wrapping_add(step);
            window.center[0] = window.center[0].wrapping_sub(1);
            window.center[1] = window.center[1].wrapping_add(state.velocity as u8);
            if [64, 160].contains(&window.center[1]) {
                state.velocity = state.velocity.wrapping_neg();
            }
            state.complete = window.extent >> 8 >= 4;
        }
        FrontendKind::Expanding => {
            window.extent = window.extent.wrapping_add(step);
            window.center[1] = window.center[1].wrapping_sub(1);
            window.center[0] = window.center[0].wrapping_add(state.velocity as u8);
            if [64, 176].contains(&window.center[0]) {
                state.velocity = state.velocity.wrapping_neg();
            }
            state.complete = window.extent >> 8 >= 4;
        }
    }
    if state.complete {
        if state.kind != FrontendKind::Scrolling {
            display.force_blank = true;
        }
        disable_masks(window, display);
    }
    Ok(())
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Countdown {
    pub state: u16,
    pub delay: u16,
}
/// Draw current map first, then word decrement/BPL; initial delay6 draws7
/// times, subsequent reload5 draws6. Renderer owns rectangle pixel encoding.
pub fn draw_countdown(
    countdown: &mut Countdown,
    layout: &CountdownLayout,
    assets: &TransitionAssets,
    memory: &mut VideoMemory,
) -> Result<()> {
    validate_layout(layout)?;
    let map = assets
        .countdown
        .get(usize::from(countdown.state))
        .context("invalid countdown state")?;
    memory::draw_rectangle(memory, &map.words, layout.destination_word, &map.rectangle)?;
    countdown.delay = countdown.delay.wrapping_sub(1);
    if (countdown.delay as i16) < 0 {
        countdown.delay = COUNTDOWN_RELOAD;
        countdown.state = countdown.state.wrapping_add(1);
    }
    Ok(())
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CountdownLayout {
    pub center: [u8; 2],
    pub destination_word: u16,
}
pub fn original_layout(track: u8, assets: &TransitionAssets) -> Result<CountdownLayout> {
    let index = usize::from(track);
    ensure!(index < ORIGINAL_TRACKS, "invalid original track layout");
    Ok(CountdownLayout {
        center: assets.centers[index],
        destination_word: assets.destinations[index],
    })
}
/// Added named tracks supply this layout instead of a fixed original index.
pub fn validate_layout(layout: &CountdownLayout) -> Result<()> {
    ensure!(
        layout.destination_word < 0x8000,
        "invalid countdown VRAM word destination"
    );
    Ok(())
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RaceTransitionKind {
    Ready,
    Postrace,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RacePhase {
    BeginWait,
    Delay,
    PrimeFirst,
    PrimeSecond,
    InitialDraw,
    Grow,
    Hold,
    CountToReady,
    Close,
    ReleaseWait,
    FinalWait,
    Complete,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RaceTransition {
    pub kind: RaceTransitionKind,
    pub phase: RacePhase,
    pub remaining_waits: u16,
    pub hold_timer: u16,
    pub countdown: Countdown,
    pub audio_commands: Vec<u16>,
    pub banner_calls: u32,
    pub animation_calls: u32,
}
/// Human gate reads the source WORD (active mask low, shifted mask high).
pub fn begin_race_transition(
    kind: RaceTransitionKind,
    human_gate_word: u16,
    state: &mut RaceTransition,
    window: &mut Window,
    layout: CountdownLayout,
    display: &mut Display,
    inhibit: &mut u8,
) {
    state.kind = kind;
    state.phase = RacePhase::BeginWait;
    state.remaining_waits = if kind == RaceTransitionKind::Ready {
        26
    } else {
        0
    };
    state.hold_timer = 0;
    state.countdown = Countdown {
        state: if kind == RaceTransitionKind::Ready {
            0
        } else {
            POSTRACE_FIRST
        },
        delay: READY_DELAY,
    };
    state.banner_calls = 0;
    state.animation_calls = 0;
    state.audio_commands.clear();
    window.center = layout.center;
    window.extent = 0;
    window.shape = ShapeId::Race;
    window.masks = [0x22, 0, 0x22];
    window.selector = 0;
    display.color_math_layers = 0x13;
    display.half_color = true;
    if kind == RaceTransitionKind::Ready {
        display.force_blank = false;
        display.brightness = 15;
        if human_gate_word != 0 {
            state.audio_commands.push(READY_AUDIO);
        }
    } else {
        *inhibit = 0;
        state.audio_commands.push(POSTRACE_AUDIO);
    }
}
/// One NMI step, real countdown drawing/window buffer/display/inhibit mutations;
/// banner/animation call counts are non-draining observations for shared owners.
pub fn advance_race_transition(
    state: &mut RaceTransition,
    window: &mut Window,
    layout: CountdownLayout,
    assets: &TransitionAssets,
    memory: &mut VideoMemory,
    display: &mut Display,
    inhibit: &mut u8,
) -> Result<()> {
    validate_layout(&layout)?;
    let post = state.kind == RaceTransitionKind::Postrace;
    let shape = &assets.shapes[1];
    match state.phase {
        RacePhase::BeginWait => {
            state.phase = if post {
                RacePhase::PrimeFirst
            } else {
                RacePhase::Delay
            };
        }
        RacePhase::Delay => {
            state.remaining_waits = state.remaining_waits.wrapping_sub(1);
            if state.remaining_waits == 0 {
                state.phase = RacePhase::PrimeFirst;
            }
        }
        RacePhase::PrimeFirst => {
            window.selector = 0;
            build_window(window, shape)?;
            build_window(window, shape)?;
            state.phase = RacePhase::PrimeSecond;
        }
        RacePhase::PrimeSecond => {
            enable_window(window, display);
            state.phase = RacePhase::InitialDraw;
        }
        RacePhase::InitialDraw => {
            draw_countdown(&mut state.countdown, &layout, assets, memory)?;
            if post {
                state.phase = RacePhase::Grow;
            } else {
                state.banner_calls = state.banner_calls.wrapping_add(1);
                state.phase = RacePhase::Grow;
            }
        }
        RacePhase::Grow => {
            if post {
                transition_draw(state, layout, assets, memory, true)?;
            }
            build_window(window, shape)?;
            show_window(window, display);
            if window.extent >= RACE_TARGET {
                if post {
                    state.hold_timer = POSTRACE_HOLD;
                    state.phase = RacePhase::Hold;
                } else {
                    state.phase = RacePhase::CountToReady;
                }
            } else {
                window.extent = window.extent.wrapping_add((window.extent >> 4).max(1));
            }
        }
        RacePhase::Hold => {
            transition_draw(state, layout, assets, memory, true)?;
            state.hold_timer = state.hold_timer.wrapping_sub(1);
            if state.hold_timer == 0 {
                state.phase = RacePhase::Close;
            }
        }
        RacePhase::CountToReady => {
            transition_draw(state, layout, assets, memory, false)?;
            if state.countdown.state >= 17 {
                state.phase = RacePhase::Close;
            }
        }
        RacePhase::Close => {
            if post || state.countdown.state < 18 {
                transition_draw(state, layout, assets, memory, false)?;
            }
            build_window(window, shape)?;
            show_window(window, display);
            let next = window.extent.wrapping_sub((window.extent >> 2).max(1));
            if (next as i16) < 0 {
                state.phase = RacePhase::ReleaseWait;
            } else {
                window.extent = next;
            }
        }
        RacePhase::ReleaseWait => {
            if !post {
                *inhibit = 0;
            }
            window.hdma_enabled = false;
            state.phase = RacePhase::FinalWait;
        }
        RacePhase::FinalWait => {
            state.phase = RacePhase::Complete;
        }
        RacePhase::Complete => {}
    }
    Ok(())
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PausePhase {
    BeginWait,
    PrimeFirst,
    PrimeSecond,
    Running,
    Expand,
    WaitRelease,
    WaitPress,
    ReleaseWait,
    FinalWait,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pause {
    pub owner_plus_one: u8,
    pub phase: PausePhase,
}
pub fn poll_pause(
    pause: &mut Pause,
    joined_pages: [u8; 2],
    automatic: [ControllerWord; CONTROLLER_COUNT],
) -> Result<()> {
    ensure!(pause.owner_plus_one <= 4, "invalid pause owner");
    let held = |slot: usize| automatic[slot].0 & BUTTON_START != 0;
    match pause.phase {
        PausePhase::Running => {
            if pause.owner_plus_one != 0 {
                if held(usize::from(pause.owner_plus_one - 1)) {
                    return Ok(());
                }
                pause.owner_plus_one = 0;
            }
            for slot in 0..4 {
                if joined_pages[slot / 2] & (1 << (slot % 2)) != 0
                    && held(if slot == 3 { 2 } else { slot })
                {
                    pause.owner_plus_one = slot as u8 + 1;
                    pause.phase = PausePhase::Expand;
                    break;
                }
            }
        }
        PausePhase::WaitRelease => {
            ensure!(pause.owner_plus_one != 0, "missing pause owner");
            if !held(usize::from(pause.owner_plus_one - 1)) {
                pause.phase = PausePhase::WaitPress;
            }
        }
        PausePhase::WaitPress => {
            ensure!(pause.owner_plus_one != 0, "missing pause owner");
            if held(usize::from(pause.owner_plus_one - 1)) {
                pause.phase = PausePhase::ReleaseWait;
            }
        }
        _ => {}
    }
    Ok(())
}
pub fn begin_pause_window(
    window: &mut Window,
    layout: CountdownLayout,
    display: &mut Display,
    memory: &mut VideoMemory,
    assets: &TransitionAssets,
    inhibit: &mut u8,
) -> Result<()> {
    validate_layout(&layout)?;
    memory::draw_rectangle(
        memory,
        &assets.pause.words,
        layout.destination_word.wrapping_add(0x41),
        &assets.pause.rectangle,
    )?;
    window.center = layout.center;
    window.extent = 0;
    window.shape = ShapeId::Pause;
    window.masks = [0x22, 0, 0x22];
    window.selector = 0;
    display.color_math_layers = 0x13;
    display.half_color = true;
    display.brightness = 15;
    display.force_blank = false;
    *inhibit = inhibit.wrapping_add(1);
    Ok(())
}
pub fn advance_pause(
    pause: &mut Pause,
    window: &mut Window,
    layout: CountdownLayout,
    assets: &TransitionAssets,
    display: &mut Display,
    inhibit: &mut u8,
    automatic: [ControllerWord; CONTROLLER_COUNT],
) -> Result<()> {
    ensure!(pause.owner_plus_one <= 4, "invalid pause owner");
    validate_layout(&layout)?;
    match pause.phase {
        PausePhase::Expand => {
            build_window(window, &assets.shapes[2])?;
            show_window(window, display);
            if window.extent >= PAUSE_TARGET {
                pause.phase = PausePhase::WaitRelease;
                poll_pause(pause, [0; 2], automatic)?;
            } else {
                window.extent = window.extent.wrapping_add(8);
            }
        }
        PausePhase::WaitRelease | PausePhase::WaitPress => {
            poll_pause(pause, [0; 2], automatic)?;
        }
        PausePhase::ReleaseWait => {
            *inhibit = 0;
            window.hdma_enabled = false;
            pause.phase = PausePhase::FinalWait;
        }
        PausePhase::FinalWait => {
            pause.phase = PausePhase::Running;
        }
        _ => {}
    }
    Ok(())
}
/// Top-level pause service includes initial frame wait, real banner draw,
/// double-buffer priming, twelve growth increments plus final threshold test,
/// release/newpress and two synchronized resume waits. No source busy loop
/// blocks the host; runtime NMI/audio counters continue while race motion stops.
// Explicit disjoint owner borrows keep mutation authority visible at this boundary.
#[allow(clippy::too_many_arguments)]
pub fn service_pause_frame(
    pause: &mut Pause,
    joined_pages: [u8; 2],
    window: &mut Window,
    layout: CountdownLayout,
    assets: &TransitionAssets,
    memory: &mut VideoMemory,
    display: &mut Display,
    inhibit: &mut u8,
    automatic: [ControllerWord; CONTROLLER_COUNT],
) -> Result<()> {
    ensure!(pause.owner_plus_one <= 4, "invalid pause owner");
    match pause.phase {
        PausePhase::Running => {
            poll_pause(pause, joined_pages, automatic)?;
            if pause.phase == PausePhase::Expand {
                pause.phase = PausePhase::BeginWait;
            }
        }
        PausePhase::BeginWait => {
            begin_pause_window(window, layout, display, memory, assets, inhibit)?;
            pause.phase = PausePhase::PrimeFirst;
        }
        PausePhase::PrimeFirst => {
            window.selector = 0;
            build_window(window, &assets.shapes[2])?;
            build_window(window, &assets.shapes[2])?;
            pause.phase = PausePhase::PrimeSecond;
        }
        PausePhase::PrimeSecond => {
            enable_window(window, display);
            pause.phase = PausePhase::Expand;
        }
        _ => advance_pause(pause, window, layout, assets, display, inhibit, automatic)?,
    }
    Ok(())
}
/// Attract overlay frame owns brightness8→normalize selected wallet→15→poll.
/// Attract owns the reentry decision; economy owns normalization semantics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlayOutcome {
    Continue,
    TimedOut,
    StartReentry,
}
pub fn advance_attract_overlay(
    wallet: &mut [u8; 6],
    timer: &mut u16,
    signature_first_byte: u8,
    display: &mut Display,
    automatic: [ControllerWord; CONTROLLER_COUNT],
) -> (OverlayOutcome, Vec<u8>) {
    display.brightness = 8;
    // A standalone selected-wallet view has no adjacent owner; the full
    // frontend panel normalizes Session directly, including preceding borrows.
    crate::frontend::shop::normalize_digits(wallet, &mut 0);
    display.brightness = 15;
    let outcome = match crate::frontend::attract::poll_start(timer, signature_first_byte, automatic)
    {
        crate::frontend::attract::Poll::Continue => OverlayOutcome::Continue,
        crate::frontend::attract::Poll::TimedOut => OverlayOutcome::TimedOut,
        crate::frontend::attract::Poll::StartReentry => OverlayOutcome::StartReentry,
    };
    (outcome, vec![8, 15])
}
#[cfg(test)]
mod tests;

fn show_window(window: &Window, display: &mut Display) {
    display.window_edges = window.buffers[usize::from(window.selector & 1)][..256].to_vec();
}
fn enable_window(window: &mut Window, display: &mut Display) {
    window.hdma_enabled = true;
    display.masked_layers = 0;
    display.window_inverted_layers = 0;
    for layer in 0..5 {
        let control = if layer < 4 {
            (window.masks[layer / 2] >> (layer % 2 * 4)) & 15
        } else {
            window.masks[2] & 15
        };
        if control & 2 != 0 {
            display.masked_layers |= 1 << layer;
            if control & 1 != 0 {
                display.window_inverted_layers |= 1 << layer;
            }
        }
    }
    show_window(window, display);
}
fn transition_draw(
    state: &mut RaceTransition,
    layout: CountdownLayout,
    assets: &TransitionAssets,
    memory: &mut VideoMemory,
    animate: bool,
) -> Result<()> {
    draw_countdown(&mut state.countdown, &layout, assets, memory)?;
    state.banner_calls = state.banner_calls.wrapping_add(1);
    if animate {
        state.animation_calls = state.animation_calls.wrapping_add(1);
    }
    if state.kind == RaceTransitionKind::Postrace && state.countdown.state >= POSTRACE_END {
        state.countdown.state = POSTRACE_FIRST;
    }
    Ok(())
}
