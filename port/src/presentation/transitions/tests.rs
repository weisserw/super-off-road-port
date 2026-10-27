//! Fixed source fixtures; no algorithmic duplicate oracle. See adjacent parent trace.
use super::*;
use crate::controls::BUTTON_START;
use crate::presentation::{Layer, OAM_BYTES, PALETTE_COLORS, Rectangle, VRAM_BYTES};
const NO_INPUT: [ControllerWord; CONTROLLER_COUNT] = [ControllerWord(0); CONTROLLER_COUNT];
fn display() -> Display {
    Display {
        layers: [Layer {
            map_byte_base: 0,
            tile_byte_base: 0,
            wide: false,
            tall: false,
            scroll: [0; 2],
            bits_per_pixel: 4,
        }; 3],
        bg3_high: true,
        object_tile_byte_base: 0,
        main_layers: 31,
        sub_layers: 0,
        main_window_layers: 31,
        sub_window_layers: 0,
        color_math_layers: 0,
        use_subscreen: false,
        half_color: false,
        subtract_color: false,
        fixed_color: 0,
        brightness: FULL_BRIGHTNESS,
        force_blank: false,
        window_edges: vec![CLOSED_EDGE; 256],
        masked_layers: 0,
        window_inverted_layers: 0,
    }
}
fn memory() -> VideoMemory {
    VideoMemory {
        vram: Box::new([0; VRAM_BYTES]),
        palette: [0; PALETTE_COLORS],
        staged_oam: [0; OAM_BYTES],
        visible_oam: [0; OAM_BYTES],
    }
}
fn window() -> Window {
    Window {
        center: [128, 112],
        extent: 256,
        shape: ShapeId::Frontend,
        masks: [0x22; 3],
        selector: 0,
        buffers: [vec![[3, 4]; WINDOW_ROWS], vec![[5, 6]; WINDOW_ROWS]],
        hdma_enabled: false,
    }
}
fn shape() -> Shape {
    Shape {
        edges: [[-16, 16]; SHAPE_ROWS],
    }
}
fn rom() -> Vec<u8> {
    std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap()
}
fn assets() -> TransitionAssets {
    TransitionAssets {
        shapes: [shape(), shape(), shape()],
        centers: [[36, 50]; ORIGINAL_TRACKS],
        destinations: [0x4861; ORIGINAL_TRACKS],
        countdown: (0..COUNTDOWN_MAPS)
            .map(|i| MapImage {
                rectangle: Rectangle {
                    width_flags: 1,
                    rows: 1,
                    palette_remap: [0; 8],
                },
                words: vec![0x2100 + i as u16],
                destination_word: 0,
            })
            .collect(),
        pause: MapImage {
            rectangle: Rectangle {
                width_flags: 1,
                rows: 1,
                palette_remap: [0; 8],
            },
            words: vec![0x2345],
            destination_word: 0,
        },
    }
}
fn layout() -> CountdownLayout {
    CountdownLayout {
        center: [36, 50],
        destination_word: 0x4861,
    }
}
fn transition(kind: RaceTransitionKind, phase: RacePhase) -> RaceTransition {
    RaceTransition {
        kind,
        phase,
        remaining_waits: 0,
        hold_timer: 0,
        countdown: Countdown {
            state: 0,
            delay: READY_DELAY,
        },
        audio_commands: vec![],
        banner_calls: 0,
        animation_calls: 0,
    }
}
fn word(m: &VideoMemory, dest: u16) -> u16 {
    let i = usize::from(dest) * 2;
    u16::from_le_bytes([m.vram[i], m.vram[i + 1]])
}
#[test]
fn original_assets_load_three_exact_shape_windows_not_preceding_unowned_words() {
    let a = load_assets(&rom()).unwrap();
    assert_eq!(a.shapes[0].edges[0], [-34, 63]);
    assert_eq!(a.shapes[1].edges[0], [-7, 7]);
    assert_eq!(a.shapes[2].edges[0], [64, -64]);
    assert_eq!(a.shapes[0].edges[127], [-63, 34]);
    assert_eq!(a.shapes[1].edges[64], [-63, 63]);
    assert_eq!(a.shapes[2].edges[127], [64, -64]);
}
#[test]
fn shape_directory_validates_stride_bank_translation_and_all_slice_bounds() {
    let r = rom();
    let a = decode_shapes(&r[0x3616..0x361f], &r).unwrap();
    assert_eq!(a[1].edges[0], [-7, 7]);
    for short in [0, 8] {
        assert!(decode_shapes(&r[0x3616..0x3616 + short], &r).is_err());
    }
    assert!(decode_shapes(&r[0x3616..0x361f], &r[..0x46599]).is_err());
    let mut bad = r[0x3616..0x361f].to_vec();
    bad[1] = 0;
    bad[2] = 0;
    assert!(decode_shapes(&bad, &r).is_err());
}
#[test]
fn all_sixteen_layouts_decode_paired_center_tables_and_word_destinations() {
    let a = load_assets(&rom()).unwrap();
    let centers = [
        [36, 50],
        [212, 50],
        [212, 170],
        [212, 50],
        [212, 50],
        [36, 170],
        [36, 50],
        [36, 50],
        [36, 50],
        [212, 50],
        [36, 170],
        [36, 170],
        [212, 50],
        [36, 50],
        [212, 50],
        [212, 50],
    ];
    let destinations = [
        0x4861, 0x4877, 0x4a57, 0x4877, 0x4877, 0x4a41, 0x4861, 0x4861, 0x4861, 0x4877, 0x4a41,
        0x4a41, 0x4877, 0x4861, 0x4877, 0x4877,
    ];
    assert_eq!(a.centers, centers);
    assert_eq!(a.destinations, destinations);
    for track in 0..ORIGINAL_TRACKS {
        assert_eq!(
            original_layout(track as u8, &a).unwrap(),
            CountdownLayout {
                center: centers[track],
                destination_word: destinations[track]
            }
        );
    }
    assert!(original_layout(16, &a).is_err());
}
#[test]
fn countdown_loads_thirtyfour_banked_map_consumers_including_repeated_frame_eleven() {
    let r = rom();
    let a = load_assets(&r).unwrap();
    assert_eq!(a.countdown.len(), COUNTDOWN_MAPS);
    for map in &a.countdown {
        assert_eq!(
            (
                map.rectangle.width_flags,
                map.rectangle.rows,
                map.words.len()
            ),
            (0x8007, 7, 49)
        );
    }
    assert_eq!(a.countdown[11].words, a.countdown[12].words);
    assert_eq!(a.countdown[11].words, a.countdown[14].words);
    assert_ne!(a.countdown[14].words, a.countdown[15].words);
    assert_eq!(
        a.countdown[0].words[0],
        u16::from_le_bytes([r[0x3d3c0], r[0x3d3c1]])
    );
    assert_eq!(
        (
            a.pause.rectangle.width_flags,
            a.pause.rectangle.rows,
            a.pause.words.len()
        ),
        (5, 3, 15)
    );
}
#[test]
fn edge_scaling_signed_samples_clamps_asymmetrically_and_uses_middle_product_byte() {
    assert_eq!(scale_edges(128, 256, [-16, 16]), [112, 144]);
    assert_eq!(scale_edges(2, 256, [-16, -16]), [1, 0]);
    assert_eq!(scale_edges(250, 256, [16, 16]), [255, 255]);
    assert_eq!(scale_edges(128, 128, [-3, 3]), [127, 129]);
    // magnitude80 remains signed80 at M7B, MPYM=80, subtraction128-128=0.
    assert_eq!(scale_edges(128, 256, [-128, -128]), [0, 0]);
    // 0400*40=010000, middle byte00: source wraps rather than wide clamp.
    assert_eq!(scale_edges(128, 1024, [64, 64]), [128, 128]);
}
#[test]
fn window_builder_toggles_display_selector_but_writes_opposite_buffer() {
    let mut w = window();
    let old = w.buffers[1].clone();
    build_window(&mut w, &shape()).unwrap();
    assert_eq!(w.selector, 1);
    assert_eq!(w.buffers[1], old);
    assert_eq!(w.buffers[0][112], [112, 144]);
    w.center[0] = 100;
    build_window(&mut w, &shape()).unwrap();
    assert_eq!(w.selector, 0);
    assert_eq!(w.buffers[1][112], [84, 116]);
    assert_eq!(w.buffers[0][112], [112, 144]);
}
#[test]
fn builder_four_row_repeats_overlap_and_last_pair_closes_three_following_rows() {
    let mut w = window();
    build_window(&mut w, &shape()).unwrap();
    // sample127 maps112+63=175; its row survives, next3 are closed.
    assert_eq!(w.buffers[0][48], [112, 144]);
    assert_eq!(w.buffers[0][175], [112, 144]);
    assert_eq!(&w.buffers[0][176..179], &[CLOSED_EDGE; 3]);
    assert_eq!(w.buffers[0][47], CLOSED_EDGE);
}
#[test]
fn zero_extent_collapses_all_rows_to_center_and_final_tail_closes() {
    let mut w = window();
    w.extent = 0;
    build_window(&mut w, &shape()).unwrap();
    assert_eq!(w.buffers[0][112], [128, 128]);
    assert_eq!(&w.buffers[0][113..116], &[CLOSED_EDGE; 3]);
    assert_eq!(w.buffers[0][111], CLOSED_EDGE);
}
#[test]
fn builder_skips_vertical_carry_and_borrow_instead_of_wrapping_rows() {
    let mut w = window();
    w.center = [128, 0];
    build_window(&mut w, &shape()).unwrap();
    assert_eq!(w.buffers[0][0], [112, 144]);
    assert_eq!(w.buffers[0][63], [112, 144]);
    assert_eq!(w.buffers[0][255], CLOSED_EDGE);
    w.selector = 0;
    w.center[1] = 255;
    build_window(&mut w, &shape()).unwrap();
    assert_eq!(w.buffers[0][191], [112, 144]);
    assert_eq!(w.buffers[0][255], [112, 144]);
    assert_eq!(&w.buffers[0][256..259], &[CLOSED_EDGE; 3]);
}
#[test]
fn setup_primes_both_buffers_then_enables_visible_geometry_after_two_waits() {
    let mut w = window();
    let mut d = display();
    let waits = prime_window(&mut w, &shape(), &mut d).unwrap();
    assert_eq!(waits, 2);
    assert_eq!(w.selector, 0);
    assert!(w.hdma_enabled);
    assert_eq!(&w.buffers[0][..256], &w.buffers[1][..256]);
    assert_eq!(w.buffers[0][259], [3, 4]);
    assert_eq!(w.buffers[1][259], [5, 6]);
    assert_eq!(d.window_edges[112], [112, 144]);
    assert_eq!(d.masked_layers, 31);
}
#[test]
fn disable_window_masks_preserves_hdma_geometry_brightness_and_scroll() {
    let mut w = window();
    w.hdma_enabled = true;
    let before = w.buffers.clone();
    let mut d = display();
    d.layers[0].scroll = [17, 19];
    disable_masks(&mut w, &mut d);
    assert_eq!(w.masks, [0x22; 3]);
    assert_eq!(d.masked_layers, 0);
    assert!(w.hdma_enabled);
    assert_eq!(w.buffers, before);
    assert_eq!((d.brightness, d.layers[0].scroll), (15, [17, 19]));
    assert_eq!(d.masked_layers, 0);
}
#[test]
fn counted_wait_zero_wraps_current_index_width_and_nonzero_waits_exact_count() {
    assert_eq!(counted_wait_boundaries(0, CountedWaitWidth::Byte), 256);
    assert_eq!(counted_wait_boundaries(0, CountedWaitWidth::Word), 65536);
    assert_eq!(counted_wait_boundaries(3, CountedWaitWidth::Word), 3);
    assert_eq!(counted_wait_boundaries(0x0101, CountedWaitWidth::Byte), 1);
}
#[test]
fn fade_out_waits_once_then_sixteen_captured_levels_before_forced_blank() {
    let mut f = Fade {
        direction: FadeDirection::Out,
        primed: false,
        complete: false,
        animation_counter: 99,
        enabled_layers: 31,
    };
    let mut d = display();
    advance_fade(&mut f, &mut d);
    assert_eq!(f.animation_counter, 0);
    assert_eq!(d.brightness, 15);
    for expected in (1..=15).rev() {
        advance_fade(&mut f, &mut d);
        assert_eq!(d.brightness, expected);
        assert!(!d.force_blank);
        assert!(!f.complete);
    }
    advance_fade(&mut f, &mut d);
    assert_eq!(d.brightness, 0);
    assert!(d.force_blank && f.complete);
}
#[test]
fn fade_in_installs_both_layer_masks_and_finishes_at_captured_counter_thirty() {
    let mut f = Fade {
        direction: FadeDirection::In,
        primed: false,
        complete: false,
        animation_counter: 99,
        enabled_layers: 19,
    };
    let mut d = display();
    d.force_blank = true;
    d.brightness = 0;
    advance_fade(&mut f, &mut d);
    assert_eq!((d.main_layers, d.main_window_layers), (19, 19));
    assert_eq!(f.animation_counter, 0);
    for call in 0..30 {
        advance_fade(&mut f, &mut d);
        assert_eq!(d.brightness, (call / 2) as u8);
        assert!(!f.complete);
    }
    advance_fade(&mut f, &mut d);
    assert_eq!(d.brightness, 15);
    assert!(!d.force_blank);
    assert!(f.complete);
}
#[test]
fn scrolling_setup_preserves_shape_but_sets_timer_center_and_masks() {
    let mut w = window();
    let mut d = display();
    let s = begin_frontend(FrontendKind::Scrolling, &mut w, &mut d);
    assert_eq!(w.center, [224, 128]);
    assert_eq!(w.extent, 0);
    assert_eq!(w.masks, [3, 0, 0]);
    assert_eq!((s.wait_timer, s.velocity), (100, -2));
    assert_eq!(d.brightness, 15);
}
#[test]
fn expanding_and_contracting_initialization_choose_source_masks_and_centers() {
    for (kind, center, extent, masks) in [
        (FrontendKind::Expanding, [128, 176], 0, [0x22; 3]),
        (FrontendKind::Contracting, [128, 112], 1024, [3, 3, 0]),
    ] {
        let mut w = window();
        let mut d = display();
        let s = begin_frontend(kind, &mut w, &mut d);
        assert_eq!((w.center, w.extent, w.masks), (center, extent, masks));
        assert!(!s.complete);
        assert_eq!(w.shape, ShapeId::Frontend);
    }
}
fn front(kind: FrontendKind) -> FrontendWindow {
    FrontendWindow {
        setup_waits: 0,
        kind,
        abort_enabled: 0,
        velocity: -2,
        wait_timer: 100,
        complete: false,
        aborted: false,
    }
}
#[test]
fn scrolling_uses_old_centers_for_byte_scroll_then_builds_old_extent_and_moves() {
    let mut w = window();
    w.center = [224, 128];
    w.extent = 16;
    let mut s = front(FrontendKind::Scrolling);
    let mut d = display();
    advance_frontend(&mut s, &mut w, &shape(), &mut d, NO_INPUT).unwrap();
    assert_eq!(d.layers[0].scroll, [164, 8]);
    assert_eq!((w.center, w.extent), ([223, 126], 17));
    assert_eq!(w.buffers[0][128], [223, 225]);
}
#[test]
fn scrolling_bounce_occurs_only_on_exact_byte_centers_and_wraps_horizontal() {
    let mut w = window();
    w.center = [0, 66];
    let mut s = front(FrontendKind::Scrolling);
    let mut d = display();
    advance_frontend(&mut s, &mut w, &shape(), &mut d, NO_INPUT).unwrap();
    assert_eq!(w.center, [255, 64]);
    assert_eq!(s.velocity, 2);
    w.center[1] = 158;
    advance_frontend(&mut s, &mut w, &shape(), &mut d, NO_INPUT).unwrap();
    assert_eq!(w.center[1], 160);
    assert_eq!(s.velocity, -2);
}
#[test]
fn expanding_bounces_x_not_y_and_grows_by_divide_sixteen_minimum_one() {
    let mut w = window();
    w.center = [66, 0];
    w.extent = 0;
    let mut s = front(FrontendKind::Expanding);
    let mut d = display();
    advance_frontend(&mut s, &mut w, &shape(), &mut d, NO_INPUT).unwrap();
    assert_eq!((w.center, w.extent, s.velocity), ([64, 255], 1, 2));
    w.extent = 256;
    advance_frontend(&mut s, &mut w, &shape(), &mut d, NO_INPUT).unwrap();
    assert_eq!(w.extent, 272);
}
#[test]
fn frontend_contraction_divides_sixteen_and_commits_negative_underflow_before_blank() {
    let mut w = window();
    let mut s = front(FrontendKind::Contracting);
    let mut d = display();
    advance_frontend(&mut s, &mut w, &shape(), &mut d, NO_INPUT).unwrap();
    assert_eq!(w.extent, 240);
    w.extent = 0;
    advance_frontend(&mut s, &mut w, &shape(), &mut d, NO_INPUT).unwrap();
    assert_eq!(w.extent, 65535);
    assert!(s.complete && d.force_blank);
    assert_eq!(w.masks, [0x22; 3]);
    assert_eq!(d.masked_layers, 0);
}
#[test]
fn normal_expansion_cleans_masks_at_extent_high_four_but_scrolling_does_not_blank() {
    for kind in [FrontendKind::Scrolling, FrontendKind::Expanding] {
        let mut w = window();
        w.extent = 1000;
        let mut s = front(kind);
        let mut d = display();
        advance_frontend(&mut s, &mut w, &shape(), &mut d, NO_INPUT).unwrap();
        assert!(s.complete);
        assert_eq!(w.extent, 1062);
        assert_eq!(w.masks, [0x22; 3]);
        assert_eq!(d.masked_layers, 0);
        assert_eq!(d.force_blank, kind == FrontendKind::Expanding);
    }
}
#[test]
fn all_frontend_start_aborts_return_without_cleanup_or_growth() {
    for kind in [
        FrontendKind::Scrolling,
        FrontendKind::Expanding,
        FrontendKind::Contracting,
    ] {
        let mut w = window();
        w.hdma_enabled = true;
        let before = (w.center, w.extent, w.masks);
        let mut s = front(kind);
        s.abort_enabled = 1;
        let mut d = display();
        let mut inputs = NO_INPUT;
        inputs[1] = ControllerWord(BUTTON_START);
        advance_frontend(&mut s, &mut w, &shape(), &mut d, inputs).unwrap();
        assert!(s.complete && s.aborted);
        assert_eq!((w.center, w.extent, w.masks), before);
        assert!(w.hdma_enabled);
        assert!(!d.force_blank);
    }
}
#[test]
fn disabled_abort_or_only_third_controller_start_does_not_exit() {
    for (enabled, slot) in [(0, 0), (1, 2), (1, 3)] {
        let mut w = window();
        let mut s = front(FrontendKind::Expanding);
        s.abort_enabled = enabled;
        let mut d = display();
        let mut input = NO_INPUT;
        input[slot] = ControllerWord(BUTTON_START);
        advance_frontend(&mut s, &mut w, &shape(), &mut d, input).unwrap();
        assert!(!s.complete);
        assert_eq!(w.extent, 272);
    }
}
#[test]
fn countdown_initial_six_is_seven_draws_then_each_reload_five_six_draws() {
    let mut c = Countdown { state: 0, delay: 6 };
    let a = assets();
    let mut m = memory();
    for remaining in (0..6).rev() {
        draw_countdown(&mut c, &layout(), &a, &mut m).unwrap();
        assert_eq!((c.state, c.delay), (0, remaining));
        assert_eq!(word(&m, 0x4861), 0x2100);
    }
    draw_countdown(&mut c, &layout(), &a, &mut m).unwrap();
    assert_eq!((c.state, c.delay), (1, 5));
    assert_eq!(word(&m, 0x4861), 0x2100);
    for _ in 0..6 {
        draw_countdown(&mut c, &layout(), &a, &mut m).unwrap();
    }
    assert_eq!((c.state, c.delay), (2, 5));
    assert_eq!(word(&m, 0x4861), 0x2101);
}
#[test]
fn countdown_delay_uses_signed_word_after_decrement_and_invalid_map_is_atomic() {
    let a = assets();
    let mut m = memory();
    let mut c = Countdown {
        state: 2,
        delay: 0x8000,
    };
    draw_countdown(&mut c, &layout(), &a, &mut m).unwrap();
    assert_eq!((c.state, c.delay), (2, 0x7fff));
    c.state = COUNTDOWN_MAPS as u16;
    let old = (c, m.clone());
    assert!(draw_countdown(&mut c, &layout(), &a, &mut m).is_err());
    assert_eq!((c, m), old);
}
#[test]
fn added_named_layouts_do_not_require_original_track_index() {
    let l = CountdownLayout {
        center: [91, 139],
        destination_word: 0x4900,
    };
    validate_layout(&l).unwrap();
    let mut c = Countdown::default();
    let mut m = memory();
    draw_countdown(&mut c, &l, &assets(), &mut m).unwrap();
    assert_eq!(word(&m, 0x4900), 0x2100);
}
#[test]
fn ready_initialization_uses_word_human_gate_track_center_and_six_delay() {
    for gate in [0, 1, 0x0100] {
        let mut s = transition(RaceTransitionKind::Ready, RacePhase::BeginWait);
        let mut w = window();
        let mut d = display();
        let mut inhibit = 7;
        begin_race_transition(
            RaceTransitionKind::Ready,
            gate,
            &mut s,
            &mut w,
            layout(),
            &mut d,
            &mut inhibit,
        );
        assert_eq!(
            (w.center, w.extent, w.shape, w.masks),
            ([36, 50], 0, ShapeId::Race, [0x22, 0, 0x22])
        );
        assert_eq!(s.countdown, Countdown { state: 0, delay: 6 });
        assert_eq!(
            s.audio_commands,
            if gate == 0 { vec![] } else { vec![READY_AUDIO] }
        );
        assert_eq!(inhibit, 7);
        assert_eq!((d.color_math_layers, d.half_color), (0x13, true));
    }
}
#[test]
fn postrace_initialization_always_audio_eighteen_state_and_clears_inhibit() {
    let mut s = transition(RaceTransitionKind::Postrace, RacePhase::BeginWait);
    let mut w = window();
    let mut d = display();
    let mut inhibit = 7;
    begin_race_transition(
        RaceTransitionKind::Postrace,
        0,
        &mut s,
        &mut w,
        layout(),
        &mut d,
        &mut inhibit,
    );
    assert_eq!(
        s.countdown,
        Countdown {
            state: 18,
            delay: 6
        }
    );
    assert_eq!(s.audio_commands, vec![POSTRACE_AUDIO]);
    assert_eq!(inhibit, 0);
}
fn step(s: &mut RaceTransition, w: &mut Window, m: &mut VideoMemory, d: &mut Display, i: &mut u8) {
    advance_race_transition(s, w, layout(), &assets(), m, d, i).unwrap();
}
#[test]
fn ready_delay_waits_twenty_six_boundaries_before_priming_and_does_not_draw() {
    let mut s = transition(RaceTransitionKind::Ready, RacePhase::Delay);
    s.remaining_waits = 26;
    let mut w = window();
    let mut m = memory();
    let old = m.clone();
    let mut d = display();
    let mut i = 1;
    for _ in 0..25 {
        step(&mut s, &mut w, &mut m, &mut d, &mut i);
        assert_eq!(s.phase, RacePhase::Delay);
    }
    step(&mut s, &mut w, &mut m, &mut d, &mut i);
    assert_eq!(s.phase, RacePhase::PrimeFirst);
    assert_eq!(m, old);
    assert_eq!(s.countdown.delay, 6);
}
#[test]
fn ready_growth_only_builds_shape_does_not_advance_countdown_or_banner() {
    let mut s = transition(RaceTransitionKind::Ready, RacePhase::Grow);
    let mut w = window();
    w.extent = 16;
    let mut m = memory();
    let old = m.clone();
    let mut d = display();
    let mut i = 1;
    step(&mut s, &mut w, &mut m, &mut d, &mut i);
    assert_eq!(w.extent, 17);
    assert_eq!(s.countdown.delay, 6);
    assert_eq!(s.banner_calls, 0);
    assert_eq!(m, old);
    w.extent = 112;
    step(&mut s, &mut w, &mut m, &mut d, &mut i);
    assert_eq!(s.phase, RacePhase::CountToReady);
    assert_eq!(w.extent, 112);
}
#[test]
fn ready_countdown_threshold_seventeen_enters_closure_and_draws_real_map_first() {
    let mut s = transition(RaceTransitionKind::Ready, RacePhase::CountToReady);
    s.countdown = Countdown {
        state: 16,
        delay: 0,
    };
    let mut w = window();
    let mut m = memory();
    let mut d = display();
    let mut i = 1;
    step(&mut s, &mut w, &mut m, &mut d, &mut i);
    assert_eq!(s.phase, RacePhase::Close);
    assert_eq!(
        s.countdown,
        Countdown {
            state: 17,
            delay: 5
        }
    );
    assert_eq!(s.banner_calls, 1);
    assert_eq!(word(&m, 0x4861), 0x2110);
}
#[test]
fn race_closure_divides_four_and_negative_step_does_not_store_underflow() {
    let mut s = transition(RaceTransitionKind::Ready, RacePhase::Close);
    s.countdown.state = 18;
    let mut w = window();
    w.extent = 112;
    let mut m = memory();
    let mut d = display();
    let mut i = 1;
    step(&mut s, &mut w, &mut m, &mut d, &mut i);
    assert_eq!(w.extent, 84);
    assert_eq!(s.banner_calls, 0);
    w.extent = 0;
    step(&mut s, &mut w, &mut m, &mut d, &mut i);
    assert_eq!(w.extent, 0);
    assert_eq!(s.phase, RacePhase::ReleaseWait);
    assert_eq!(i, 1);
}
#[test]
fn ready_closure_draws_only_while_state_less_than_eighteen() {
    for state in [17, 18] {
        let mut s = transition(RaceTransitionKind::Ready, RacePhase::Close);
        s.countdown = Countdown { state, delay: 5 };
        let mut w = window();
        let mut m = memory();
        let mut d = display();
        let mut i = 1;
        step(&mut s, &mut w, &mut m, &mut d, &mut i);
        assert_eq!(s.banner_calls, if state == 17 { 1 } else { 0 });
        assert_eq!(s.countdown.delay, if state == 17 { 4 } else { 5 });
    }
}
#[test]
fn final_release_uses_two_waits_ready_clears_inhibit_postrace_preserves_it() {
    for kind in [RaceTransitionKind::Ready, RaceTransitionKind::Postrace] {
        let mut s = transition(kind, RacePhase::ReleaseWait);
        let mut w = window();
        w.hdma_enabled = true;
        let mut m = memory();
        let mut d = display();
        let mut i = 7;
        step(&mut s, &mut w, &mut m, &mut d, &mut i);
        assert_eq!(s.phase, RacePhase::FinalWait);
        assert!(!w.hdma_enabled);
        assert_eq!(
            i,
            if kind == RaceTransitionKind::Ready {
                0
            } else {
                7
            }
        );
        step(&mut s, &mut w, &mut m, &mut d, &mut i);
        assert_eq!(s.phase, RacePhase::Complete);
    }
}
#[test]
fn postrace_growth_draws_animations_and_wraps_thirtyfour_to_eighteen() {
    let mut s = transition(RaceTransitionKind::Postrace, RacePhase::Grow);
    s.countdown = Countdown {
        state: 33,
        delay: 0,
    };
    let mut w = window();
    w.extent = 16;
    let mut m = memory();
    let mut d = display();
    let mut i = 0;
    step(&mut s, &mut w, &mut m, &mut d, &mut i);
    assert_eq!((s.countdown.state, w.extent), (18, 17));
    assert_eq!((s.banner_calls, s.animation_calls), (1, 1));
    assert_eq!(word(&m, 0x4861), 0x2121);
    w.extent = 112;
    step(&mut s, &mut w, &mut m, &mut d, &mut i);
    assert_eq!(s.phase, RacePhase::Hold);
    assert_eq!(s.hold_timer, 256);
}
#[test]
fn postrace_hold_decrements_word_timer_and_then_closure_stops_sprite_animation() {
    let mut s = transition(RaceTransitionKind::Postrace, RacePhase::Hold);
    s.hold_timer = 1;
    s.countdown = Countdown {
        state: 33,
        delay: 0,
    };
    let mut w = window();
    w.extent = 112;
    let mut m = memory();
    let mut d = display();
    let mut i = 0;
    step(&mut s, &mut w, &mut m, &mut d, &mut i);
    assert_eq!((s.hold_timer, s.countdown.state), (0, 18));
    assert_eq!(s.phase, RacePhase::Close);
    assert_eq!(s.animation_calls, 1);
    step(&mut s, &mut w, &mut m, &mut d, &mut i);
    assert_eq!(s.animation_calls, 1);
    assert_eq!(s.banner_calls, 2);
    assert_eq!(w.extent, 84);
}
#[test]
fn pause_candidates_prioritize_first_joined_pressed_and_fourth_reads_third() {
    let mut p = Pause {
        owner_plus_one: 0,
        phase: PausePhase::Running,
    };
    let input = [ControllerWord(BUTTON_START); 4];
    poll_pause(&mut p, [3, 3], input).unwrap();
    assert_eq!((p.owner_plus_one, p.phase), (1, PausePhase::Expand));
    let mut p = Pause {
        owner_plus_one: 0,
        phase: PausePhase::Running,
    };
    let mut input = NO_INPUT;
    input[2] = ControllerWord(BUTTON_START);
    poll_pause(&mut p, [0, 2], input).unwrap();
    assert_eq!(p.owner_plus_one, 4);
    let mut p = Pause {
        owner_plus_one: 0,
        phase: PausePhase::Running,
    };
    let mut input = NO_INPUT;
    input[3] = ControllerWord(BUTTON_START);
    poll_pause(&mut p, [0, 2], input).unwrap();
    assert_eq!(p.owner_plus_one, 0);
}
#[test]
fn existing_pause_owner_held_returns_running_but_release_can_find_new_owner_same_poll() {
    let mut p = Pause {
        owner_plus_one: 2,
        phase: PausePhase::Running,
    };
    let mut input = NO_INPUT;
    input[1] = ControllerWord(BUTTON_START);
    poll_pause(&mut p, [3, 0], input).unwrap();
    assert_eq!((p.owner_plus_one, p.phase), (2, PausePhase::Running));
    input[1] = ControllerWord(0);
    input[0] = ControllerWord(BUTTON_START);
    poll_pause(&mut p, [3, 0], input).unwrap();
    assert_eq!((p.owner_plus_one, p.phase), (1, PausePhase::Expand));
}
#[test]
fn pause_owner_four_waits_actual_fourth_device_not_triggering_third() {
    let mut p = Pause {
        owner_plus_one: 4,
        phase: PausePhase::WaitRelease,
    };
    let mut input = NO_INPUT;
    input[2] = ControllerWord(BUTTON_START);
    poll_pause(&mut p, [0, 2], input).unwrap();
    assert_eq!(p.phase, PausePhase::WaitPress);
    poll_pause(&mut p, [0, 2], input).unwrap();
    assert_eq!(p.phase, PausePhase::WaitPress);
    input[3] = ControllerWord(BUTTON_START);
    poll_pause(&mut p, [0, 2], input).unwrap();
    assert_eq!(p.phase, PausePhase::ReleaseWait);
    assert_eq!(p.owner_plus_one, 4);
}
#[test]
fn pause_requires_release_then_new_press_and_ignores_nonowner_start() {
    let mut p = Pause {
        owner_plus_one: 1,
        phase: PausePhase::WaitRelease,
    };
    let mut input = NO_INPUT;
    input[0] = ControllerWord(BUTTON_START);
    poll_pause(&mut p, [1, 0], input).unwrap();
    assert_eq!(p.phase, PausePhase::WaitRelease);
    input[0] = ControllerWord(0);
    input[1] = ControllerWord(BUTTON_START);
    poll_pause(&mut p, [1, 0], input).unwrap();
    assert_eq!(p.phase, PausePhase::WaitPress);
    poll_pause(&mut p, [1, 0], input).unwrap();
    assert_eq!(p.phase, PausePhase::WaitPress);
    input[0] = ControllerWord(BUTTON_START);
    poll_pause(&mut p, [1, 0], input).unwrap();
    assert_eq!(p.phase, PausePhase::ReleaseWait);
}
#[test]
fn pause_setup_increments_byte_inhibit_and_draws_banner_at_plus_sixtyfive_words() {
    let mut w = window();
    let mut d = display();
    let mut m = memory();
    let a = assets();
    let mut i = 255;
    begin_pause_window(&mut w, layout(), &mut d, &mut m, &a, &mut i).unwrap();
    assert_eq!(i, 0);
    assert_eq!(
        (w.center, w.extent, w.shape, w.masks),
        ([36, 50], 0, ShapeId::Pause, [0x22, 0, 0x22])
    );
    assert_eq!(word(&m, 0x48a2), 0x2345);
    assert_eq!((d.color_math_layers, d.half_color), (0x13, true));
}
#[test]
fn pause_growth_uses_eight_and_tests_threshold_before_add_then_waits_owner() {
    let mut p = Pause {
        owner_plus_one: 1,
        phase: PausePhase::Expand,
    };
    let mut w = window();
    w.extent = 88;
    let mut d = display();
    let mut i = 1;
    advance_pause(
        &mut p,
        &mut w,
        layout(),
        &assets(),
        &mut d,
        &mut i,
        NO_INPUT,
    )
    .unwrap();
    assert_eq!(w.extent, 96);
    assert_eq!(p.phase, PausePhase::Expand);
    advance_pause(
        &mut p,
        &mut w,
        layout(),
        &assets(),
        &mut d,
        &mut i,
        NO_INPUT,
    )
    .unwrap();
    assert_eq!(w.extent, 96);
    assert_eq!(p.phase, PausePhase::WaitPress);
}
#[test]
fn pause_resume_synchronizes_hdma_release_then_second_wait_retains_owner_until_release() {
    let mut p = Pause {
        owner_plus_one: 1,
        phase: PausePhase::ReleaseWait,
    };
    let mut w = window();
    w.hdma_enabled = true;
    let mut d = display();
    let mut i = 7;
    advance_pause(
        &mut p,
        &mut w,
        layout(),
        &assets(),
        &mut d,
        &mut i,
        NO_INPUT,
    )
    .unwrap();
    assert_eq!(i, 0);
    assert!(!w.hdma_enabled);
    assert_eq!(p.phase, PausePhase::FinalWait);
    assert_eq!(p.owner_plus_one, 1);
    advance_pause(
        &mut p,
        &mut w,
        layout(),
        &assets(),
        &mut d,
        &mut i,
        NO_INPUT,
    )
    .unwrap();
    assert_eq!(p.phase, PausePhase::Running);
    assert_eq!(p.owner_plus_one, 1);
}
#[test]
fn invalid_pause_owner_rejects_without_mutation() {
    let mut p = Pause {
        owner_plus_one: 5,
        phase: PausePhase::WaitPress,
    };
    let old = p.clone();
    assert!(poll_pause(&mut p, [3, 3], NO_INPUT).is_err());
    assert_eq!(p, old);
}

#[test]
fn negative_eighty_sample_preserves_signed_multiplier_corner_case() {
    // abs(-128) remains80, signed M7B=-128; 0080*-128=C000, MPYM=C0.
    assert_eq!(scale_edges(128, 128, [-128, -128]), [1, 0]);
}
#[test]
fn ready_growth_can_overshoot_target_without_clamping() {
    let mut s = transition(RaceTransitionKind::Ready, RacePhase::Grow);
    let mut w = window();
    w.extent = 109;
    let mut m = memory();
    let mut d = display();
    let mut i = 1;
    step(&mut s, &mut w, &mut m, &mut d, &mut i);
    assert_eq!(w.extent, 115);
    step(&mut s, &mut w, &mut m, &mut d, &mut i);
    assert_eq!(w.extent, 115);
    assert_eq!(s.phase, RacePhase::CountToReady);
}
#[test]
fn frontend_abort_build_order_differs_expanding_from_scrolling_and_contracting() {
    for kind in [
        FrontendKind::Scrolling,
        FrontendKind::Expanding,
        FrontendKind::Contracting,
    ] {
        let mut w = window();
        let mut s = front(kind);
        s.abort_enabled = 1;
        let mut d = display();
        let mut input = NO_INPUT;
        input[0] = ControllerWord(BUTTON_START);
        advance_frontend(&mut s, &mut w, &shape(), &mut d, input).unwrap();
        assert_eq!(
            w.selector,
            if kind == FrontendKind::Expanding {
                1
            } else {
                0
            }
        );
    }
}
#[test]
fn attract_overlay_normalizes_real_wallet_before_timeout_and_restores_brightness() {
    let mut wallet = [0, 0, 0, 0, 0, 15];
    let mut timer = 0;
    let mut d = display();
    let (outcome, brightness) =
        advance_attract_overlay(&mut wallet, &mut timer, 0, &mut d, NO_INPUT);
    assert_eq!(wallet, [0, 0, 0, 0, 1, 5]);
    assert_eq!(timer, 65535);
    assert_eq!(outcome, OverlayOutcome::TimedOut);
    assert_eq!(brightness, [8, 15]);
    assert_eq!(d.brightness, 15);
}
#[test]
fn overlay_start_checks_all_devices_but_only_signature_first_byte_bypasses_timer() {
    for signature in [0, 0x12] {
        let mut wallet = [0; 6];
        let mut timer = 100;
        let mut d = display();
        let mut input = NO_INPUT;
        input[3] = ControllerWord(BUTTON_START);
        let (outcome, brightness) =
            advance_attract_overlay(&mut wallet, &mut timer, signature, &mut d, input);
        assert_eq!(
            outcome,
            if signature == 0 {
                OverlayOutcome::Continue
            } else {
                OverlayOutcome::StartReentry
            }
        );
        assert_eq!(timer, if signature == 0 { 99 } else { 100 });
        assert_eq!(brightness, [8, 15]);
    }
}

#[test]
fn frontend_setup_has_two_waits_and_no_geometry_motion_until_primed() {
    let mut w = window();
    let mut d = display();
    let mut s = begin_frontend(FrontendKind::Expanding, &mut w, &mut d);
    assert_eq!(s.setup_waits, 2);
    advance_frontend(&mut s, &mut w, &shape(), &mut d, NO_INPUT).unwrap();
    assert_eq!(s.setup_waits, 1);
    assert_eq!(w.extent, 0);
    assert!(!w.hdma_enabled);
    advance_frontend(&mut s, &mut w, &shape(), &mut d, NO_INPUT).unwrap();
    assert_eq!(s.setup_waits, 0);
    assert!(w.hdma_enabled);
    assert_eq!(w.extent, 0);
}
#[test]
fn complete_pause_service_waits_before_drawing_then_primes_before_growth() {
    let mut p = Pause {
        owner_plus_one: 0,
        phase: PausePhase::Running,
    };
    let mut w = window();
    let mut d = display();
    let mut m = memory();
    let a = assets();
    let mut i = 0;
    let mut input = NO_INPUT;
    input[0] = ControllerWord(BUTTON_START);
    service_pause_frame(
        &mut p,
        [1, 0],
        &mut w,
        layout(),
        &a,
        &mut m,
        &mut d,
        &mut i,
        input,
    )
    .unwrap();
    assert_eq!(p.phase, PausePhase::BeginWait);
    assert_eq!(i, 0);
    assert_eq!(word(&m, 0x48a2), 0);
    service_pause_frame(
        &mut p,
        [1, 0],
        &mut w,
        layout(),
        &a,
        &mut m,
        &mut d,
        &mut i,
        input,
    )
    .unwrap();
    assert_eq!(p.phase, PausePhase::PrimeFirst);
    assert_eq!(i, 1);
    assert_eq!(word(&m, 0x48a2), 0x2345);
    assert_eq!(w.extent, 0);
    service_pause_frame(
        &mut p,
        [1, 0],
        &mut w,
        layout(),
        &a,
        &mut m,
        &mut d,
        &mut i,
        input,
    )
    .unwrap();
    assert_eq!(p.phase, PausePhase::PrimeSecond);
    assert!(!w.hdma_enabled);
    service_pause_frame(
        &mut p,
        [1, 0],
        &mut w,
        layout(),
        &a,
        &mut m,
        &mut d,
        &mut i,
        input,
    )
    .unwrap();
    assert_eq!(p.phase, PausePhase::Expand);
    assert!(w.hdma_enabled);
    assert_eq!(w.extent, 0);
    service_pause_frame(
        &mut p,
        [1, 0],
        &mut w,
        layout(),
        &a,
        &mut m,
        &mut d,
        &mut i,
        input,
    )
    .unwrap();
    assert_eq!(w.extent, 8);
}
#[test]
fn malformed_native_scanline_capacity_rejects_before_selector_or_buffer_mutation() {
    let mut w = window();
    w.buffers[1].truncate(255);
    let old = w.clone();
    assert!(build_window(&mut w, &shape()).is_err());
    assert_eq!(w, old);
}

#[test]
fn source03_inverts_window_one_while22_masks_inside_with_fixed_pixel_colors() {
    let mut w = window();
    w.buffers = [vec![[10, 20]; WINDOW_ROWS], vec![[10, 20]; WINDOW_ROWS]];
    let mut d = display();
    d.main_layers = 1;
    d.main_window_layers = 1;
    d.sub_layers = 0;
    d.layers[0].map_byte_base = 0x8000;
    d.layers[0].tile_byte_base = 0;
    let mut m = memory();
    m.palette[0] = 0x7c00;
    m.palette[1] = 0x001f;
    for row in 0..8 {
        m.vram[row * 2] = 0xff;
    }
    w.masks = [3, 0, 0];
    enable_window(&mut w, &mut d);
    assert_eq!(d.window_inverted_layers, 1);
    assert_eq!(
        crate::presentation::pixel(&m, &d, 15, 0).unwrap(),
        [255, 0, 0, 255]
    );
    assert_eq!(
        crate::presentation::pixel(&m, &d, 9, 0).unwrap(),
        [0, 0, 255, 255]
    );
    w.masks = [0x22, 0, 0x22];
    enable_window(&mut w, &mut d);
    assert_eq!(d.window_inverted_layers, 0);
    assert_eq!(
        crate::presentation::pixel(&m, &d, 15, 0).unwrap(),
        [0, 0, 255, 255]
    );
    assert_eq!(
        crate::presentation::pixel(&m, &d, 9, 0).unwrap(),
        [255, 0, 0, 255]
    );
}
