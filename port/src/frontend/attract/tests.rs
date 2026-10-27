//! Literal branch/width fixtures independently derived in attract.trace.md.
use super::*;
use crate::{
    controls::BUTTON_START,
    presentation::{
        Layer, OAM_BYTES, PALETTE_COLORS, Rectangle, VRAM_BYTES, transitions::FadeDirection,
    },
    trucks::motion::Upgrades,
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
fn map(width: u16, rows: u16, dest: u16, value: u16) -> MapImage {
    MapImage {
        rectangle: Rectangle {
            width_flags: width,
            rows,
            palette_remap: [0, 1, 2, 3, 4, 5, 6, 7],
        },
        words: vec![value; usize::from(width) * usize::from(rows)],
        destination_word: dest,
    }
}
fn assets() -> FrontendAssets {
    FrontendAssets {
        signature: [0x12, 0x0f, 0x16, 0x16, 0x19],
        attract_slots: [0, 1, 2, 3],
        frontend_tiles: vec![0x12; 0x2580],
        frontend_tail: vec![0x34; 0x940],
        frontend_palette: vec![0x2345; 112],
        attract_tiles: vec![0x56; 0x4d00],
        attract_map: vec![0x3a5b; 0x300],
        attract_palette: vec![0x4567; 128],
        license: map(18, 25, 0x4047, 0x0002),
        software: map(23, 17, 0x40c3, 0x0003),
    }
}
fn trucks() -> [Truck; 4] {
    std::array::from_fn(|_| Truck {
        nitro: crate::trucks::nitro::NitroStock::from_encoded(0x42),
        upgrades: Upgrades {
            steering: 3,
            speed: 4,
            acceleration: 5,
            suspension: 6,
        },
        speed: 0x1234,
        heading: 0x5678,
        nitro_button_history: 0x80,
        ..Truck::default()
    })
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
fn automatic(slot: usize) -> [ControllerWord; 4] {
    let mut a = [ControllerWord(0); 4];
    a[slot] = ControllerWord(BUTTON_START);
    a
}
fn splash(phase: SplashPhase, timer: u16) -> Splash {
    Splash {
        phase,
        timer,
        fade: Fade {
            direction: FadeDirection::In,
            primed: true,
            complete: false,
            animation_counter: 0,
            enabled_layers: 1,
        },
    }
}
#[test]
fn full_signature_requires_each_of_five_bytes_not_only_first() {
    let a = assets();
    assert!(signature_matches(&a.signature, &a.signature));
    for i in 0..5 {
        let mut s = a.signature;
        s[i] ^= 1;
        assert!(!signature_matches(&s, &a.signature));
    }
}
#[test]
fn valid_entry_preserves_records_before_money_phase_and_configures_audio_display() {
    let mut s = session();
    let before = s.records.clone();
    let mut d = display();
    begin_entry(&mut s, &mut d, &assets());
    assert_eq!(s.records, before);
    assert_eq!(s.wallets, [[9; 6]; 4]);
    assert_eq!(s.phase, EntryPhase::InitializeMoney);
    assert_eq!(s.audio_commands, [FRONTEND_AUDIO]);
    assert_eq!(s.continuation_progress, 0);
    assert_eq!(s.page_limit, 0);
    assert_eq!(s.window_shape, 0);
    assert!(s.nmi_and_automatic_inputs);
    assert_eq!(d.layers[0].scroll, [0, 0]);
    assert_eq!(d.color_math_layers, 0);
    assert!(!d.use_subscreen);
    assert!(!d.half_color);
    assert!(!d.subtract_color);
    assert_eq!(d.fixed_color, 0x1234);
}
#[test]
fn any_invalid_signature_byte_clears_only_first_then_runs_initial_license_before_defaults() {
    for i in 0..5 {
        let mut s = session();
        s.signature[i] ^= 1;
        let old = s.signature;
        begin_entry(&mut s, &mut display(), &assets());
        assert_eq!(s.phase, EntryPhase::InitialLicense);
        assert_eq!(s.signature[0], 0);
        assert_eq!(&s.signature[1..], &old[1..]);
        assert_eq!(s.records, records(0x77));
        assert_eq!(s.wallets, [[9; 6]; 4]);
    }
}
#[test]
fn cold_record_defaults_copy_only_after_initial_splashes_return() {
    let mut s = session();
    s.phase = EntryPhase::InitialLicense;
    let expected = records(0x2a);
    finish_initial_splashes(&mut s, &expected);
    assert_eq!(s.records, expected);
    assert_eq!(s.phase, EntryPhase::InitializeMoney);
    assert_eq!(s.wallets, [[9; 6]; 4]);
}
#[test]
fn warm_and_cold_shared_setup_reset_wallets_twenty_stock_upgrade_bytes_but_not_motion() {
    for initial in [EntryPhase::InitializeMoney, EntryPhase::InitialLicense] {
        let mut s = session();
        s.phase = initial;
        let mut t = trucks();
        let mut d = display();
        initialize_session(&mut s, &mut t, &mut d);
        assert_eq!(s.wallets, [[0, 0, 1, 0, 0, 0]; 4]);
        assert_eq!(s.selected_wallet, 0);
        assert_eq!(s.records, records(0x77));
        for truck in t {
            assert_eq!(truck.nitro.encoded(), 0);
            assert_eq!(truck.upgrades, Upgrades::default());
            assert_eq!(truck.speed, 0x1234);
            assert_eq!(truck.heading, 0x5678);
            assert_eq!(truck.nitro_button_history, 0x80);
        }
        assert_eq!(s.inhibit_video, 0x81);
        assert_eq!(d.main_layers, 0x1f);
        assert_eq!(d.main_window_layers, 0x1f);
        assert_eq!(s.enabled_layers, 5);
        assert_eq!(s.phase, EntryPhase::AttractPanel);
    }
}
#[test]
fn nonzero_reentry_is_consumed_after_common_reset_and_bypasses_attract_panel() {
    let mut s = session();
    s.start_reentry = 0xff;
    let rotation = s.attract_rotation;
    initialize_session(&mut s, &mut trucks(), &mut display());
    assert_eq!(s.start_reentry, 0);
    assert_eq!(s.phase, EntryPhase::DriverSelection);
    assert_eq!(s.attract_rotation, rotation);
    assert_eq!(s.wallets, [[0, 0, 1, 0, 0, 0]; 4]);
}
#[test]
fn attract_rotates_before_lookup_and_wraps_byte_counter_then_masks() {
    for (before, after) in [(0, 1), (2, 3), (3, 0), (0xff, 0)] {
        let mut s = session();
        s.attract_rotation = before;
        let mut t = trucks();
        let mut d = display();
        let mut a = assets();
        a.attract_slots = [7, 11, 15, 19];
        finish_attract_presentation(&mut s, &mut t, &mut d, &a);
        assert_eq!(s.attract_rotation, after);
        assert_eq!(s.order_slot, [7, 11, 15, 19][after as usize]);
        assert_eq!(s.active_humans, 0);
        assert!(t.iter().all(|t| t.nitro == ATTRACT_NITRO));
        assert_eq!(s.inhibit_video, 0x81);
        assert!(d.force_blank);
        assert_eq!(d.masked_layers, 0);
        assert_eq!(s.phase, EntryPhase::ReadyForRace);
    }
}
#[test]
fn joined_second_page_counts_even_if_first_zero_and_combines_shifted_bits() {
    let mut s = session();
    s.joined_pages = [0, 2];
    let mut t = trucks();
    finish_driver_selection(&mut s, &mut t);
    assert_eq!(s.active_humans, 8);
    assert_eq!(s.order_slot, 0);
    assert_eq!(t[3].nitro.encoded(), 0x42);
    for truck in &t[..3] {
        assert_eq!(truck.nitro.encoded(), AI_INITIAL_NITRO.encoded());
    }
    assert_eq!(s.phase, EntryPhase::ReadyForRace);
}
#[test]
fn joined_mask_combine_is_byte_shift_or_not_a_global_low_four_mask() {
    let mut s = session();
    s.joined_pages = [0x81, 0x42];
    let mut t = trucks();
    finish_driver_selection(&mut s, &mut t);
    assert_eq!(s.active_humans, 0x89);
    assert_eq!(t[0].nitro.encoded(), 0x42);
    assert_eq!(t[3].nitro.encoded(), 0x42);
    assert_eq!(t[1].nitro.encoded(), AI_INITIAL_NITRO.encoded());
}
#[test]
fn nobody_joined_restarts_entry_without_ai_stock_or_order_reset() {
    let mut s = session();
    s.joined_pages = [0, 0];
    let mut t = trucks();
    let before = t.clone();
    finish_driver_selection(&mut s, &mut t);
    assert_eq!(t, before);
    assert_eq!(s.order_slot, 19);
    assert_eq!(s.phase, EntryPhase::RestartEntry);
}
#[test]
fn poll_each_of_four_starts_reenters_on_first_signature_byte_and_preserves_timer() {
    for slot in 0..4 {
        for first in [1, 0x12, 0xff] {
            let mut timer = 7;
            assert_eq!(
                poll_start(&mut timer, first, automatic(slot)),
                Poll::StartReentry
            );
            assert_eq!(timer, 7);
        }
    }
}
#[test]
fn cold_start_cannot_skip_splash_and_still_decrements_timer() {
    for slot in 0..4 {
        let mut timer = 1;
        assert_eq!(poll_start(&mut timer, 0, automatic(slot)), Poll::Continue);
        assert_eq!(timer, 0);
        assert_eq!(poll_start(&mut timer, 0, automatic(slot)), Poll::TimedOut);
        assert_eq!(timer, 0xffff);
    }
}
#[test]
fn unrelated_buttons_do_not_start_and_timer_negative_comes_from_word_decrement() {
    let mut timer = 0x8000;
    let inputs = [ControllerWord(0xefff); 4];
    assert_eq!(poll_start(&mut timer, 0x12, inputs), Poll::Continue);
    assert_eq!(timer, 0x7fff);
    let mut timer = 0x8001;
    assert_eq!(
        poll_start(&mut timer, 0x12, [ControllerWord(0); 4]),
        Poll::TimedOut
    );
    assert_eq!(timer, 0x8000);
}
#[test]
fn timer140_and110_waits_are_inclusive141_and111_nmi_polls() {
    for initial in [LICENSE_WAIT, SOFTWARE_WAIT] {
        let mut timer = initial;
        for _ in 0..initial {
            assert_eq!(
                poll_start(&mut timer, 0, [ControllerWord(0); 4]),
                Poll::Continue
            );
        }
        assert_eq!(timer, 0);
        assert_eq!(
            poll_start(&mut timer, 0, [ControllerWord(0); 4]),
            Poll::TimedOut
        );
        assert_eq!(timer, 0xffff);
    }
}
#[test]
fn licensed_graphics_upload_uses_two_spans_and_partial_palette_preserving_other_slots() {
    let a = assets();
    let mut m = memory();
    let old = m.clone();
    let mut inhibit = 0xff;
    install_splash_graphics(&a, &mut m, &mut inhibit).unwrap();
    assert_eq!(inhibit, 0);
    assert_eq!(&m.vram[..0x2580], &a.frontend_tiles);
    assert_eq!(&m.vram[0x2580..0x2ec0], &a.frontend_tail);
    assert_eq!(&m.vram[0x2ec0..0x8000], &old.vram[0x2ec0..0x8000]);
    assert_eq!(m.palette[0], 0);
    assert!(m.palette[1..16].iter().all(|&v| v == 0x6abc));
    assert_eq!(&m.palette[16..128], &a.frontend_palette);
    assert_eq!(&m.palette[128..], &old.palette[128..]);
    assert_eq!(m.staged_oam, old.staged_oam);
    assert_eq!(&m.vram[0x8000..0x8802], vec![0; 0x802]);
}
#[test]
fn attract_upload_copies_full_graphics_map_palette_and_three_inclusive_clears() {
    let a = assets();
    let mut m = memory();
    let old = m.clone();
    install_attract_graphics(&a, &mut m).unwrap();
    assert_eq!(&m.vram[..0x4d00], &a.attract_tiles);
    assert_eq!(m.vram[0x4d00], old.vram[0x4d00]);
    assert_eq!(&m.palette[..128], &a.attract_palette);
    assert_eq!(&m.palette[128..], &old.palette[128..]);
    assert_eq!(&m.vram[0x8000..0x8080], vec![0; 0x80]);
    assert_eq!(&m.vram[0x8080..0x8680], [0x5b, 0x3a].repeat(0x300));
    assert_eq!(&m.vram[0x8680..0x8802], vec![0; 0x182]);
    assert_eq!(&m.vram[0x9000..0xa002], vec![0; 0x1002]);
    assert_eq!(m.vram[0xa002], 0x5a);
    assert_eq!(m.visible_oam, old.visible_oam);
}
#[test]
fn malformed_decoded_resources_reject_transactionally_before_any_upload() {
    let mut a = assets();
    a.frontend_tail.pop();
    let mut m = memory();
    let old = m.clone();
    let mut inhibit = 9;
    assert!(install_splash_graphics(&a, &mut m, &mut inhibit).is_err());
    assert_eq!(m, old);
    assert_eq!(inhibit, 9);
    a = assets();
    a.attract_map.pop();
    assert!(validate_assets(&a).is_err());
}
#[test]
fn begin_splashes_installs_license_map_real_display_and_preserves_signature_until_waits() {
    let mut m = memory();
    let mut d = display();
    let mut inhibit = 4;
    let a = assets();
    let state = begin_splashes(&a, &mut m, &mut d, &mut inhibit).unwrap();
    assert_eq!(state.phase, SplashPhase::FadeInLicense);
    assert_eq!(state.fade.direction, FadeDirection::In);
    assert_eq!(state.fade.enabled_layers, 1);
    assert_eq!(inhibit, 5);
    let pos = usize::from(a.license.destination_word) * 2;
    assert_eq!(&m.vram[pos..pos + 2], &2u16.to_le_bytes());
    assert_eq!(d.main_layers, 1);
    assert_eq!(d.main_window_layers, 1);
    assert!(d.force_blank);
}
#[test]
fn license_wait_underflow_starts_fadeout_without_installing_software_or_signature() {
    let mut state = splash(SplashPhase::WaitLicense, 0);
    let mut sig = [0; 5];
    let mut m = memory();
    let old = m.clone();
    advance_splashes(
        &mut state,
        &mut sig,
        &assets(),
        &mut m,
        &mut display(),
        [ControllerWord(0); 4],
    )
    .unwrap();
    assert_eq!(state.phase, SplashPhase::FadeOutLicense);
    assert_eq!(state.timer, 0xffff);
    assert_eq!(sig, [0; 5]);
    assert_eq!(m, old);
    assert_eq!(state.fade.direction, FadeDirection::Out);
}
#[test]
fn software_wait_underflow_copies_all_signature_bytes_before_last_fade() {
    let mut state = splash(SplashPhase::WaitSoftware, 0);
    let mut sig = [0; 5];
    advance_splashes(
        &mut state,
        &mut sig,
        &assets(),
        &mut memory(),
        &mut display(),
        automatic(3),
    )
    .unwrap();
    assert_eq!(state.phase, SplashPhase::FadeOutSoftware);
    assert_eq!(sig, [0x12, 0x0f, 0x16, 0x16, 0x19]);
    assert_eq!(state.fade.direction, FadeDirection::Out);
}
#[test]
fn warm_start_reentry_interrupts_wait_without_timer_signature_or_map_mutation() {
    for phase in [SplashPhase::WaitLicense, SplashPhase::WaitSoftware] {
        let mut state = splash(phase, 15);
        let mut sig = [0x12, 0, 0, 0, 0];
        let mut m = memory();
        let old = m.clone();
        advance_splashes(
            &mut state,
            &mut sig,
            &assets(),
            &mut m,
            &mut display(),
            automatic(2),
        )
        .unwrap();
        assert_eq!(state.phase, SplashPhase::Reentry);
        assert_eq!(state.timer, 15);
        assert_eq!(sig, [0x12, 0, 0, 0, 0]);
        assert_eq!(m, old);
    }
}
#[test]
fn fade_completion_installs_software_map_and_starts_new_real_fade() {
    let mut state = splash(SplashPhase::FadeOutLicense, 0xffff);
    state.fade.direction = FadeDirection::Out;
    state.fade.complete = true;
    let mut m = memory();
    let mut sig = [0; 5];
    advance_splashes(
        &mut state,
        &mut sig,
        &assets(),
        &mut m,
        &mut display(),
        [ControllerWord(0); 4],
    )
    .unwrap();
    assert_eq!(state.phase, SplashPhase::FadeInSoftware);
    assert_eq!(state.fade.direction, FadeDirection::In);
    let pos = 0x40c3 * 2;
    assert_eq!(&m.vram[pos..pos + 2], &3u16.to_le_bytes());
    assert_eq!(sig, [0; 5]);
}
#[test]
fn fadein_completions_start_distinct_source_wait_timers() {
    for (phase, next, timer) in [
        (
            SplashPhase::FadeInLicense,
            SplashPhase::WaitLicense,
            LICENSE_WAIT,
        ),
        (
            SplashPhase::FadeInSoftware,
            SplashPhase::WaitSoftware,
            SOFTWARE_WAIT,
        ),
    ] {
        let mut state = splash(phase, 0);
        state.fade.complete = true;
        advance_splashes(
            &mut state,
            &mut [0; 5],
            &assets(),
            &mut memory(),
            &mut display(),
            [ControllerWord(0); 4],
        )
        .unwrap();
        assert_eq!(state.phase, next);
        assert_eq!(state.timer, timer);
    }
}
#[test]
fn final_fade_completion_returns_without_erasing_installed_signature() {
    let mut state = splash(SplashPhase::FadeOutSoftware, 0xffff);
    state.fade.direction = FadeDirection::Out;
    state.fade.complete = true;
    let mut sig = assets().signature;
    advance_splashes(
        &mut state,
        &mut sig,
        &assets(),
        &mut memory(),
        &mut display(),
        [ControllerWord(0); 4],
    )
    .unwrap();
    assert_eq!(state.phase, SplashPhase::Done);
    assert_eq!(sig, assets().signature);
}
#[test]
fn highscore_scroll_stores_preincrement_and_caps_at_source_limit() {
    let mut d = display();
    let mut s = Highscores {
        scroll: 0x118,
        timer: 500,
        finished: false,
        reentry: false,
    };
    advance_highscores(&mut s, 0x12, &mut d, [ControllerWord(0); 4]);
    assert_eq!(s.scroll, 0x119);
    assert_eq!(d.layers[2].scroll[1], 0x118);
    assert_eq!(s.timer, 499);
    s.scroll = 0x1d5;
    advance_highscores(&mut s, 0x12, &mut d, [ControllerWord(0); 4]);
    assert_eq!(s.scroll, 0x1d6);
    assert_eq!(d.layers[2].scroll[1], 0x1d5);
    advance_highscores(&mut s, 0x12, &mut d, [ControllerWord(0); 4]);
    assert_eq!(s.scroll, 0x1d6);
    assert_eq!(d.layers[2].scroll[1], 0x1d6);
}
#[test]
fn highscore_timeout_or_start_after_scroll_selects_completion_without_double_poll() {
    for start in [false, true] {
        let mut s = Highscores {
            scroll: 0x118,
            timer: 0,
            finished: false,
            reentry: false,
        };
        let mut d = display();
        advance_highscores(
            &mut s,
            0x12,
            &mut d,
            if start {
                automatic(0)
            } else {
                [ControllerWord(0); 4]
            },
        );
        assert_eq!(s.scroll, 0x119);
        assert_eq!(d.layers[2].scroll[1], 0x118);
        assert_eq!(s.reentry, start);
        assert_eq!(s.finished, !start);
        assert_eq!(s.timer, if start { 0 } else { 0xffff });
    }
}
#[test]
fn highscore_entry_initializes_ram_and_display_to_same_scroll_and500_timer() {
    let mut s = Highscores {
        scroll: 0xffff,
        timer: 0,
        finished: true,
        reentry: true,
    };
    let mut d = display();
    begin_highscores(&mut s, &mut d);
    assert_eq!(
        s,
        Highscores {
            scroll: 0x118,
            timer: 500,
            finished: false,
            reentry: false
        }
    );
    assert_eq!(d.layers[2].scroll[1], 0x118);
}
#[test]
fn local_rom_loads_all_splash_maps_signature_slots_and_exact_consumer_spans() {
    let rom = std::fs::read("../input/game.rom").unwrap();
    let a = load_assets(&rom).unwrap();
    assert_eq!(a.signature, [0x12, 0x0f, 0x16, 0x16, 0x19]);
    assert_eq!(a.attract_slots, [0, 1, 2, 3]);
    assert_eq!(&a.frontend_tiles, &rom[FRONTEND_TILES_ROM]);
    assert_eq!(&a.frontend_tail, &rom[FRONTEND_TAIL_ROM]);
    assert_eq!(&a.attract_tiles, &rom[ATTRACT_TILES_ROM]);
    assert_eq!(a.frontend_palette.len(), 112);
    assert_eq!(a.attract_palette.len(), 128);
    assert_eq!(
        (a.license.rectangle.width_flags, a.license.rectangle.rows),
        (18, 25)
    );
    assert_eq!(
        (a.software.rectangle.width_flags, a.software.rectangle.rows),
        (23, 17)
    );
    assert_eq!(a.license.words.len(), 450);
    assert_eq!(a.software.words.len(), 391);
    assert_eq!(a.license.destination_word, 0x4047);
    assert_eq!(a.software.destination_word, 0x40c3);
    assert_eq!(
        a.license.words[0],
        u16::from_le_bytes(rom[0x2e4e0..0x2e4e2].try_into().unwrap())
    );
}
#[test]
fn asset_loader_rejects_truncated_last_palette_instead_of_silent_partial_default() {
    let rom = std::fs::read("../input/game.rom").unwrap();
    assert!(load_assets(&rom[..ATTRACT_PALETTE_ROM.end - 1]).is_err());
}
#[test]
fn panel_initialization_resets_stock_not_wallets_and_installs_real_map_before_window_waits() {
    let mut s = session();
    let mut t = trucks();
    let mut m = memory();
    let mut d = display();
    let p = begin_panel(&mut s, &mut t, &assets(), &mut m, &mut d).unwrap();
    assert_eq!(p.phase, PanelPhase::Motion);
    assert_eq!(p.motion.setup_waits, 2);
    assert_eq!(p.motion.abort_enabled, 1);
    assert_eq!(p.window.center, [0xe0, 0x80]);
    assert_eq!(p.window.masks, [3, 0, 0]);
    assert_eq!(s.wallets, [[9; 6]; 4]);
    for truck in t {
        assert_eq!(truck.nitro.encoded(), 0);
        assert_eq!(truck.upgrades, Upgrades::default());
    }
    assert_eq!(s.enabled_layers, 5);
    assert_eq!(d.main_layers, 5);
    assert_eq!(d.main_window_layers, 5);
    assert_eq!(&m.vram[..0x4d00], vec![0x56; 0x4d00]);
}
fn overlay_panel() -> Panel {
    use crate::presentation::transitions::{
        CLOSED_EDGE, FrontendKind, FrontendWindow, WINDOW_ROWS, Window,
    };
    Panel {
        phase: PanelPhase::Overlay,
        window: Window {
            center: [0xe0, 0x80],
            extent: 0x400,
            shape: crate::presentation::transitions::ShapeId::Frontend,
            masks: [3, 0, 0],
            selector: 0,
            buffers: [
                vec![CLOSED_EDGE; WINDOW_ROWS],
                vec![CLOSED_EDGE; WINDOW_ROWS],
            ],
            hdma_enabled: true,
        },
        motion: FrontendWindow {
            setup_waits: 0,
            kind: FrontendKind::Scrolling,
            abort_enabled: 1,
            velocity: -2,
            wait_timer: 0,
            complete: true,
            aborted: false,
        },
        timer: 1,
    }
}
#[test]
fn panel_overlay_normalizes_selected_wallet_only_restores_brightness_and_decrements_timer() {
    let mut s = session();
    s.selected_wallet = 2;
    s.wallets[2] = [0, 0, 0, 0, 0, 12];
    let mut p = overlay_panel();
    let mut d = display();
    let shape = crate::presentation::transitions::Shape {
        edges: [[0, 0]; 128],
    };
    advance_panel(&mut p, &mut s, &shape, &mut d, [ControllerWord(0); 4]).unwrap();
    assert_eq!(s.wallets[2], [0, 0, 0, 0, 1, 2]);
    assert_eq!(s.wallets[0], [9; 6]);
    assert_eq!(p.timer, 0);
    assert_eq!(p.phase, PanelPhase::Overlay);
    assert_eq!(d.brightness, 15);
    advance_panel(&mut p, &mut s, &shape, &mut d, [ControllerWord(0); 4]).unwrap();
    assert_eq!(p.timer, 0xffff);
    assert_eq!(p.phase, PanelPhase::Complete);
}
#[test]
fn panel_start_overlay_reentry_preserves_timer_but_still_performs_prior_normalization() {
    let mut s = session();
    s.wallets[3] = [0, 0, 0, 0, 0, 12];
    let mut p = overlay_panel();
    let mut d = display();
    let shape = crate::presentation::transitions::Shape {
        edges: [[0, 0]; 128],
    };
    advance_panel(&mut p, &mut s, &shape, &mut d, automatic(3)).unwrap();
    assert_eq!(p.phase, PanelPhase::Reentry);
    assert_eq!(p.timer, 1);
    assert_eq!(s.wallets[3], [0, 0, 0, 0, 1, 2]);
    assert_eq!(d.brightness, 15);
}
