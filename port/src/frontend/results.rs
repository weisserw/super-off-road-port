//! Podium, session records, loss/continue and score rows;
//! independent width/branch/resource derivations are in results.trace.md.
use super::attract::{Session, SessionRecords};
use super::shop;
use crate::presentation::{self, memory, sprites};
use crate::{
    controls::ControllerWord,
    presentation::{Display, MapImage, VideoMemory},
    race::{Progress, RaceTimer},
    trucks::motion::Truck,
};
use anyhow::{Context, Result, ensure};
use std::collections::BTreeMap;
pub const TRUCKS: usize = 4;
pub use super::leaderboard::SCORE_ROWS;
pub const PODIUM_RANKS: usize = 3;
pub const AWARDS_TIMEOUT: u16 = 400;
pub const RECORD_COMMIT_TIME: u16 = 250;
pub const GAME_OVER_TIMEOUT: u16 = 300;
pub const PODIUM_AUDIO: u16 = 0x0401;
pub const LOSS_AUDIO: u16 = 0x1001;
pub const PODIUM_TILES_ROM: std::ops::Range<usize> = 0x38000..0x3c000;
pub const PODIUM_PALETTE_ROM: std::ops::Range<usize> = 0x3b480..0x3b680;
pub const PODIUM_MAP_ROM: std::ops::Range<usize> = 0x3ae40..0x3b480;
pub const POSTRACE_MAP_ROM: std::ops::Range<usize> = 0x3dad2..0x3e892;
pub const LOSS_TILES_A_ROM: std::ops::Range<usize> = 0x4d510..0x4ff90;
pub const LOSS_TILES_B_ROM: std::ops::Range<usize> = 0x55aba..0x56d9a;
pub const LOSS_PALETTE_ROM: std::ops::Range<usize> = 0x4659a..0x4665a;
pub const LOSS_MAP_ROM: std::ops::Range<usize> = 0x4665a..0x46c1a;
pub const LOSS_REPLACEMENTS_ROM: std::ops::Range<usize> = 0x46c1a..0x46d3a;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResultAssets {
    pub defaults: SessionRecords,
    pub awards: [u8; PODIUM_RANKS],
    pub award_destinations: [u16; PODIUM_RANKS],
    pub name_destinations: [u16; PODIUM_RANKS],
    pub podium_colors: [u8; PODIUM_RANKS],
    pub truck_palettes: [[u16; 4]; TRUCKS],
    pub podium_tiles: Vec<u8>,
    pub podium_palette: Vec<u16>,
    pub podium_map: Vec<u16>,
    pub postrace_map: Vec<u16>,
    pub loss_tiles_a: Vec<u8>,
    pub loss_tiles_b: Vec<u8>,
    pub loss_palette: Vec<u16>,
    pub loss_map: MapImage,
    pub loss_replacements: [Vec<u16>; 3],
    pub game_over_text: Vec<u8>,
    pub continue_text: Vec<u8>,
    pub choices_text: Vec<u8>,
    pub score_heading: Vec<u8>,
    pub score_destinations: [u16; SCORE_ROWS],
    pub score_name_destinations: [u16; SCORE_ROWS],
}
pub fn load_assets(rom: &[u8]) -> Result<ResultAssets> {
    let bytes = |start: usize, length: usize| -> Result<Vec<u8>> {
        Ok(rom
            .get(start..start + length)
            .context("truncated results asset")?
            .to_vec())
    };
    let words = |start, length| -> Result<Vec<u16>> {
        Ok(bytes(start, length * 2)?
            .chunks_exact(2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .collect())
    };
    let text = |start: usize| -> Result<Vec<u8>> {
        let tail = rom.get(start..).context("missing result text")?;
        let end = tail
            .iter()
            .position(|&b| b == 0xff)
            .context("unterminated results text")?;
        Ok(tail[..=end].to_vec())
    };
    let defaults = SessionRecords::from_legacy(
        bytes(0x3688, 48)?.try_into().unwrap(),
        bytes(0x36b8, 32)?.try_into().unwrap(),
        bytes(0x3815, 60)?.try_into().unwrap(),
        bytes(0x3851, 30)?.try_into().unwrap(),
        bytes(0x386f, 10)?.try_into().unwrap(),
        bytes(0x36d8, 12)?.try_into().unwrap(),
    );
    let palettes = words(0x3b5a0, 16)?;
    let descriptor = memory::decode_rectangle(&bytes(0x2a62, 20)?)?;
    let assets = ResultAssets {
        defaults,
        awards: bytes(0x2c28, 3)?.try_into().unwrap(),
        award_destinations: words(0x2c2c, 3)?.try_into().unwrap(),
        name_destinations: words(0x2c32, 3)?.try_into().unwrap(),
        podium_colors: bytes(0x2aa2, 3)?.try_into().unwrap(),
        truck_palettes: std::array::from_fn(|i| palettes[i * 4..i * 4 + 4].try_into().unwrap()),
        podium_tiles: bytes(PODIUM_TILES_ROM.start, PODIUM_TILES_ROM.len())?,
        podium_palette: words(PODIUM_PALETTE_ROM.start, PODIUM_PALETTE_ROM.len() / 2)?,
        podium_map: words(PODIUM_MAP_ROM.start, PODIUM_MAP_ROM.len() / 2)?,
        postrace_map: words(POSTRACE_MAP_ROM.start, POSTRACE_MAP_ROM.len() / 2)?,
        loss_tiles_a: bytes(LOSS_TILES_A_ROM.start, LOSS_TILES_A_ROM.len())?,
        loss_tiles_b: bytes(LOSS_TILES_B_ROM.start, LOSS_TILES_B_ROM.len())?,
        loss_palette: words(LOSS_PALETTE_ROM.start, LOSS_PALETTE_ROM.len() / 2)?,
        loss_map: MapImage {
            words: memory::read_rom_rectangle(rom, 0x08e65a, &descriptor)?,
            rectangle: descriptor,
            destination_word: 0x4020,
        },
        loss_replacements: [
            words(LOSS_REPLACEMENTS_ROM.start, 48)?,
            words(LOSS_REPLACEMENTS_ROM.start + 96, 48)?,
            words(LOSS_REPLACEMENTS_ROM.start + 192, 48)?,
        ],
        game_over_text: text(0x2a32)?,
        continue_text: text(0x2a43)?,
        choices_text: text(0x2a4e)?,
        score_heading: text(0x3a11)?,
        score_destinations: words(0x3a1b, 10)?.try_into().unwrap(),
        score_name_destinations: words(0x3a2f, 10)?.try_into().unwrap(),
    };
    validate_assets(&assets)?;
    Ok(assets)
}
pub fn validate_assets(assets: &ResultAssets) -> Result<()> {
    ensure!(
        assets.podium_tiles.len() == PODIUM_TILES_ROM.len()
            && assets.podium_palette.len() == 256
            && assets.podium_map.len() == PODIUM_MAP_ROM.len() / 2
            && assets.postrace_map.len() == POSTRACE_MAP_ROM.len() / 2
            && assets.loss_tiles_a.len() == LOSS_TILES_A_ROM.len()
            && assets.loss_tiles_b.len() == LOSS_TILES_B_ROM.len()
            && assets.loss_palette.len() == 96
            && assets.loss_replacements.iter().all(|p| p.len() == 48),
        "invalid results asset span"
    );
    let mut order = assets.defaults.scores.order;
    order.sort_unstable();
    ensure!(
        order == std::array::from_fn(|i| i as u8),
        "invalid default score order"
    );
    let (width, rows) = memory::validate_rectangle(&assets.loss_map.rectangle)?;
    ensure!(
        assets.loss_map.words.len() == width * rows,
        "invalid loss map"
    );
    ensure!(
        [
            &assets.game_over_text,
            &assets.continue_text,
            &assets.choices_text,
            &assets.score_heading
        ]
        .iter()
        .all(|t| t.last() == Some(&0xff)),
        "invalid results text"
    );
    ensure!(
        assets.podium_colors.iter().all(|&i| i <= 252),
        "invalid podium colors"
    );
    Ok(())
}
/// Cold signature/default-copy entry only. Warm frontend entry retains records.
pub fn copy_record_defaults(records: &mut SessionRecords, defaults: &SessionRecords) {
    *records = defaults.clone();
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RecordKey {
    Original(u8),
    Added(String),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrackRecord {
    #[serde(with = "crate::snapshot::array")]
    pub time: [u8; 2],
    #[serde(with = "crate::snapshot::array")]
    pub name: [u8; 3],
}
/// Added records extend the catalogue without a fixed total count. Original
/// records remain solely in Session.records; no duplicate original record owner.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AddedRecords(pub BTreeMap<String, TrackRecord>);
pub fn read_record(
    records: &SessionRecords,
    added: &AddedRecords,
    key: &RecordKey,
) -> Result<TrackRecord> {
    match key {
        RecordKey::Original(id) => {
            ensure!(*id < 16, "invalid original record");
            let i = usize::from(*id);
            Ok(TrackRecord {
                time: records.track_times[i * 2..i * 2 + 2].try_into().unwrap(),
                name: records.track_names[i * 3..i * 3 + 3].try_into().unwrap(),
            })
        }
        RecordKey::Added(id) => added.0.get(id).copied().context("missing named record"),
    }
}
pub fn write_record(
    records: &mut SessionRecords,
    added: &mut AddedRecords,
    key: &RecordKey,
    record: TrackRecord,
) -> Result<()> {
    match key {
        RecordKey::Original(id) => {
            ensure!(*id < 16, "invalid original record");
            let i = usize::from(*id);
            records.track_times[i * 2..i * 2 + 2].copy_from_slice(&record.time);
            records.track_names[i * 3..i * 3 + 3].copy_from_slice(&record.name);
        }
        RecordKey::Added(id) => {
            ensure!(!id.is_empty(), "empty record identity");
            added.0.insert(id.clone(), record);
        }
    }
    Ok(())
}
pub fn compare_record(pending: &mut u8, time: RaceTimer, stored: [u8; 2]) {
    if time.record_encoded() < stored {
        *pending = pending.wrapping_add(1);
    } else {
        *pending = 0;
    }
}
pub fn normalize_checkpoints(progress: &mut Progress, route_count: u8) {
    for driver in &mut progress.drivers {
        if driver.checkpoint > route_count {
            driver.checkpoint = driver.checkpoint.wrapping_sub(route_count);
        }
    }
}
/// Byte increment/cap at every eighth source order; retain continuation high byte.
pub fn award_continuation(session: &mut Session) {
    let low = session.continuation_progress as u8;
    if session.order_slot & 7 == 0 && low < 3 {
        session.continuation_progress =
            (session.continuation_progress & 0xff00) | u16::from(low.wrapping_add(1));
    }
}
pub fn install_postrace_map(assets: &ResultAssets, memory: &mut VideoMemory) -> Result<()> {
    validate_assets(assets)?;
    for (i, &word) in assets.postrace_map.iter().enumerate() {
        memory::write_word(memory, 0x55c8 + i as u16, word);
    }
    Ok(())
}
pub fn install_podium(
    assets: &ResultAssets,
    memory: &mut VideoMemory,
    display: &mut Display,
) -> Result<()> {
    validate_assets(assets)?;
    memory::fill_map(memory, 0x4000, 0);
    memory.vram[..assets.podium_tiles.len()].copy_from_slice(&assets.podium_tiles);
    memory.palette.copy_from_slice(&assets.podium_palette);
    for (i, &word) in assets.podium_map.iter().enumerate() {
        memory::write_word(memory, 0x4040 + i as u16, word);
    }
    let _ = display;
    Ok(())
}
pub fn color_podium(
    assets: &ResultAssets,
    rank: [u8; TRUCKS],
    memory: &mut VideoMemory,
) -> Result<()> {
    validate_assets(assets)?;
    for place in (0..PODIUM_RANKS).rev() {
        let at = usize::from(assets.podium_colors[place]);
        memory.palette[at..at + 4]
            .copy_from_slice(&assets.truck_palettes[usize::from(rank[place] & 3)]);
    }
    Ok(())
}
/// Rank award is credited now; displayed amount comes from pickup pending bytes,
/// credited separately at dismissal. Borrow the single WorldObjects array.
pub fn draw_awards(
    assets: &ResultAssets,
    session: &mut Session,
    pending_awards: &[u8; TRUCKS],
    rank: [u8; TRUCKS],
    memory: &mut VideoMemory,
) -> Result<()> {
    validate_assets(assets)?;
    ensure!(rank.iter().all(|&id| id < 4), "invalid podium rank");
    for place in (0..PODIUM_RANKS).rev() {
        let slot = usize::from(rank[place]);
        session.selected_wallet = rank[place];
        shop::add_award(
            session,
            assets.awards[place],
            shop::AwardDigit::TenThousands,
        )?;
        let count = pending_awards[slot];
        let leading = if count >= 10 { 2 } else { 0 };
        let remainder = if count >= 10 {
            count.wrapping_sub(10)
        } else {
            count
        };
        let attributes = 0x2000 + ((place as u16 + 1) * 0x400);
        for (i, digit) in [leading, remainder.wrapping_add(1), 1]
            .into_iter()
            .enumerate()
        {
            memory::write_word(
                memory,
                assets.award_destinations[place] + i as u16,
                attributes | u16::from(digit),
            );
        }
        for (i, &glyph) in session.records.truck_names[slot * 3..slot * 3 + 3]
            .iter()
            .enumerate()
        {
            memory::write_word(
                memory,
                assets.name_destinations[place] + i as u16,
                0x2800 | u16::from(glyph),
            );
        }
    }
    Ok(())
}
pub fn draw_times(
    session: &Session,
    added: &AddedRecords,
    key: &RecordKey,
    timer: RaceTimer,
    winner: u8,
    memory: &mut VideoMemory,
) -> Result<()> {
    ensure!(winner < 4, "invalid winner");
    let record = read_record(&session.records, added, key)?;
    draw_time_row(
        memory,
        0x4869,
        0x4872,
        session.records.truck_names[usize::from(winner) * 3..usize::from(winner) * 3 + 3]
            .try_into()
            .unwrap(),
        timer.record_encoded(),
    );
    draw_time_row(memory, 0x4889, 0x4892, record.name, record.time);
    Ok(())
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum AwardsFrame {
    Waiting,
    CommitWait,
    Dismissed,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Awards {
    pub timer: u16,
    pub new_record_pending: u8,
    pub phase: AwardsFrame,
}
/// One ordinary frame; timer>250 with a pending record bypasses input polling.
/// Commit at250 suspends for an additional NMI before redraw and input polling.
// Explicit disjoint owner borrows keep mutation authority visible at this boundary.
#[allow(clippy::too_many_arguments)]
pub fn advance_awards(
    state: &mut Awards,
    session: &mut Session,
    added: &mut AddedRecords,
    key: &RecordKey,
    time: RaceTimer,
    rank: [u8; TRUCKS],
    animation: u8,
    inputs: [ControllerWord; TRUCKS],
    memory: &mut VideoMemory,
) -> Result<()> {
    ensure!(rank.iter().all(|&id| id < 4), "invalid award rank");
    if state.phase != AwardsFrame::Waiting {
        return Ok(());
    }
    if state.new_record_pending != 0 && state.timer >= RECORD_COMMIT_TIME {
        if state.timer == RECORD_COMMIT_TIME {
            let winner = usize::from(rank[0]);
            let name = session.records.truck_names[winner * 3..winner * 3 + 3]
                .try_into()
                .unwrap();
            write_record(
                &mut session.records,
                added,
                key,
                TrackRecord {
                    time: time.record_encoded(),
                    name,
                },
            )?;
            state.phase = AwardsFrame::CommitWait;
            return Ok(());
        }
        if animation & 0x10 == 0 {
            for at in 0x4869..=0x4875 {
                memory::write_word(memory, at, 0x2000);
            }
        } else {
            draw_time_row(
                memory,
                0x4869,
                0x4872,
                session.records.truck_names[usize::from(rank[0]) * 3..usize::from(rank[0]) * 3 + 3]
                    .try_into()
                    .unwrap(),
                time.record_encoded(),
            );
        }
        tick_awards(state, [ControllerWord(0); TRUCKS]);
    } else {
        tick_awards(state, inputs);
    }
    Ok(())
}
// Explicit disjoint owner borrows keep mutation authority visible at this boundary.
#[allow(clippy::too_many_arguments)]
pub fn finish_record_wait(
    state: &mut Awards,
    session: &Session,
    added: &AddedRecords,
    key: &RecordKey,
    time: RaceTimer,
    rank: [u8; TRUCKS],
    inputs: [ControllerWord; TRUCKS],
    memory: &mut VideoMemory,
) -> Result<()> {
    if state.phase != AwardsFrame::CommitWait {
        return Ok(());
    }
    draw_times(session, added, key, time, rank[0], memory)?;
    state.phase = AwardsFrame::Waiting;
    tick_awards(state, inputs);
    Ok(())
}
pub fn settle_pickup_awards(
    session: &mut Session,
    pending_awards: &mut [u8; TRUCKS],
) -> Result<()> {
    for slot in (0..TRUCKS).rev() {
        session.selected_wallet = slot as u8;
        if pending_awards[slot] != 0 {
            shop::add_award(
                session,
                pending_awards[slot],
                shop::AwardDigit::TenThousands,
            )?;
            pending_awards[slot] = 0;
        }
    }
    Ok(())
}
pub fn insert_selected_score(session: &mut Session) -> Result<bool> {
    let selected = usize::from(session.selected_wallet);
    ensure!(
        selected < 4 && session.records.scores.valid_ids(),
        "invalid score indirection"
    );
    let score = super::leaderboard::Score::from_digits(session.wallets[selected]);
    let name = super::leaderboard::Initials(
        session.records.truck_names[selected * 3..selected * 3 + 3]
            .try_into()
            .unwrap(),
    );
    session.records.scores.insert(score, name)
}
pub fn insert_all_scores(session: &mut Session) -> Result<()> {
    ensure!(
        session.records.scores.valid_ids(),
        "invalid score indirection"
    );
    for slot in (0..TRUCKS).rev() {
        session.selected_wallet = slot as u8;
        shop::normalize_wallet(session)?;
        insert_selected_score(session)?;
    }
    Ok(())
}
pub fn draw_highscores(
    assets: &ResultAssets,
    records: &SessionRecords,
    memory: &mut VideoMemory,
) -> Result<()> {
    validate_assets(assets)?;
    ensure!(records.scores.valid_ids(), "invalid highscore ID");
    memory::draw_text(
        memory,
        &assets.score_heading,
        0x480b,
        memory::TextSize::Paired,
    )?;
    for row in (0..SCORE_ROWS).rev() {
        let id = usize::from(records.scores.order[row]);
        let base = assets.score_destinations[row];
        let mut significant = false;
        for (digit, &value) in records.scores.entries[id].score.digits().iter().enumerate() {
            significant |= value != 0;
            let tile = if significant {
                value.wrapping_add(1).wrapping_mul(2)
            } else {
                0
            };
            paired(memory, base + digit as u16, tile);
        }
        paired(memory, base + 6, 2);
        paired(memory, base + 7, 2);
        for (character, &glyph) in records.scores.entries[id].name.0.iter().enumerate() {
            paired(
                memory,
                assets.score_name_destinations[row] + character as u16,
                glyph.wrapping_mul(2),
            );
        }
    }
    Ok(())
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LossRoute {
    Shop,
    Penalty,
    RestartFrontend,
}
/// Call after loss prelude's inactive conversion/highscore insertion. Counts
/// low four participation bits; single-human byte DEC may force frontend unwind.
pub fn choose_loss_route(session: &mut Session, loser: u8) -> Result<LossRoute> {
    ensure!(loser < 4, "invalid losing truck");
    if session.active_humans & (1 << loser) == 0 {
        return Ok(LossRoute::Shop);
    }
    if (session.active_humans & 15).count_ones() == 1 {
        let low = (session.continuation_progress as u8).wrapping_sub(1);
        session.continuation_progress = (session.continuation_progress & 0xff00) | u16::from(low);
        if (low as i8) < 0 {
            toggle_participation(session, loser)?;
            return Ok(LossRoute::RestartFrontend);
        }
    }
    Ok(LossRoute::Penalty)
}
pub fn penalize_loser(
    session: &mut Session,
    trucks: &mut [Truck; TRUCKS],
    loser: u8,
) -> Result<()> {
    ensure!(loser < 4, "invalid losing truck");
    let slot = usize::from(loser);
    let upgrades = &mut trucks[slot].upgrades;
    for value in [
        &mut upgrades.speed,
        &mut upgrades.acceleration,
        &mut upgrades.steering,
        &mut upgrades.suspension,
    ] {
        let next = value.wrapping_sub(2);
        *value = if (next as i8) < 0 { 0 } else { next };
    }
    session.wallets[slot] = [0, 0, 1, 0, 0, 0];
    Ok(())
}
pub fn toggle_participation(session: &mut Session, truck: u8) -> Result<()> {
    ensure!(truck < 4, "invalid participant");
    session.active_humans ^= 1 << truck;
    session.joined_pages = [session.active_humans & 3, session.active_humans >> 2];
    session.continuation_progress &= 0xff00;
    Ok(())
}
pub fn resolve_choice(session: &mut Session, loser: u8, choice: u8) -> Result<LossRoute> {
    ensure!(loser < 4, "invalid losing truck");
    if choice != 0 {
        toggle_participation(session, loser)?;
        if session.active_humans == 0 {
            return Ok(LossRoute::RestartFrontend);
        }
    }
    Ok(LossRoute::Shop)
}
pub fn install_loss(
    assets: &ResultAssets,
    loser: u8,
    memory: &mut VideoMemory,
    display: &mut Display,
) -> Result<()> {
    ensure!(loser < 4, "invalid losing truck");
    validate_assets(assets)?;
    memory::fill_map(memory, 0x4000, 0);
    memory.vram[..assets.loss_tiles_a.len()].copy_from_slice(&assets.loss_tiles_a);
    let at = assets.loss_tiles_a.len();
    memory.vram[at..at + assets.loss_tiles_b.len()].copy_from_slice(&assets.loss_tiles_b);
    memory.palette[0x20..0x80].copy_from_slice(&assets.loss_palette);
    if loser != 0 {
        memory.palette[0x29..0x59]
            .copy_from_slice(&assets.loss_replacements[usize::from(loser - 1)]);
    }
    memory::draw_rectangle(
        memory,
        &assets.loss_map.words,
        assets.loss_map.destination_word,
        &assets.loss_map.rectangle,
    )?;
    let _ = display;
    Ok(())
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContinuePrompt {
    pub choice: u8,
    pub loser: u8,
    pub dismissed: bool,
}
pub fn initialize_continue(
    state: &mut ContinuePrompt,
    session: &mut Session,
    assets: &ResultAssets,
    cursor_tiles: &[u16; 9],
    memory: &mut VideoMemory,
    display: &mut Display,
) -> Result<()> {
    stage_continue(state, assets, cursor_tiles, memory)?;
    release_continue(state, session, memory, display);
    Ok(())
}
pub fn poll_continue(
    state: &mut ContinuePrompt,
    inputs: [ControllerWord; TRUCKS],
    memory: &mut VideoMemory,
) -> Result<()> {
    ensure!(state.loser < 4, "invalid losing truck");
    if state.dismissed {
        return Ok(());
    }
    let high = (inputs[usize::from(state.loser)].0 >> 8) as u8;
    if high & 0x80 != 0 {
        state.dismissed = true;
        for record in memory.staged_oam[..512].chunks_exact_mut(4) {
            record[0] = 0xf0;
            record[1] = 0xf0;
        }
        return Ok(());
    }
    if high & 2 != 0 {
        if state.choice != 0 {
            state.choice = 0;
            position_continue(memory, 0x24, memory.staged_oam[1]);
        }
    } else if high & 1 != 0 && state.choice == 0 {
        state.choice = 1;
        position_continue(memory, 0xa8, memory.staged_oam[1]);
    }
    Ok(())
}
/// Attract/nonhuman prompt uses all four HIGH bytes only, DEC before input.
pub fn poll_game_over(remaining: &mut u16, inputs: [ControllerWord; TRUCKS]) -> bool {
    *remaining = remaining.wrapping_sub(1);
    (*remaining as i16) < 0 || inputs.iter().any(|word| word.0 & 0xff00 != 0)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PostracePhase {
    HudWait,
    Window,
    FadeOut,
    PodiumSetup,
    FadeIn,
    Awards,
    RecordWait,
    Contract,
    LossPrelude,
    LossUploadWait,
    LossPrompt,
    Shop,
    ConvertInactive,
    Restart,
    Complete,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Postrace {
    pub loss_route: Option<LossRoute>,
    pub route: Option<crate::world::tracks::Route>,
    pub route_count: u8,
    pub phase: PostracePhase,
    #[serde(with = "crate::snapshot::array")]
    pub rank: [u8; TRUCKS],
    pub awards: Awards,
}
/// Actual shared state/presentation composition, never just an effect-list oracle.
/// Caller supplies transition completion; an NMI step cannot busy-wait.
pub fn begin_postrace(
    trucks: &[Truck; TRUCKS],
    state: &mut Postrace,
    session: &mut Session,
    progress: &mut Progress,
    timer: &mut RaceTimer,
    memory: &mut VideoMemory,
    display: &mut Display,
) {
    presentation::update_hud_digits(
        memory,
        progress.drivers.map(|driver| driver.lap),
        trucks.each_ref().map(|truck| truck.nitro),
        timer,
    );
    state.phase = PostracePhase::HudWait;
    let _ = (session, display);
}
// Explicit disjoint owner borrows keep mutation authority visible at this boundary.
#[allow(clippy::too_many_arguments)]
pub fn advance_postrace(
    timer: RaceTimer,
    key: &RecordKey,
    added: &mut AddedRecords,
    state: &mut Postrace,
    session: &mut Session,
    progress: &mut Progress,
    trucks: &mut [Truck; TRUCKS],
    pending_awards: &mut [u8; TRUCKS],
    assets: &ResultAssets,
    memory: &mut VideoMemory,
    display: &mut Display,
    transition_complete: bool,
) -> Result<()> {
    validate_assets(assets)?;
    match state.phase {
        PostracePhase::HudWait => {
            award_continuation(session);
            session.inhibit_video = session.inhibit_video.wrapping_add(1);
            install_postrace_map(assets, memory)?;
            session.inhibit_video = 0;
            state.phase = PostracePhase::Window;
        }
        PostracePhase::Window if transition_complete => {
            normalize_checkpoints(progress, state.route_count);
            let route = state
                .route
                .as_ref()
                .context("missing finished race route")?;
            state.rank =
                crate::trucks::ai::ranking(trucks, &progress.drivers, route)?.map(|id| id as u8);
            state.phase = PostracePhase::FadeOut;
        }
        PostracePhase::FadeOut if transition_complete => state.phase = PostracePhase::PodiumSetup,
        PostracePhase::PodiumSetup => {
            session.inhibit_video = session.inhibit_video.wrapping_add(1);
            display.use_subscreen = false;
            display.color_math_layers = 0;
            display.half_color = false;
            display.subtract_color = false;
            display.layers[2].scroll[1] = 0;
            display.force_blank = true;
            session.enabled_layers = 5;
            install_podium(assets, memory, display)?;
            color_podium(assets, state.rank, memory)?;
            draw_awards(assets, session, pending_awards, state.rank, memory)?;
            draw_times(session, added, key, timer, state.rank[0], memory)?;
            let record = read_record(&session.records, added, key)?;
            compare_record(&mut state.awards.new_record_pending, timer, record.time);
            session.audio_commands.push(PODIUM_AUDIO);
            progress.animation_frame = 0;
            state.phase = PostracePhase::FadeIn;
        }
        PostracePhase::FadeIn if transition_complete => {
            state.awards.timer = AWARDS_TIMEOUT;
            state.awards.phase = AwardsFrame::Waiting;
            state.phase = PostracePhase::Awards;
        }
        PostracePhase::Awards if state.awards.phase == AwardsFrame::CommitWait => {
            state.phase = PostracePhase::RecordWait
        }
        PostracePhase::Awards | PostracePhase::RecordWait
            if state.awards.phase == AwardsFrame::Dismissed =>
        {
            state.phase = PostracePhase::Contract
        }
        PostracePhase::RecordWait if state.awards.phase == AwardsFrame::Waiting => {
            state.phase = PostracePhase::Awards
        }
        PostracePhase::Contract if transition_complete => {
            settle_pickup_awards(session, pending_awards)?;
            let loser = state.rank[3];
            state.phase = if session.active_humans & (1 << loser) != 0 {
                PostracePhase::LossPrelude
            } else {
                PostracePhase::Shop
            };
        }
        PostracePhase::LossPrelude => {
            shop::convert_inactive(session, trucks)?;
            insert_all_scores(session)?;
            let loser = state.rank[3];
            let route = choose_loss_route(session, loser)?;
            if route == LossRoute::Penalty {
                penalize_loser(session, trucks, loser)?;
            }
            session.inhibit_video = session.inhibit_video.wrapping_add(1);
            state.loss_route = Some(route);
            state.phase = PostracePhase::LossUploadWait;
        }
        PostracePhase::LossUploadWait => {
            install_loss(assets, state.rank[3], memory, display)?;
            session.audio_commands.push(LOSS_AUDIO);
            state.phase = if state.loss_route == Some(LossRoute::RestartFrontend) {
                PostracePhase::Restart
            } else {
                PostracePhase::LossPrompt
            };
        }
        PostracePhase::Shop if transition_complete => state.phase = PostracePhase::ConvertInactive,
        PostracePhase::ConvertInactive => {
            shop::convert_inactive(session, trucks)?;
            state.phase = PostracePhase::Complete;
        }
        _ => {}
    }
    Ok(())
}
#[cfg(test)]
mod tests;

fn draw_time_row(
    memory: &mut VideoMemory,
    name_at: u16,
    time_at: u16,
    name: [u8; 3],
    time: [u8; 2],
) {
    for (i, glyph) in name.into_iter().enumerate() {
        memory::write_word(memory, name_at + i as u16, 0x2800 | u16::from(glyph));
    }
    for (i, word) in [
        0x2801 + u16::from(time[0] & 15),
        0x2801 + u16::from(time[1] >> 4),
        0x2827,
        0x2801 + u16::from(time[1] & 15),
    ]
    .into_iter()
    .enumerate()
    {
        memory::write_word(memory, time_at + i as u16, word);
    }
}
fn paired(memory: &mut VideoMemory, at: u16, tile: u8) {
    memory::write_word(memory, at, 0x2000 | u16::from(tile));
    memory::write_word(
        memory,
        at.wrapping_add(32),
        0x2000 | u16::from(tile.wrapping_add(1)),
    );
}
fn tick_awards(state: &mut Awards, inputs: [ControllerWord; TRUCKS]) {
    if inputs[..2].iter().any(|word| word.0 != 0) {
        state.phase = AwardsFrame::Dismissed;
    } else {
        state.timer = state.timer.wrapping_sub(1);
        if (state.timer as i16) < 0 {
            state.phase = AwardsFrame::Dismissed;
        }
    }
}
fn position_continue(memory: &mut VideoMemory, x: u8, y: u8) {
    for part in 0..9 {
        memory.staged_oam[part * 4] = x.wrapping_add((part % 3) as u8 * 16);
        memory.staged_oam[part * 4 + 1] = y.wrapping_add((part / 3) as u8 * 16);
    }
}

/// Before the source prompt's NMI wait: text and tile words are staged while
/// race video remains inhibited. Release/cursor placement happens afterward.
pub fn stage_continue(
    state: &ContinuePrompt,
    assets: &ResultAssets,
    cursor_tiles: &[u16; 9],
    memory: &mut VideoMemory,
) -> Result<()> {
    ensure!(state.loser < 4, "invalid losing truck");
    validate_assets(assets)?;
    memory::draw_text(
        memory,
        &assets.continue_text,
        0x4acb,
        memory::TextSize::Paired,
    )?;
    memory::draw_text(
        memory,
        &assets.choices_text,
        0x4b06,
        memory::TextSize::Paired,
    )?;
    let mut staging = sprites::SpriteArrays::default();
    sprites::initialize_arrays(&mut staging);
    sprites::pack_sorted(&staging, &mut memory.staged_oam)?;
    for player in 0..2 {
        for (part, &tile) in cursor_tiles.iter().enumerate() {
            let at = player * 0x24 + part * 4 + 2;
            memory.staged_oam[at..at + 2].copy_from_slice(&tile.to_le_bytes());
        }
    }
    Ok(())
}
pub fn release_continue(
    state: &mut ContinuePrompt,
    session: &mut Session,
    memory: &mut VideoMemory,
    display: &mut Display,
) {
    state.choice = 0;
    state.dismissed = false;
    session.inhibit_video = 0;
    session.enabled_layers = 0x15;
    display.main_layers = 0x15;
    display.main_window_layers = 0x15;
    position_continue(memory, 0x24, 0xae);
}
