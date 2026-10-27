//! Six-digit wallets and the two-player speed shop;
//! independently derived source contracts and branch reasoning: shop.trace.md.
use super::{
    attract::Session,
    names::{NameAssets, Selection},
};
use crate::presentation::{
    memory, sprites,
    transitions::{self, FadeDirection},
};
use crate::{
    controls::{ControllerWord, DirectionTables, MenuDelta},
    presentation::{
        Display, VideoMemory,
        memory::SharedGraphics,
        transitions::{Fade, Window},
    },
    trucks::motion::Truck,
};
use anyhow::{Context, Result, ensure};
pub const ITEMS: usize = 5;
pub const CURSOR_SPRITES: usize = 9;
pub const PURCHASE_DIGIT: usize = 3;
pub const MONEY_DIGITS: usize = 6;
pub const SHOP_AUDIO: u16 = 0x0301;
pub const COMPLETE_AUDIO: u16 = 0xe803;
pub const ACK_AUDIO: u16 = 0x0101;
pub const INACTIVE_NITRO_LIMIT: u8 = 0x80;
pub const COSTS_ROM: usize = 0x2f3c;
pub const CAPS_ROM: usize = 0x2f41;
pub const BAR_DESTINATIONS_ROM: usize = 0x310d;
pub const CURSOR_TILEWORDS_ROM: usize = 0x2da2;
pub const CURSOR_DELTAS_ROM: usize = 0x2f2a;
pub const HEADING_ROM: [usize; 2] = [0x2db4, 0x2dcd];
pub const COMPLETE_TEXT_ROM: usize = 0x2de6;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShopAssets {
    pub costs: [u8; ITEMS],
    pub caps: [u8; ITEMS],
    pub cursor_tiles: [u16; CURSOR_SPRITES],
    pub cursor_x: [u8; CURSOR_SPRITES],
    pub cursor_y: [u8; CURSOR_SPRITES],
    pub grid_x: [u8; 12],
    pub grid_y: [u8; 12],
    pub bar_destinations: [[u16; ITEMS]; 2],
    pub headings: [Vec<u8>; 2],
    pub complete_text: Vec<u8>,
}
pub fn load_assets(rom: &[u8]) -> Result<ShopAssets> {
    let bytes = |start: usize, length: usize| -> Result<Vec<u8>> {
        Ok(rom
            .get(start..start + length)
            .context("truncated shop asset")?
            .to_vec())
    };
    let words = |start, length| -> Result<Vec<u16>> {
        Ok(bytes(start, length * 2)?
            .chunks_exact(2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .collect())
    };
    let text = |start: usize| -> Result<Vec<u8>> {
        let tail = rom.get(start..).context("missing shop text")?;
        let end = tail
            .iter()
            .position(|&b| b == 0xff)
            .context("unterminated shop text")?;
        Ok(tail[..=end].to_vec())
    };
    let destinations = words(BAR_DESTINATIONS_ROM, 10)?;
    let assets = ShopAssets {
        costs: bytes(COSTS_ROM, ITEMS)?.try_into().unwrap(),
        caps: bytes(CAPS_ROM, ITEMS)?.try_into().unwrap(),
        cursor_tiles: words(CURSOR_TILEWORDS_ROM, CURSOR_SPRITES)?
            .try_into()
            .unwrap(),
        cursor_x: bytes(CURSOR_DELTAS_ROM, CURSOR_SPRITES)?
            .try_into()
            .unwrap(),
        cursor_y: bytes(CURSOR_DELTAS_ROM + CURSOR_SPRITES, CURSOR_SPRITES)?
            .try_into()
            .unwrap(),
        grid_x: bytes(super::names::CELL_X_ROM, 12)?.try_into().unwrap(),
        grid_y: bytes(super::names::CELL_Y_ROM, 12)?.try_into().unwrap(),
        bar_destinations: std::array::from_fn(|player| {
            std::array::from_fn(|i| destinations[player * 5 + i])
        }),
        headings: [text(HEADING_ROM[0])?, text(HEADING_ROM[1])?],
        complete_text: text(COMPLETE_TEXT_ROM)?,
    };
    validate_assets(&assets)?;
    Ok(assets)
}
pub fn validate_assets(assets: &ShopAssets) -> Result<()> {
    ensure!(
        assets.headings.iter().all(|t| t.last() == Some(&0xff))
            && assets.complete_text.last() == Some(&0xff),
        "unterminated shop text"
    );
    ensure!(
        assets
            .bar_destinations
            .iter()
            .flatten()
            .all(|&d| d < 0x8000),
        "invalid shop bar destination"
    );
    Ok(())
}
/// Wallets/selection stay in the shared frontend Session. Record defaults belong
/// to results and are not implicitly reset by this wallet initializer.
pub fn initialize_wallets(session: &mut Session) {
    session.wallets = [[0, 0, 1, 0, 0, 0]; 4];
    session.selected_wallet = 0;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AwardDigit {
    TenThousands,
    Thousands,
    Hundreds,
}
/// Source decimal-clear byte add, followed separately by normalization. TenThousands
/// is stored digit3; the displayed two appended zeros make it a $10000 unit.
pub fn add_award(session: &mut Session, amount: u8, digit: AwardDigit) -> Result<()> {
    let selected = usize::from(session.selected_wallet);
    ensure!(selected < 4, "invalid selected wallet");
    let at = match digit {
        AwardDigit::TenThousands => 3,
        AwardDigit::Thousands => 4,
        AwardDigit::Hundreds => 5,
    };
    session.wallets[selected][at] = session.wallets[selected][at].wrapping_add(amount);
    Ok(())
}
/// Carries do not extend the highest digit. Conditional highest-digit borrows
/// touch the previous wallet tail, or selected-wallet byte for truck0. This is
/// modeled using adjacent domain owners, never an emulated memory map.
pub fn normalize_wallet(session: &mut Session) -> Result<()> {
    let selected = usize::from(session.selected_wallet);
    anyhow::ensure!(selected < 4, "invalid selected wallet");
    let mut preceding = if selected == 0 {
        session.selected_wallet
    } else {
        session.wallets[selected - 1][5]
    };
    normalize_digits(&mut session.wallets[selected], &mut preceding);
    if selected == 0 {
        session.selected_wallet = preceding;
    } else {
        session.wallets[selected - 1][5] = preceding;
    }
    Ok(())
}
/// Descending truck order; the source decrements before testing capped nitro.
/// Only digits0..3 participate in its balance test. No display refresh occurs.
pub fn convert_inactive(session: &mut Session, trucks: &mut [Truck; 4]) -> Result<()> {
    for slot in (0..4).rev() {
        if session.active_humans & (1 << slot) != 0 {
            continue;
        }
        while session.wallets[slot][..4].iter().any(|&digit| digit != 0) {
            session.wallets[slot][3] = session.wallets[slot][3].wrapping_sub(1);
            if trucks[slot].nitro_stock_bcd >= INACTIVE_NITRO_LIMIT {
                break;
            }
            trucks[slot].nitro_stock_bcd = increment_decimal(trucks[slot].nitro_stock_bcd);
            session.selected_wallet = slot as u8;
            normalize_wallet(session)?;
        }
    }
    Ok(())
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item {
    Nitro,
    Acceleration,
    Steering,
    Speed,
    Suspension,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Purchase {
    Bought,
    InsufficientFunds,
    AtCap,
}
pub fn purchase(
    session: &mut Session,
    trucks: &mut [Truck; 4],
    page: u8,
    player: usize,
    item: Item,
    assets: &ShopAssets,
) -> Result<Purchase> {
    ensure!(player < 2 && page < 2, "invalid shop participant");
    validate_assets(assets)?;
    let slot = usize::from(page) * 2 + player;
    let item = index(item);
    let wallet = &mut session.wallets[slot];
    if wallet[..3].iter().all(|&d| d == 0) && wallet[PURCHASE_DIGIT] < assets.costs[item] {
        return Ok(Purchase::InsufficientFunds);
    }
    let stock = stock_mut(&mut trucks[slot], item);
    if *stock >= assets.caps[item] {
        return Ok(Purchase::AtCap);
    }
    *stock = increment_decimal(*stock);
    wallet[PURCHASE_DIGIT] = wallet[PURCHASE_DIGIT].wrapping_sub(assets.costs[item]);
    Ok(Purchase::Bought)
}
/// Despite wrap labels, source branches clamp each end of the 2x3 grid.
pub fn move_cursor(selection: &mut Selection, player: usize, delta: MenuDelta) -> Result<()> {
    ensure!(player < 2, "invalid shop cursor player");
    let editor = &mut selection.editors[player];
    let clamp = |value: u8, step: i8, maximum: u8| {
        let next = value.wrapping_add(step as u8);
        if (next as i8) < 0 {
            0
        } else {
            next.min(maximum)
        }
    };
    editor.column = clamp(editor.column, delta.columns, 1);
    editor.row = clamp(editor.row, delta.rows, 2);
    Ok(())
}
pub fn position_cursor(
    selection: &Selection,
    player: usize,
    assets: &ShopAssets,
    memory: &mut VideoMemory,
) -> Result<()> {
    ensure!(player < 2, "invalid shop cursor player");
    validate_assets(assets)?;
    let editor = selection.editors[player];
    ensure!(editor.column < 2 && editor.row < 3, "invalid shop grid");
    let cell = usize::from(editor.row) * 5 + usize::from(editor.column);
    let x = assets.grid_x[cell]
        .wrapping_mul(3)
        .wrapping_add(if player == 0 { 0x10 } else { 0x90 });
    let y = assets.grid_y[cell].wrapping_mul(2).wrapping_add(0x60);
    for part in (0..CURSOR_SPRITES).rev() {
        let at = player * 0x24 + part * 4;
        memory.staged_oam[at] = x.wrapping_add(assets.cursor_x[part]);
        memory.staged_oam[at + 1] = y.wrapping_add(assets.cursor_y[part]);
    }
    Ok(())
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Phase {
    Setup,
    FadeIn,
    WaitFrame,
    FirstDoneWait,
    FinalWait,
    Expand,
    Ack,
    Complete,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Shop {
    /// Moves between names/shop: retained repeat/button history is one value.
    pub selection: Selection,
    pub redraw_field: u16,
    pub phase: Phase,
    pub fade: Option<Fade>,
    pub window: Option<Window>,
    pub window_motion: Option<transitions::FrontendWindow>,
}
pub fn initialize_page(
    shop: &mut Shop,
    session: &mut Session,
    assets: &ShopAssets,
    memory: &mut VideoMemory,
) -> Result<()> {
    ensure!(shop.selection.page < 2, "invalid shop page");
    validate_assets(assets)?;
    shop.selection.done_mask = session.joined_pages[usize::from(shop.selection.page)] ^ 3;
    for player in 0..2 {
        shop.selection.editors[player].column = 0;
        shop.selection.editors[player].row = 0;
        shop.selection.sampled[player].0 &= 0xff;
        if shop.selection.done_mask & (1 << player) == 0 {
            position_cursor(&shop.selection, player, assets, memory)?;
        }
    }
    Ok(())
}
/// Confirmation comes from high-byte B; Start has no shop completion shortcut.
/// First player's done path suspends for an extra NMI; second has no such wait.
// Explicit disjoint owner borrows keep mutation authority visible at this boundary.
#[allow(clippy::too_many_arguments)]
pub fn handle_player(
    shop: &mut Shop,
    session: &mut Session,
    trucks: &mut [Truck; 4],
    player: usize,
    input: ControllerWord,
    directions: &DirectionTables,
    assets: &ShopAssets,
    memory: &mut VideoMemory,
) -> Result<()> {
    ensure!(
        player < 2 && shop.selection.page < 2,
        "invalid shop player/page"
    );
    validate_assets(assets)?;
    if shop.selection.done_mask & (1 << player) != 0 {
        return Ok(());
    }
    let high = (input.0 >> 8) as u8;
    let editor = &mut shop.selection.editors[player];
    let move_now = if high == 0 {
        editor.repeat = 0;
        false
    } else if editor.repeat != 0 {
        editor.repeat -= 1;
        false
    } else {
        editor.repeat = super::names::REPEAT_DELAY;
        high & 15 != 0
    };
    if move_now {
        move_cursor(&mut shop.selection, player, directions.frontend_delta(high))?;
        position_cursor(&shop.selection, player, assets, memory)?;
    }
    let history = &mut shop.selection.editors[player].button_history;
    *history = (*history >> 1)
        | if input.0 & crate::controls::BUTTON_B != 0 {
            0x80
        } else {
            0
        };
    if *history & 0xc0 != 0x80 {
        return Ok(());
    }
    let editor = shop.selection.editors[player];
    let item = usize::from(editor.row) * 2 + usize::from(editor.column);
    if item == ITEMS {
        if player == 0 {
            shop.phase = Phase::FirstDoneWait;
        } else {
            finish_player(shop, player, assets, memory)?;
        }
    } else {
        let item = [
            Item::Nitro,
            Item::Acceleration,
            Item::Steering,
            Item::Speed,
            Item::Suspension,
        ][item];
        purchase(session, trucks, shop.selection.page, player, item, assets)?;
    }
    Ok(())
}
pub fn draw_money(
    digits: &[u8; MONEY_DIGITS],
    destination_word: u16,
    memory: &mut VideoMemory,
) -> Result<()> {
    for column in 0..9 {
        memory::write_word(memory, destination_word.wrapping_add(column), 0x2000);
    }
    let mut at = destination_word;
    let mut significant = false;
    for &digit in digits {
        if digit != 0 {
            significant = true;
        }
        if significant {
            memory::write_word(memory, at, 0x2000 | u16::from(digit.wrapping_add(1)));
            at = at.wrapping_add(1);
        }
    }
    memory::write_word(memory, at, 0x2001);
    memory::write_word(memory, at.wrapping_add(1), 0x2001);
    Ok(())
}
pub fn refresh_money(shop: &Shop, session: &mut Session, memory: &mut VideoMemory) -> Result<()> {
    ensure!(shop.selection.page < 2, "invalid shop money page");
    let base = usize::from(shop.selection.page) * 2;
    for player in 0..2 {
        let slot = base + player;
        session.selected_wallet = slot as u8;
        normalize_wallet(session)?;
        if session.joined_pages[usize::from(shop.selection.page)] & (1 << player) != 0 {
            draw_money(
                &session.wallets[slot],
                if player == 0 { 0x4964 } else { 0x4974 },
                memory,
            )?;
        }
    }
    Ok(())
}
pub fn cycle_upgrade_field(
    shop: &mut Shop,
    trucks: &[Truck; 4],
    assets: &ShopAssets,
    memory: &mut VideoMemory,
) -> Result<()> {
    ensure!(shop.selection.page < 2, "invalid shop bar page");
    validate_assets(assets)?;
    shop.redraw_field = shop.redraw_field.wrapping_add(1);
    if shop.redraw_field >= ITEMS as u16 {
        shop.redraw_field = 0;
    }
    let field = usize::from(shop.redraw_field);
    for player in 0..2 {
        if shop.selection.done_mask & (1 << player) != 0 {
            continue;
        }
        let mut count = stock(
            &trucks[usize::from(shop.selection.page) * 2 + player],
            field,
        )
        .min(6);
        let mut at = assets.bar_destinations[player][field];
        while count != 0 {
            memory::write_word(memory, at, if count >= 2 { 0x283a } else { 0x283b });
            count = count.saturating_sub(2);
            at = at.wrapping_sub(32);
        }
    }
    Ok(())
}
// Explicit disjoint owner borrows keep mutation authority visible at this boundary.
#[allow(clippy::too_many_arguments)]
pub fn install_page(
    shop: &mut Shop,
    session: &mut Session,
    trucks: &[Truck; 4],
    assets: &ShopAssets,
    names: &NameAssets,
    shared: &SharedGraphics,
    memory: &mut VideoMemory,
    display: &mut Display,
) -> Result<()> {
    validate_assets(assets)?;
    super::names::validate_assets(names)?;
    ensure!(shop.selection.page < 2, "invalid shop page");
    session.inhibit_video = session.inhibit_video.wrapping_add(1);
    display.masked_layers = 0;
    display.window_inverted_layers = 0;
    super::names::install_driver_panels(
        names,
        shop.selection.page,
        session.joined_pages[usize::from(shop.selection.page)],
        false,
        memory,
        display,
    )?;
    memory::upload_fonts(memory, shared, false)?;
    memory::upload_text_palette(memory, shared)?;
    memory::upload_cursor(memory, shared)?;
    let mut staging = sprites::SpriteArrays::default();
    sprites::initialize_arrays(&mut staging);
    sprites::hide_frontend(&mut staging, memory)?;
    display.layers[0].tile_byte_base = 0;
    display.layers[1].tile_byte_base = 0x8000;
    initialize_page(shop, session, assets, memory)?;
    session.enabled_layers = 0x15;
    memory::fill_map(memory, 0x4800, 0);
    memory::fill_map(memory, 0x4400, 0);
    memory::draw_text(
        memory,
        &assets.headings[usize::from(shop.selection.page != 0)],
        0x4904,
        memory::TextSize::Single,
    )?;
    for player in 0..2 {
        for part in 0..CURSOR_SPRITES {
            let at = player * 0x24 + part * 4 + 2;
            memory.staged_oam[at..at + 2].copy_from_slice(&assets.cursor_tiles[part].to_le_bytes());
        }
    }
    refresh_money(shop, session, memory)?;
    for _ in 0..6 {
        cycle_upgrade_field(shop, trucks, assets, memory)?;
    }
    session.inhibit_video = 0;
    session.audio_commands.push(SHOP_AUDIO);
    shop.fade = Some(Fade {
        direction: FadeDirection::In,
        primed: false,
        complete: false,
        animation_counter: 0,
        enabled_layers: 0x15,
    });
    shop.phase = Phase::FadeIn;
    Ok(())
}
pub fn finish_page(shop: &mut Shop, session: &mut Session) {
    session.audio_commands.push(ACK_AUDIO);
    shop.selection.page = shop.selection.page.wrapping_add(1);
    shop.phase = if shop.selection.page <= session.page_limit {
        Phase::Setup
    } else {
        Phase::Complete
    };
    shop.fade = None;
    shop.window = None;
    shop.window_motion = None;
}
/// Full resumable NMI service; refresh and one bar precede sampling/handlers.
/// Real Display/window/VRAM/session mutations are required, not a phase log.
// Explicit disjoint owner borrows keep mutation authority visible at this boundary.
#[allow(clippy::too_many_arguments)]
pub fn advance(
    shop: &mut Shop,
    session: &mut Session,
    trucks: &mut [Truck; 4],
    assets: &ShopAssets,
    names: &NameAssets,
    shared: &SharedGraphics,
    directions: &DirectionTables,
    memory: &mut VideoMemory,
    display: &mut Display,
    automatic: [ControllerWord; 4],
) -> Result<()> {
    validate_assets(assets)?;
    match shop.phase {
        Phase::Setup => {
            install_page(
                shop, session, trucks, assets, names, shared, memory, display,
            )?;
            return Ok(());
        }
        Phase::FadeIn => {
            let fade = shop.fade.as_mut().context("missing shop fade")?;
            transitions::advance_fade(fade, display);
            if fade.complete {
                shop.fade = None;
                shop.phase = Phase::WaitFrame;
            }
            return Ok(());
        }
        Phase::FinalWait => {
            session.inhibit_video = session.inhibit_video.wrapping_add(1);
            session.audio_commands.push(COMPLETE_AUDIO);
            let mut window = super::names::new_window();
            let motion = transitions::begin_frontend(
                transitions::FrontendKind::Expanding,
                &mut window,
                display,
            );
            shop.window = Some(window);
            shop.window_motion = Some(motion);
            shop.phase = Phase::Expand;
            return Ok(());
        }
        Phase::Expand => {
            let window = shop.window.as_mut().context("missing shop window")?;
            let motion = shop.window_motion.as_mut().context("missing shop motion")?;
            transitions::advance_frontend(
                motion,
                window,
                &names.frontend_shape,
                display,
                automatic,
            )?;
            if motion.complete {
                shop.phase = Phase::Ack;
            }
            return Ok(());
        }
        Phase::Ack => {
            finish_page(shop, session);
            return Ok(());
        }
        Phase::Complete => return Ok(()),
        Phase::FirstDoneWait => {
            finish_player(shop, 0, assets, memory)?;
            handle_player(
                shop,
                session,
                trucks,
                1,
                shop.selection.sampled[1],
                directions,
                assets,
                memory,
            )?;
        }
        Phase::WaitFrame => {
            refresh_money(shop, session, memory)?;
            cycle_upgrade_field(shop, trucks, assets, memory)?;
            shop.selection.sampled =
                crate::controls::sample_frontend(shop.selection.page, automatic).players;
            handle_player(
                shop,
                session,
                trucks,
                0,
                shop.selection.sampled[0],
                directions,
                assets,
                memory,
            )?;
            if shop.phase == Phase::FirstDoneWait {
                return Ok(());
            }
            handle_player(
                shop,
                session,
                trucks,
                1,
                shop.selection.sampled[1],
                directions,
                assets,
                memory,
            )?;
        }
    }
    if shop.selection.done_mask == 3 {
        for record in memory.staged_oam[..512].chunks_exact_mut(4) {
            record[0] = 0xf0;
            record[1] = 0xf0;
        }
        shop.phase = Phase::FinalWait;
    } else {
        shop.phase = Phase::WaitFrame;
    }
    Ok(())
}
#[cfg(test)]
mod tests;

/// Source CMP10/BPL selects carry through89, borrow from8A; preceding is
/// the selected-wallet byte for record0 or previous record's final digit.
pub fn normalize_digits(wallet: &mut [u8; 6], preceding: &mut u8) {
    for digit in (0..6).rev() {
        while wallet[digit] >= 10 {
            if (wallet[digit].wrapping_sub(10) as i8) >= 0 {
                wallet[digit] = wallet[digit].wrapping_sub(10);
                if digit != 0 {
                    wallet[digit - 1] = wallet[digit - 1].wrapping_add(1);
                }
            } else {
                wallet[digit] = wallet[digit].wrapping_add(10);
                if digit != 0 {
                    wallet[digit - 1] = wallet[digit - 1].wrapping_sub(1);
                } else {
                    *preceding = preceding.wrapping_sub(1);
                }
            }
        }
    }
}

fn index(item: Item) -> usize {
    match item {
        Item::Nitro => 0,
        Item::Acceleration => 1,
        Item::Steering => 2,
        Item::Speed => 3,
        Item::Suspension => 4,
    }
}
fn stock(truck: &Truck, item: usize) -> u8 {
    match item {
        0 => truck.nitro_stock_bcd,
        1 => truck.upgrades.acceleration,
        2 => truck.upgrades.steering,
        3 => truck.upgrades.speed,
        _ => truck.upgrades.suspension,
    }
}
fn stock_mut(truck: &mut Truck, item: usize) -> &mut u8 {
    match item {
        0 => &mut truck.nitro_stock_bcd,
        1 => &mut truck.upgrades.acceleration,
        2 => &mut truck.upgrades.steering,
        3 => &mut truck.upgrades.speed,
        _ => &mut truck.upgrades.suspension,
    }
}
fn increment_decimal(value: u8) -> u8 {
    let low = (value & 15) + 1;
    let adjusted = if low > 9 { low + 6 } else { low };
    let high = (value & 0xf0).wrapping_add(adjusted);
    if high > 0x99 {
        high.wrapping_add(0x60)
    } else {
        high
    }
}
fn finish_player(
    shop: &mut Shop,
    player: usize,
    assets: &ShopAssets,
    memory: &mut VideoMemory,
) -> Result<()> {
    memory::draw_text(
        memory,
        &assets.complete_text,
        if player == 0 { 0x4903 } else { 0x4913 },
        memory::TextSize::Single,
    )?;
    shop.selection.done_mask |= 1 << player;
    for part in 0..CURSOR_SPRITES {
        memory.staged_oam[player * 0x24 + part * 4 + 1] = 0xf0;
    }
    Ok(())
}
