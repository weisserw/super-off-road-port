//! Independent fixed vectors derived in ../shop.trace.md.
use super::*;
use crate::{
    controls::*,
    frontend::{
        attract::{EntryPhase, SessionRecords},
        names::{self, Editor},
    },
    presentation::{Layer, OAM_BYTES, PALETTE_COLORS, VRAM_BYTES},
};
fn records(value: u8) -> SessionRecords {
    SessionRecords {
        track_names: [value; 48],
        track_times: [value; 32],
        score_digits: [value; 60],
        score_names: [value; 30],
        score_order: [value; 10],
        truck_names: [value; 12],
    }
}
fn session() -> Session {
    Session {
        signature: [0x12, 0x0f, 0x16, 0x16, 0x19],
        records: records(0x77),
        wallets: [[9; 6]; 4],
        selected_wallet: 3,
        continuation_progress: 0x1234,
        page_limit: 2,
        joined_pages: [0; 2],
        active_humans: 0x85,
        order_slot: 19,
        attract_rotation: 0,
        start_reentry: 0,
        inhibit_video: 0x80,
        enabled_layers: 0x1f,
        window_shape: 0x1234,
        nmi_and_automatic_inputs: false,
        audio_commands: vec![],
        phase: EntryPhase::InitializeMoney,
    }
}
fn display() -> Display {
    Display {
        layers: [Layer {
            map_byte_base: 0x8000,
            tile_byte_base: 0,
            wide: false,
            tall: false,
            scroll: [0x123, 0x456],
            bits_per_pixel: 4,
        }; 3],
        bg3_high: true,
        object_tile_byte_base: 0xc000,
        main_layers: 0x1f,
        sub_layers: 0x17,
        main_window_layers: 0x1f,
        sub_window_layers: 0,
        color_math_layers: 0x1f,
        use_subscreen: true,
        half_color: true,
        subtract_color: true,
        fixed_color: 0x1234,
        brightness: 9,
        force_blank: false,
        window_edges: vec![[2, 250]; 224],
        masked_layers: 0x1f,
        window_inverted_layers: 0,
    }
}
fn memory() -> VideoMemory {
    VideoMemory {
        vram: Box::new([0x5a; VRAM_BYTES]),
        palette: [0x6abc; PALETTE_COLORS],
        staged_oam: [0x69; OAM_BYTES],
        visible_oam: [0x96; OAM_BYTES],
    }
}
fn selection() -> Selection {
    Selection {
        sequence: None,
        resume: names::Resume::Sample,
        page: 0,
        done_mask: 0,
        countdown: names::JOIN_COUNTDOWN,
        editors: [Editor::default(); 2],
        temporary: [[0; names::TEMPORARY_BYTES]; 2],
        phase: names::Phase::WaitFrame,
        sampled: [ControllerWord(0); 2],
        window_abort_enabled: true,
    }
}
fn tables() -> DirectionTables {
    DirectionTables {
        first: [0, 1, -1, 0, 0, 1, -1, 0, 0, 1, -1, 0, 0, 0, 0, 0],
        second: [0, 0, 0, 0, 1, 1, 1, 0, -1, -1, -1, 0, 0, 0, 0, 0],
    }
}
fn word(memory: &VideoMemory, at: u16) -> u16 {
    let i = usize::from(at) * 2;
    u16::from_le_bytes([memory.vram[i], memory.vram[i + 1]])
}
fn fixture() -> ShopAssets {
    ShopAssets {
        costs: [1, 8, 4, 10, 6],
        caps: [0x99, 6, 6, 6, 6],
        cursor_tiles: [
            0x2020, 0x2022, 0x6020, 0x2024, 0x202c, 0x6024, 0xa020, 0xa022, 0xe020,
        ],
        cursor_x: [0, 16, 32, 0, 16, 32, 0, 16, 32],
        cursor_y: [0, 0, 0, 16, 16, 16, 32, 32, 32],
        grid_x: [0, 16, 32, 48, 64, 0, 16, 32, 48, 64, 0, 16],
        grid_y: [0, 0, 0, 0, 0, 16, 16, 16, 16, 16, 32, 32],
        bar_destinations: [
            [0x4a02, 0x4a0d, 0x4a82, 0x4a8d, 0x4b02],
            [0x4a12, 0x4a1d, 0x4a92, 0x4a9d, 0x4b12],
        ],
        headings: [vec![2, 0xff], vec![4, 0xff]],
        complete_text: vec![17, 15, 30, 0, 28, 15, 11, 14, 35, 0xff],
    }
}
fn shop() -> Shop {
    Shop {
        selection: selection(),
        redraw_field: 4,
        phase: Phase::WaitFrame,
        fade: None,
        window: None,
        window_motion: None,
    }
}
fn trucks() -> [Truck; 4] {
    std::array::from_fn(|_| Truck::default())
}
fn blank_wallets() -> Session {
    let mut s = session();
    s.wallets = [[0; 6]; 4];
    s.selected_wallet = 0;
    s
}
#[test]
fn loader_uses_parallel_byte_costs_caps_shared_grid_and_full_text_terminators() {
    let rom = std::fs::read("../input/game.rom").unwrap();
    let a = load_assets(&rom).unwrap();
    let expected = fixture();
    assert_eq!(a.costs, expected.costs);
    assert_eq!(a.caps, expected.caps);
    assert_eq!(a.cursor_tiles, expected.cursor_tiles);
    assert_eq!(a.cursor_x, expected.cursor_x);
    assert_eq!(a.cursor_y, expected.cursor_y);
    assert_eq!(a.grid_x, expected.grid_x);
    assert_eq!(a.grid_y, expected.grid_y);
    assert_eq!(a.bar_destinations, expected.bar_destinations);
    assert_eq!(a.headings[0].len(), 25);
    assert_eq!(a.headings[1].len(), 25);
    assert_eq!(a.headings[0][23], 3);
    assert_eq!(a.headings[1][23], 5);
    assert_eq!(a.complete_text, expected.complete_text);
}
#[test]
fn loader_rejects_missing_grid_tail_and_cost_bounds() {
    assert!(load_assets(&vec![0; 0x4015 + 11]).is_err());
    assert!(load_assets(&vec![0; COSTS_ROM + 4]).is_err());
}
#[test]
fn validation_rejects_unterminated_headings_before_display_changes() {
    let mut a = fixture();
    a.headings[1] = vec![1, 2];
    assert!(validate_assets(&a).is_err());
}
#[test]
fn wallet_initialize_resets_four_records_selected_only_preserving_session_records() {
    let mut s = session();
    let old = s.records.clone();
    initialize_wallets(&mut s);
    assert_eq!(s.wallets, [[0, 0, 1, 0, 0, 0]; 4]);
    assert_eq!(s.selected_wallet, 0);
    assert_eq!(s.records, old);
    assert_eq!(s.active_humans, 0x85);
}
#[test]
fn ordinary_award_adds_digit_three_with_byte_wrap_without_normalizing() {
    let mut s = blank_wallets();
    s.selected_wallet = 2;
    s.wallets[2] = [1, 2, 3, 250, 8, 9];
    add_award(&mut s, 10, AwardDigit::TenThousands).unwrap();
    assert_eq!(s.wallets[2], [1, 2, 3, 4, 8, 9]);
    assert_eq!(s.wallets[1], [0; 6]);
}
#[test]
fn conditional_finer_awards_keep_four_and_five_distinct() {
    let mut s = blank_wallets();
    s.selected_wallet = 3;
    add_award(&mut s, 12, AwardDigit::Thousands).unwrap();
    add_award(&mut s, 255, AwardDigit::Hundreds).unwrap();
    assert_eq!(s.wallets[3], [0, 0, 0, 0, 12, 255]);
}
#[test]
fn invalid_selected_wallet_rejects_atomically() {
    let mut s = blank_wallets();
    s.selected_wallet = 4;
    let old = s.clone();
    assert!(normalize_wallet(&mut s).is_err());
    assert_eq!(s, old);
}
#[test]
fn normalization_cascades_carry_low_to_high_but_drops_highest_overflow() {
    let mut s = blank_wallets();
    s.wallets[0] = [9, 9, 9, 9, 9, 19];
    normalize_wallet(&mut s).unwrap();
    assert_eq!(s.wallets[0], [0, 0, 0, 0, 0, 9]);
    assert_eq!(s.selected_wallet, 0);
}
#[test]
fn normalization_borrows_across_zero_digits_to_upper_balance() {
    let mut s = blank_wallets();
    s.wallets[0] = [0, 0, 1, 255, 0, 0];
    normalize_wallet(&mut s).unwrap();
    assert_eq!(s.wallets[0], [0, 0, 0, 9, 0, 0]);
}
#[test]
fn compare_negative_flag_changes_at_eight_a_not_eight_zero() {
    for (raw, digits) in [
        (0x80, [0, 0, 1, 1, 2, 8]),
        (0x89, [0, 0, 1, 1, 3, 7]),
        (0x8a, [0, 0, 0, 8, 8, 2]),
    ] {
        let mut s = blank_wallets();
        s.wallets[0] = [0, 0, 1, 0, 0, raw];
        normalize_wallet(&mut s).unwrap();
        assert_eq!(s.wallets[0], digits, "raw={raw:02x}");
    }
}
#[test]
fn highest_digit_underflow_mutates_selected_byte_for_first_wallet() {
    let mut s = blank_wallets();
    s.wallets[0][0] = 255;
    normalize_wallet(&mut s).unwrap();
    assert_eq!(s.wallets[0], [9, 0, 0, 0, 0, 0]);
    assert_eq!(s.selected_wallet, 255);
}
#[test]
fn highest_digit_underflow_mutates_previous_wallet_tail_without_renormalizing_it() {
    let mut s = blank_wallets();
    s.selected_wallet = 1;
    s.wallets[1][0] = 255;
    normalize_wallet(&mut s).unwrap();
    assert_eq!(s.wallets[1], [9, 0, 0, 0, 0, 0]);
    assert_eq!(s.wallets[0][5], 255);
    assert_eq!(s.selected_wallet, 1);
}
#[test]
fn inactive_conversion_ignores_last_two_digits_and_skips_joined_humans() {
    let mut s = blank_wallets();
    s.wallets[0] = [0, 0, 0, 0, 9, 9];
    s.wallets[1] = [0, 0, 0, 2, 0, 0];
    s.active_humans = 2;
    let mut t = trucks();
    convert_inactive(&mut s, &mut t).unwrap();
    assert_eq!(s.wallets[0], [0, 0, 0, 0, 9, 9]);
    assert_eq!(s.wallets[1][3], 2);
    assert_eq!(t[0].nitro_stock_bcd, 0);
    assert_eq!(t[1].nitro_stock_bcd, 0);
}
#[test]
fn inactive_conversion_decimal_seventy_nine_to_eighty_consumes_extra_unit_before_cap() {
    let mut s = blank_wallets();
    s.active_humans = 0;
    s.wallets[2] = [0, 0, 0, 3, 0, 0];
    let mut t = trucks();
    t[2].nitro_stock_bcd = 0x79;
    convert_inactive(&mut s, &mut t).unwrap();
    assert_eq!(t[2].nitro_stock_bcd, 0x80);
    assert_eq!(s.wallets[2], [0, 0, 0, 1, 0, 0]);
    assert_eq!(s.selected_wallet, 2);
}
#[test]
fn inactive_already_capped_decrements_without_normalization_or_selected_change() {
    let mut s = blank_wallets();
    s.active_humans = 0;
    s.selected_wallet = 3;
    s.wallets[0] = [0, 0, 1, 0, 0, 0];
    let mut t = trucks();
    t[0].nitro_stock_bcd = 0x80;
    convert_inactive(&mut s, &mut t).unwrap();
    assert_eq!(s.wallets[0], [0, 0, 1, 255, 0, 0]);
    assert_eq!(s.selected_wallet, 3);
    assert_eq!(t[0].nitro_stock_bcd, 0x80);
}
#[test]
fn inactive_conversion_visits_descending_slots_and_consumes_hundreds_balance() {
    let mut s = blank_wallets();
    s.active_humans = 0;
    s.wallets[3] = [0, 0, 0, 2, 0, 0];
    s.wallets[1] = [0, 0, 0, 1, 0, 0];
    let mut t = trucks();
    t[3].nitro_stock_bcd = 9;
    convert_inactive(&mut s, &mut t).unwrap();
    assert_eq!(t[3].nitro_stock_bcd, 0x11);
    assert_eq!(t[1].nitro_stock_bcd, 1);
    assert_eq!(s.selected_wallet, 1);
}
#[test]
fn exact_funds_acceleration_purchase_subtracts_digit_three_not_final_digit() {
    let mut s = blank_wallets();
    s.wallets[0] = [0, 0, 0, 8, 9, 9];
    let mut t = trucks();
    assert_eq!(
        purchase(&mut s, &mut t, 0, 0, Item::Acceleration, &fixture()).unwrap(),
        Purchase::Bought
    );
    assert_eq!(s.wallets[0], [0, 0, 0, 0, 9, 9]);
    assert_eq!(t[0].upgrades.acceleration, 1);
}
#[test]
fn upper_funds_allow_raw_negative_purchase_until_next_refresh() {
    let mut s = blank_wallets();
    s.wallets[0] = [0, 0, 1, 0, 0, 0];
    let mut t = trucks();
    purchase(&mut s, &mut t, 0, 0, Item::Speed, &fixture()).unwrap();
    assert_eq!(s.wallets[0], [0, 0, 1, 246, 0, 0]);
    assert_eq!(t[0].upgrades.speed, 1);
}
#[test]
fn insufficient_funds_and_capped_stock_leave_wallet_and_truck_unchanged() {
    let mut s = blank_wallets();
    let mut t = trucks();
    let old = s.clone();
    assert_eq!(
        purchase(&mut s, &mut t, 0, 0, Item::Nitro, &fixture()).unwrap(),
        Purchase::InsufficientFunds
    );
    assert_eq!(s, old);
    s.wallets[0][3] = 9;
    t[0].nitro_stock_bcd = 0x99;
    let old = s.clone();
    assert_eq!(
        purchase(&mut s, &mut t, 0, 0, Item::Nitro, &fixture()).unwrap(),
        Purchase::AtCap
    );
    assert_eq!(s, old);
}
#[test]
fn each_item_uses_its_cost_cap_and_stock_field_without_cross_mutation() {
    for (item, cost) in [
        (Item::Nitro, 1),
        (Item::Acceleration, 8),
        (Item::Steering, 4),
        (Item::Speed, 10),
        (Item::Suspension, 6),
    ] {
        let mut s = blank_wallets();
        s.wallets[0][3] = 20;
        let mut t = trucks();
        purchase(&mut s, &mut t, 0, 0, item, &fixture()).unwrap();
        assert_eq!(s.wallets[0][3], 20 - cost);
        assert_eq!(t[0].nitro_stock_bcd, u8::from(item == Item::Nitro));
        assert_eq!(t[0].upgrades.steering, u8::from(item == Item::Steering));
        assert_eq!(t[0].upgrades.speed, u8::from(item == Item::Speed));
        assert_eq!(
            t[0].upgrades.acceleration,
            u8::from(item == Item::Acceleration)
        );
        assert_eq!(t[0].upgrades.suspension, u8::from(item == Item::Suspension));
    }
}
#[test]
fn nonzero_page_maps_purchase_to_trucks_two_three_and_second_wallet() {
    for player in 0..2 {
        let mut s = blank_wallets();
        s.wallets[2 + player][3] = 4;
        let mut t = trucks();
        purchase(&mut s, &mut t, 1, player, Item::Steering, &fixture()).unwrap();
        assert_eq!(t[2 + player].upgrades.steering, 1);
        assert_eq!(s.wallets[2 + player][3], 0);
        assert_eq!(s.wallets[player][3], 0);
    }
}
#[test]
fn decimal_nitro_increment_nine_to_ten_and_upgrade_six_reject() {
    let mut s = blank_wallets();
    s.wallets[0][3] = 20;
    let mut t = trucks();
    t[0].nitro_stock_bcd = 9;
    purchase(&mut s, &mut t, 0, 0, Item::Nitro, &fixture()).unwrap();
    assert_eq!(t[0].nitro_stock_bcd, 0x10);
    t[0].upgrades.speed = 6;
    assert_eq!(
        purchase(&mut s, &mut t, 0, 0, Item::Speed, &fixture()).unwrap(),
        Purchase::AtCap
    );
    assert_eq!(s.wallets[0][3], 19);
}
#[test]
fn cursor_branches_clamp_negative_to_zero_and_positive_to_last_cell() {
    let mut s = selection();
    move_cursor(
        &mut s,
        0,
        MenuDelta {
            columns: -1,
            rows: -1,
        },
    )
    .unwrap();
    assert_eq!((s.editors[0].column, s.editors[0].row), (0, 0));
    s.editors[0].column = 1;
    s.editors[0].row = 2;
    move_cursor(
        &mut s,
        0,
        MenuDelta {
            columns: 1,
            rows: 1,
        },
    )
    .unwrap();
    assert_eq!((s.editors[0].column, s.editors[0].row), (1, 2));
}
#[test]
fn cursor_geometry_uses_five_column_stride_three_times_x_and_byte_wrap() {
    let mut s = selection();
    s.editors[0].column = 1;
    s.editors[0].row = 2;
    s.editors[1] = s.editors[0];
    let mut m = memory();
    position_cursor(&s, 0, &fixture(), &mut m).unwrap();
    position_cursor(&s, 1, &fixture(), &mut m).unwrap();
    assert_eq!(&m.staged_oam[..2], &[64, 160]);
    assert_eq!(&m.staged_oam[0x24..0x26], &[192, 160]);
    assert_eq!(m.staged_oam[0x20], 96);
    assert_eq!(m.staged_oam[0x45], 192);
    assert_eq!(m.staged_oam[2], 0x69);
}
#[test]
fn page_initialization_clears_coordinates_sampled_high_but_preserves_history_repeat_low_inputs() {
    let mut h = shop();
    h.selection.editors[0] = Editor {
        column: 1,
        row: 2,
        repeat: 7,
        button_history: 0xe1,
        cursor_byte: 6,
    };
    h.selection.sampled = [ControllerWord(0xffff); 2];
    let mut s = session();
    s.joined_pages = [1, 2];
    let mut m = memory();
    initialize_page(&mut h, &mut s, &fixture(), &mut m).unwrap();
    assert_eq!(h.selection.done_mask, 2);
    assert_eq!(
        (h.selection.editors[0].column, h.selection.editors[0].row),
        (0, 0)
    );
    assert_eq!(
        (
            h.selection.editors[0].repeat,
            h.selection.editors[0].button_history
        ),
        (7, 0xe1)
    );
    assert_eq!(h.selection.sampled, [ControllerWord(0x00ff); 2]);
}
#[test]
fn confirm_uses_b_edge_not_start_and_holds_do_not_repeat_purchases() {
    let mut h = shop();
    let mut s = blank_wallets();
    s.wallets[0][3] = 5;
    let mut t = trucks();
    let mut m = memory();
    for input in [BUTTON_START, BUTTON_B, BUTTON_B] {
        handle_player(
            &mut h,
            &mut s,
            &mut t,
            0,
            ControllerWord(input),
            &tables(),
            &fixture(),
            &mut m,
        )
        .unwrap();
    }
    assert_eq!(t[0].nitro_stock_bcd, 1);
    assert_eq!(s.wallets[0][3], 4);
    assert_eq!(h.selection.done_mask, 0);
    assert_eq!(h.selection.editors[0].button_history & 0xc0, 0xc0);
}
#[test]
fn done_players_do_not_mutate_repeat_or_edge_history_or_balance() {
    let mut h = shop();
    h.selection.done_mask = 1;
    h.selection.editors[0].repeat = 7;
    h.selection.editors[0].button_history = 0x55;
    let old = h.clone();
    let mut s = blank_wallets();
    handle_player(
        &mut h,
        &mut s,
        &mut trucks(),
        0,
        ControllerWord(BUTTON_B | BUTTON_RIGHT),
        &tables(),
        &fixture(),
        &mut memory(),
    )
    .unwrap();
    assert_eq!(h, old);
}
#[test]
fn held_direction_reload_eight_means_nine_calls_and_high_button_consumes_repeat() {
    let mut h = shop();
    let mut s = blank_wallets();
    let mut t = trucks();
    let mut m = memory();
    handle_player(
        &mut h,
        &mut s,
        &mut t,
        0,
        ControllerWord(BUTTON_DOWN),
        &tables(),
        &fixture(),
        &mut m,
    )
    .unwrap();
    assert_eq!(h.selection.editors[0].row, 1);
    for _ in 0..8 {
        handle_player(
            &mut h,
            &mut s,
            &mut t,
            0,
            ControllerWord(BUTTON_DOWN),
            &tables(),
            &fixture(),
            &mut m,
        )
        .unwrap();
        assert_eq!(h.selection.editors[0].row, 1);
    }
    handle_player(
        &mut h,
        &mut s,
        &mut t,
        0,
        ControllerWord(BUTTON_DOWN),
        &tables(),
        &fixture(),
        &mut m,
    )
    .unwrap();
    assert_eq!(h.selection.editors[0].row, 2);
    assert_eq!(h.selection.editors[0].repeat, 8);
}
#[test]
fn low_byte_only_input_resets_repeat_and_released_b_allows_new_purchase() {
    let mut h = shop();
    h.selection.editors[0].repeat = 8;
    h.selection.editors[0].button_history = 0x80;
    let mut s = blank_wallets();
    s.wallets[0][3] = 2;
    let mut t = trucks();
    let mut m = memory();
    handle_player(
        &mut h,
        &mut s,
        &mut t,
        0,
        ControllerWord(BUTTON_A),
        &tables(),
        &fixture(),
        &mut m,
    )
    .unwrap();
    assert_eq!(h.selection.editors[0].repeat, 0);
    handle_player(
        &mut h,
        &mut s,
        &mut t,
        0,
        ControllerWord(BUTTON_B),
        &tables(),
        &fixture(),
        &mut m,
    )
    .unwrap();
    assert_eq!(t[0].nitro_stock_bcd, 1);
}
#[test]
fn first_done_path_waits_before_drawing_second_done_has_no_extra_wait() {
    for player in 0..2 {
        let mut h = shop();
        h.selection.editors[player].row = 2;
        h.selection.editors[player].column = 1;
        let mut s = blank_wallets();
        let mut m = memory();
        handle_player(
            &mut h,
            &mut s,
            &mut trucks(),
            player,
            ControllerWord(BUTTON_B),
            &tables(),
            &fixture(),
            &mut m,
        )
        .unwrap();
        if player == 0 {
            assert_eq!(h.phase, Phase::FirstDoneWait);
            assert_eq!(h.selection.done_mask, 0);
        } else {
            assert_eq!(h.selection.done_mask, 2);
            assert_eq!(h.phase, Phase::WaitFrame);
            for i in 0..9 {
                assert_eq!(m.staged_oam[0x25 + 4 * i], 0xf0);
            }
            assert_eq!(word(&m, 0x4913), 0x2011);
        }
    }
}
#[test]
fn money_draw_left_compacts_significant_digits_then_appends_two_zero_glyphs() {
    let mut m = memory();
    draw_money(&[0, 0, 1, 0, 2, 0], 0x4964, &mut m).unwrap();
    let got: Vec<_> = (0..9).map(|i| word(&m, 0x4964 + i)).collect();
    assert_eq!(
        got,
        [
            0x2002, 0x2001, 0x2003, 0x2001, 0x2001, 0x2001, 0x2000, 0x2000, 0x2000
        ]
    );
}
#[test]
fn all_zero_money_field_emits_only_two_appended_zero_tiles_after_nine_clear() {
    let mut m = memory();
    draw_money(&[0; 6], 0x4974, &mut m).unwrap();
    assert_eq!(word(&m, 0x4974), 0x2001);
    assert_eq!(word(&m, 0x4975), 0x2001);
    assert_eq!(word(&m, 0x4976), 0x2000);
    assert_eq!(word(&m, 0x497d), 0x5a5a);
}
#[test]
fn refresh_normalizes_both_even_absent_but_draws_only_joined_and_leaves_second_selected() {
    let mut h = shop();
    h.selection.page = 1;
    let mut s = blank_wallets();
    s.joined_pages = [0, 2];
    s.wallets[2] = [0, 0, 0, 10, 0, 0];
    s.wallets[3] = [0, 0, 0, 2, 0, 0];
    let mut m = memory();
    refresh_money(&h, &mut s, &mut m).unwrap();
    assert_eq!(s.wallets[2], [0, 0, 1, 0, 0, 0]);
    assert_eq!(s.selected_wallet, 3);
    assert_eq!(word(&m, 0x4964), 0x5a5a);
    assert_eq!(word(&m, 0x4974), 0x2003);
}
#[test]
fn redraw_preincrements_field_and_zero_stock_does_not_clear_stale_tiles() {
    let mut h = shop();
    h.redraw_field = 4;
    let mut m = memory();
    cycle_upgrade_field(&mut h, &trucks(), &fixture(), &mut m).unwrap();
    assert_eq!(h.redraw_field, 0);
    assert_eq!(word(&m, 0x4a02), 0x5a5a);
}
#[test]
fn redraw_caps_at_six_and_draws_pairs_upwards_with_odd_tail() {
    let mut h = shop();
    h.redraw_field = 0;
    let mut t = trucks();
    t[0].upgrades.acceleration = 5;
    t[1].upgrades.acceleration = 9;
    let mut m = memory();
    cycle_upgrade_field(&mut h, &t, &fixture(), &mut m).unwrap();
    assert_eq!(h.redraw_field, 1);
    assert_eq!(word(&m, 0x4a0d), 0x283a);
    assert_eq!(word(&m, 0x49ed), 0x283a);
    assert_eq!(word(&m, 0x49cd), 0x283b);
    assert_eq!(word(&m, 0x4a1d), 0x283a);
    assert_eq!(word(&m, 0x49fd), 0x283a);
    assert_eq!(word(&m, 0x49dd), 0x283a);
}
#[test]
fn done_mask_skips_bar_player_and_nonzero_page_uses_other_truck_pair() {
    let mut h = shop();
    h.selection.page = 1;
    h.selection.done_mask = 1;
    let mut t = trucks();
    t[2].nitro_stock_bcd = 3;
    t[3].nitro_stock_bcd = 1;
    let mut m = memory();
    cycle_upgrade_field(&mut h, &t, &fixture(), &mut m).unwrap();
    assert_eq!(word(&m, 0x4a02), 0x5a5a);
    assert_eq!(word(&m, 0x4a12), 0x283b);
}
#[test]
fn redraw_word_overflow_wraps_before_five_bound_check() {
    let mut h = shop();
    h.redraw_field = 0xffff;
    cycle_upgrade_field(&mut h, &trucks(), &fixture(), &mut memory()).unwrap();
    assert_eq!(h.redraw_field, 0);
}
#[test]
fn page_finish_ack_precedes_byte_increment_and_limit_is_inclusive() {
    let mut h = shop();
    let mut s = session();
    s.audio_commands.clear();
    s.page_limit = 1;
    finish_page(&mut h, &mut s);
    assert_eq!(h.selection.page, 1);
    assert_eq!(h.phase, Phase::Setup);
    assert_eq!(s.audio_commands, [ACK_AUDIO]);
    finish_page(&mut h, &mut s);
    assert_eq!(h.selection.page, 2);
    assert_eq!(h.phase, Phase::Complete);
}
fn names_fixture() -> NameAssets {
    use crate::presentation::{MapImage, Rectangle};
    let image = |destination_word| MapImage {
        rectangle: Rectangle {
            width_flags: 1,
            rows: 1,
            palette_remap: [0; 8],
        },
        words: vec![0x2222],
        destination_word,
    };
    NameAssets {
        frontend_shape: crate::presentation::transitions::Shape {
            edges: [[0; 2]; 128],
        },
        defaults: [0; names::TEMPORARY_BYTES],
        cell_x: [0; names::GRID_CELLS],
        cell_y: [0; names::GRID_CELLS],
        glyph: [0; names::GRID_CELLS],
        grid_rows: vec![0xff; 7],
        instructions: vec![0xff],
        headings: std::array::from_fn(|_| vec![0xff]),
        heading_records: [(0, 0); 4],
        driver_tiles: vec![0x44; 0x4da0],
        base_palette: vec![0x1234; 128],
        alternate_palette: vec![0x5678; 32],
        continued_palettes: [vec![0x3333; 16], vec![0x4444; 16]],
        second_palettes: [vec![0x5555; 16], vec![0x6666; 16]],
        heading: image(0x4020),
        standard_panels: [image(0x4121), image(0x4131)],
        continued_panels: [image(0x4121), image(0x4131)],
        inline_heading: vec![0xff],
    }
}
fn shared_fixture() -> SharedGraphics {
    SharedGraphics {
        font_a: vec![0xa1; 0x800],
        font_b: vec![0xb2; 0x800],
        cursor: vec![0xc3; 0x400],
        cursor_palette: vec![0x1111; 32],
        text_palette: vec![0x2222; 16],
        race_palette: vec![0; 256],
        sprite_palette: vec![0; 128],
        sprite_prefix: vec![0; 0x3000],
        sprite_continuation: vec![0; 0x1040],
        hud: vec![0; 0x1000],
    }
}
#[test]
fn actual_page_installs_graphics_heading_nine_tilegroups_six_bar_primes_then_audio_fade() {
    let mut h = shop();
    h.phase = Phase::Setup;
    let mut s = blank_wallets();
    s.joined_pages = [3, 0];
    s.inhibit_video = 7;
    let mut m = memory();
    let mut d = display();
    let mut t = trucks();
    t[0].upgrades.acceleration = 1;
    install_page(
        &mut h,
        &mut s,
        &t,
        &fixture(),
        &names_fixture(),
        &shared_fixture(),
        &mut m,
        &mut d,
    )
    .unwrap();
    assert_eq!(h.redraw_field, 0);
    assert_eq!(h.phase, Phase::FadeIn);
    assert!(h.fade.is_some());
    assert_eq!(s.inhibit_video, 0);
    assert_eq!(s.enabled_layers, 0x15);
    assert_eq!(s.audio_commands, [SHOP_AUDIO]);
    assert_eq!(d.layers[0].tile_byte_base, 0);
    assert_eq!(d.layers[1].tile_byte_base, 0x8000);
    assert_eq!(word(&m, 0x4904), 0x2002);
    assert_eq!(word(&m, 0x4a0d), 0x283b);
    assert_eq!(
        u16::from_le_bytes([m.staged_oam[2], m.staged_oam[3]]),
        0x2020
    );
    assert_eq!(
        u16::from_le_bytes([m.staged_oam[0x46], m.staged_oam[0x47]]),
        0xe020
    );
}
#[test]
fn service_refreshes_wallet_and_bar_before_purchase_leaves_new_debit_for_next_boundary() {
    let mut h = shop();
    let mut s = blank_wallets();
    s.joined_pages = [1, 0];
    s.wallets[0] = [0, 0, 0, 1, 0, 0];
    let mut m = memory();
    let mut d = display();
    let mut t = trucks();
    advance(
        &mut h,
        &mut s,
        &mut t,
        &fixture(),
        &names_fixture(),
        &shared_fixture(),
        &tables(),
        &mut m,
        &mut d,
        [
            ControllerWord(BUTTON_B),
            ControllerWord(0),
            ControllerWord(BUTTON_B),
            ControllerWord(0),
        ],
    )
    .unwrap();
    assert_eq!(t[0].nitro_stock_bcd, 1);
    assert_eq!(t[2].nitro_stock_bcd, 0);
    assert_eq!(s.wallets[0][3], 0);
    assert_eq!(word(&m, 0x4964), 0x2002);
    assert_eq!(h.redraw_field, 0);
    assert_eq!(word(&m, 0x4a02), 0x5a5a);
    assert!(s.audio_commands.is_empty());
}
#[test]
fn suspended_first_done_wait_resumes_without_resampling_second_controller() {
    let mut h = shop();
    h.phase = Phase::FirstDoneWait;
    h.selection.sampled = [ControllerWord(BUTTON_B), ControllerWord(BUTTON_B)];
    h.selection.editors[0].row = 2;
    h.selection.editors[0].column = 1;
    h.selection.editors[0].button_history = 0x80;
    let mut s = blank_wallets();
    s.joined_pages = [3, 0];
    s.wallets[1][3] = 2;
    let mut t = trucks();
    let mut m = memory();
    advance(
        &mut h,
        &mut s,
        &mut t,
        &fixture(),
        &names_fixture(),
        &shared_fixture(),
        &tables(),
        &mut m,
        &mut display(),
        [ControllerWord(0); 4],
    )
    .unwrap();
    assert_eq!(h.selection.done_mask, 1);
    assert_eq!(t[1].nitro_stock_bcd, 1);
    assert_eq!(s.wallets[1][3], 1);
    assert_eq!(word(&m, 0x4903), 0x2011);
    for i in 0..9 {
        assert_eq!(m.staged_oam[1 + i * 4], 0xf0);
    }
    assert_eq!(h.redraw_field, 4);
}
#[test]
fn completed_frame_hides_then_waits_before_inhibit_audio_window_expansion() {
    let mut h = shop();
    h.selection.done_mask = 3;
    let mut s = blank_wallets();
    s.inhibit_video = 5;
    let mut t = trucks();
    let mut m = memory();
    advance(
        &mut h,
        &mut s,
        &mut t,
        &fixture(),
        &names_fixture(),
        &shared_fixture(),
        &tables(),
        &mut m,
        &mut display(),
        [ControllerWord(0); 4],
    )
    .unwrap();
    assert_eq!(h.phase, Phase::FinalWait);
    assert_eq!(s.inhibit_video, 5);
    assert!(s.audio_commands.is_empty());
    advance(
        &mut h,
        &mut s,
        &mut t,
        &fixture(),
        &names_fixture(),
        &shared_fixture(),
        &tables(),
        &mut m,
        &mut display(),
        [ControllerWord(0); 4],
    )
    .unwrap();
    assert_eq!(s.inhibit_video, 6);
    assert_eq!(s.audio_commands, [COMPLETE_AUDIO]);
    assert_eq!(h.phase, Phase::Expand);
    assert!(h.window.is_some());
}
#[test]
fn purchase_rejects_invalid_local_player_without_touching_session() {
    let mut s = blank_wallets();
    let old = s.clone();
    assert!(purchase(&mut s, &mut trucks(), 0, 2, Item::Nitro, &fixture()).is_err());
    assert_eq!(s, old);
}
