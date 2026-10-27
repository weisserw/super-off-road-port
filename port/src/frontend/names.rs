//! Human joining, live name editing and shared driver panels. See names.trace.md.
use super::attract::Session;
use crate::presentation::{
    memory, sprites,
    transitions::{self, Fade, FadeDirection},
};
use crate::{
    controls::{ControllerWord, DirectionTables},
    presentation::{Display, MapImage, VideoMemory, memory::SharedGraphics, sprites::SpriteArrays},
};
use anyhow::{Context, Result, ensure};
pub const GRID_COLUMNS: usize = 5;
pub const GRID_ROWS: usize = 7;
pub const GRID_CELLS: usize = GRID_COLUMNS * GRID_ROWS;
pub const TEMPORARY_BYTES: usize = 10;
pub const JOIN_COUNTDOWN: u8 = 0x5f;
pub const REPEAT_DELAY: u8 = 8;
pub const CONFIRM_CURSOR_LIMIT: u8 = 6;
pub const PLACEHOLDER: u8 = 0x2a;
pub const ERASE_TOKEN: u8 = 0x80;
pub const EXIT_TOKEN: u8 = 0xff;
pub const SELECTION_AUDIO: u16 = 0x0501;
pub const COMPLETE_AUDIO: u16 = 0xe803;
pub const ACK_AUDIO: u16 = 0x0101;
pub const DEFAULTS_ROM: usize = 0x3fe8;
pub const CELL_X_ROM: usize = 0x3ff2;
pub const CELL_Y_ROM: usize = 0x4015;
pub const CELL_GLYPH_ROM: usize = 0x4038;
pub const DRIVER_TILES_ROM: std::ops::Range<usize> = 0x48000..0x4cda0;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NameAssets {
    pub frontend_shape: transitions::Shape,
    pub defaults: [u8; TEMPORARY_BYTES],
    pub cell_x: [u8; GRID_CELLS],
    pub cell_y: [u8; GRID_CELLS],
    pub glyph: [u8; GRID_CELLS],
    pub grid_rows: Vec<u8>,
    pub instructions: Vec<u8>,
    pub headings: [Vec<u8>; 4],
    pub heading_records: [(u16, u16); 4],
    pub driver_tiles: Vec<u8>,
    pub base_palette: Vec<u16>,
    pub alternate_palette: Vec<u16>,
    pub continued_palettes: [Vec<u16>; 2],
    pub second_palettes: [Vec<u16>; 2],
    pub heading: MapImage,
    pub standard_panels: [MapImage; 2],
    pub continued_panels: [MapImage; 2],
    pub inline_heading: Vec<u8>,
}
pub fn load_assets(rom: &[u8]) -> Result<NameAssets> {
    let bytes = |start: usize, length: usize| -> Result<Vec<u8>> {
        Ok(rom
            .get(start..start + length)
            .context("truncated name resource")?
            .to_vec())
    };
    let palette = |start, length| memory::decode_palette(&bytes(start, length)?);
    let map = |source, descriptor, destination_word| -> Result<MapImage> {
        let rectangle = memory::decode_rectangle(&bytes(descriptor, 20)?)?;
        Ok(MapImage {
            words: memory::read_rom_rectangle(rom, source, &rectangle)?,
            rectangle,
            destination_word,
        })
    };
    let records = bytes(0x3aac, 16)?;
    let heading_records = std::array::from_fn(|i| {
        (
            u16::from_le_bytes(records[i * 4..i * 4 + 2].try_into().unwrap()),
            u16::from_le_bytes(records[i * 4 + 2..i * 4 + 4].try_into().unwrap()),
        )
    });
    let text = |pointer: u16| -> Result<Vec<u8>> {
        let start = usize::from(pointer & 0x7fff);
        let tail = rom.get(start..).context("missing name text")?;
        let end = tail
            .iter()
            .position(|&b| b == 0xff)
            .context("unterminated name text")?;
        Ok(tail[..=end].to_vec())
    };
    let mut headings = Vec::new();
    for &(_, pointer) in &heading_records {
        headings.push(text(pointer)?);
    }
    let assets = NameAssets {
        frontend_shape: transitions::decode_shapes(
            rom.get(0x3616..0x361f).context("missing window shapes")?,
            rom,
        )?[0]
            .clone(),
        defaults: bytes(DEFAULTS_ROM, TEMPORARY_BYTES)?.try_into().unwrap(),
        cell_x: bytes(CELL_X_ROM, GRID_CELLS)?.try_into().unwrap(),
        cell_y: bytes(CELL_Y_ROM, GRID_CELLS)?.try_into().unwrap(),
        glyph: bytes(CELL_GLYPH_ROM, GRID_CELLS)?.try_into().unwrap(),
        grid_rows: bytes(0x3a43, 71)?,
        instructions: bytes(0x3a8a, 22)?,
        headings: headings.try_into().unwrap(),
        heading_records,
        driver_tiles: bytes(DRIVER_TILES_ROM.start, DRIVER_TILES_ROM.len())?,
        base_palette: palette(0x4d410, 256)?,
        alternate_palette: palette(0x4d158, 64)?,
        continued_palettes: [palette(0x4d390, 32)?, palette(0x4d3d0, 32)?],
        second_palettes: [palette(0x4d3b0, 32)?, palette(0x4d3f0, 32)?],
        heading: map(0x09cda0, 0x33c2, 0x4020)?,
        standard_panels: [
            map(0x09cf60, 0x33ea, 0x4121)?,
            map(0x09cf60, 0x3412, 0x4131)?,
        ],
        continued_panels: [
            map(0x09d198, 0x33d6, 0x4121)?,
            map(0x09d198, 0x33fe, 0x4131)?,
        ],
        inline_heading: text(0xb463)?,
    };
    validate_assets(&assets)?;
    Ok(assets)
}
pub fn validate_assets(assets: &NameAssets) -> Result<()> {
    ensure!(
        assets.grid_rows.iter().filter(|&&b| b == 0xff).count() == 7
            && assets.grid_rows.last() == Some(&0xff),
        "invalid seven-row grid"
    );
    ensure!(
        assets.driver_tiles.len() == DRIVER_TILES_ROM.len()
            && assets.base_palette.len() == 128
            && assets.alternate_palette.len() == 32
            && assets
                .continued_palettes
                .iter()
                .chain(assets.second_palettes.iter())
                .all(|p| p.len() == 16),
        "invalid driver asset span"
    );
    ensure!(
        assets.instructions.last() == Some(&0xff)
            && assets.headings.iter().all(|t| t.last() == Some(&0xff))
            && assets.inline_heading.last() == Some(&0xff),
        "invalid driver text"
    );
    for map in std::iter::once(&assets.heading)
        .chain(assets.standard_panels.iter())
        .chain(assets.continued_panels.iter())
    {
        let (width, rows) = memory::validate_rectangle(&map.rectangle)?;
        ensure!(map.words.len() == width * rows, "invalid driver map");
    }
    Ok(())
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Editor {
    pub column: u8,
    pub row: u8,
    pub cursor_byte: u8,
    pub repeat: u8,
    pub button_history: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Phase {
    EntryFade,
    PageSetup,
    FadeIn,
    WaitFrame,
    JoinDraw(u8),
    ClearPanel(u8),
    FinalWait,
    ContractWindow,
    Ack,
    Complete,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    pub sequence: Option<Sequence>,
    pub resume: Resume,
    pub page: u8,
    pub done_mask: u8,
    pub countdown: u8,
    #[serde(with = "crate::snapshot::array")]
    pub editors: [Editor; 2],
    #[serde(with = "crate::snapshot::array")]
    pub temporary: [[u8; TEMPORARY_BYTES]; 2],
    pub phase: Phase,
    #[serde(with = "crate::snapshot::array")]
    pub sampled: [ControllerWord; 2],
    /// Completion is a resumable source call sequence, never a blocking host loop.
    pub window_abort_enabled: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Sequence {
    Fade(Fade),
    Window {
        window: transitions::Window,
        motion: transitions::FrontendWindow,
    },
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Resume {
    #[default]
    Sample,
    Join(usize),
    Edit(usize),
    Complete,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JoinResult {
    Skipped,
    NoEdge,
    AwaitGrid,
}
pub fn begin_selection(session: &mut Session, selection: &mut Selection) {
    session.inhibit_video = session.inhibit_video.wrapping_add(1);
    session.joined_pages = [0; 2];
    selection.page = 0;
    selection.phase = Phase::EntryFade;
    selection.sequence = Some(Sequence::Fade(fade(FadeDirection::Out)));
    selection.resume = Resume::Sample;
}
/// Page initialization preserves repeat/history; source does not explicitly clear either.
pub fn initialize_page(selection: &mut Selection, session: &mut Session, assets: &NameAssets) {
    session.joined_pages[usize::from(selection.page != 0)] = 0;
    session.enabled_layers = 0x15;
    selection.done_mask = 0;
    selection.countdown = JOIN_COUNTDOWN;
    selection.temporary = [assets.defaults; 2];
    for editor in &mut selection.editors {
        editor.column = 0;
        editor.row = 0;
        editor.cursor_byte = 0;
    }
}
pub fn try_join(
    selection: &mut Selection,
    session: &mut Session,
    player: usize,
    input: ControllerWord,
) -> Result<JoinResult> {
    ensure!(
        player < 2 && selection.page < 2,
        "invalid joining player/page"
    );
    let bit = 1 << player;
    if (session.joined_pages[usize::from(selection.page)] | selection.done_mask) & bit != 0 {
        return Ok(JoinResult::Skipped);
    }
    if edge(
        &mut selection.editors[player].button_history,
        input.0 & crate::controls::BUTTON_SELECT != 0,
    ) {
        session.joined_pages[usize::from(selection.page)] |= bit;
        Ok(JoinResult::AwaitGrid)
    } else {
        Ok(JoinResult::NoEdge)
    }
}
pub fn draw_joined_grid(
    selection: &Selection,
    player: usize,
    assets: &NameAssets,
    memory: &mut VideoMemory,
    sprites: &mut SpriteArrays,
) -> Result<()> {
    ensure!(player < 2, "invalid grid player");
    validate_assets(assets)?;
    let _ = selection;
    memory::draw_seven_rows(
        memory,
        &assets.grid_rows,
        if player == 0 { 0x4943 } else { 0x4954 },
    )?;
    let base = player * 2;
    sprites.tile_attributes[base] = 0x2026;
    sprites.tile_attributes[base + 1] = 0x2028;
    sprites.x[base] = if player == 0 { 0x15 } else { 0x9d };
    sprites.x[base + 1] = sprites.x[base];
    sprites.y[base] = 0x48;
    sprites.y[base + 1] = 0x58;
    Ok(())
}
/// Advances one local editor, including live preview even with no confirmation.
pub fn edit_player(
    selection: &mut Selection,
    session: &Session,
    player: usize,
    input: ControllerWord,
    tables: &DirectionTables,
    assets: &NameAssets,
    sprites: &mut SpriteArrays,
) -> Result<()> {
    ensure!(
        player < 2 && selection.page < 2,
        "invalid editor player/page"
    );
    validate_assets(assets)?;
    let bit = 1 << player;
    if session.joined_pages[usize::from(selection.page)] & bit == 0
        || selection.done_mask & bit != 0
    {
        return Ok(());
    }
    let editor = &mut selection.editors[player];
    ensure!(
        usize::from(editor.column) < GRID_COLUMNS
            && usize::from(editor.row) < GRID_ROWS
            && editor.cursor_byte <= CONFIRM_CURSOR_LIMIT,
        "invalid name editor cursor"
    );
    let high = (input.0 >> 8) as u8;
    if high == 0 {
        editor.repeat = 0;
    } else if editor.repeat != 0 {
        editor.repeat = editor.repeat.wrapping_sub(1);
    } else {
        editor.repeat = REPEAT_DELAY;
        let delta = tables.frontend_delta(high);
        editor.column = wrap(editor.column, delta.columns, GRID_COLUMNS as u8);
        editor.row = wrap(editor.row, delta.rows, GRID_ROWS as u8);
    }
    let cell = usize::from(editor.row) * GRID_COLUMNS + usize::from(editor.column);
    let x = assets.cell_x[cell].wrapping_add(if player == 0 { 0x15 } else { 0x9d });
    let y = assets.cell_y[cell].wrapping_add(0x48);
    for part in 0..2 {
        let slot = player * 2 + part;
        sprites.x[slot] = (sprites.x[slot] & 0xff00) | u16::from(x);
        sprites.y[slot] = y.wrapping_add(part as u8 * 16);
    }
    let glyph = assets.glyph[cell];
    selection.temporary[player][usize::from(editor.cursor_byte)] = if (glyph as i8) < 0 {
        PLACEHOLDER
    } else {
        glyph
    };
    if edge(
        &mut editor.button_history,
        input.0 & crate::controls::BUTTON_B != 0,
    ) {
        if glyph == EXIT_TOKEN {
            selection.done_mask |= bit;
        } else if (glyph as i8) < 0 {
            editor.cursor_byte = editor.cursor_byte.saturating_sub(2);
        } else if editor.cursor_byte < CONFIRM_CURSOR_LIMIT {
            editor.cursor_byte += 2;
        }
    }
    Ok(())
}
pub fn update_countdown(
    selection: &mut Selection,
    session: &Session,
    frame: u8,
    memory: &mut VideoMemory,
) -> Result<()> {
    ensure!(selection.page < 2, "invalid countdown page");
    if selection.countdown == 0 {
        return Ok(());
    }
    selection.temporary[0][7] = 1;
    selection.temporary[0][8] = (selection.countdown >> 4).wrapping_add(1);
    for player in 0..2 {
        if session.joined_pages[usize::from(selection.page)] & (1 << player) == 0 {
            memory::draw_text(
                memory,
                &[selection.temporary[0][7], selection.temporary[0][8], 0xff],
                if player == 0 { 0x4a67 } else { 0x4a77 },
                memory::TextSize::Paired,
            )?;
        }
    }
    if frame & 3 == 0 {
        selection.countdown = selection.countdown.wrapping_sub(1);
        if selection.countdown == 0 {
            selection.done_mask |= session.joined_pages[usize::from(selection.page)] ^ 3;
        }
    }
    Ok(())
}
pub fn hide_done(selection: &Selection, memory: &mut VideoMemory, sprites: &mut SpriteArrays) {
    for player in 0..2 {
        if selection.done_mask & (1 << player) != 0 {
            clear_player(player, memory, sprites);
        }
    }
}
pub fn commit_names(selection: &Selection, session: &mut Session) {
    let base = if selection.page == 0 { 0 } else { 6 };
    for player in 0..2 {
        for character in 0..3 {
            session.records.truck_names[base + player * 3 + character] =
                selection.temporary[player][character * 2];
        }
    }
}
pub fn finish_page(selection: &mut Selection, session: &mut Session) {
    session.audio_commands.push(ACK_AUDIO);
    selection.page = selection.page.wrapping_add(1);
    selection.phase = if selection.page <= session.page_limit {
        Phase::PageSetup
    } else {
        Phase::Complete
    };
    selection.sequence = None;
}
/// Installs source graphics/palette and standard or continued panel variants.
/// Standard=true is the unconditional pair used by names/attract; shop=false selects joined variants.
pub fn install_driver_panels(
    assets: &NameAssets,
    page: u8,
    joined: u8,
    standard: bool,
    memory: &mut VideoMemory,
    display: &mut Display,
) -> Result<()> {
    validate_assets(assets)?;
    ensure!(page < 2, "invalid driver page");
    memory::fill_map(memory, 0x4000, 0);
    memory.vram[..assets.driver_tiles.len()].copy_from_slice(&assets.driver_tiles);
    memory.palette[..128].copy_from_slice(&assets.base_palette);
    if page != 0 {
        memory.palette[64..96].copy_from_slice(&assets.alternate_palette);
    }
    if !standard {
        memory::draw_rectangle(
            memory,
            &assets.heading.words,
            assets.heading.destination_word,
            &assets.heading.rectangle,
        )?;
    }
    for player in 0..2 {
        let continued = !standard && joined & (1 << player) != 0;
        let map = if continued {
            &assets.continued_panels[player]
        } else {
            &assets.standard_panels[player]
        };
        memory::draw_rectangle(memory, &map.words, map.destination_word, &map.rectangle)?;
        if continued {
            let colors = if player == 0 {
                &assets.continued_palettes[usize::from(page != 0)]
            } else {
                &assets.second_palettes[usize::from(page != 0)]
            };
            let at = 64 + player * 16;
            memory.palette[at..at + 16].copy_from_slice(colors);
        }
    }
    if !standard {
        memory::fill_map(memory, 0x4800, 0);
    }
    let _ = display;
    Ok(())
}
pub fn install_name_page(
    selection: &Selection,
    assets: &NameAssets,
    shared: &SharedGraphics,
    memory: &mut VideoMemory,
    display: &mut Display,
    sprites: &mut SpriteArrays,
) -> Result<()> {
    validate_assets(assets)?;
    memory::fill_map(memory, 0x4800, 0);
    memory::fill_map(memory, 0x4000, 0);
    install_driver_panels(assets, selection.page, 0, true, memory, display)?;
    memory::draw_rectangle(
        memory,
        &assets.heading.words,
        assets.heading.destination_word,
        &assets.heading.rectangle,
    )?;
    memory::upload_fonts(memory, shared, true)?;
    memory::upload_text_palette(memory, shared)?;
    memory::upload_cursor(memory, shared)?;
    for index in (0..4).rev() {
        memory::draw_text(
            memory,
            &assets.headings[index],
            assets.heading_records[index].0,
            memory::TextSize::Paired,
        )?;
    }
    memory::draw_text(
        memory,
        &assets.instructions[..9],
        0x4904,
        memory::TextSize::Single,
    )?;
    memory::draw_text(
        memory,
        &assets.instructions[9..18],
        0x4914,
        memory::TextSize::Single,
    )?;
    if selection.page != 0 {
        memory::draw_text(
            memory,
            &assets.instructions[18..20],
            0x490b,
            memory::TextSize::Single,
        )?;
        memory::draw_text(
            memory,
            &assets.instructions[20..],
            0x491b,
            memory::TextSize::Single,
        )?;
    }
    display.layers[2].scroll[1] = 0;
    sprites::initialize_arrays(sprites);
    sprites::hide_frontend(sprites, memory)?;
    Ok(())
}
/// One real NMI boundary resumes all nonblocking work until next source wait.
/// Countdown precedes sampling; join/clear waits resume without resampling.
// Explicit disjoint owner borrows keep mutation authority visible at this boundary.
#[allow(clippy::too_many_arguments)]
pub fn advance(
    selection: &mut Selection,
    session: &mut Session,
    assets: &NameAssets,
    shared: &SharedGraphics,
    tables: &DirectionTables,
    memory: &mut VideoMemory,
    display: &mut Display,
    sprites: &mut SpriteArrays,
    automatic: [ControllerWord; 4],
    animation_frame: u8,
) -> Result<()> {
    ensure!(
        selection.page < 2 || selection.phase == Phase::Complete,
        "invalid frontend page"
    );
    validate_assets(assets)?;
    match selection.phase {
        Phase::EntryFade => {
            let Some(Sequence::Fade(state)) = &mut selection.sequence else {
                anyhow::bail!("missing selection fade");
            };
            transitions::advance_fade(state, display);
            if state.complete {
                display.layers[0].scroll = [0; 2];
                display.masked_layers = 0;
                display.window_inverted_layers = 0;
                session.audio_commands.push(SELECTION_AUDIO);
                selection.phase = Phase::PageSetup;
                selection.sequence = None;
            }
            return Ok(());
        }
        Phase::PageSetup => {
            initialize_page(selection, session, assets);
            install_name_page(selection, assets, shared, memory, display, sprites)?;
            session.inhibit_video = 0;
            selection.sequence = Some(Sequence::Fade(fade(FadeDirection::In)));
            selection.phase = Phase::FadeIn;
            return Ok(());
        }
        Phase::FadeIn => {
            let Some(Sequence::Fade(state)) = &mut selection.sequence else {
                anyhow::bail!("missing name fade");
            };
            transitions::advance_fade(state, display);
            if state.complete {
                selection.sequence = None;
                selection.phase = Phase::WaitFrame;
            }
            return Ok(());
        }
        Phase::FinalWait => {
            session.inhibit_video = session.inhibit_video.wrapping_add(1);
            selection.window_abort_enabled = false;
            session.audio_commands.push(COMPLETE_AUDIO);
            let mut window = new_window();
            let motion = transitions::begin_frontend(
                transitions::FrontendKind::Contracting,
                &mut window,
                display,
            );
            selection.sequence = Some(Sequence::Window { window, motion });
            selection.phase = Phase::ContractWindow;
            return Ok(());
        }
        Phase::ContractWindow => {
            let Some(Sequence::Window { window, motion }) = &mut selection.sequence else {
                anyhow::bail!("missing name contraction");
            };
            // Original frontend shape belongs to the shared transition assets;
            // caller supplies that resource through the sequence before driving.
            transitions::advance_frontend(
                motion,
                window,
                &assets.frontend_shape,
                display,
                automatic,
            )?;
            if motion.complete {
                selection.phase = Phase::Ack;
            }
            return Ok(());
        }
        Phase::Ack => {
            finish_page(selection, session);
            return Ok(());
        }
        Phase::Complete => return Ok(()),
        Phase::JoinDraw(player) => {
            draw_joined_grid(selection, usize::from(player), assets, memory, sprites)?;
            selection.resume = Resume::Join(usize::from(player) + 1);
        }
        Phase::ClearPanel(player) => {
            clear_player(usize::from(player), memory, sprites);
            if player == 0 && selection.done_mask & 2 != 0 {
                selection.phase = Phase::ClearPanel(1);
                return Ok(());
            }
        }
        Phase::WaitFrame => {
            for player in (0..2).rev() {
                if session.joined_pages[usize::from(selection.page)] & (1 << player) != 0 {
                    memory::draw_text(
                        memory,
                        &selection.temporary[player],
                        if player == 0 { 0x4b05 } else { 0x4b16 },
                        memory::TextSize::Paired,
                    )?;
                }
            }
            let old_done = selection.done_mask;
            update_countdown(selection, session, animation_frame, memory)?;
            selection.resume = Resume::Sample;
            if selection.done_mask != old_done {
                selection.phase =
                    Phase::ClearPanel(if selection.done_mask & 1 != 0 { 0 } else { 1 });
                return Ok(());
            }
        }
    }
    loop {
        match selection.resume {
            Resume::Sample => {
                selection.sampled =
                    crate::controls::sample_frontend(selection.page, automatic).players;
                sprites::pack_sorted(sprites, &mut memory.staged_oam)?;
                selection.resume = Resume::Join(0);
            }
            Resume::Join(player) => {
                if player == 2 {
                    selection.resume = Resume::Edit(0);
                    continue;
                }
                if try_join(selection, session, player, selection.sampled[player])?
                    == JoinResult::AwaitGrid
                {
                    selection.phase = Phase::JoinDraw(player as u8);
                    return Ok(());
                }
                selection.resume = Resume::Join(player + 1);
            }
            Resume::Edit(player) => {
                if player == 2 {
                    selection.resume = Resume::Complete;
                    continue;
                }
                let old_done = selection.done_mask;
                edit_player(
                    selection,
                    session,
                    player,
                    selection.sampled[player],
                    tables,
                    assets,
                    sprites,
                )?;
                selection.resume = Resume::Edit(player + 1);
                if old_done != selection.done_mask {
                    selection.phase =
                        Phase::ClearPanel(if selection.done_mask & 1 != 0 { 0 } else { 1 });
                    return Ok(());
                }
            }
            Resume::Complete => {
                if selection.done_mask == 3 {
                    commit_names(selection, session);
                    sprites::hide_frontend(sprites, memory)?;
                    selection.phase = Phase::FinalWait;
                } else {
                    selection.phase = Phase::WaitFrame;
                }
                return Ok(());
            }
        }
    }
}
/// Conditional source entry only; no new player-facing control is authorized.
pub fn draw_inline_heading(assets: &NameAssets, memory: &mut VideoMemory) -> Result<()> {
    validate_assets(assets)?;
    memory::draw_text(
        memory,
        &assets.inline_heading,
        0x408a,
        memory::TextSize::Single,
    )?;
    Ok(())
}
#[cfg(test)]
mod tests;

fn edge(history: &mut u8, pressed: bool) -> bool {
    *history = (*history >> 1) | if pressed { 0x80 } else { 0 };
    *history & 0xc0 == 0x80
}
fn wrap(value: u8, delta: i8, bound: u8) -> u8 {
    let result = value.wrapping_add(delta as u8);
    if (result as i8) < 0 {
        bound - 1
    } else if result >= bound {
        0
    } else {
        result
    }
}
fn clear_player(player: usize, memory: &mut VideoMemory, sprites: &mut SpriteArrays) {
    memory::clear_name_panel(memory, if player == 0 { 0x4941 } else { 0x4954 });
    sprites.y[player * 2..player * 2 + 2].fill(0xf0);
}
fn fade(direction: FadeDirection) -> Fade {
    Fade {
        direction,
        primed: false,
        complete: false,
        animation_counter: 0,
        enabled_layers: 0x15,
    }
}
pub fn new_window() -> transitions::Window {
    transitions::Window {
        center: [0; 2],
        extent: 0,
        shape: transitions::ShapeId::Frontend,
        masks: [0; 3],
        selector: 0,
        buffers: std::array::from_fn(|_| vec![transitions::CLOSED_EDGE; transitions::WINDOW_ROWS]),
        hdma_enabled: false,
    }
}
