//! Fixed expectations independently derived in ../names.trace.md.
use super::*;
use crate::{
    controls::*,
    frontend::attract::{EntryPhase, SessionRecords},
    presentation::{Layer, OAM_BYTES, PALETTE_COLORS, SORTED_SPRITES, VRAM_BYTES},
};
fn records(value: u8) -> SessionRecords {
    SessionRecords::from_legacy(
        [value; 48],
        [value; 32],
        [value; 60],
        [value; 30],
        [value; 10],
        [value; 12],
    )
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
        resume: Resume::Sample,
        page: 0,
        done_mask: 0,
        countdown: JOIN_COUNTDOWN,
        editors: [Editor::default(); 2],
        temporary: [[0; TEMPORARY_BYTES]; 2],
        phase: Phase::WaitFrame,
        sampled: [ControllerWord(0); 2],
        window_abort_enabled: true,
    }
}
fn sprites() -> SpriteArrays {
    SpriteArrays {
        order: std::array::from_fn(|i| i as u8),
        x: [0x1aa; SORTED_SPRITES],
        y: [0x99; SORTED_SPRITES],
        tile_attributes: [0x5555; SORTED_SPRITES],
        wheel_sample_heights: [0; SORTED_SPRITES],
    }
}
fn assets() -> NameAssets {
    load_assets(&std::fs::read("../input/game.rom").unwrap()).unwrap()
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
#[test]
fn original_assets_load_full_driver_span_all_grid_cells_and_raw_default_bytes() {
    let rom = std::fs::read("../input/game.rom").unwrap();
    let a = load_assets(&rom).unwrap();
    assert_eq!(
        a.defaults,
        [0x2a, 0, 0x2a, 0, 0x2a, 0xff, 0xff, 0xff, 0xff, 0xff]
    );
    assert_eq!(a.driver_tiles, rom[DRIVER_TILES_ROM].to_vec());
    assert_eq!(a.base_palette.len(), 128);
    assert_eq!(a.alternate_palette.len(), 32);
    assert_eq!(
        &a.glyph[25..],
        [
            0x24,
            0x27,
            0,
            0,
            ERASE_TOKEN,
            EXIT_TOKEN,
            EXIT_TOKEN,
            EXIT_TOKEN,
            EXIT_TOKEN,
            EXIT_TOKEN
        ]
    );
    assert_eq!(&a.cell_x[25..], [0, 16, 36, 36, 64, 36, 36, 36, 36, 36]);
    assert_eq!(&a.cell_y[25..], [80, 80, 80, 80, 80, 96, 96, 96, 96, 96]);
    assert_eq!(
        a.heading_records,
        [
            (0x49e6, 0xbaa0),
            (0x4a25, 0xbaa5),
            (0x49f6, 0xbaa0),
            (0x4a35, 0xbaa5)
        ]
    );
    assert_eq!(a.grid_rows.iter().filter(|&&g| g == 0xff).count(), 7);
    assert_eq!(a.inline_heading.len(), 12);
    assert_eq!(a.inline_heading.last(), Some(&0xff));
    for image in a.standard_panels.iter().chain(a.continued_panels.iter()) {
        assert_eq!(image.rectangle.width_flags & 0x7fff, 14);
        assert_eq!(image.rectangle.rows, 18);
        assert_eq!(image.words.len(), 252);
    }
}
#[test]
fn asset_loader_rejects_each_missing_consumer_span() {
    let rom = std::fs::read("../input/game.rom").unwrap();
    for end in [DEFAULTS_ROM + 9, CELL_GLYPH_ROM + 34, 0x4d410, 0x4d50f] {
        assert!(load_assets(&rom[..end]).is_err());
    }
}
#[test]
fn asset_validation_rejects_bad_grid_rows_defaults_driver_maps_and_palette_lengths() {
    let a = assets();
    for variant in 0..5 {
        let mut bad = a.clone();
        match variant {
            0 => {
                bad.grid_rows.pop();
            }
            1 => {
                bad.driver_tiles.pop();
            }
            2 => {
                bad.base_palette.pop();
            }
            3 => {
                bad.heading.words.pop();
            }
            _ => {
                bad.continued_palettes[1].pop();
            }
        };
        assert!(validate_assets(&bad).is_err());
    }
}
#[test]
fn entry_resets_both_page_join_masks_and_increments_video_inhibit_with_wrap() {
    let mut session = session();
    session.joined_pages = [3, 2];
    session.inhibit_video = 255;
    let records = session.records.clone();
    let mut s = selection();
    s.page = 1;
    begin_selection(&mut session, &mut s);
    assert_eq!(session.joined_pages, [0, 0]);
    assert_eq!(session.inhibit_video, 0);
    assert_eq!(s.page, 0);
    assert_eq!(s.phase, Phase::EntryFade);
    assert_eq!(session.records, records);
}
#[test]
fn page_reset_preserves_history_and_repeat_but_resets_cursor_done_and_countdown() {
    let a = assets();
    let mut session = session();
    session.joined_pages = [1, 3];
    let mut s = selection();
    s.page = 1;
    s.editors = [Editor {
        column: 4,
        row: 6,
        cursor_byte: 6,
        repeat: 7,
        button_history: 0xa5,
    }; 2];
    s.done_mask = 3;
    s.countdown = 0;
    initialize_page(&mut s, &mut session, &a);
    assert_eq!(session.joined_pages, [1, 0]);
    assert_eq!(s.done_mask, 0);
    assert_eq!(s.countdown, 0x5f);
    assert_eq!(s.temporary, [a.defaults; 2]);
    for e in s.editors {
        assert_eq!((e.column, e.row, e.cursor_byte), (0, 0, 0));
        assert_eq!((e.repeat, e.button_history), (7, 0xa5));
    }
    assert_eq!(session.enabled_layers, 0x15);
}
#[test]
fn select_join_ror_history_requires_new_press_for_both_players() {
    for player in 0..2 {
        let mut s = selection();
        let mut session = session();
        assert_eq!(
            try_join(&mut s, &mut session, player, ControllerWord(BUTTON_SELECT)).unwrap(),
            JoinResult::AwaitGrid
        );
        assert_eq!(session.joined_pages[0], 1 << player);
        assert_eq!(s.editors[player].button_history, 0x80);
        session.joined_pages[0] = 0;
        assert_eq!(
            try_join(&mut s, &mut session, player, ControllerWord(BUTTON_SELECT)).unwrap(),
            JoinResult::NoEdge
        );
        assert_eq!(s.editors[player].button_history, 0xc0);
        try_join(&mut s, &mut session, player, ControllerWord(0)).unwrap();
        assert_eq!(s.editors[player].button_history, 0x60);
        assert_eq!(
            try_join(&mut s, &mut session, player, ControllerWord(BUTTON_SELECT)).unwrap(),
            JoinResult::AwaitGrid
        );
    }
}
#[test]
fn joined_or_done_guard_leaves_history_untouched_and_ignores_select() {
    for player in 0..2 {
        for joined in [false, true] {
            let mut s = selection();
            let mut session = session();
            s.editors[player].button_history = 0x31;
            if joined {
                session.joined_pages[0] = 1 << player;
            } else {
                s.done_mask = 1 << player;
            }
            assert_eq!(
                try_join(&mut s, &mut session, player, ControllerWord(BUTTON_SELECT)).unwrap(),
                JoinResult::Skipped
            );
            assert_eq!(s.editors[player].button_history, 0x31);
        }
    }
}
#[test]
fn joining_nonzero_page_changes_second_mask_while_sampling_still_first_devices() {
    let mut s = selection();
    s.page = 1;
    let mut session = session();
    let sample = sample_frontend(
        1,
        [
            ControllerWord(0),
            ControllerWord(BUTTON_SELECT),
            ControllerWord(BUTTON_SELECT),
            ControllerWord(0),
        ],
    );
    try_join(&mut s, &mut session, 1, sample.players[1]).unwrap();
    assert_eq!(session.joined_pages, [0, 2]);
}
#[test]
fn joined_grids_install_correct_panel_coordinates_and_cursor_tiles() {
    let a = assets();
    for player in 0..2 {
        let s = selection();
        let mut mem = memory();
        let mut sp = sprites();
        draw_joined_grid(&s, player, &a, &mut mem, &mut sp).unwrap();
        let pair = player * 2;
        assert_eq!(
            &sp.x[pair..pair + 2],
            if player == 0 {
                &[0x15, 0x15]
            } else {
                &[0x9d, 0x9d]
            }
        );
        assert_eq!(&sp.y[pair..pair + 2], &[0x48, 0x58]);
        assert_eq!(&sp.tile_attributes[pair..pair + 2], &[0x2026, 0x2028]);
        let at = if player == 0 { 0x4943 } else { 0x4954 };
        assert_eq!(word(&mem, at), 0x2016);
        assert_eq!(word(&mem, at + 32), 0x2017);
    }
}
#[test]
fn editor_guards_preserve_names_repeat_history_and_sprites() {
    let a = assets();
    for player in 0..2 {
        let mut s = selection();
        let session = session();
        let before = s.clone();
        let mut sp = sprites();
        let old = sp.clone();
        edit_player(
            &mut s,
            &session,
            player,
            ControllerWord(BUTTON_B | BUTTON_RIGHT),
            &tables(),
            &a,
            &mut sp,
        )
        .unwrap();
        assert_eq!(s, before);
        assert_eq!(sp, old);
    }
}
#[test]
fn repeat_reload_eight_produces_nine_call_movement_cadence() {
    let a = assets();
    let mut s = selection();
    let mut session = session();
    session.joined_pages[0] = 1;
    let mut sp = sprites();
    edit_player(
        &mut s,
        &session,
        0,
        ControllerWord(BUTTON_RIGHT),
        &tables(),
        &a,
        &mut sp,
    )
    .unwrap();
    assert_eq!((s.editors[0].column, s.editors[0].repeat), (1, 8));
    for expected in (0..8).rev() {
        edit_player(
            &mut s,
            &session,
            0,
            ControllerWord(BUTTON_RIGHT),
            &tables(),
            &a,
            &mut sp,
        )
        .unwrap();
        assert_eq!((s.editors[0].column, s.editors[0].repeat), (1, expected));
    }
    edit_player(
        &mut s,
        &session,
        0,
        ControllerWord(BUTTON_RIGHT),
        &tables(),
        &a,
        &mut sp,
    )
    .unwrap();
    assert_eq!((s.editors[0].column, s.editors[0].repeat), (2, 8));
}
#[test]
fn zero_high_byte_resets_repeat_but_any_high_action_consumes_delay() {
    let a = assets();
    let mut s = selection();
    let mut session = session();
    session.joined_pages[0] = 1;
    let mut sp = sprites();
    s.editors[0].repeat = 6;
    edit_player(
        &mut s,
        &session,
        0,
        ControllerWord(BUTTON_A),
        &tables(),
        &a,
        &mut sp,
    )
    .unwrap();
    assert_eq!(s.editors[0].repeat, 0);
    edit_player(
        &mut s,
        &session,
        0,
        ControllerWord(BUTTON_START),
        &tables(),
        &a,
        &mut sp,
    )
    .unwrap();
    assert_eq!(s.editors[0].repeat, 8);
    assert_eq!(s.editors[0].column, 0);
}
#[test]
fn both_player_grid_coordinates_wrap_five_columns_and_seven_rows() {
    let a = assets();
    for player in 0..2 {
        for (input, start, end) in [
            (BUTTON_LEFT, (0, 0), (4, 0)),
            (BUTTON_RIGHT, (4, 0), (0, 0)),
            (BUTTON_UP, (0, 0), (0, 6)),
            (BUTTON_DOWN, (0, 6), (0, 0)),
            (BUTTON_UP | BUTTON_LEFT, (0, 0), (4, 6)),
        ] {
            let mut s = selection();
            let mut session = session();
            session.joined_pages[0] = 1 << player;
            let mut sp = sprites();
            s.editors[player].column = start.0;
            s.editors[player].row = start.1;
            edit_player(
                &mut s,
                &session,
                player,
                ControllerWord(input),
                &tables(),
                &a,
                &mut sp,
            )
            .unwrap();
            assert_eq!((s.editors[player].column, s.editors[player].row), end);
        }
    }
}
#[test]
fn cursor_x_updates_low_byte_only_preserving_source_word_high() {
    let a = assets();
    for player in 0..2 {
        let mut s = selection();
        let mut session = session();
        session.joined_pages[0] = 1 << player;
        s.editors[player].row = 5;
        s.editors[player].column = 2;
        let mut sp = sprites();
        edit_player(
            &mut s,
            &session,
            player,
            ControllerWord(0),
            &tables(),
            &a,
            &mut sp,
        )
        .unwrap();
        let pair = player * 2;
        assert_eq!(sp.x[pair], if player == 0 { 0x139 } else { 0x1c1 });
        assert_eq!(sp.x[pair + 1], sp.x[pair]);
        assert_eq!(&sp.y[pair..pair + 2], &[0x98, 0xa8]);
    }
}
#[test]
fn normal_glyph_preview_occurs_without_b_and_preserves_interleaved_bytes() {
    let a = assets();
    let mut s = selection();
    s.temporary = [[0x7a; 10]; 2];
    let mut session = session();
    session.joined_pages[0] = 3;
    let mut sp = sprites();
    for player in 0..2 {
        edit_player(
            &mut s,
            &session,
            player,
            ControllerWord(0),
            &tables(),
            &a,
            &mut sp,
        )
        .unwrap();
        assert_eq!(s.temporary[player][0], 0x0b);
        assert_eq!(s.temporary[player][1], 0x7a);
        assert_eq!(s.editors[player].cursor_byte, 0);
    }
}
#[test]
fn b_confirms_ordinary_glyph_only_on_rising_edge_and_advances_through_byte_six() {
    let a = assets();
    let mut s = selection();
    let mut session = session();
    session.joined_pages[0] = 1;
    let mut sp = sprites();
    for expected in [2, 4, 6, 6] {
        edit_player(
            &mut s,
            &session,
            0,
            ControllerWord(0),
            &tables(),
            &a,
            &mut sp,
        )
        .unwrap();
        edit_player(
            &mut s,
            &session,
            0,
            ControllerWord(BUTTON_B),
            &tables(),
            &a,
            &mut sp,
        )
        .unwrap();
        assert_eq!(s.editors[0].cursor_byte, expected);
        edit_player(
            &mut s,
            &session,
            0,
            ControllerWord(BUTTON_B),
            &tables(),
            &a,
            &mut sp,
        )
        .unwrap();
        assert_eq!(s.editors[0].cursor_byte, expected);
    }
    assert_eq!(s.temporary[0][6], 0x0b);
}
#[test]
fn control_cells_preview_placeholder_before_confirm_and_do_not_write_negative_token() {
    let a = assets();
    for cell in [29, 30, 31, 32, 33, 34] {
        let mut s = selection();
        let mut session = session();
        session.joined_pages[0] = 1;
        s.editors[0].row = cell / 5;
        s.editors[0].column = cell % 5;
        let mut sp = sprites();
        edit_player(
            &mut s,
            &session,
            0,
            ControllerWord(0),
            &tables(),
            &a,
            &mut sp,
        )
        .unwrap();
        assert_eq!(s.temporary[0][0], PLACEHOLDER);
        assert_eq!(s.done_mask, 0);
    }
}
#[test]
fn erase_decrements_two_without_clearing_previous_preview_and_stops_at_zero() {
    let a = assets();
    for cursor in [0, 2, 4, 6] {
        let mut s = selection();
        let mut session = session();
        session.joined_pages[0] = 2;
        s.editors[1].row = 5;
        s.editors[1].column = 4;
        s.editors[1].cursor_byte = cursor;
        s.temporary[1] = [0x55; 10];
        let mut sp = sprites();
        edit_player(
            &mut s,
            &session,
            1,
            ControllerWord(BUTTON_B),
            &tables(),
            &a,
            &mut sp,
        )
        .unwrap();
        assert_eq!(s.editors[1].cursor_byte, cursor.saturating_sub(2));
        assert_eq!(s.temporary[1][cursor as usize], PLACEHOLDER);
        assert_eq!(s.done_mask, 0);
    }
}
#[test]
fn every_exit_cell_sets_own_done_bit_and_preserves_other_bit() {
    let a = assets();
    for player in 0..2 {
        for column in 0..5 {
            let mut s = selection();
            let mut session = session();
            session.joined_pages[0] = 3;
            s.done_mask = 1 << (1 - player);
            s.editors[player].row = 6;
            s.editors[player].column = column;
            let mut sp = sprites();
            edit_player(
                &mut s,
                &session,
                player,
                ControllerWord(BUTTON_B),
                &tables(),
                &a,
                &mut sp,
            )
            .unwrap();
            assert_eq!(s.done_mask, 3);
        }
    }
}
#[test]
fn join_select_history_can_suppress_b_same_iteration_then_next_iteration_confirms() {
    let a = assets();
    let mut s = selection();
    let mut session = session();
    let mut sp = sprites();
    let both = ControllerWord(BUTTON_SELECT | BUTTON_B);
    try_join(&mut s, &mut session, 0, both).unwrap();
    assert_eq!(s.editors[0].button_history, 0x80);
    edit_player(&mut s, &session, 0, both, &tables(), &a, &mut sp).unwrap();
    assert_eq!(s.editors[0].cursor_byte, 0);
    assert_eq!(s.editors[0].button_history, 0xc0);
    edit_player(
        &mut s,
        &session,
        0,
        ControllerWord(0),
        &tables(),
        &a,
        &mut sp,
    )
    .unwrap();
    edit_player(
        &mut s,
        &session,
        0,
        ControllerWord(BUTTON_B),
        &tables(),
        &a,
        &mut sp,
    )
    .unwrap();
    assert_eq!(s.editors[0].cursor_byte, 2);
}
#[test]
fn countdown_draws_old_upper_nibble_plus_one_only_for_absent_slots() {
    let mut s = selection();
    let mut session = session();
    session.joined_pages[0] = 1;
    let mut mem = memory();
    update_countdown(&mut s, &session, 1, &mut mem).unwrap();
    assert_eq!(s.countdown, 0x5f);
    assert_eq!(s.temporary[0][7..9], [1, 6]);
    assert_eq!(word(&mem, 0x4a67), 0x5a5a);
    assert_eq!(word(&mem, 0x4a77), 0x2002);
    assert_eq!(word(&mem, 0x4a78), 0x200c);
}
#[test]
fn countdown_ticks_only_every_fourth_animation_frame() {
    let session = session();
    for frame in [0, 1, 2, 3, 4, 255] {
        let mut s = selection();
        s.countdown = 2;
        let mut mem = memory();
        update_countdown(&mut s, &session, frame, &mut mem).unwrap();
        assert_eq!(s.countdown, if frame & 3 == 0 { 1 } else { 2 });
    }
}
#[test]
fn countdown_zero_returns_without_repeat_completion_or_redraw() {
    let session = session();
    let mut s = selection();
    s.countdown = 0;
    let before = s.clone();
    let mut mem = memory();
    let old = mem.clone();
    update_countdown(&mut s, &session, 0, &mut mem).unwrap();
    assert_eq!(s, before);
    assert_eq!(mem, old);
}
#[test]
fn countdown_expiry_marks_exactly_absent_players_done_by_xor_then_or() {
    for joined in 0..4 {
        let mut s = selection();
        s.countdown = 1;
        s.done_mask = joined;
        let mut session = session();
        session.joined_pages[0] = joined;
        let mut mem = memory();
        update_countdown(&mut s, &session, 0, &mut mem).unwrap();
        assert_eq!(s.countdown, 0);
        assert_eq!(s.done_mask, 3);
    }
}
#[test]
fn final_countdown_tick_precedes_select_and_prevents_late_join() {
    let mut s = selection();
    s.countdown = 1;
    let mut session = session();
    let mut mem = memory();
    update_countdown(&mut s, &session, 0, &mut mem).unwrap();
    assert_eq!(
        try_join(&mut s, &mut session, 0, ControllerWord(BUTTON_SELECT)).unwrap(),
        JoinResult::Skipped
    );
    assert_eq!(session.joined_pages[0], 0);
}
#[test]
fn panel_hide_clears_eleven_by_fourteen_rows_and_hides_only_done_cursor_pairs() {
    for done in 0..4 {
        let mut s = selection();
        s.done_mask = done;
        let mut mem = memory();
        let mut sp = sprites();
        hide_done(&s, &mut mem, &mut sp);
        for (bit, pair, base) in [(1, 0, 0x4941), (2, 2, 0x4954)] {
            for row in 0..14 {
                for col in 0..11 {
                    assert_eq!(
                        word(&mem, base + row * 32 + col),
                        if done & bit != 0 { 0x2000 } else { 0x5a5a }
                    );
                }
            }
            assert_eq!(
                &sp.y[pair..pair + 2],
                if done & bit != 0 {
                    &[0xf0, 0xf0]
                } else {
                    &[0x99, 0x99]
                }
            );
        }
    }
}
#[test]
fn commit_alternating_first_three_bytes_for_both_names_and_nonzero_page_offset() {
    for page in [0, 1] {
        let mut s = selection();
        s.page = page;
        s.temporary = [
            [1, 0xa1, 2, 0xa2, 3, 0xa3, 4, 0xa4, 5, 0xff],
            [6, 0xb1, 7, 0xb2, 8, 0xb3, 9, 0xb4, 10, 0xff],
        ];
        let mut session = session();
        commit_names(&s, &mut session);
        let base = if page == 0 { 0 } else { 6 };
        assert_eq!(
            &session.records.truck_names[base..base + 6],
            &[1, 2, 3, 6, 7, 8]
        );
        assert_eq!(
            &session.records.truck_names[(6 - base)..(12 - base)],
            &[0x77; 6]
        );
    }
}
#[test]
fn page_limit_is_inclusive_byte_comparison_after_ack_restores_m8() {
    for (limit, page, phase) in [(0, 1, Phase::Complete), (1, 1, Phase::PageSetup)] {
        let mut s = selection();
        let mut session = session();
        session.page_limit = limit;
        finish_page(&mut s, &mut session);
        assert_eq!(s.page, page);
        assert_eq!(s.phase, phase);
        assert_eq!(session.audio_commands.last(), Some(&ACK_AUDIO));
    }
}
#[test]
fn standard_driver_install_preserves_upper_palette_and_sets_alternate_subset() {
    let a = assets();
    for page in [0, 1] {
        let mut mem = memory();
        let mut d = display();
        install_driver_panels(&a, page, 3, true, &mut mem, &mut d).unwrap();
        assert_eq!(&mem.vram[..a.driver_tiles.len()], a.driver_tiles.as_slice());
        assert_eq!(&mem.palette[128..], &[0x6abc; 128]);
        if page == 0 {
            assert_eq!(&mem.palette[..128], a.base_palette.as_slice());
        } else {
            assert_eq!(&mem.palette[64..96], a.alternate_palette.as_slice());
        }
        assert_eq!(word(&mem, 0x4121), a.standard_panels[0].words[0]);
        assert_eq!(word(&mem, 0x4131), a.standard_panels[1].words[0]);
    }
}
#[test]
fn continued_driver_choices_select_distinct_page_palettes_and_clear_text_map() {
    let a = assets();
    for page in [0, 1] {
        for joined in 0..4 {
            let mut mem = memory();
            let mut d = display();
            install_driver_panels(&a, page, joined, false, &mut mem, &mut d).unwrap();
            if joined & 1 != 0 {
                assert_eq!(
                    &mem.palette[64..80],
                    a.continued_palettes[page as usize].as_slice()
                );
            }
            if joined & 2 != 0 {
                assert_eq!(
                    &mem.palette[80..96],
                    a.second_palettes[page as usize].as_slice()
                );
            }
            assert_eq!(word(&mem, 0x4800), 0);
            assert_eq!(word(&mem, 0x4c00), 0);
        }
    }
}
#[test]
fn malformed_driver_assets_are_rejected_without_partial_video_mutation() {
    let mut a = assets();
    a.continued_panels[0].words.clear();
    let mut mem = memory();
    let old = mem.clone();
    let mut d = display();
    let oldd = d.clone();
    assert!(install_driver_panels(&a, 0, 1, false, &mut mem, &mut d).is_err());
    assert_eq!(mem, old);
    assert_eq!(d, oldd);
}
#[test]
fn real_page_install_draws_heading_instructions_and_hides_staged_cursor_sprites() {
    let rom = std::fs::read("../input/game.rom").unwrap();
    let a = assets();
    let shared = crate::presentation::memory::load_shared(&rom).unwrap();
    let s = selection();
    let mut mem = memory();
    let mut d = display();
    let mut sp = sprites();
    install_name_page(&s, &a, &shared, &mut mem, &mut d, &mut sp).unwrap();
    assert_eq!(word(&mem, 0x4904), 0x208e);
    assert_eq!(word(&mem, 0x4914), 0x208e);
    assert_eq!(sp.y, [0xf0; 16]);
    assert_eq!(d.layers[2].scroll[1], 0);
}
#[test]
fn full_service_samples_only_joy_one_two_after_countdown_and_stages_before_join_grid_wait() {
    let rom = std::fs::read("../input/game.rom").unwrap();
    let a = assets();
    let shared = crate::presentation::memory::load_shared(&rom).unwrap();
    let mut s = selection();
    let mut session = session();
    let mut mem = memory();
    let mut d = display();
    let mut sp = sprites();
    let previous = mem.visible_oam;
    advance(
        &mut s,
        &mut session,
        &a,
        &shared,
        &tables(),
        &mut mem,
        &mut d,
        &mut sp,
        [
            ControllerWord(BUTTON_SELECT),
            ControllerWord(0),
            ControllerWord(BUTTON_SELECT),
            ControllerWord(BUTTON_SELECT),
        ],
        0,
    )
    .unwrap();
    assert_eq!(s.countdown, 0x5e);
    assert_eq!(session.joined_pages, [1, 0]);
    assert_eq!(s.phase, Phase::JoinDraw(0));
    assert_eq!(
        s.sampled,
        [ControllerWord(BUTTON_SELECT), ControllerWord(0)]
    );
    assert_eq!(mem.visible_oam, previous);
}
#[test]
fn full_completion_commits_names_before_final_wait_then_inhibits_and_sends_ordered_audio() {
    let rom = std::fs::read("../input/game.rom").unwrap();
    let a = assets();
    let shared = crate::presentation::memory::load_shared(&rom).unwrap();
    let mut s = selection();
    s.done_mask = 3;
    s.temporary[0] = [1, 0, 2, 0, 3, 0xff, 0xff, 0xff, 0xff, 0xff];
    let mut session = session();
    session.inhibit_video = 0;
    let mut mem = memory();
    let mut d = display();
    let mut sp = sprites();
    advance(
        &mut s,
        &mut session,
        &a,
        &shared,
        &tables(),
        &mut mem,
        &mut d,
        &mut sp,
        [ControllerWord(0); 4],
        1,
    )
    .unwrap();
    assert_eq!(&session.records.truck_names[..3], &[1, 2, 3]);
    assert_eq!(s.phase, Phase::FinalWait);
    assert_eq!(session.inhibit_video, 0);
    advance(
        &mut s,
        &mut session,
        &a,
        &shared,
        &tables(),
        &mut mem,
        &mut d,
        &mut sp,
        [ControllerWord(0); 4],
        2,
    )
    .unwrap();
    assert_eq!(session.inhibit_video, 1);
    assert!(!s.window_abort_enabled);
    assert_eq!(session.audio_commands.last(), Some(&COMPLETE_AUDIO));
    assert_eq!(s.phase, Phase::ContractWindow);
}

#[test]
fn conditional_inline_heading_renders_source_glyphs_at_fixed_destination_without_new_ui() {
    let a = assets();
    let mut memory = memory();
    draw_inline_heading(&a, &mut memory).unwrap();
    assert_eq!(word(&memory, 0x408a), 0x201a);
    assert_eq!(word(&memory, 0x4094), 0x201e);
    assert_eq!(word(&memory, 0x4095), 0x5a5a);
}
#[test]
fn joining_and_character_confirmation_do_not_invent_additional_sound_commands() {
    let a = assets();
    let mut s = selection();
    let mut session = session();
    let mut sp = sprites();
    try_join(&mut s, &mut session, 0, ControllerWord(BUTTON_SELECT)).unwrap();
    edit_player(
        &mut s,
        &session,
        0,
        ControllerWord(0),
        &tables(),
        &a,
        &mut sp,
    )
    .unwrap();
    edit_player(
        &mut s,
        &session,
        0,
        ControllerWord(BUTTON_B),
        &tables(),
        &a,
        &mut sp,
    )
    .unwrap();
    assert!(session.audio_commands.is_empty());
}
