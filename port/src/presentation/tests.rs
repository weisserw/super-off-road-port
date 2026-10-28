//! Fixed synthetic vectors and local ROM expectations; see rendering.trace.md.
use super::memory::*;
use super::sprites::*;
use super::*;
fn memory() -> VideoMemory {
    VideoMemory {
        vram: Box::new([0x5a; VRAM_BYTES]),
        palette: [0; PALETTE_COLORS],
        staged_oam: [0x69; OAM_BYTES],
        visible_oam: [0x96; OAM_BYTES],
    }
}
fn display() -> Display {
    Display {
        layers: [
            Layer {
                map_byte_base: 0x8000,
                tile_byte_base: 0,
                wide: false,
                tall: false,
                scroll: [0, 0],
                bits_per_pixel: 4,
            },
            Layer {
                map_byte_base: 0x8800,
                tile_byte_base: 0,
                wide: false,
                tall: false,
                scroll: [0, 0],
                bits_per_pixel: 4,
            },
            Layer {
                map_byte_base: 0x9000,
                tile_byte_base: 0xa000,
                wide: false,
                tall: false,
                scroll: [0, 0],
                bits_per_pixel: 2,
            },
        ],
        bg3_high: true,
        object_tile_byte_base: 0xc000,
        main_layers: 0x1f,
        sub_layers: 0,
        main_window_layers: 0,
        sub_window_layers: 0,
        color_math_layers: 0,
        use_subscreen: false,
        half_color: false,
        subtract_color: false,
        fixed_color: 0,
        brightness: 15,
        force_blank: false,
        window_edges: vec![[1, 0]; 224],
        masked_layers: 0,
        window_inverted_layers: 0,
    }
}
fn arrays() -> SpriteArrays {
    SpriteArrays {
        order: std::array::from_fn(|i| i as u8),
        x: [0; 16],
        y: [HIDDEN_Y; 16],
        tile_attributes: [0x2345; 16],
        wheel_sample_heights: [0x55; 16],
    }
}
fn rom() -> Vec<u8> {
    std::fs::read("../input/game.rom").unwrap()
}
fn word(m: &VideoMemory, destination_word: usize) -> u16 {
    let a = (destination_word * 2) & 0xffff;
    u16::from_le_bytes([m.vram[a], m.vram[(a + 1) & 0xffff]])
}
fn rectangle(width: u16, rows: u16) -> Rectangle {
    Rectangle {
        width_flags: width,
        rows,
        palette_remap: [0, 1, 2, 3, 4, 5, 6, 7],
    }
}
fn draw_words(m: &mut VideoMemory, destination: u16, words: &[u16]) {
    for (i, w) in words.iter().enumerate() {
        let a = (usize::from(destination) * 2 + i * 2) & 0xffff;
        m.vram[a..a + 2].copy_from_slice(&w.to_le_bytes());
    }
}
#[test]
fn planar_decoding_uses_snes_paired_plane_rows_and_msb_left() {
    let mut bytes = [0; 32];
    bytes[0] = 0x80;
    bytes[1] = 0x40;
    bytes[16] = 0x20;
    bytes[17] = 0x10;
    bytes[14] = 0x01;
    let tiles = decode_planar(&bytes, 4).unwrap();
    assert_eq!(tiles.len(), 1);
    assert_eq!(&tiles[0][..8], &[1, 2, 4, 8, 0, 0, 0, 0]);
    assert_eq!(tiles[0][63], 1);
    let t = decode_planar(&bytes[..16], 2).unwrap();
    assert_eq!(&t[0][..4], &[1, 2, 0, 0]);
}
#[test]
fn planar_and_palette_validation_reject_incomplete_units() {
    assert!(decode_planar(&[0; 31], 4).is_err());
    assert!(decode_planar(&[0; 32], 3).is_err());
    assert!(decode_palette(&[0]).is_err());
}
#[test]
fn palette_decode_masks_unstored_color_bit_and_keeps_native_bgr555() {
    assert_eq!(
        decode_palette(&[0x1f, 0x80, 0xe0, 0x03, 0, 0x7c]).unwrap(),
        vec![0x001f, 0x03e0, 0x7c00]
    );
}
#[test]
fn semantic_loader_uses_full_overlapping_font_cursor_palette_hud_windows() {
    let b = rom();
    let a = load_shared(&b).unwrap();
    assert_eq!(a.font_a, b[0x45440..0x45c40]);
    assert_eq!(a.font_b, b[0x45a20..0x46220]);
    assert_eq!(a.cursor, b[0x45de0..0x461e0]);
    assert_eq!(a.hud, b[0x3b600..0x3d600]);
    assert_eq!(a.sprite_prefix, b[SHARED_ROM_PREFIX]);
    assert_eq!(a.sprite_continuation, b[0x25ac0..0x26b00]);
    assert_eq!(a.race_palette.len(), 256);
    assert_eq!(&a.race_palette[..4], &[0, 0x67bd, 0x4a72, 0x420f]);
    // Full consumer extends beyond the368-byte named palette prefix. Mask CGRAM bit15.
    let tail = u16::from_le_bytes([b[0x20c5], b[0x20c6]]) & 0x7fff;
    assert_eq!(a.race_palette[255], tail);
    assert_eq!(a.cursor_palette.len(), 32);
    assert_eq!(a.text_palette.len(), 16);
    assert_eq!(a.sprite_palette.len(), 128);
    assert!(load_shared(&b[..0x45fff]).is_err());
}
#[test]
fn fill_writes_401_words_including_next_map_start_and_wraps_vram() {
    let mut m = memory();
    fill_map(&mut m, 0x4800, 0x2ddb);
    assert_eq!(word(&m, 0x47ff), 0x5a5a);
    assert_eq!(word(&m, 0x4800), 0x2ddb);
    assert_eq!(word(&m, 0x4c00), 0x2ddb);
    assert_eq!(word(&m, 0x4c01), 0x5a5a);
    fill_map(&mut m, 0x7fff, 0x1234);
    assert_eq!(word(&m, 0x7fff), 0x1234);
    assert_eq!(word(&m, 0), 0x1234);
    assert_eq!(word(&m, 0x3ff), 0x1234);
    assert_eq!(word(&m, 0x400), 0x5a5a);
}
#[test]
fn rectangle_descriptor_is_twenty_little_endian_bytes_and_validates_dimensions() {
    let b = [2, 128, 2, 0, 7, 0, 6, 0, 5, 0, 4, 0, 3, 0, 2, 0, 1, 0, 0, 0];
    assert_eq!(
        decode_rectangle(&b).unwrap(),
        Rectangle {
            width_flags: 0x8002,
            rows: 2,
            palette_remap: [7, 6, 5, 4, 3, 2, 1, 0]
        }
    );
    assert!(decode_rectangle(&b[..19]).is_err());
    let mut bad = b;
    bad[0] = 0;
    bad[1] = 0;
    assert!(decode_rectangle(&bad).is_err());
    bad = b;
    bad[2] = 0;
    assert!(decode_rectangle(&bad).is_err());
}
#[test]
fn rectangle_remaps_palette_only_retaining_priority_flips_tile_and_row_stride() {
    let mut m = memory();
    let mut d = rectangle(2, 2);
    d.palette_remap = [7, 6, 5, 4, 3, 2, 1, 0];
    draw_rectangle(&mut m, &[0xe001, 0x0402, 0x1c03, 0x3004], 0x4000, &d).unwrap();
    assert_eq!(
        [
            word(&m, 0x4000),
            word(&m, 0x4001),
            word(&m, 0x4020),
            word(&m, 0x4021)
        ],
        [0xfc01, 0x1802, 0x0003, 0x2c04]
    );
    assert_eq!(word(&m, 0x4002), 0x5a5a);
    assert_eq!(word(&m, 0x401f), 0x5a5a);
}
#[test]
fn rectangle_width_sign_bypasses_remap_and_native_short_source_is_atomic() {
    let mut m = memory();
    let d = rectangle(0x8002, 1);
    draw_rectangle(&mut m, &[0x7c01, 0xffff], 0x4800, &d).unwrap();
    assert_eq!(word(&m, 0x4800), 0x7c01);
    assert_eq!(word(&m, 0x4801), 0xffff);
    let old = m.clone();
    assert!(draw_rectangle(&mut m, &[1], 0x4800, &d).is_err());
    assert_eq!(m, old);
}
#[test]
fn rom_rectangle_source_low_word_wrap_retains_bank_and_crossing_byte_read() {
    let mut b = rom();
    b[0x7ffff] = 0x11;
    b[0] = 0x22;
    // 0F:FFFF first low byte ROM7FFFF, high byte10:0000 is mirrored WRAM,
    // so unsupported ROM-only reads must reject rather than concatenate file bytes.
    let mut m = memory();
    assert!(draw_rom_rectangle(&mut m, &b, 0x0fffff, 0x4000, &rectangle(2, 1)).is_err());
    assert_eq!(word(&m, 0x4000), 0x5a5a);
}
#[test]
fn single_text_stops_at_ff_and_keeps_separator_unconsumed() {
    let mut m = memory();
    assert_eq!(
        draw_text(&mut m, &[0x80, 2, 0xff, 7], 0x4800, TextSize::Single).unwrap(),
        2
    );
    assert_eq!(
        [word(&m, 0x4800), word(&m, 0x4801), word(&m, 0x4802)],
        [0x2080, 0x2002, 0x5a5a]
    );
    assert_eq!(
        draw_text(&mut m, &[0xff], 0x4800, TextSize::Single).unwrap(),
        0
    );
}
#[test]
fn paired_text_doubles_in_byte_width_and_places_vertical_second_tiles() {
    let mut m = memory();
    draw_text(&mut m, &[0x7f, 0x80, 0xff], 0x4800, TextSize::Paired).unwrap();
    assert_eq!(
        [
            word(&m, 0x4800),
            word(&m, 0x4820),
            word(&m, 0x4801),
            word(&m, 0x4821)
        ],
        [0x20fe, 0x20ff, 0x2000, 0x2001]
    );
    assert_eq!(word(&m, 0x4802), 0x5a5a);
}
#[test]
fn unterminated_text_is_rejected_without_partial_video_mutation() {
    let mut m = memory();
    let old = m.clone();
    assert!(draw_text(&mut m, &[1, 2], 0x4800, TextSize::Single).is_err());
    assert_eq!(m, old);
    assert!(draw_seven_rows(&mut m, &[0xff; 6], 0x4800).is_err());
    assert_eq!(m, old);
}
#[test]
fn seven_large_rows_consume_separators_and_advance_40_words() {
    let mut m = memory();
    let bytes = [
        1, 0xff, 2, 0xff, 3, 0xff, 4, 0xff, 5, 0xff, 6, 0xff, 7, 0xff, 99,
    ];
    assert_eq!(draw_seven_rows(&mut m, &bytes, 0x4800).unwrap(), 14);
    assert_eq!(
        [
            word(&m, 0x4800),
            word(&m, 0x4820),
            word(&m, 0x4840),
            word(&m, 0x4980),
            word(&m, 0x49a0)
        ],
        [0x2002, 0x2003, 0x2004, 0x200e, 0x200f]
    );
    assert_eq!(word(&m, 0x49c0), 0x5a5a);
}
#[test]
fn name_panel_clear_is_exact_eleven_by_fourteen_with_32_word_stride() {
    let mut m = memory();
    clear_name_panel(&mut m, 0x48a5);
    assert_eq!(word(&m, 0x48a5), 0x2000);
    assert_eq!(word(&m, 0x48af), 0x2000);
    assert_eq!(word(&m, 0x4a4f), 0x2000);
    assert_eq!(word(&m, 0x48b0), 0x5a5a);
    assert_eq!(word(&m, 0x4a65), 0x5a5a);
}
#[test]
fn frontend_fonts_concatenate_both_800_spans_or_only_font_b_then_clear() {
    let a = load_shared(&rom()).unwrap();
    let mut m = memory();
    upload_fonts(&mut m, &a, true).unwrap();
    assert_eq!(&m.vram[0xa000..0xa800], &a.font_a);
    assert_eq!(&m.vram[0xa800..0xb000], &a.font_b);
    assert_eq!(word(&m, 0x4c00), 0);
    assert_eq!(word(&m, 0x4c01), 0x5a5a);
    let mut one = memory();
    upload_fonts(&mut one, &a, false).unwrap();
    assert_eq!(&one.vram[0xa000..0xa800], &a.font_b);
    assert_eq!(one.vram[0xa800], 0x5a);
}
#[test]
fn cursor_upload_keeps_separate_text_and_upper_cgram_palettes() {
    let a = load_shared(&rom()).unwrap();
    let mut m = memory();
    m.palette.fill(0x1234);
    upload_cursor(&mut m, &a).unwrap();
    assert_eq!(&m.vram[0xc400..0xc800], &a.cursor);
    assert_eq!(&m.palette[128..160], &a.cursor_palette);
    assert_eq!(m.palette[127], 0x1234);
    assert_eq!(m.palette[160], 0x1234);
}
#[test]
fn shared_sprite_upload_preserves_prefix_then_second_transfer_overwrites_all_wrapped_tail() {
    let a = load_shared(&rom()).unwrap();
    let mut m = memory();
    upload_shared_sprites(&mut m, &a).unwrap();
    assert_eq!(&m.vram[0xc400..0xefc0], &a.sprite_prefix[..0x2bc0]);
    // First tail F400..FFFF, and prefix's final440 bytes, are all overwritten.
    assert_eq!(&m.vram[0xefc0..0x10000], &a.sprite_continuation);
    assert_eq!(m.vram[0xc3ff], 0x5a);
}
#[test]
fn conditional_hex_encoding_preserves_nonoutputs_and_wraps_byte_cursor() {
    let mut b = [0x99; 256];
    let mut cursor = 254;
    append_hex_byte(&mut b, &mut cursor, 0xaf);
    assert_eq!([b[254], b[255], b[0]], [11, 16, 0x99]);
    assert_eq!(cursor, 0);
}
#[test]
fn conditional_diagnostic_fields_stage_high_then_low_and_leave_separators() {
    let mut b = [0x99; 256];
    let mut cursor = 200;
    stage_diagnostic_fields(&mut b, &mut cursor, [0xab12, 0x00ff]);
    assert_eq!(&b[2..12], &[11, 12, 2, 3, 0x99, 1, 1, 16, 16, 0x99]);
    assert_eq!(cursor, 12);
}
#[test]
fn sprite_array_init_retains_attributes_but_resets_ids_positions_and_samples() {
    let mut s = arrays();
    s.order.reverse();
    s.x.fill(0xffff);
    s.y.fill(3);
    initialize_arrays(&mut s);
    assert_eq!(s.order, std::array::from_fn(|i| i as u8));
    assert_eq!(s.x, [0; 16]);
    assert_eq!(s.y, [0xf0; 16]);
    assert_eq!(s.wheel_sample_heights, [0; 16]);
    assert_eq!(s.tile_attributes, [0x2345; 16]);
}
#[test]
fn y_sort_is_descending_stable_and_moves_ids_only() {
    let mut s = arrays();
    s.order = [3, 1, 2, 0, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    s.y = [10, 20, 20, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let old = s.clone();
    sort_by_y(&mut s);
    assert_eq!(
        s.order,
        [1, 2, 0, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
    );
    assert_eq!(s.y, old.y);
    assert_eq!(s.x, old.x);
    assert_eq!(s.tile_attributes, old.tile_attributes);
}
#[test]
fn oam_pack_uses_sorted_ids_x_bit8_forced_large_and_preserves_other_slots() {
    let mut s = arrays();
    s.x[..4].copy_from_slice(&[0x100, 0, 0x1ff, 0x200]);
    s.y[0] = 42;
    s.tile_attributes[0] = 0xe321;
    let mut o = [0x69; OAM_BYTES];
    pack_sorted(&s, &mut o).unwrap();
    assert_eq!(&o[0x100..0x104], &[0, 42, 0x21, 0xe3]);
    assert_eq!(&o[0x210..0x214], &[0xbb, 0xaa, 0xaa, 0xaa]);
    assert_eq!(o[0xff], 0x69);
    assert_eq!(o[0x140], 0x69);
    assert_eq!(o[0x20f], 0x69);
    assert_eq!(o[0x214], 0x69);
    s.order[0] = 16;
    let old = o;
    assert!(pack_sorted(&s, &mut o).is_err());
    assert_eq!(o, old);
}
#[test]
fn frontend_hide_preserves_tiles_then_repack_overwrites_sorted_x_only() {
    let mut m = memory();
    let mut s = arrays();
    s.x.fill(0x123);
    s.tile_attributes.fill(0x4567);
    hide_frontend(&mut s, &mut m).unwrap();
    assert_eq!(&m.staged_oam[..4], &[0xf0, 0xf0, 0x69, 0x69]);
    assert_eq!(&m.staged_oam[0x100..0x104], &[0x23, 0xf0, 0x67, 0x45]);
    assert_eq!(s.y, [0xf0; 16]);
    assert_eq!(s.x, [0x123; 16]);
    assert_eq!(m.staged_oam[0x200], 0xaa);
    assert_eq!(m.staged_oam[0x210], 0xff);
    assert_eq!(m.visible_oam, [0x96; OAM_BYTES]);
}
#[test]
fn race_oam_defaults_and_two_high_overrides_install_source_color_math() {
    let mut m = memory();
    let mut d = display();
    initialize_race_oam(&mut m, &mut d);
    assert_eq!(&m.staged_oam[..8], &[0, 0xf0, 0, 0x21, 0, 0xf0, 0, 0x29]);
    assert_eq!(&m.staged_oam[508..512], &[0, 0xf0, 0, 0x21]);
    assert_eq!(m.staged_oam[0x208], 0xa2);
    assert_eq!(m.staged_oam[0x218], 0);
    assert_eq!(m.staged_oam[0x200], 0xaa);
    assert_eq!(
        (
            d.color_math_layers,
            d.use_subscreen,
            d.half_color,
            d.subtract_color,
            d.sub_layers
        ),
        (0x17, true, true, false, 7)
    );
}
#[test]
fn full_31_animation_pairs_include_unselected_suffix_without_invented_access() {
    let t = load_animations(&rom()).unwrap();
    assert_eq!(
        &t.tiles[..10],
        &[0xff, 0x20, 0x22, 0x24, 0x26, 0x28, 0x2a, 0x2c, 0x2e, 0xff]
    );
    assert_eq!(&t.tiles[24..], &[4, 5, 6, 7, 8, 9, 0xff]);
    assert_eq!(
        &t.attributes[24..],
        &[0x2b, 0x2b, 0x2b, 0x2b, 0x2b, 0x2b, 0xff]
    );
}
#[test]
fn animation_writes_before_fourth_frame_increment_and_preserves_xy() {
    let t = load_animations(&rom()).unwrap();
    let mut m = memory();
    let mut cursors = [0; 16];
    cursors[3] = 1;
    animate(&mut m, &mut cursors, 4, &t).unwrap();
    assert_eq!(&m.staged_oam[0x15c..0x160], &[0x69, 0x69, 0x20, 0x2a]);
    assert_eq!(cursors[3], 2);
    animate(&mut m, &mut cursors, 5, &t).unwrap();
    assert_eq!(m.staged_oam[0x15e], 0x22);
    assert_eq!(cursors[3], 2);
}
#[test]
fn animation_ff_hides_resets_even_offpace_fe_preserves_everything() {
    let mut t = load_animations(&rom()).unwrap();
    t.tiles[2] = 0xfe;
    let mut m = memory();
    let mut c = [0; 16];
    c[0] = 2;
    c[1] = 9;
    animate(&mut m, &mut c, 1, &t).unwrap();
    assert_eq!(&m.staged_oam[0x150..0x154], &[0x69; 4]);
    assert_eq!(c[0], 2);
    assert_eq!(&m.staged_oam[0x154..0x158], &[0x69, 0xf0, 0x69, 0x69]);
    assert_eq!(c[1], 0);
}
#[test]
fn animation_invalid_cursors_reject_atomically_and_clear_visits_all16() {
    let t = load_animations(&rom()).unwrap();
    let mut m = memory();
    let old = m.clone();
    let mut c = [1; 16];
    c[0] = 31;
    let previous = c;
    assert!(animate(&mut m, &mut c, 0, &t).is_err());
    assert_eq!(m, old);
    assert_eq!(c, previous);
    clear_cursors(&mut c);
    assert_eq!(c, [0; 16]);
}
#[test]
fn pose_upload_plan_selects_bank_once_and_keeps_two_strips_per_truck() {
    let b = rom();
    let mut m = memory();
    present_legacy_poses(&mut m, &b, [0, 0x7fc0, 0x8000, 0xc1c0]).unwrap();
    for (destination, physical) in [
        (0xc000, 0x8000),
        (0xc200, 0x8200),
        (0xc040, 0xffc0),
        (0xc240, 0x81c0),
        (0xc080, 0x10000),
        (0xc280, 0x10200),
        (0xc0c0, 0x141c0),
        (0xc2c0, 0x143c0),
    ] {
        assert_eq!(
            &m.vram[destination..destination + 64],
            &b[physical..physical + 64]
        );
    }
}
#[test]
fn pose_upload_copies_real_rom_strips_and_leaves_interstrip_space() {
    let b = rom();
    let mut m = memory();
    present_legacy_poses(&mut m, &b, [0, 0, 0x8000, 0xc1c0]).unwrap();
    assert_eq!(&m.vram[0xc000..0xc040], &b[0x8000..0x8040]);
    assert_eq!(&m.vram[0xc200..0xc240], &b[0x8200..0x8240]);
    assert_eq!(&m.vram[0xc080..0xc0c0], &b[0x10000..0x10040]);
    assert_eq!(&m.vram[0xc2c0..0xc300], &b[0x143c0..0x14400]);
    assert_eq!(m.vram[0xc100], 0x5a);
}

#[test]
fn iteration_three_pose_every_legacy_selector_preserves_both_graphics_spans() {
    let b = rom();
    let mut m = memory();
    // All16-bit selectors were accepted by legacy saves. Source selects bank
    // once; source+0200 wraps the word before setting bit15 for the second row.
    // These are source-layout byte expectations, not rendered-port observations.
    let atlas = poses::PoseAtlas::load(&b, &poses::PoseRules::load(&b).unwrap()).unwrap();
    for selector in 0..=u16::MAX {
        let mut selection = std::array::from_fn(|_| poses::PoseSelection::from_encoded(selector));
        atlas.resolve(&mut selection).unwrap();
        atlas.present(&mut m, &selection).unwrap();
        let bank_base = if selector & 0x8000 == 0 {
            0x8000
        } else {
            0x10000
        };
        let upper = bank_base + usize::from(selector & 0x7fff);
        let lower = bank_base + usize::from(selector.wrapping_add(0x200) & 0x7fff);
        assert_eq!(&m.vram[0xc000..0xc040], &b[upper..upper + 64]);
        assert_eq!(&m.vram[0xc200..0xc240], &b[lower..lower + 64]);
        assert_eq!(m.vram[0xc100], 0x5a);
    }
}
#[test]
fn truck_projection_wraps_coordinates_and_track8_band_priority_thresholds() {
    let mut s = arrays();
    s.tile_attributes[0] = 0x4301;
    project_truck(&mut s, 0, 8, 0x1d80, 0x2e, [0x20, 0xf0]).unwrap();
    assert_eq!(s.x[0], 0xffe0);
    assert_eq!(s.y[0], 0x18);
    assert_eq!(s.tile_attributes[0], 0x7301);
    project_truck(&mut s, 0, 8, 0x2580, 0xff, [0, 0]).unwrap();
    assert_eq!(s.tile_attributes[0], 0x6301);
    project_truck(&mut s, 0, 8, 0x257f, 0x2d, [0, 0]).unwrap();
    assert_eq!(s.tile_attributes[0], 0x6301);
    project_truck(&mut s, 0, 8, 0x257f, 0x2e, [0, 0]).unwrap();
    assert_eq!(s.tile_attributes[0], 0x7301);
    project_truck(&mut s, 0, 7, 0, 0, [0, 0]).unwrap();
    assert_eq!(s.tile_attributes[0], 0x7301);
}
#[test]
fn original_track_titles_all16_resolve_parallel_cpu_rectangles_and_anchors() {
    let b = rom();
    let expected = [
        (7, 0x4b01, [4, 145]),
        (8, 0x4b20, [252, 137]),
        (9, 0x4b21, [172, 1]),
        (10, 0x49c8, [252, 81]),
        (8, 0x49b7, [4, 81]),
        (13, 0x4b12, [92, 1]),
        (12, 0x4a8a, [60, 113]),
        (7, 0x49c1, [180, 81]),
        (11, 0x4852, [180, 161]),
        (10, 0x4b33, [4, 1]),
        (13, 0x4b30, [172, 73]),
        (7, 0x4b33, [4, 65]),
        (13, 0x4b31, [252, 73]),
        (8, 0x48d1, [92, 121]),
        (7, 0x4b37, [252, 1]),
        (12, 0x49a6, [84, 105]),
    ];
    for (i, (width, destination, anchor)) in expected.into_iter().enumerate() {
        let c = load_track_presentation(&b, i as u8).unwrap();
        assert_eq!(c.title.rectangle.width_flags, width);
        assert_eq!(c.title.rectangle.rows, 1);
        assert_eq!(c.title.words.len(), usize::from(width));
        assert_eq!(c.title.destination_word, destination);
        assert_eq!(c.hud_anchor, anchor);
        validate_track_presentation(&c).unwrap();
    }
    assert!(load_track_presentation(&b, 16).is_err());
}
#[test]
fn track8_and14_overlays_use_complete_descriptors_others_none() {
    let b = rom();
    let a = load_track_presentation(&b, 8).unwrap();
    assert_eq!(a.overlays.len(), 1);
    assert_eq!(
        (
            a.overlays[0].destination_word,
            a.overlays[0].rectangle.width_flags,
            a.overlays[0].rectangle.rows,
            a.overlays[0].words.len()
        ),
        (0x4281, 3, 3, 9)
    );
    let c = load_track_presentation(&b, 14).unwrap();
    assert_eq!(
        c.overlays
            .iter()
            .map(|m| (
                m.destination_word,
                m.rectangle.width_flags,
                m.rectangle.rows,
                m.words.len()
            ))
            .collect::<Vec<_>>(),
        vec![(0x40d9, 2, 2, 4), (0x40ca, 2, 2, 4)]
    );
    assert!(load_track_presentation(&b, 7).unwrap().overlays.is_empty());
}
#[test]
fn hud_tables_keep_source_reversed_sprite_iteration_and_complete_banner() {
    let t = load_hud_tables(&rom()).unwrap();
    assert_eq!(
        t.relative_x,
        [
            25, 30, 35, 38, 25, 36, 47, 58, 21, 26, 32, 37, 43, 48, 54, 59
        ]
    );
    assert_eq!(
        t.relative_y,
        [
            16, 16, 16, 16, 25, 25, 25, 25, 34, 34, 34, 34, 34, 34, 34, 34
        ]
    );
    assert_eq!(
        t.attributes,
        [
            55, 55, 55, 55, 49, 51, 53, 55, 49, 49, 51, 51, 53, 53, 55, 55
        ]
    );
    assert_eq!(
        (
            t.banner.rectangle.width_flags,
            t.banner.rectangle.rows,
            t.banner.words.len()
        ),
        (8, 4, 32)
    );
}
#[test]
fn hud_initialization_uploads_graphics_border_titles_and_resets_only_time() {
    let b = rom();
    let c = load_track_presentation(&b, 0).unwrap();
    let t = load_hud_tables(&b).unwrap();
    let mut m = memory();
    let mut clock = RaceTimer::from_encoded([0x12, 0x34, 5]);
    initialize_hud(&mut m, &b, &c, &t, false, &mut clock).unwrap();
    assert_eq!(&m.vram[0xa000..0xc000], &b[0x3b600..0x3d600]);
    assert_eq!(
        [
            word(&m, 0x4800),
            word(&m, 0x483f),
            word(&m, 0x4b40),
            word(&m, 0x4b7f)
        ],
        [0x2ddb; 4]
    );
    assert_eq!(clock, RaceTimer::default());
    assert_eq!(&m.staged_oam[..4], &[63, 179, 0x15, 55]);
    assert_eq!(m.staged_oam[0x36], 0x1f);
    assert_eq!(&m.staged_oam[0x200..0x204], &[0; 4]);
    assert_eq!(m.staged_oam[0x204], 0x69);
}
#[test]
fn reversed_flag_alone_gates_special_overlays_then_title_still_draws() {
    let b = rom();
    let c = load_track_presentation(&b, 8).unwrap();
    let t = load_hud_tables(&b).unwrap();
    let mut forward = memory();
    let mut reverse = memory();
    initialize_hud(&mut forward, &b, &c, &t, false, &mut RaceTimer::default()).unwrap();
    initialize_hud(&mut reverse, &b, &c, &t, true, &mut RaceTimer::default()).unwrap();
    assert_eq!(word(&forward, 0x4281), 0x5a5a);
    assert_ne!(word(&reverse, 0x4281), word(&forward, 0x4281));
    assert_eq!(
        word(&forward, c.title.destination_word as usize),
        word(&reverse, c.title.destination_word as usize)
    );
}
#[test]
fn hud_digit_slots_reverse_truck_order_and_call_real_timer_even_after_winner() {
    let mut m = memory();
    let mut t = RaceTimer::from_encoded([0x90, 0x12, 3]);
    update_hud_digits(
        &mut m,
        [1, 2, 3, 5],
        [0x12, 0x34, 0x56, 0x78].map(crate::trucks::nitro::NitroStock::from_encoded),
        &mut t,
    );
    assert_eq!(
        [
            m.staged_oam[2],
            m.staged_oam[6],
            m.staged_oam[10],
            m.staged_oam[14],
            m.staged_oam[26],
            m.staged_oam[30]
        ],
        [0x1d, 0x1c, 0x1b, 0x1a, 0x17, 0x16]
    );
    assert_eq!(
        [
            m.staged_oam[0x22],
            m.staged_oam[0x26],
            m.staged_oam[0x2a],
            m.staged_oam[0x2e]
        ],
        [0x1a, 0x18, 0x17, 0x16]
    );
    assert_eq!(t, RaceTimer::from_encoded([6, 0x13, 3]));
    assert_eq!(
        [m.staged_oam[0x3e], m.staged_oam[0x3a], m.staged_oam[0x32]],
        [0x18, 0x16, 0x18]
    );
}
#[test]
fn hud_time_saturation_preserves_fraction_and_draws_all_three_visible_nibbles() {
    let mut m = memory();
    let mut t = RaceTimer::from_encoded([0x77, 0x99, 9]);
    update_hud_digits(
        &mut m,
        [1; 4],
        [0; 4].map(crate::trucks::nitro::NitroStock::from_encoded),
        &mut t,
    );
    assert_eq!(t.encoded()[0], 0x77);
    assert_eq!(
        [m.staged_oam[0x3e], m.staged_oam[0x3a], m.staged_oam[0x32]],
        [0x1e; 3]
    );
}
#[test]
fn track_video_copies_raw_bg_and_forces_priority_on_second_map_only() {
    let b = rom();
    let descriptors =
        crate::world::tracks::decode_descriptors(&b[0x123a..0x132a], &b[0x13ef..0x141f]).unwrap();
    let c = crate::world::tracks::load_content(&b, &descriptors[0], 0).unwrap();
    let mut m = memory();
    install_track_video(&mut m, &c, &[0x1234; 256]).unwrap();
    assert_eq!(&m.vram[..0x8000], &c.graphics);
    assert_eq!(&m.vram[0x8080..0x8680], &c.backgrounds[0]);
    let source = u16::from_le_bytes([c.backgrounds[1][0], c.backgrounds[1][1]]);
    assert_eq!(word(&m, 0x4440), source | 0x2000);
    assert_eq!(word(&m, 0x4000), 0);
    assert_eq!(word(&m, 0x4c00), 0);
    assert_eq!(m.palette, [0x1234; 256]);
}
#[test]
fn added_track_presentation_accepts_custom_title_without_original_index_and_rejects_truncation() {
    let mut c = TrackPresentation {
        countdown: crate::presentation::transitions::CountdownLayout {
            center: [91, 139],
            destination_word: 0x4900,
        },
        title: MapImage {
            rectangle: rectangle(2, 1),
            words: vec![0x2001, 0x2002],
            destination_word: 0x4900,
        },
        banner_destination_word: 0x4880,
        hud_anchor: [7, 9],
        overlays: vec![],
    };
    validate_track_presentation(&c).unwrap();
    c.title.words.pop();
    assert!(validate_track_presentation(&c).is_err());
    c.title.words.push(0x2002);
    c.title.rectangle.palette_remap[0] = 8;
    assert!(validate_track_presentation(&c).is_err());
}
#[test]
fn mode1_setup_uses_byte_bases_4_4_2_planes_and_bg3_priority() {
    let mut d = display();
    d.layers[0].scroll = [123, 456];
    configure_race(&mut d);
    assert_eq!(
        d.layers,
        [
            Layer {
                map_byte_base: 0x8000,
                tile_byte_base: 0,
                wide: false,
                tall: false,
                scroll: [123, 456],
                bits_per_pixel: 4
            },
            Layer {
                map_byte_base: 0x8800,
                tile_byte_base: 0,
                wide: false,
                tall: false,
                scroll: [0, 0],
                bits_per_pixel: 4
            },
            Layer {
                map_byte_base: 0x9000,
                tile_byte_base: 0xa000,
                wide: false,
                tall: false,
                scroll: [0, 0],
                bits_per_pixel: 2
            }
        ]
    );
    assert!(d.bg3_high);
    assert_eq!(d.object_tile_byte_base, 0xc000);
}
#[test]
fn display_reset_forces_blank_and_clears_scroll_window_math_but_not_memory() {
    let mut d = display();
    d.layers[0].scroll = [123, 456];
    d.color_math_layers = 63;
    d.fixed_color = 0x1234;
    d.window_edges.fill([8, 99]);
    reset_display(&mut d);
    assert!(d.force_blank);
    assert_eq!(d.brightness, 15);
    assert_eq!(
        (
            d.main_layers,
            d.sub_layers,
            d.main_window_layers,
            d.sub_window_layers,
            d.color_math_layers
        ),
        (0, 0, 0, 0, 0)
    );
    assert_eq!(d.layers[0].scroll, [0, 0]);
    assert_eq!(d.fixed_color, 0x1234); // COLDATA=0 selects no components; retained fixed color is not zeroed.
}
fn scene() -> (VideoMemory, Display) {
    let mut m = memory();
    m.vram.fill(0);
    m.staged_oam.fill(0);
    m.visible_oam.fill(0);
    for i in 0..128 {
        m.visible_oam[i * 4 + 1] = 0xf0;
    }
    m.palette[0] = 0x7c00;
    m.palette[1] = 0x001f;
    m.palette[2] = 0x03e0;
    m.palette[3] = 0x7fff;
    let mut d = display();
    d.main_layers = 1;
    draw_words(&mut m, 0x4000, &[1]);
    m.vram[32] = 0xff;
    (m, d)
}
#[test]
fn compositor_samples_planar_background_palette_and_scroll_from_visible_state() {
    let (mut m, mut d) = scene();
    assert_eq!(pixel(&m, &d, 0, 0).unwrap(), [255, 0, 0, 255]);
    d.layers[0].scroll = [8, 0];
    assert_eq!(pixel(&m, &d, 0, 0).unwrap(), [0, 0, 255, 255]);
    draw_words(&mut m, 0x4001, &[0x0401]);
    m.palette[17] = 0x03e0;
    assert_eq!(pixel(&m, &d, 0, 0).unwrap(), [0, 255, 0, 255]);
}
#[test]
fn compositor_bg3_high_priority_beats_objects_and_low_bg3_falls_behind_bg1() {
    let (mut m, mut d) = scene();
    d.main_layers = 0x15;
    draw_words(&mut m, 0x4800, &[0x2001]);
    m.vram[0xa010] = 0xff;
    m.palette[1] = 0x03e0; // BG3 shares palette; change BG1 to palette1 red.
    draw_words(&mut m, 0x4000, &[0x0401]);
    m.palette[17] = 0x001f;
    m.visible_oam[..4].copy_from_slice(&[0, 0, 0, 0x30]);
    m.vram[0xc000] = 0xff;
    m.palette[129] = 0x7fff;
    assert_eq!(pixel(&m, &d, 0, 0).unwrap(), [0, 255, 0, 255]);
    draw_words(&mut m, 0x4800, &[1]);
    assert_eq!(pixel(&m, &d, 0, 0).unwrap(), [255, 255, 255, 255]);
    d.main_layers = 5;
    assert_eq!(pixel(&m, &d, 0, 0).unwrap(), [255, 0, 0, 255]);
}
#[test]
fn compositor_resolves_first_obj_before_later_higher_bg_priority_and_uses_signed_x() {
    let (mut m, mut d) = scene();
    d.main_layers = 0x10;
    m.palette[129] = 0x001f;
    m.palette[145] = 0x03e0;
    m.vram[0xc000] = 0xff;
    m.visible_oam[..8].copy_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0x32]);
    assert_eq!(pixel(&m, &d, 0, 0).unwrap(), [255, 0, 0, 255]);
    m.visible_oam[512] = 1; // first signed X=-256, second green now visible.
    assert_eq!(pixel(&m, &d, 0, 0).unwrap(), [0, 255, 0, 255]);
}
#[test]
fn compositor_oam_flip_and_size_name_table_addressing_use_game_obj_base() {
    let (mut m, mut d) = scene();
    d.main_layers = 0x10;
    m.palette[129] = 0x001f;
    m.visible_oam[..4].copy_from_slice(&[0, 0, 0, 0x41]);
    m.vram[0xe000] = 1; // name-table1 baseE000, xflip chooses rightmost bit0.
    assert_eq!(pixel(&m, &d, 0, 0).unwrap(), [255, 0, 0, 255]);
    assert_eq!(pixel(&m, &d, 7, 0).unwrap(), [0, 0, 255, 255]);
    m.visible_oam[512] = 2;
    m.visible_oam[3] = 1;
    m.vram[0xe020] = 0x80;
    assert_eq!(pixel(&m, &d, 8, 0).unwrap(), [255, 0, 0, 255]);
}
#[test]
fn compositor_window_masks_inclusive_edges_only_when_main_enable_selects_layer() {
    let (m, mut d) = scene();
    d.masked_layers = 1;
    d.main_window_layers = 1;
    d.window_edges[0] = [2, 4];
    assert_eq!(pixel(&m, &d, 1, 0).unwrap(), [255, 0, 0, 255]);
    assert_eq!(pixel(&m, &d, 2, 0).unwrap(), [0, 0, 255, 255]);
    assert_eq!(pixel(&m, &d, 4, 0).unwrap(), [0, 0, 255, 255]);
    assert_eq!(pixel(&m, &d, 5, 0).unwrap(), [255, 0, 0, 255]);
    d.main_window_layers = 0;
    assert_eq!(pixel(&m, &d, 2, 0).unwrap(), [255, 0, 0, 255]);
}
#[test]
fn compositor_uses_subscreen_add_half_and_subtract_and_backdrop_nohalf_rule() {
    let (mut m, mut d) = scene();
    m.palette[1] = 0x0010;
    m.palette[2] = 8;
    draw_words(&mut m, 0x4400, &[2]);
    m.vram[65] = 0xff;
    d.sub_layers = 2;
    d.use_subscreen = true;
    d.color_math_layers = 1;
    d.half_color = true;
    assert_eq!(pixel(&m, &d, 0, 0).unwrap(), [99, 0, 0, 255]);
    d.subtract_color = true;
    assert_eq!(pixel(&m, &d, 0, 0).unwrap(), [33, 0, 0, 255]);
    d.sub_layers = 0;
    d.fixed_color = 8;
    assert_eq!(pixel(&m, &d, 0, 0).unwrap(), [66, 0, 0, 255]);
}
#[test]
fn obj_palettes_zero_to_three_exclude_math_but_upper_palettes_participate() {
    let (mut m, mut d) = scene();
    d.main_layers = 0x10;
    d.color_math_layers = 0x10;
    d.fixed_color = 0x0010;
    m.palette[129] = 0x000f;
    m.palette[193] = 0x000f;
    m.vram[0xc000] = 0xff;
    m.visible_oam[..4].copy_from_slice(&[0, 0, 0, 0]);
    assert_eq!(pixel(&m, &d, 0, 0).unwrap(), [123, 0, 0, 255]);
    m.visible_oam[3] = 8;
    assert_eq!(pixel(&m, &d, 0, 0).unwrap(), [255, 0, 0, 255]);
}
#[test]
fn brightness_and_force_blank_apply_after_native_color_math_with_no_state_mutation() {
    let (m, mut d) = scene();
    d.brightness = 7;
    let old = m.clone();
    let previous = d.clone();
    assert_eq!(pixel(&m, &d, 0, 0).unwrap(), [119, 0, 0, 255]);
    assert_eq!(m, old);
    assert_eq!(d, previous);
    d.force_blank = true;
    assert_eq!(pixel(&m, &d, 0, 0).unwrap(), [0, 0, 0, 255]);
    d.force_blank = false;
    d.brightness = 0;
    assert_eq!(pixel(&m, &d, 0, 0).unwrap(), [0, 0, 0, 255]);
}
#[test]
fn render_produces_256_by224_read_only_frame_and_ignores_future_staged_oam() {
    let (mut m, d) = scene();
    let before = m.clone();
    let frame = render(&m, &d).unwrap();
    assert_eq!(frame.len(), 256 * 224 * 4);
    // The synthetic scene's console row one is blue. Framebuffer row zero
    // samples that first visible scanline, not the offscreen console row zero.
    assert_eq!(&frame[..4], &[0, 0, 255, 255]);
    assert_eq!(m, before);
    m.staged_oam[..4].copy_from_slice(&[0, 0, 0, 0x30]);
    assert_eq!(render(&m, &d).unwrap(), frame);
}

#[test]
fn text_palette_upload_changes_only_first_sixteen_colors() {
    let a = load_shared(&rom()).unwrap();
    let mut m = memory();
    m.palette.fill(0x1234);
    upload_text_palette(&mut m, &a).unwrap();
    assert_eq!(&m.palette[..16], &a.text_palette);
    assert_eq!(m.palette[16], 0x1234);
}

#[test]
fn framebuffer_first_row_samples_console_visible_scanline_one() {
    let mut memory = VideoMemory::default();
    let mut display = display();
    display.main_layers = 1;
    memory.palette[1] = 0x001f;
    // Tile zero row one alone has color one at X=0. Console line zero is blank.
    memory.vram[2] = 0x80;
    let rendered = render(&memory, &display).unwrap();
    assert_eq!(&rendered[..4], &[255, 0, 0, 255]);
    assert_eq!(pixel(&memory, &display, 0, 0).unwrap(), [0, 0, 0, 255]);
    assert_eq!(pixel(&memory, &display, 0, 1).unwrap(), [255, 0, 0, 255]);
}

#[test]
fn frontend_bg3_scroll_uses_vertical_second_map_page_instead_of_wrapping() {
    let mut memory = VideoMemory::default();
    let mut display = display();
    display.main_layers = 4;
    display.layers[2].tall = true;
    display.layers[2].scroll[1] = 256;
    memory.palette[1] = 0x03e0;
    // First page tile zero is green; second page tile one is transparent.
    memory.vram[usize::from(display.layers[2].tile_byte_base)] = 0x80;
    draw_words(&mut memory, 0x4c00, &[1]);
    assert_eq!(pixel(&memory, &display, 0, 0).unwrap(), [0, 0, 0, 255]);
    display.layers[2].tall = false;
    assert_eq!(pixel(&memory, &display, 0, 0).unwrap(), [0, 255, 0, 255]);
}

#[test]
fn framebuffer_oam_y_zero_starts_first_row_independent_of_background_origin() {
    let mut memory = VideoMemory::default();
    let mut display = display();
    display.main_layers = 0x10;
    memory
        .visible_oam
        .chunks_exact_mut(4)
        .take(128)
        .for_each(|record| record[1] = HIDDEN_Y);
    memory.visible_oam[..4].copy_from_slice(&[0, 0, 0, 0]);
    memory.palette[129] = 0x001f;
    memory.vram[usize::from(display.object_tile_byte_base)] = 0x80;
    let rendered = render(&memory, &display).unwrap();
    assert_eq!(&rendered[..4], &[255, 0, 0, 255]);
    assert_eq!(&rendered[256 * 4..256 * 4 + 4], &[0, 0, 0, 255]);
}

#[test]
fn framebuffer_hdma_first_pair_applies_to_first_visible_row() {
    let mut memory = VideoMemory::default();
    let mut display = display();
    display.main_layers = 1;
    display.main_window_layers = 1;
    display.masked_layers = 1;
    memory.palette[1] = 0x001f;
    memory.vram[2] = 0x80;
    display.window_edges[0] = [0, 0];
    display.window_edges[1] = [1, 0];
    assert_eq!(&render(&memory, &display).unwrap()[..4], &[0, 0, 0, 255]);
    display.window_edges[0] = [1, 0];
    assert_eq!(&render(&memory, &display).unwrap()[..4], &[255, 0, 0, 255]);
}

// Source-format fixture adapter only. Live Game resolves at load/import and
// presents decoded frame selections, without rebuilding an atlas at each NMI.
fn present_legacy_poses(
    memory: &mut VideoMemory,
    rom: &[u8],
    selectors: [u16; 4],
) -> anyhow::Result<()> {
    let atlas = poses::PoseAtlas::load(rom, &poses::PoseRules::load(rom)?)?;
    let mut selection = selectors.map(poses::PoseSelection::from_encoded);
    atlas.resolve(&mut selection)?;
    atlas.present(memory, &selection)
}

#[test]
fn native_pose_assets_apply_overridden_tilt_resolution_and_graphics_together() {
    let mut b = rom();
    // Center tilt becomes source pose100: heading31 resolves to15, encoded85C0,
    // hence physical bank2 rows105C0/107C0. Expectations follow source layout.
    b[0xcba..0xcbc].copy_from_slice(&0x100u16.to_le_bytes());
    b[0x105c0..0x10600].fill(0xde);
    b[0x107c0..0x10800].fill(0xad);
    let rules = poses::PoseRules::load(&b).unwrap();
    let frame = rules.frame(2, 2, 31);
    assert_eq!((frame.working_heading(), frame.encoded()), (15, 0x85c0));
    let atlas = poses::PoseAtlas::load(&b, &rules).unwrap();
    let selected = std::array::from_fn(|_| poses::PoseSelection::frame(frame));
    let mut m = memory();
    atlas.present(&mut m, &selected).unwrap();
    assert_eq!(m.vram[0xc000..0xc040], [0xde; 64]);
    assert_eq!(m.vram[0xc200..0xc240], [0xad; 64]);
    assert_eq!(m.vram[0xc100], 0x5a);
    let mut incomplete = selected;
    incomplete[3] = poses::PoseSelection::from_encoded(0);
    let old = m.clone();
    assert!(atlas.present(&mut m, &incomplete).is_err());
    assert_eq!(
        m, old,
        "a later unresolved selection must not partially present earlier trucks"
    );
}

/// game.asm $07BB..$0804, $0804/$0823 paired31-cell streams.
/// All accepted legacy starts, all pacing residues, and every presentation slot.
#[test]
fn modernization_animation_all_entries_pacing_slots_and_retained_oam() {
    const TILES: [u8; 31] = [
        0xff, 0x20, 0x22, 0x24, 0x26, 0x28, 0x2a, 0x2c, 0x2e, 0xff, 0x62, 0x64, 0x66, 0x68, 0x6a,
        0x6c, 0x6e, 0x80, 0xff, 0x82, 0x84, 0x86, 0x88, 0xff, 4, 5, 6, 7, 8, 9, 0xff,
    ];
    let tables = load_animations(&rom()).unwrap();
    for start in 0..31u8 {
        for residue in 0..4u8 {
            let mut m = memory();
            let before = m.clone();
            let mut cursors = [start; 16];
            animate(&mut m, &mut cursors, residue, &tables).unwrap();
            let tile = TILES[usize::from(start)];
            let stopped = tile == 0xff;
            let expected = if stopped {
                0
            } else {
                start + u8::from(residue == 0)
            };
            assert_eq!(cursors, [expected; 16], "start={start} residue={residue}");
            let mut expected_oam = before.staged_oam;
            for slot in 0..16 {
                let at = 0x150 + slot * 4;
                if stopped {
                    expected_oam[at + 1] = 0xf0;
                } else {
                    expected_oam[at + 2] = tile;
                    expected_oam[at + 3] = match start {
                        1..=8 => 0x2a,
                        10..=17 | 19..=22 => 0x2c,
                        24..=29 => 0x2b,
                        _ => unreachable!(),
                    };
                }
            }
            assert_eq!(m.staged_oam, expected_oam);
            assert_eq!(m.visible_oam, before.visible_oam);
            assert_eq!(m.vram, before.vram);
            assert_eq!(m.palette, before.palette);
        }
    }
}
