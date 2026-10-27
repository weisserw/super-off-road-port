//! Licensed/software splashes, session entry and attract presentation.
//! Independent source derivations and flow boundaries: attract.trace.md.
use crate::controls::BUTTON_START;
use crate::presentation::{
    memory,
    transitions::{self, FadeDirection},
};
use crate::trucks::nitro::NitroStock;
use crate::{
    controls::{CONTROLLER_COUNT, ControllerWord},
    presentation::{Display, MapImage, VideoMemory, transitions::Fade},
    trucks::motion::Truck,
};
use anyhow::{Context, Result, ensure};
pub const SIGNATURE_BYTES: usize = 5;
pub const LICENSE_WAIT: u16 = 140;
pub const SOFTWARE_WAIT: u16 = 110;
pub const HIGHSCORE_WAIT: u16 = 500;
pub const HIGHSCORE_INITIAL_SCROLL: u16 = 0x118;
pub const HIGHSCORE_FINAL_SCROLL: u16 = 0x1d6;
pub const ATTRACT_NITRO: NitroStock = NitroStock::new(99);
pub const AI_INITIAL_NITRO: NitroStock = NitroStock::new(15);
pub const FRONTEND_AUDIO: u16 = 0x0201;
pub const FRONTEND_TILES_ROM: std::ops::Range<usize> = 0x35400..0x37980;
pub const FRONTEND_TAIL_ROM: std::ops::Range<usize> = 0x2dac0..0x2e400;
pub const FRONTEND_PALETTE_ROM: std::ops::Range<usize> = 0x2e400..0x2e4e0;
pub const ATTRACT_TILES_ROM: std::ops::Range<usize> = 0x40000..0x44d00;
pub const ATTRACT_MAP_ROM: std::ops::Range<usize> = 0x44d00..0x45300;
pub const ATTRACT_PALETTE_ROM: std::ops::Range<usize> = 0x45340..0x45440;
/// Session-only records: future results owner refines methods, keeping this one
/// shared value rather than a second attract copy. Cartridge has no battery.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionRecords {
    #[serde(with = "crate::snapshot::array")]
    pub track_names: [u8; 48],
    #[serde(with = "crate::snapshot::array")]
    pub track_times: [u8; 32],
    #[serde(with = "crate::snapshot::array")]
    pub score_digits: [u8; 60],
    #[serde(with = "crate::snapshot::array")]
    pub score_names: [u8; 30],
    #[serde(with = "crate::snapshot::array")]
    pub score_order: [u8; 10],
    #[serde(with = "crate::snapshot::array")]
    pub truck_names: [u8; 12],
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendAssets {
    pub signature: [u8; SIGNATURE_BYTES],
    pub attract_slots: [u8; 4],
    pub frontend_tiles: Vec<u8>,
    pub frontend_tail: Vec<u8>,
    pub frontend_palette: Vec<u16>,
    pub attract_tiles: Vec<u8>,
    pub attract_map: Vec<u16>,
    pub attract_palette: Vec<u16>,
    pub license: MapImage,
    pub software: MapImage,
}
pub fn load_assets(rom: &[u8]) -> Result<FrontendAssets> {
    let slice = |span: std::ops::Range<usize>| -> Result<Vec<u8>> {
        Ok(rom
            .get(span)
            .context("truncated frontend resource")?
            .to_vec())
    };
    let map = |source, descriptor, destination_word| -> Result<MapImage> {
        let rectangle = memory::decode_rectangle(
            rom.get(descriptor..descriptor + 20)
                .context("missing splash descriptor")?,
        )?;
        Ok(MapImage {
            words: memory::read_rom_rectangle(rom, source, &rectangle)?,
            rectangle,
            destination_word,
        })
    };
    let assets = FrontendAssets {
        signature: rom
            .get(0x21ae..0x21b3)
            .context("missing signature")?
            .try_into()
            .unwrap(),
        attract_slots: rom
            .get(0x20de..0x20e2)
            .context("missing attract slots")?
            .try_into()
            .unwrap(),
        frontend_tiles: slice(FRONTEND_TILES_ROM)?,
        frontend_tail: slice(FRONTEND_TAIL_ROM)?,
        frontend_palette: memory::decode_palette(&slice(FRONTEND_PALETTE_ROM)?)?,
        attract_tiles: slice(ATTRACT_TILES_ROM)?,
        attract_map: slice(ATTRACT_MAP_ROM)?
            .chunks_exact(2)
            .map(|v| u16::from_le_bytes([v[0], v[1]]))
            .collect(),
        attract_palette: memory::decode_palette(&slice(ATTRACT_PALETTE_ROM)?)?,
        license: map(0x05e4e0, 0x21b3, 0x4047)?,
        software: map(0x05e864, 0x21c7, 0x40c3)?,
    };
    validate_assets(&assets)?;
    Ok(assets)
}
pub fn validate_assets(assets: &FrontendAssets) -> Result<()> {
    ensure!(
        assets.frontend_tiles.len() == FRONTEND_TILES_ROM.len()
            && assets.frontend_tail.len() == FRONTEND_TAIL_ROM.len()
            && assets.frontend_palette.len() == 112
            && assets.attract_tiles.len() == ATTRACT_TILES_ROM.len()
            && assets.attract_map.len() == ATTRACT_MAP_ROM.len() / 2
            && assets.attract_palette.len() == 128,
        "invalid frontend resource span"
    );
    for map in [&assets.license, &assets.software] {
        let (width, rows) = memory::validate_rectangle(&map.rectangle)?;
        ensure!(map.words.len() == width * rows, "invalid splash map");
    }
    Ok(())
}
pub fn signature_matches(actual: &[u8; SIGNATURE_BYTES], expected: &[u8; SIGNATURE_BYTES]) -> bool {
    actual == expected
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum EntryPhase {
    InitialLicense,
    RestartEntry,
    InitializeMoney,
    AttractPanel,
    DriverSelection,
    ReadyForRace,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    #[serde(with = "crate::snapshot::array")]
    pub signature: [u8; SIGNATURE_BYTES],
    pub records: SessionRecords,
    #[serde(with = "crate::snapshot::array")]
    pub wallets: [[u8; 6]; 4],
    pub selected_wallet: u8,
    pub continuation_progress: u16,
    pub page_limit: u8,
    #[serde(with = "crate::snapshot::array")]
    pub joined_pages: [u8; 2],
    pub active_humans: u8,
    pub order_slot: u8,
    pub attract_rotation: u8,
    pub start_reentry: u8,
    pub inhibit_video: u8,
    pub enabled_layers: u8,
    pub window_shape: u16,
    pub nmi_and_automatic_inputs: bool,
    pub audio_commands: Vec<u16>,
    pub phase: EntryPhase,
}
/// Initial hardware/display boundary, before optional blocking splash. It does
/// not yet clear wallets or copy records; ordering matters for early Start.
pub fn begin_entry(session: &mut Session, display: &mut Display, assets: &FrontendAssets) {
    session.audio_commands.push(FRONTEND_AUDIO);
    session.continuation_progress = 0;
    session.page_limit = 0;
    session.window_shape = 0;
    session.nmi_and_automatic_inputs = true;
    display.layers[0].scroll = [0; 2];
    display.layers[2].map_byte_base = 0x9000;
    // Frontend BG3SC=$4A selects a vertical second tilemap page. Highscore
    // scrolling must reach that page rather than wrap back through row zero.
    display.layers[2].tall = true;
    display.color_math_layers = 0;
    display.use_subscreen = false;
    display.half_color = false;
    display.subtract_color = false;
    session.phase = if signature_matches(&session.signature, &assets.signature) {
        EntryPhase::InitializeMoney
    } else {
        session.signature[0] = 0;
        EntryPhase::InitialLicense
    };
}
/// Called after the cold initial licensed/software sequence returns. Source
/// initializes records only here, after signature installation, never before.
pub fn finish_initial_splashes(session: &mut Session, defaults: &SessionRecords) {
    session.records = defaults.clone();
    session.phase = EntryPhase::InitializeMoney;
}
/// Shared warm/cold money init, contiguous stock/upgrade clear, then reentry
/// bypass or panel selection. Does not reset unrelated motion history.
pub fn initialize_session(session: &mut Session, trucks: &mut [Truck; 4], display: &mut Display) {
    super::shop::initialize_wallets(session);
    session.inhibit_video = session.inhibit_video.wrapping_add(1);
    display.main_layers = 31;
    display.main_window_layers = 31;
    session.enabled_layers = 5;
    clear_stock_upgrades(trucks);
    session.phase = if session.start_reentry != 0 {
        session.start_reentry = 0;
        EntryPhase::DriverSelection
    } else {
        EntryPhase::AttractPanel
    };
}
/// Invoked after panel→highscores→licensed/software return. Counter increments
/// before the table lookup; high bits are masked after wrapping byte increment.
pub fn finish_attract_presentation(
    session: &mut Session,
    trucks: &mut [Truck; 4],
    display: &mut Display,
    assets: &FrontendAssets,
) {
    session.active_humans = 0;
    session.attract_rotation = session.attract_rotation.wrapping_add(1) & 3;
    session.order_slot = assets.attract_slots[usize::from(session.attract_rotation)];
    for truck in trucks {
        truck.nitro = ATTRACT_NITRO;
    }
    finish_blank(session, display);
    session.phase = EntryPhase::ReadyForRace;
}
/// Source calls selection THEN shop even when nobody joined. Callers run those
/// owners before this completion; nonzero word gate precedes byte mask combine.
pub fn finish_driver_selection(session: &mut Session, trucks: &mut [Truck; 4]) {
    if session.joined_pages == [0; 2] {
        session.phase = EntryPhase::RestartEntry;
        return;
    }
    session.active_humans = session.joined_pages[0] | session.joined_pages[1].wrapping_shl(2);
    for (slot, truck) in trucks.iter_mut().enumerate() {
        if session.active_humans & (1 << slot) == 0 {
            truck.nitro = AI_INITIAL_NITRO;
        }
    }
    session.order_slot = 0;
    session.inhibit_video = session.inhibit_video.wrapping_add(1);
    session.phase = EntryPhase::ReadyForRace;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Poll {
    Continue,
    TimedOut,
    StartReentry,
}
/// Any of four Starts, first signature byte only. A Start reentry preserves the
/// timer; ordinary poll DEC word returns N from that decrement (SEP preserves N).
pub fn poll_start(
    timer: &mut u16,
    signature_first: u8,
    automatic: [ControllerWord; CONTROLLER_COUNT],
) -> Poll {
    if signature_first != 0 && automatic.iter().any(|word| word.0 & BUTTON_START != 0) {
        return Poll::StartReentry;
    }
    *timer = timer.wrapping_sub(1);
    if (*timer as i16) < 0 {
        Poll::TimedOut
    } else {
        Poll::Continue
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SplashPhase {
    FadeInLicense,
    WaitLicense,
    FadeOutLicense,
    FadeInSoftware,
    WaitSoftware,
    FadeOutSoftware,
    Done,
    Reentry,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Splash {
    pub phase: SplashPhase,
    pub timer: u16,
    pub fade: Fade,
}
pub fn begin_splashes(
    assets: &FrontendAssets,
    memory: &mut VideoMemory,
    display: &mut Display,
    inhibit: &mut u8,
) -> Result<Splash> {
    validate_assets(assets)?;
    display.force_blank = true;
    display.main_layers = 1;
    display.main_window_layers = 1;
    install_splash_graphics(assets, memory, inhibit)?;
    memory::draw_rectangle(
        memory,
        &assets.license.words,
        assets.license.destination_word,
        &assets.license.rectangle,
    )?;
    Ok(Splash {
        phase: SplashPhase::FadeInLicense,
        timer: 0,
        fade: new_fade(FadeDirection::In),
    })
}
/// One NMI boundary: executes real transition fades and source rectangle writes,
/// not merely a list of requested phases. Signature copy follows software wait.
pub fn advance_splashes(
    state: &mut Splash,
    signature: &mut [u8; SIGNATURE_BYTES],
    assets: &FrontendAssets,
    memory: &mut VideoMemory,
    display: &mut Display,
    automatic: [ControllerWord; CONTROLLER_COUNT],
) -> Result<()> {
    validate_assets(assets)?;
    match state.phase {
        SplashPhase::WaitLicense | SplashPhase::WaitSoftware => {
            match poll_start(&mut state.timer, signature[0], automatic) {
                Poll::Continue => {}
                Poll::StartReentry => state.phase = SplashPhase::Reentry,
                Poll::TimedOut => {
                    if state.phase == SplashPhase::WaitSoftware {
                        *signature = assets.signature;
                        state.phase = SplashPhase::FadeOutSoftware;
                    } else {
                        state.phase = SplashPhase::FadeOutLicense;
                    }
                    state.fade = new_fade(FadeDirection::Out);
                }
            }
        }
        SplashPhase::Done | SplashPhase::Reentry => {}
        _ => {
            transitions::advance_fade(&mut state.fade, display);
            if state.fade.complete {
                match state.phase {
                    SplashPhase::FadeInLicense => {
                        state.timer = LICENSE_WAIT;
                        state.phase = SplashPhase::WaitLicense;
                    }
                    SplashPhase::FadeInSoftware => {
                        state.timer = SOFTWARE_WAIT;
                        state.phase = SplashPhase::WaitSoftware;
                    }
                    SplashPhase::FadeOutLicense => {
                        memory::draw_rectangle(
                            memory,
                            &assets.software.words,
                            assets.software.destination_word,
                            &assets.software.rectangle,
                        )?;
                        state.fade = new_fade(FadeDirection::In);
                        state.phase = SplashPhase::FadeInSoftware;
                    }
                    SplashPhase::FadeOutSoftware => state.phase = SplashPhase::Done,
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
/// Source DMA mechanics are replaced by named contiguous native copies. Palette
/// untouched slots remain intact; font/driver resource installers are other owners.
pub fn install_splash_graphics(
    assets: &FrontendAssets,
    memory: &mut VideoMemory,
    inhibit: &mut u8,
) -> Result<()> {
    validate_assets(assets)?;
    *inhibit = inhibit.wrapping_add(1);
    memory.vram[0x8000..0x8802].fill(0);
    memory.vram[..0x2580].copy_from_slice(&assets.frontend_tiles);
    memory.vram[0x2580..0x2ec0].copy_from_slice(&assets.frontend_tail);
    memory.palette[0] = 0;
    memory.palette[16..128].copy_from_slice(&assets.frontend_palette);
    Ok(())
}
pub fn install_attract_graphics(assets: &FrontendAssets, memory: &mut VideoMemory) -> Result<()> {
    validate_assets(assets)?;
    memory.vram[0x8000..0x8802].fill(0);
    memory.vram[0x9000..0xa002].fill(0);
    memory.vram[..0x4d00].copy_from_slice(&assets.attract_tiles);
    memory.palette[..128].copy_from_slice(&assets.attract_palette);
    for (i, &word) in assets.attract_map.iter().enumerate() {
        memory::write_word(memory, 0x4040 + i as u16, word);
    }
    Ok(())
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Highscores {
    pub scroll: u16,
    pub timer: u16,
    pub finished: bool,
    pub reentry: bool,
}
pub fn begin_highscores(state: &mut Highscores, display: &mut Display) {
    state.scroll = HIGHSCORE_INITIAL_SCROLL;
    state.timer = HIGHSCORE_WAIT;
    state.finished = false;
    state.reentry = false;
    display.layers[2].scroll[1] = state.scroll;
}
/// Writes PREincrement scroll, then polls. The record renderer/contract owns rows.
pub fn advance_highscores(
    state: &mut Highscores,
    signature_first: u8,
    display: &mut Display,
    automatic: [ControllerWord; CONTROLLER_COUNT],
) {
    if state.finished || state.reentry {
        return;
    }
    let old = state.scroll;
    if old < HIGHSCORE_FINAL_SCROLL {
        state.scroll = old.wrapping_add(1);
    }
    display.layers[2].scroll[1] = old;
    match poll_start(&mut state.timer, signature_first, automatic) {
        Poll::Continue => {}
        Poll::TimedOut => state.finished = true,
        Poll::StartReentry => state.reentry = true,
    }
}
/// Attract panel owns real graphics, transition window and selected-wallet overlay.
/// Standard driver/high-score installation follows only after this service returns.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PanelPhase {
    Motion,
    Overlay,
    Complete,
    Reentry,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Panel {
    pub phase: PanelPhase,
    pub window: crate::presentation::transitions::Window,
    pub motion: crate::presentation::transitions::FrontendWindow,
    pub timer: u16,
}
pub fn begin_panel(
    session: &mut Session,
    trucks: &mut [Truck; 4],
    assets: &FrontendAssets,
    memory: &mut VideoMemory,
    display: &mut Display,
) -> Result<Panel> {
    validate_assets(assets)?;
    display.force_blank = true;
    session.enabled_layers = 5;
    clear_stock_upgrades(trucks);
    install_attract_graphics(assets, memory)?;
    display.main_layers = 5;
    display.main_window_layers = 5;
    let mut window = transitions::Window {
        center: [0; 2],
        extent: 0,
        shape: transitions::ShapeId::Frontend,
        masks: [0; 3],
        selector: 0,
        buffers: std::array::from_fn(|_| vec![transitions::CLOSED_EDGE; transitions::WINDOW_ROWS]),
        hdma_enabled: false,
    };
    let mut motion =
        transitions::begin_frontend(transitions::FrontendKind::Scrolling, &mut window, display);
    motion.abort_enabled = 1;
    Ok(Panel {
        phase: PanelPhase::Motion,
        window,
        motion,
        timer: 100,
    })
}
pub fn advance_panel(
    panel: &mut Panel,
    session: &mut Session,
    shape: &crate::presentation::transitions::Shape,
    display: &mut Display,
    automatic: [ControllerWord; CONTROLLER_COUNT],
) -> Result<()> {
    match panel.phase {
        PanelPhase::Motion => {
            transitions::advance_frontend(
                &mut panel.motion,
                &mut panel.window,
                shape,
                display,
                automatic,
            )?;
            if panel.motion.complete {
                panel.phase = PanelPhase::Overlay;
            }
        }
        PanelPhase::Overlay => {
            display.brightness = 8;
            super::shop::normalize_wallet(session)?;
            display.brightness = 15;
            match poll_start(&mut panel.timer, session.signature[0], automatic) {
                Poll::Continue => {}
                Poll::TimedOut => panel.phase = PanelPhase::Complete,
                Poll::StartReentry => panel.phase = PanelPhase::Reentry,
            }
        }
        _ => {}
    }
    Ok(())
}
#[cfg(test)]
mod tests;

fn clear_stock_upgrades(trucks: &mut [Truck; 4]) {
    for truck in trucks {
        truck.nitro = NitroStock::default();
        truck.upgrades = Default::default();
    }
}
fn finish_blank(session: &mut Session, display: &mut Display) {
    session.inhibit_video = session.inhibit_video.wrapping_add(1);
    display.masked_layers = 0;
    display.window_inverted_layers = 0;
    display.force_blank = true;
}
fn new_fade(direction: FadeDirection) -> Fade {
    Fade {
        direction,
        primed: false,
        complete: false,
        animation_counter: 0,
        enabled_layers: 1,
    }
}
