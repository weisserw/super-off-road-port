//! Source-derived fixed branch vectors; see ../results.trace.md.
use super::*;
use crate::{
    controls::*,
    frontend::attract::EntryPhase,
    presentation::{Layer, OAM_BYTES, PALETTE_COLORS, Rectangle, VRAM_BYTES},
    trucks::{ai::Driver, motion::Upgrades},
};
fn records(value: u8) -> SessionRecords {
    SessionRecords::from_legacy(
        [value; 48],
        [value; 32],
        [value; 60],
        [value; 30],
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
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
fn fixture() -> ResultAssets {
    ResultAssets {
        defaults: records(7),
        awards: [15, 10, 5],
        award_destinations: [0x4aed, 0x4ae3, 0x4af7],
        name_destinations: [0x4b2e, 0x4b24, 0x4b38],
        podium_colors: [0x35, 0x25, 0x45],
        truck_palettes: [
            [1, 2, 3, 4],
            [5, 6, 7, 8],
            [9, 10, 11, 12],
            [13, 14, 15, 16],
        ],
        podium_tiles: vec![0x12; 0x4000],
        podium_palette: vec![0x1234; 256],
        podium_map: vec![0x4567; 0x320],
        postrace_map: vec![0x2345; 0x6e0],
        loss_tiles_a: vec![0x34; 0x2a80],
        loss_tiles_b: vec![0x56; 0x12e0],
        loss_palette: vec![0x3210; 96],
        loss_map: MapImage {
            rectangle: Rectangle {
                width_flags: 32,
                rows: 23,
                palette_remap: [0, 1, 2, 3, 4, 5, 6, 7],
            },
            words: vec![0x2001; 32 * 23],
            destination_word: 0x4020,
        },
        loss_replacements: [vec![0x1111; 48], vec![0x2222; 48], vec![0x3333; 48]],
        game_over_text: vec![2, 0xff],
        continue_text: vec![3, 0xff],
        choices_text: vec![4, 0xff],
        score_heading: vec![5, 0xff],
        score_destinations: [
            0x4a10, 0x49b0, 0x4950, 0x48f0, 0x4890, 0x4a01, 0x49a1, 0x4941, 0x48e1, 0x4881,
        ],
        score_name_destinations: [
            0x4a19, 0x49b9, 0x4959, 0x48f9, 0x4899, 0x4a0a, 0x49aa, 0x494a, 0x48ea, 0x488a,
        ],
    }
}
fn word(m: &VideoMemory, at: u16) -> u16 {
    let i = usize::from(at) * 2;
    u16::from_le_bytes([m.vram[i], m.vram[i + 1]])
}
fn progress() -> Progress {
    Progress {
        drivers: [Driver::default(); 4],
        active_humans: 1,
        animation_frame: 0,
        winner_plus_one: 1,
        banner_state: 0,
    }
}
fn time() -> RaceTimer {
    RaceTimer::from_encoded([0x99, 0x23, 1])
}
fn awards(timer: u16, pending: u8) -> Awards {
    Awards {
        timer,
        new_record_pending: pending,
        phase: AwardsFrame::Waiting,
    }
}
fn ranking_session() -> Session {
    let mut s = session();
    s.wallets = [[0; 6]; 4];
    s.records.scores.set_legacy_digits([0; 60]);
    s.records.scores.set_legacy_names([0; 30]);
    for i in 0..10 {
        s.records.scores.set_legacy_digit(i * 6 + 3, i as u8);
        s.records
            .scores
            .set_legacy_name_slice(i * 3..i * 3 + 3, &[i as u8; 3]);
    }
    s.records.truck_names = [11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22];
    s
}
#[test]
fn loader_reads_all_parallel_defaults_and_exact_podium_loss_consumers() {
    let rom = std::fs::read("../input/game.rom").unwrap();
    let a = load_assets(&rom).unwrap();
    assert_eq!(a.awards, [15, 10, 5]);
    assert_eq!(a.award_destinations, [0x4aed, 0x4ae3, 0x4af7]);
    assert_eq!(a.name_destinations, [0x4b2e, 0x4b24, 0x4b38]);
    assert_eq!(a.podium_colors, [0x35, 0x25, 0x45]);
    assert_eq!(a.defaults.track_names.as_slice(), &rom[0x3688..0x36b8]);
    assert_eq!(a.defaults.track_times.as_slice(), &rom[0x36b8..0x36d8]);
    assert_eq!(
        a.defaults.scores.legacy_digits().as_slice(),
        &rom[0x3815..0x3851]
    );
    assert_eq!(
        a.defaults.scores.legacy_names().as_slice(),
        &rom[0x3851..0x386f]
    );
    assert_eq!(a.defaults.scores.order.as_slice(), &rom[0x386f..0x3879]);
    assert_eq!(a.defaults.truck_names.as_slice(), &rom[0x36d8..0x36e4]);
    assert_eq!(a.podium_tiles, &rom[PODIUM_TILES_ROM]);
    assert_eq!(a.loss_tiles_a, &rom[LOSS_TILES_A_ROM]);
    assert_eq!(a.loss_tiles_b, &rom[LOSS_TILES_B_ROM]);
    assert_eq!(a.podium_palette.len(), 256);
    assert_eq!(a.podium_map.len(), 0x320);
    assert_eq!(a.postrace_map.len(), 0x6e0);
    assert_eq!(a.loss_palette.len(), 96);
    assert_eq!(a.loss_map.words.len(), 736);
    assert_eq!(a.loss_replacements.each_ref().map(Vec::len), [48; 3]);
    assert_eq!(a.loss_map.rectangle.width_flags, 32);
    assert_eq!(a.loss_map.rectangle.rows, 23);
    assert_eq!(a.score_destinations, fixture().score_destinations);
    assert_eq!(a.score_name_destinations, fixture().score_name_destinations);
}
#[test]
fn loader_rejects_truncated_last_loss_tiles_and_invalid_decoded_lengths() {
    assert!(load_assets(&vec![0; LOSS_TILES_B_ROM.end - 1]).is_err());
    let mut a = fixture();
    a.podium_palette.pop();
    assert!(validate_assets(&a).is_err());
    a = fixture();
    a.loss_replacements[2].pop();
    assert!(validate_assets(&a).is_err());
    a = fixture();
    a.defaults.scores.order[9] = 0;
    assert!(validate_assets(&a).is_err());
    a = fixture();
    a.loss_map.words.pop();
    assert!(validate_assets(&a).is_err());
}
#[test]
fn record_defaults_copy_every_parallel_array_without_touching_wallets() {
    let mut s = session();
    let a = fixture();
    let wallets = s.wallets;
    copy_record_defaults(&mut s.records, &a.defaults);
    assert_eq!(s.records, a.defaults);
    assert_eq!(s.wallets, wallets);
}
#[test]
fn original_record_stride_and_named_extension_records_share_no_fixed_total_limit() {
    let mut r = records(0);
    r.track_times[30..32].copy_from_slice(&[2, 0x34]);
    r.track_names[45..48].copy_from_slice(&[5, 6, 7]);
    let mut a = AddedRecords::default();
    assert_eq!(
        read_record(&r, &a, &RecordKey::Original(15)).unwrap(),
        TrackRecord {
            time: [2, 0x34],
            name: [5, 6, 7]
        }
    );
    for i in 0..24 {
        write_record(
            &mut r,
            &mut a,
            &RecordKey::Added(format!("extra-{i}")),
            TrackRecord {
                time: [3, 0x45],
                name: [8, 9, 10],
            },
        )
        .unwrap();
    }
    assert_eq!(a.0.len(), 24);
    assert_eq!(
        read_record(&r, &a, &RecordKey::Added("extra-23".into()))
            .unwrap()
            .name,
        [8, 9, 10]
    );
    assert_eq!(r.track_names[45..48], [5, 6, 7]);
    assert!(read_record(&r, &a, &RecordKey::Original(16)).is_err());
}
#[test]
fn missing_named_record_or_invalid_original_write_is_transactional() {
    let mut r = records(3);
    let old = r.clone();
    let mut a = AddedRecords::default();
    assert!(read_record(&r, &a, &RecordKey::Added("missing".into())).is_err());
    assert!(
        write_record(
            &mut r,
            &mut a,
            &RecordKey::Original(16),
            TrackRecord {
                time: [0; 2],
                name: [0; 3]
            }
        )
        .is_err()
    );
    assert_eq!(r, old);
    assert!(a.0.is_empty());
}
#[test]
fn faster_time_compares_high_then_middle_bytes_and_increments_pending_instead_of_setting() {
    let mut p = 7;
    compare_record(&mut p, time(), [2, 0]);
    assert_eq!(p, 8);
    p = 255;
    compare_record(&mut p, time(), [1, 0x24]);
    assert_eq!(p, 0);
    p = 9;
    compare_record(&mut p, time(), [1, 0x23]);
    assert_eq!(p, 0);
    p = 9;
    compare_record(&mut p, time(), [0, 0x99]);
    assert_eq!(p, 0);
    let t = RaceTimer::from_encoded([0, time().encoded()[1], time().encoded()[2]]);
    p = 3;
    compare_record(&mut p, t, [1, 0x23]);
    assert_eq!(p, 0);
}
#[test]
fn postrace_checkpoint_wrap_is_strict_greater_and_subtracts_only_once() {
    let mut p = progress();
    for (d, c) in p.drivers.iter_mut().zip([3, 4, 5, 255]) {
        d.checkpoint = c;
        d.lap = 5;
    }
    normalize_checkpoints(&mut p, 4);
    assert_eq!(p.drivers.map(|d| d.checkpoint), [3, 4, 1, 251]);
    assert_eq!(p.drivers.map(|d| d.lap), [5; 4]);
}
#[test]
fn continuation_award_uses_every_eighth_order_caps_low_byte_and_preserves_high() {
    for (slot, before, after) in [
        (0, 0xab00, 0xab01),
        (8, 0xab02, 0xab03),
        (16, 0xab03, 0xab03),
        (1, 0xab01, 0xab01),
        (63, 0xab00, 0xab00),
    ] {
        let mut s = session();
        s.order_slot = slot;
        s.continuation_progress = before;
        award_continuation(&mut s);
        assert_eq!(s.continuation_progress, after);
    }
}
#[test]
fn postrace_map_installs_at_word55c8_without_touching_neighbor_bytes() {
    let a = fixture();
    let mut m = memory();
    install_postrace_map(&a, &mut m).unwrap();
    assert_eq!(word(&m, 0x55c8), 0x2345);
    assert_eq!(word(&m, 0x5ca7), 0x2345);
    assert_eq!(word(&m, 0x55c7), 0x5a5a);
    assert_eq!(word(&m, 0x5ca8), 0x5a5a);
}
#[test]
fn podium_installs_full_graphics_palette_map_and_disables_window_staging() {
    let a = fixture();
    let mut m = memory();
    let mut d = display();
    install_podium(&a, &mut m, &mut d).unwrap();
    assert_eq!(&m.vram[..0x4000], a.podium_tiles);
    assert_eq!(m.palette, [0x1234; 256]);
    assert_eq!(word(&m, 0x4040), 0x4567);
    assert_eq!(word(&m, 0x435f), 0x4567);
    assert_eq!(word(&m, 0x4000), 0);
    assert_eq!(word(&m, 0x4360), 0);
    assert_eq!(d.main_window_layers, 0x1f);
}
#[test]
fn podium_palette_patches_top_three_ranked_trucks_masked_to_two_bits() {
    let a = fixture();
    let mut m = memory();
    color_podium(&a, [7, 4, 6, 1], &mut m).unwrap();
    assert_eq!(&m.palette[0x35..0x39], &[13, 14, 15, 16]);
    assert_eq!(&m.palette[0x25..0x29], &[1, 2, 3, 4]);
    assert_eq!(&m.palette[0x45..0x49], &[9, 10, 11, 12]);
    assert_eq!(m.palette[0x39], 0x6abc);
}
#[test]
fn podium_credits_rank_money_now_but_draws_pending_pickup_count_without_consuming() {
    let a = fixture();
    let mut s = ranking_session();
    let mut m = memory();
    let pending = [25, 0, 10, 9];
    draw_awards(&a, &mut s, &pending, [3, 2, 0, 1], &mut m).unwrap();
    assert_eq!(s.wallets.map(|w| w[3]), [5, 0, 10, 15]);
    assert_eq!(pending, [25, 0, 10, 9]);
    assert_eq!(
        [word(&m, 0x4aed), word(&m, 0x4aee), word(&m, 0x4aef)],
        [0x2400, 0x240a, 0x2401]
    );
    assert_eq!([word(&m, 0x4ae3), word(&m, 0x4ae4)], [0x2802, 0x2801]);
    assert_eq!([word(&m, 0x4af7), word(&m, 0x4af8)], [0x2c02, 0x2c10]);
    assert_eq!(
        [word(&m, 0x4b2e), word(&m, 0x4b2f), word(&m, 0x4b30)],
        [0x2814, 0x2815, 0x2816]
    );
    assert_eq!(s.selected_wallet, 3);
}
#[test]
fn time_rows_use_three_name_tiles_and_only_display_selected_bcd_nibbles() {
    let mut s = ranking_session();
    s.records.track_names[6..9].copy_from_slice(&[31, 32, 33]);
    s.records.track_times[4..6].copy_from_slice(&[0xa2, 0x45]);
    let mut m = memory();
    draw_times(
        &s,
        &AddedRecords::default(),
        &RecordKey::Original(2),
        RaceTimer::from_encoded([0x99, 0x23, 0xf1]),
        3,
        &mut m,
    )
    .unwrap();
    assert_eq!(
        [word(&m, 0x4869), word(&m, 0x486a), word(&m, 0x486b)],
        [0x2814, 0x2815, 0x2816]
    );
    assert_eq!(
        [
            word(&m, 0x4872),
            word(&m, 0x4873),
            word(&m, 0x4874),
            word(&m, 0x4875)
        ],
        [0x2802, 0x2803, 0x2827, 0x2804]
    );
    assert_eq!(
        [
            word(&m, 0x4892),
            word(&m, 0x4893),
            word(&m, 0x4894),
            word(&m, 0x4895)
        ],
        [0x2803, 0x2805, 0x2827, 0x2806]
    );
    assert_eq!(word(&m, 0x4889), 0x281f);
}
#[test]
fn pending_record_above250_blanks_thirteen_words_and_skips_controller_dismissal() {
    let mut a = awards(400, 1);
    let mut s = ranking_session();
    let old = s.records.clone();
    let mut m = memory();
    advance_awards(
        &mut a,
        &mut s,
        &mut AddedRecords::default(),
        &RecordKey::Original(0),
        time(),
        [3, 2, 1, 0],
        0,
        [ControllerWord(BUTTON_B); 4],
        &mut m,
    )
    .unwrap();
    assert_eq!(a.timer, 399);
    assert_eq!(a.phase, AwardsFrame::Waiting);
    assert_eq!(s.records, old);
    for at in 0x4869..=0x4875 {
        assert_eq!(word(&m, at), 0x2000);
    }
    assert_eq!(word(&m, 0x4876), 0x5a5a);
}
#[test]
fn pending_record_blink_bit10_redraws_time_still_skipping_input() {
    let mut a = awards(251, 3);
    let mut s = ranking_session();
    let mut m = memory();
    advance_awards(
        &mut a,
        &mut s,
        &mut AddedRecords::default(),
        &RecordKey::Original(0),
        time(),
        [3, 2, 1, 0],
        0x10,
        [ControllerWord(BUTTON_START); 4],
        &mut m,
    )
    .unwrap();
    assert_eq!(a.timer, 250);
    assert_eq!(a.phase, AwardsFrame::Waiting);
    assert_eq!(word(&m, 0x4869), 0x2814);
    assert_eq!(word(&m, 0x4872), 0x2802);
}
#[test]
fn timer250_commits_time_and_ranked_winner_name_before_extra_nmi_input_poll() {
    let mut a = awards(250, 1);
    let mut s = ranking_session();
    let mut m = memory();
    advance_awards(
        &mut a,
        &mut s,
        &mut AddedRecords::default(),
        &RecordKey::Original(15),
        time(),
        [2, 3, 1, 0],
        0,
        [ControllerWord(BUTTON_B); 4],
        &mut m,
    )
    .unwrap();
    assert_eq!(&s.records.track_times[30..32], &[1, 0x23]);
    assert_eq!(&s.records.track_names[45..48], &[17, 18, 19]);
    assert_eq!(a.timer, 250);
    assert_eq!(a.phase, AwardsFrame::CommitWait);
    finish_record_wait(
        &mut a,
        &s,
        &AddedRecords::default(),
        &RecordKey::Original(15),
        time(),
        [2, 3, 1, 0],
        [ControllerWord(BUTTON_B); 4],
        &mut m,
    )
    .unwrap();
    assert_eq!(a.phase, AwardsFrame::Dismissed);
    assert_eq!(a.timer, 250);
    assert_eq!(word(&m, 0x4889), 0x2811);
}
#[test]
fn pending_below250_or_no_pending_polls_any_bit_on_first_two_ports_only() {
    for (pending, timer) in [(1, 249), (0, 400)] {
        let mut a = awards(timer, pending);
        let mut s = ranking_session();
        let mut m = memory();
        advance_awards(
            &mut a,
            &mut s,
            &mut AddedRecords::default(),
            &RecordKey::Original(0),
            time(),
            [0, 1, 2, 3],
            0,
            [
                ControllerWord(0),
                ControllerWord(0),
                ControllerWord(BUTTON_B),
                ControllerWord(BUTTON_START),
            ],
            &mut m,
        )
        .unwrap();
        assert_eq!(a.phase, AwardsFrame::Waiting);
        assert_eq!(a.timer, timer - 1);
        advance_awards(
            &mut a,
            &mut s,
            &mut AddedRecords::default(),
            &RecordKey::Original(0),
            time(),
            [0, 1, 2, 3],
            0,
            [
                ControllerWord(0),
                ControllerWord(BUTTON_R),
                ControllerWord(0),
                ControllerWord(0),
            ],
            &mut m,
        )
        .unwrap();
        assert_eq!(a.phase, AwardsFrame::Dismissed);
        assert_eq!(a.timer, timer - 1);
    }
}
#[test]
fn awards_timeout_underflows_word_and_committed_record_does_not_recommit_below250() {
    let mut a = awards(0, 0);
    let mut s = ranking_session();
    let mut m = memory();
    advance_awards(
        &mut a,
        &mut s,
        &mut AddedRecords::default(),
        &RecordKey::Original(0),
        time(),
        [0, 1, 2, 3],
        0,
        [ControllerWord(0); 4],
        &mut m,
    )
    .unwrap();
    assert_eq!(a.timer, 0xffff);
    assert_eq!(a.phase, AwardsFrame::Dismissed);
}
#[test]
fn pending_pickup_awards_credit_descending_once_byte_wrapped_without_normalization() {
    let mut s = ranking_session();
    s.wallets[2][3] = 250;
    let mut pending = [1, 0, 15, 255];
    settle_pickup_awards(&mut s, &mut pending).unwrap();
    assert_eq!(pending, [0; 4]);
    assert_eq!(s.wallets.map(|w| w[3]), [1, 0, 9, 255]);
    assert_eq!(s.selected_wallet, 0);
    let before = s.wallets;
    settle_pickup_awards(&mut s, &mut pending).unwrap();
    assert_eq!(s.wallets, before);
}
#[test]
fn highscore_order_is_ascending_and_reuses_lowest_storage_with_indirect_shift() {
    let mut s = ranking_session();
    s.selected_wallet = 2;
    s.wallets[2] = [0, 0, 0, 5, 0, 0];
    assert!(insert_selected_score(&mut s).unwrap());
    assert_eq!(s.records.scores.order, [1, 2, 3, 4, 0, 5, 6, 7, 8, 9]);
    assert_eq!(&s.records.scores.legacy_digits()[..6], &[0, 0, 0, 5, 0, 0]);
    assert_eq!(&s.records.scores.legacy_names()[..3], &[17, 18, 19]);
    assert_eq!(&s.records.scores.legacy_digits()[54..], &[0, 0, 0, 9, 0, 0]);
}
#[test]
fn equal_all_scores_does_not_insert_but_equal_highest_can_insert_below_it() {
    let mut s = ranking_session();
    s.selected_wallet = 0;
    s.wallets[0] = [0; 6];
    s.records.scores.set_legacy_digits([0; 60]);
    let old = s.records.clone();
    assert!(!insert_selected_score(&mut s).unwrap());
    assert_eq!(s.records, old);
    s = ranking_session();
    s.selected_wallet = 0;
    s.wallets[0] = [0, 0, 0, 9, 0, 0];
    assert!(insert_selected_score(&mut s).unwrap());
    assert_eq!(s.records.scores.order, [1, 2, 3, 4, 5, 6, 7, 8, 0, 9]);
}
#[test]
fn scores_compare_first_difference_not_sum_and_reject_invalid_indirection_atomically() {
    let mut s = ranking_session();
    s.selected_wallet = 1;
    s.wallets[1] = [1, 0, 0, 0, 0, 0];
    insert_selected_score(&mut s).unwrap();
    assert_eq!(s.records.scores.order, [1, 2, 3, 4, 5, 6, 7, 8, 9, 0]);
    assert_eq!(s.records.scores.legacy_digits()[..6], [1, 0, 0, 0, 0, 0]);
    s.records.scores.order[0] = 10;
    let old = s.clone();
    assert!(insert_selected_score(&mut s).is_err());
    assert_eq!(s, old);
}
#[test]
fn all_scores_normalize_before_descending_insertion_including_nonhuman_trucks() {
    let mut s = ranking_session();
    s.records.scores.set_legacy_digits([0; 60]);
    s.wallets = [[0, 0, 0, 0, 0, 10], [0; 6], [0; 6], [0, 0, 0, 0, 0, 11]];
    s.active_humans = 0;
    insert_all_scores(&mut s).unwrap();
    assert_eq!(s.wallets[0], [0, 0, 0, 0, 1, 0]);
    assert_eq!(s.wallets[3], [0, 0, 0, 0, 1, 1]);
    assert_eq!(s.selected_wallet, 0);
    assert_eq!(s.records.scores.order[9], 0);
    assert_eq!(s.records.scores.legacy_digits()[..6], [0, 0, 0, 0, 1, 1]);
}
#[test]
fn highscore_rows_use_order_indirection_blank_pairs_leading_zero_and_fixed_borders() {
    let a = fixture();
    let mut r = records(0);
    r.scores
        .set_legacy_digit_slice(9 * 6..10 * 6, &[0, 0, 1, 0, 2, 0]);
    r.scores.set_legacy_name_slice(9 * 3..10 * 3, &[0x80, 2, 3]);
    r.scores.order = [9, 1, 2, 3, 4, 5, 6, 7, 8, 0];
    let mut m = memory();
    draw_highscores(&a, &r, &mut m).unwrap();
    let at = a.score_destinations[0];
    assert_eq!(
        [
            word(&m, at),
            word(&m, at + 32),
            word(&m, at + 1),
            word(&m, at + 33)
        ],
        [0x2000, 0x2001, 0x2000, 0x2001]
    );
    assert_eq!(
        [
            word(&m, at + 2),
            word(&m, at + 34),
            word(&m, at + 3),
            word(&m, at + 35)
        ],
        [0x2004, 0x2005, 0x2002, 0x2003]
    );
    assert_eq!(
        [
            word(&m, at + 6),
            word(&m, at + 38),
            word(&m, at + 7),
            word(&m, at + 39)
        ],
        [0x2002, 0x2003, 0x2002, 0x2003]
    );
    let n = a.score_name_destinations[0];
    assert_eq!(
        [
            word(&m, n),
            word(&m, n + 32),
            word(&m, n + 1),
            word(&m, n + 33)
        ],
        [0x2000, 0x2001, 0x2004, 0x2005]
    );
}
#[test]
fn no_losing_human_goes_shop_without_spending_continuation() {
    let mut s = session();
    s.active_humans = 1;
    s.continuation_progress = 0xab02;
    assert_eq!(choose_loss_route(&mut s, 2).unwrap(), LossRoute::Shop);
    assert_eq!(s.continuation_progress, 0xab02);
}
#[test]
fn single_human_decrements_low_progress_and_underflow_toggles_out_without_upgrade_penalty() {
    let mut s = session();
    s.active_humans = 4;
    s.continuation_progress = 0xab00;
    assert_eq!(
        choose_loss_route(&mut s, 2).unwrap(),
        LossRoute::RestartFrontend
    );
    assert_eq!(s.active_humans, 0);
    assert_eq!(s.joined_pages, [0, 0]);
    assert_eq!(s.continuation_progress, 0xab00);
    s.active_humans = 4;
    s.continuation_progress = 0xab02;
    assert_eq!(choose_loss_route(&mut s, 2).unwrap(), LossRoute::Penalty);
    assert_eq!(s.continuation_progress, 0xab01);
}
#[test]
fn multiple_humans_do_not_spend_progress_and_only_low_four_bits_count() {
    let mut s = session();
    s.active_humans = 0x85;
    s.continuation_progress = 0xab00;
    assert_eq!(choose_loss_route(&mut s, 2).unwrap(), LossRoute::Penalty);
    assert_eq!(s.continuation_progress, 0xab00);
}
#[test]
fn penalty_minus_two_signed_byte_clamps_and_resets_same_wallet_to_starter_cash() {
    let mut s = ranking_session();
    let mut t = std::array::from_fn(|_| Truck::default());
    t[2].upgrades = Upgrades {
        speed: 0,
        acceleration: 1,
        steering: 2,
        suspension: 6,
    };
    t[2].nitro = crate::trucks::nitro::NitroStock::from_encoded(0x53);
    s.wallets[2] = [9; 6];
    let others = s.wallets;
    penalize_loser(&mut s, &mut t, 2).unwrap();
    assert_eq!(
        t[2].upgrades,
        Upgrades {
            speed: 0,
            acceleration: 0,
            steering: 0,
            suspension: 4
        }
    );
    assert_eq!(t[2].nitro.encoded(), 0x53);
    assert_eq!(s.wallets[2], [0, 0, 1, 0, 0, 0]);
    assert_eq!(s.wallets[0], others[0]);
    assert_eq!(s.wallets[3], others[3]);
}
#[test]
fn conditional_large_upgrade_bytes_follow_dec_dec_bpl_not_saturating_sub() {
    let mut s = session();
    let mut t = std::array::from_fn(|_| Truck::default());
    t[0].upgrades = Upgrades {
        speed: 0x80,
        acceleration: 0x81,
        steering: 0x82,
        suspension: 0xff,
    };
    penalize_loser(&mut s, &mut t, 0).unwrap();
    assert_eq!(
        t[0].upgrades,
        Upgrades {
            speed: 0x7e,
            acceleration: 0x7f,
            steering: 0,
            suspension: 0
        }
    );
}
#[test]
fn participation_is_xor_preserves_upper_mask_and_splits_second_page_without_masking() {
    let mut s = session();
    s.active_humans = 0x85;
    s.continuation_progress = 0xab03;
    toggle_participation(&mut s, 0).unwrap();
    assert_eq!(s.active_humans, 0x84);
    assert_eq!(s.joined_pages, [0, 0x21]);
    assert_eq!(s.continuation_progress, 0xab00);
    toggle_participation(&mut s, 0).unwrap();
    assert_eq!(s.active_humans, 0x85);
}
#[test]
fn continue_yes_preserves_participants_no_removes_last_and_returns_shop_or_restart() {
    let mut s = session();
    s.active_humans = 5;
    s.continuation_progress = 0xab02;
    assert_eq!(resolve_choice(&mut s, 2, 0).unwrap(), LossRoute::Shop);
    assert_eq!(s.active_humans, 5);
    assert_eq!(s.continuation_progress, 0xab02);
    assert_eq!(resolve_choice(&mut s, 2, 1).unwrap(), LossRoute::Shop);
    assert_eq!(s.active_humans, 1);
    assert_eq!(s.continuation_progress, 0xab00);
    assert_eq!(
        resolve_choice(&mut s, 0, 1).unwrap(),
        LossRoute::RestartFrontend
    );
    assert_eq!(s.active_humans, 0);
}
#[test]
fn loss_graphics_upload_contiguous_spans_partial_palette_and_ranked_replacement() {
    let a = fixture();
    let mut m = memory();
    let mut d = display();
    install_loss(&a, 3, &mut m, &mut d).unwrap();
    assert_eq!(&m.vram[..0x2a80], a.loss_tiles_a);
    assert_eq!(&m.vram[0x2a80..0x3d60], a.loss_tiles_b);
    assert_eq!(m.palette[0x20], 0x3210);
    assert_eq!(&m.palette[0x29..0x59], &vec![0x3333; 48]);
    assert_eq!(m.palette[0x80], 0x6abc);
    assert_eq!(word(&m, 0x4020), 0x2001);
    assert_eq!(word(&m, 0x42ff), 0x2001);
    assert_eq!(word(&m, 0x4300), 0);
}
#[test]
fn zero_loser_keeps_base_loss_palette_and_bad_loser_rejects_without_writes() {
    let a = fixture();
    let mut m = memory();
    let mut d = display();
    install_loss(&a, 0, &mut m, &mut d).unwrap();
    assert_eq!(m.palette[0x29], 0x3210);
    let old = m.clone();
    assert!(install_loss(&a, 4, &mut m, &mut d).is_err());
    assert_eq!(m, old);
}
#[test]
fn continue_held_b_dismisses_selected_loser_without_edge_and_overrides_directions() {
    let mut p = ContinuePrompt {
        choice: 1,
        loser: 3,
        dismissed: false,
    };
    let mut m = memory();
    poll_continue(
        &mut p,
        [
            ControllerWord(0),
            ControllerWord(0),
            ControllerWord(0),
            ControllerWord(BUTTON_B | BUTTON_LEFT),
        ],
        &mut m,
    )
    .unwrap();
    assert!(p.dismissed);
    assert_eq!(p.choice, 1);
}
#[test]
fn continue_left_priority_clamps_choices_and_repositions_nine_cursor_sprites() {
    let mut p = ContinuePrompt {
        choice: 0,
        loser: 2,
        dismissed: false,
    };
    let mut m = memory();
    poll_continue(
        &mut p,
        [
            ControllerWord(0),
            ControllerWord(0),
            ControllerWord(BUTTON_LEFT | BUTTON_RIGHT),
            ControllerWord(0),
        ],
        &mut m,
    )
    .unwrap();
    assert_eq!(p.choice, 0);
    assert_eq!(m.staged_oam[0], 0x69);
    poll_continue(
        &mut p,
        [
            ControllerWord(0),
            ControllerWord(0),
            ControllerWord(BUTTON_RIGHT),
            ControllerWord(0),
        ],
        &mut m,
    )
    .unwrap();
    assert_eq!(p.choice, 1);
    assert_eq!(m.staged_oam[0], 0xa8);
    assert_eq!(m.staged_oam[4], 0xb8);
    poll_continue(
        &mut p,
        [
            ControllerWord(0),
            ControllerWord(0),
            ControllerWord(BUTTON_LEFT),
            ControllerWord(0),
        ],
        &mut m,
    )
    .unwrap();
    assert_eq!(p.choice, 0);
    assert_eq!(m.staged_oam[0], 0x24);
    assert_eq!(m.staged_oam[32], 0x44);
}
#[test]
fn continue_input_ignores_other_trucks_start_and_low_byte_buttons() {
    let mut p = ContinuePrompt {
        choice: 0,
        loser: 1,
        dismissed: false,
    };
    let mut m = memory();
    poll_continue(
        &mut p,
        [
            ControllerWord(BUTTON_B),
            ControllerWord(BUTTON_START | BUTTON_A),
            ControllerWord(BUTTON_B),
            ControllerWord(BUTTON_B),
        ],
        &mut m,
    )
    .unwrap();
    assert!(!p.dismissed);
    assert_eq!(p.choice, 0);
}
#[test]
fn game_over_polls_all_four_high_bytes_after_decrement_and_expires_on301st_wait() {
    let mut t = GAME_OVER_TIMEOUT;
    for _ in 0..300 {
        assert!(!poll_game_over(&mut t, [ControllerWord(0); 4]));
    }
    assert_eq!(t, 0);
    assert!(poll_game_over(&mut t, [ControllerWord(0); 4]));
    assert_eq!(t, 0xffff);
    t = 20;
    assert!(!poll_game_over(&mut t, [ControllerWord(BUTTON_A); 4]));
    assert_eq!(t, 19);
    assert!(poll_game_over(
        &mut t,
        [
            ControllerWord(0),
            ControllerWord(0),
            ControllerWord(0),
            ControllerWord(BUTTON_SELECT)
        ]
    ));
    assert_eq!(t, 18);
}
#[test]
fn initialize_continue_sets_zero_choice_layers_paired_cursor_tiles_and_actual_text() {
    let mut s = session();
    let mut d = display();
    let mut m = memory();
    let a = fixture();
    let mut p = ContinuePrompt {
        choice: 1,
        loser: 2,
        dismissed: true,
    };
    let tiles = [
        0x2020, 0x2022, 0x6020, 0x2024, 0x202c, 0x6024, 0xa020, 0xa022, 0xe020,
    ];
    initialize_continue(&mut p, &mut s, &a, &tiles, &mut m, &mut d).unwrap();
    assert_eq!(p.choice, 0);
    assert!(!p.dismissed);
    assert_eq!(s.inhibit_video, 0);
    assert_eq!(s.enabled_layers, 0x15);
    assert_eq!(m.staged_oam[0], 0x24);
    assert_eq!(m.staged_oam[1], 0xae);
    assert_eq!(word_oam(&m, 2), 0x2020);
    assert_eq!(word_oam(&m, 0x26), 0x2020);
    assert_eq!(word(&m, 0x4acb), 0x2006);
    assert_eq!(word(&m, 0x4b06), 0x2008);
}
fn word_oam(m: &VideoMemory, at: usize) -> u16 {
    u16::from_le_bytes([m.staged_oam[at], m.staged_oam[at + 1]])
}
#[test]
fn postrace_entry_services_hud_time_before_first_wait_without_early_continuation_increment() {
    let mut p = Postrace {
        route: None,
        loss_route: None,
        route_count: 4,
        phase: PostracePhase::Complete,
        rank: [0, 1, 2, 3],
        awards: awards(0, 0),
    };
    let mut s = session();
    s.continuation_progress = 0xab00;
    s.order_slot = 0;
    let mut r = progress();
    r.drivers[0].lap = 5;
    let mut t = RaceTimer::default();
    let mut m = memory();
    let mut d = display();
    begin_postrace(
        &std::array::from_fn(|_| Truck::default()),
        &mut p,
        &mut s,
        &mut r,
        &mut t,
        &mut m,
        &mut d,
    );
    assert_eq!(p.phase, PostracePhase::HudWait);
    assert_eq!(t.encoded()[0], 0x16);
    assert_eq!(s.continuation_progress, 0xab00);
    assert_ne!(m.staged_oam, [0x69; OAM_BYTES]);
}
#[test]
fn postrace_first_nmi_installs_map_and_defers_checkpoint_rank_until_window_done() {
    let mut p = Postrace {
        route: None,
        loss_route: None,
        route_count: 4,
        phase: PostracePhase::HudWait,
        rank: [0, 1, 2, 3],
        awards: awards(0, 0),
    };
    let mut s = session();
    s.continuation_progress = 0xab00;
    s.order_slot = 8;
    let mut r = progress();
    r.drivers[0].checkpoint = 7;
    let mut trucks = std::array::from_fn(|_| Truck::default());
    let mut pending = [1, 2, 3, 4];
    let mut m = memory();
    let mut d = display();
    advance_postrace(
        time(),
        &RecordKey::Original(0),
        &mut AddedRecords::default(),
        &mut p,
        &mut s,
        &mut r,
        &mut trucks,
        &mut pending,
        &fixture(),
        &mut m,
        &mut d,
        false,
    )
    .unwrap();
    assert_eq!(s.continuation_progress, 0xab01);
    assert_eq!(word(&m, 0x55c8), 0x2345);
    assert_eq!(r.drivers[0].checkpoint, 7);
    assert_eq!(p.phase, PostracePhase::Window);
    assert_eq!(s.inhibit_video, 0);
    assert_eq!(pending, [1, 2, 3, 4]);
}
#[test]
fn postrace_podium_setup_mutates_money_graphics_audio_and_starts400_timer() {
    let mut p = Postrace {
        route: None,
        loss_route: None,
        route_count: 4,
        phase: PostracePhase::PodiumSetup,
        rank: [3, 2, 0, 1],
        awards: awards(0, 0),
    };
    let mut s = ranking_session();
    let mut r = progress();
    let mut trucks = std::array::from_fn(|_| Truck::default());
    let mut pending = [3, 0, 2, 1];
    let mut m = memory();
    let mut d = display();
    advance_postrace(
        time(),
        &RecordKey::Original(0),
        &mut AddedRecords::default(),
        &mut p,
        &mut s,
        &mut r,
        &mut trucks,
        &mut pending,
        &fixture(),
        &mut m,
        &mut d,
        true,
    )
    .unwrap();
    assert_eq!(s.wallets.map(|w| w[3]), [5, 0, 10, 15]);
    assert_eq!(pending, [3, 0, 2, 1]);
    assert!(s.audio_commands.contains(&PODIUM_AUDIO));
    assert_eq!(s.enabled_layers, 5);
    assert_eq!(d.color_math_layers, 0);
    assert_eq!(d.layers[2].scroll[1], 0);
    assert_eq!(p.phase, PostracePhase::FadeIn);
}
#[test]
fn postrace_completed_contraction_settles_pickups_once_before_nonhuman_shop_route() {
    let mut p = Postrace {
        route: None,
        loss_route: None,
        route_count: 4,
        phase: PostracePhase::Contract,
        rank: [0, 1, 2, 3],
        awards: awards(0, 0),
    };
    let mut s = ranking_session();
    s.active_humans = 1;
    let mut r = progress();
    let mut trucks = std::array::from_fn(|_| Truck::default());
    let mut pending = [2, 3, 4, 5];
    let mut m = memory();
    let mut d = display();
    advance_postrace(
        time(),
        &RecordKey::Original(0),
        &mut AddedRecords::default(),
        &mut p,
        &mut s,
        &mut r,
        &mut trucks,
        &mut pending,
        &fixture(),
        &mut m,
        &mut d,
        true,
    )
    .unwrap();
    assert_eq!(pending, [0; 4]);
    assert_eq!(s.wallets.map(|w| w[3]), [2, 3, 4, 5]);
    assert_eq!(p.phase, PostracePhase::Shop);
}

#[test]
fn modernization_score_duplicate_ids_keep_shared_storage_and_strict_comparison() {
    let mut s = ranking_session();
    s.records.scores.order = [0; 10];
    s.selected_wallet = 0;
    s.wallets[0] = [0, 0, 0, 0, 0, 1];
    assert!(insert_selected_score(&mut s).unwrap());
    assert_eq!(s.records.scores.order, [0; 10]);
    assert_eq!(&s.records.scores.legacy_digits()[..6], &[0, 0, 0, 0, 0, 1]);
    assert_eq!(&s.records.scores.legacy_names()[..3], &[11, 12, 13]);
    let retained = s.clone();
    assert!(!insert_selected_score(&mut s).unwrap());
    assert_eq!(s, retained);
}

#[test]
fn modernization_score_nondecimal_rows_compare_lexically_without_normalizing_wallet() {
    let mut s = ranking_session();
    s.records
        .scores
        .set_legacy_digit_slice(54..60, &[0, 10, 0, 0, 0, 0]);
    s.selected_wallet = 0;
    s.wallets[0] = [0, 9, 255, 255, 255, 255];
    assert!(insert_selected_score(&mut s).unwrap());
    assert_eq!(s.records.scores.order, [1, 2, 3, 4, 5, 6, 7, 8, 0, 9]);
    assert_eq!(
        &s.records.scores.legacy_digits()[..6],
        &[0, 9, 255, 255, 255, 255]
    );
    assert_eq!(
        &s.records.scores.legacy_digits()[54..],
        &[0, 10, 0, 0, 0, 0]
    );
    assert_eq!(s.wallets[0], [0, 9, 255, 255, 255, 255]);
    let wire = serde_json::to_value(&s.records).unwrap();
    let restored: SessionRecords = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(restored, s.records);
    assert_eq!(serde_json::to_value(restored).unwrap(), wire);
}
