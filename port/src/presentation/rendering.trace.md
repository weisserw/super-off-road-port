# Rendering independent source contracts

Worker `/root/behavior_contracts/rendering`, Stage3; no behavior implementation.
Adopted frozen Round29 `reference/source/game.asm` SHA256
`4e3e1a45e4409ce63a3354740a2cdd3962cd3d9975d88d22ecee40c9d29ded7f`.
ROM identity/config unchanged. Physical offsets below differ from bank-zero CPU
aliases by8000; other CPU sources require LoROM translation. The inventory was
navigation; instructions, source data and cross-call readers/writers were reread.

Platform references consulted: toolkit docs/development.md platform table,
docs/snes.md, docs/snes-native.md and independent snes/ppu.rs helpers. Hardware
reference [Fullsnes](https://problemkaputt.de/fullsnes.htm), sections DMA A1Tx/A1Bx,
PPU BG Control/VRAM tile format/OAM/Windows/Color Math. DMA keeps bank constant
while its low16 address wraps. BGMODE09 is Mode1, 4/4/2bpp, BG3 high enabled;
not Mode7. VRAM addresses passed to toolkit primitives are BYTES; source VMADD
is WORDS. Source byte glyph arithmetic uses M8 even when the destination/index
is word-wide. All test fixture values are synthetic or read from local ROM;
no original image/audio content embedded. Unit names below are full test names
under `presentation::tests` in tests.rs; that file imports the pending domain
interfaces, not a second implementation/oracle.

## State, timing and source composition

VideoMemory holds actual native video storage and staged/visible OAM. This is
video data, not a CPU/RAM/register emulator. Display holds named layer bases,
scrolling, masks, main/sub enabled layers, math and brightness/window state.
Renderer signatures are read-only and cannot advance contact cursors or audio.
Race/Game must compose these actual owners, all sprites/animation/pose sources,
objects/race and transition state; phase lists do not implement orchestration.
Full clone/snapshot/debug support must include visible/staged video and retained
contents. Shared immutable decoded assets may be shared, mutable video may not.

Source reset $1C2A writes INIDISP8F (forced blank, brightness15), clears background
and window state/scroll latches twice, main/sub/window layer enables and math.
COLDATA=0 selects NO color components and therefore retains fixed color; do not
mistake it for writing black. Source initialize_race_oam writes CGWSEL02,
CGADSUB57: subscreen operand, addition/halving, BG1/2/3+OBJ math layers17;
TS07 enables three BG sub layers. Its COLDATA0 similarly leaves fixed color.
Native power-on fixed-color selection remains a documented deterministic
assumption until original/runtime evidence; reset/reentry must preserve it.
Race layer setup game.asm:19/110 sets word map bases4000/4400/4800→byte
8000/8800/9000, BG1/2 tiles at0 and BG3 atA000, 8px tiles. initialize_sprite_dma
sets OBSEL03: OBJ base byteC000, name1 baseE000, small8/large16. Configure-race
retains existing scrolls; preceding reset/scene code owns clearing them.

NMI $1BCA checks inhibit BEFORE upload_oam_shadow $1A07 and pose upload $104C,
then runtime counter/heartbeat service. It copies all544 OAM bytes (not only
512), then current8 pose strips; inhibit suppresses both transfers. Source
services mutate shadow after the boundary, so render visible OAM from that
boundary, never future shadow. NMI also selects HUD word4820; actual domain
HUD writes name destinations explicitly. Runtime contracts cover counters/audio;
presentation contracts cover actual visible OAM/VRAM mutation and inhibit.
`nmi_presents_prior_staging_and_pose_state_without_mutating_staged_oam` and
`render_produces_256_by224_read_only_frame_and_ignores_future_staged_oam` establish
both boundaries without a rendering side effect or mock-only phase list.

Compositor fixtures construct real synthetic planar tiles/maps/OAM in those
native buffers and assert RGBA: 2/4bpp transparency, palette offset and scrolling;
Mode1 priority BG3-high over highest OBJ; BG3-low below BG1; first surviving OBJ
wins against later OBJ even if later has higher BG-comparison rank; signed9bit
X, name table, large size and horizontal flip; inclusive one-window masks gated
by TMW/TSW; separate main/sub enables; 5bit add/half/subtract; subscreen backdrop
uses fixed color without halving; OBJ palettes0..3 exclude math and4..7 include;
brightness7 of full red gives119 by native digital 5bit conversion, forced blank
and brightness0 black. Full render dimensions256x224x4, no mutation, checked.
Source natural windows select single noninverted window via22 fields; transitions
owns generation/double buffering and frame timing of scanline edges. Rendering
consumes those edges; no duplicate HDMA state machine. Analog display/gamma,
hardware lag/flicker/overflow are not claimed by these contracts.

## Sprite arrays, OAM and projection

initialize_sprite_arrays $1139 visits15..0, restores order IDs, clears all32 X
bytes and16 wheel samples, sets16 Y bytesF0. It does NOT clear tile/attributes;
that retained-state boundary is tested. sort_sprites_by_screen_y $11B3 performs
stable descending unsigned-byte Y insertion on IDs only; `$0428,x` is previous
ID (`sprite_sort_ids=$0429`). Equal values retain original order, including
arbitrary existing equal-ID order; coordinate/attribute arrays stay intact.

pack_sorted entry$1155 visits16 current IDs, writes each tile/attribute word and
X word at OAM offset100+4i, then overwrites X's high byte with Y. LSR X-high
extracts original bit8 into carry; first ROR shifts that bit into15 of a word,
SEC/ROR forces large bit. After8 sprites all16 scratch bits are overwritten;
first sprite appears in low two bits (xbit,large). Two groups of8 store4 high
bytes at offsets210..213, preserving the other124 OAM entries/high bytes.
Fixed X inputs100,0,1FF,200 give first high byteBB; bit9 from200 is ignored.
Source assumes valid IDs; added/restore invalid ID16 errors atomically in native.

hide_frontend_sprites $40AA writes X andY low bytesF0 to all128 OAM records,
preserves tile/attrs; sets all16 array YF0 and32 high bytesAA, THEN repacks16
sorted records, so sorted X reappears from retained X array, Y remainsF0, high
bytes210..213 may becomeFF. It does not immediately publish visible OAM.
initialize_race_oam_shadow $196E sets all128 tuples (0,F0,0,21), changes second
sprite attr29, high table allAA then offset208=A2,218=00. Fixed tests assert both
special overrides. Object service must retain untouched OAM slots after shorter
composite/placement updates; sorting/packing must not clear the entire shadow.

project_truck_to_screen $10C2 loads committed truck Y for (track&0F)==8.
Unsigned (worldY-1D80)<0800 selects inclusive1D80..257F; height>=2E sets both
priority bits3000, otherwise clears ONLY1000 (keeps2000 and unrelated bits).
Other tracks leave attrs. ScreenX=(projectedX-40) word-wrap, screenY=low(projectedY)
+28 byte-wrap. Tests include both world edges, surface2D/2E and retaining2000.
Physics/terrain supply world/sample/projection state; no float substitute here.

## Contact cursor animation and truck poses

clear cursors $07B0 visits16. animate $07BB visits15..0, reads attribute BEFORE
tile, FE skips EVERYTHING including increment, FF hides only shadow Y and resets
cursor0 regardless pacing. Normal tile/attribute at shadow150+4slot+2/3 written
before incrementing cursor when (animation_frame&3)==0. Coordinates stay intact.
The LDA2A is overwritten by actual attribute scratch; never force2A globally.
Paired arrays physical804..823 and823..842 each31 cells; suffix24..30 is a
supported allocation/conditional entry (04..09,FF with attrs2B), no natural
writer selecting24 established. Local-ROM contract covers complete arrays;
synthetic FE fixture covers skip branch absent from current natural stream.
Native invalid>=31 cursor rejects atomically; this is supported-state validation,
not source memory clamp. Objects chooses start/response cursors (0/1/10/19);
rendering advances/hides them, preserving object contact use of12/17 states.

upload_four_truck_pose_tiles $104C chooses bank1 or2 by selector bit15 ONCE,
source first low=(selector|8000), second=((selector+0200) wrapping16)|8000.
Each strip64bytes; destination first word6000+20*slot→byteC000+40*slot,
second word6100+20*slot→byteC200+40*slot. No generalized flat pose frame index.
Selector7FC0 produces first01:FFC0 but second01:81C0, NOT next bank. Selector
8000 chooses02:8000/8200. Physics owns pose producer/packing (all416 pose words),
rendering owns actual ROM strips/upload. Local ROM tests assert source bytes and
untouched byteC100 between strip areas. Transfer lengths end at bank boundary
without carrying DMA bank; selector producer outputs aligned strips.

## Shared semantic assets and source lifetime

load_shared uses named CPU/physical consumer windows, not archive disjoint regions:

| Asset | CPU start | Physical | Length |
|---|---:|---:|---:|
| Frontend font A |08:D440|45440|800|
| Frontend font B |08:DA20|45A20|800|
| Cursor sprites |08:DDE0|45DE0|400|
| Text palette |08:E1E0|461E0|20|
| Cursor palette |08:E200|46200|40|
| Common banner map |08:E220|46220|40|
| Race HUD tiles |07:B600|3B600|2000|
| Shared sprite ROM prefix |02:D000|15000|3000|
| Shared sprite continuation |04:DAC0|25AC0|1040|
| Shared sprite palette |04:EB00|26B00|100|
| Race palette |00:9EC7|1EC7|200|

The368-byte classified race palette prefix is SHORTER than the512-byte consumer;
load the entire200 bytes, including adjacent physical2037..20C7 storage, mask
CGRAM top bit. `semantic_loader_uses_full_overlapping_font_cursor_palette_hud_windows`
checks all spans, full256 colors and last word, not only prefix. Font A/B overlap,
cursor/text/color/banner overlap too; preserve actual source windows. BGR555
little endian strips stored top bit. Planar fixture has planes0/1 at row bytes
0/1, planes2/3 at16/17 and bit7 left; invalid depth/incomplete units rejected.
No original graphics embedded in executable/test fixtures.

Superseded initial DMA-tail hypothesis: I initially reported that the wrapped
C00 tail needed explicit domain-provided low-WRAM state. Further consumer/lifetime
trace corrected this BEFORE handoff: upload_shared_sprite_tiles $1A9C first
copies3C00 bytes from02:D000 to byteC400..FFFF. DMA bank constant means prefix
3000 ROM bytes then C00 bytes from02:0000..0BFF (WRAM mirrors), NOT next ROM bank.
The immediately following1040-byte transfer from04:DAC0 targets word77E0→byte
EFC0..FFFF; it overwrites the ENTIRE wrapped tail F400..FFFF, plus the first
transfer's final440 ROM bytes. Both run synchronously during race initializer
with video inhibited and forced blank BEFORE source fade/NMI display activation.
No supported consumer observes intermediate overwritten bytes. Native final
installation therefore copies surviving2BC0 prefix bytes toC400..EFBF and full
1040 continuation toEFC0..FFFF; no RAM emulator, fabricated zero or omitted
visible source data. Test explicitly checks surviving prefix and full overwrite.
This optimization removes an original transfer mechanism; it does not classify
WRAM bytes as original ROM assets or exclude gameplay behavior.

load_font_and_clear_bg $1B7B copies font B800bytes at word5000 then clears4800.
upload_frontend_fonts $40DB copies font A800 then font B800 contiguously (no
reset of VMADD between loops), destinationsA000..AFFF bytes. upload_cursor$44B2
copies400 bytes to word6200 (byteC400) and40 palette bytes to CGRAM color128..159;
upload_text_base_palette$4485 transfers20bytes to colors0..15. Tests preserve
neighbors, exact map-fill endpoint and font overlap. Attract/names/shop/results
own when their scene assets/layout are installed; these mechanisms are shared.

## Map, text and conditional diagnostics

fill_vram_common $1C13 writes X=0400 down through0 inclusive:401 WORDS, not400;
this touches first word of next map. Clears and caller-word fill share body,
wrap at VRAM byteFFFF→0. Does NOT reset PPU despite combined routine label.
Three track maps initialized4000/4400/4800, then raw bg1 bytes600 at4040 and bg2
600 at4440 with high byte OR20 (priority bit2000), not premodified TrackContent.
Contract asserts original graphics8000 bytes, raw bg1 and forced bg2 bit, palette.

Rectangle $44FA copies20 descriptor bytes: width/flag word, row word,8 remap
WORDS. Width low15 controls inner count, sign bypasses all remapping. Source tile
palette=((tile>>10)&7); remap word shifted twice then XBA; normal bounded0..7
remap replaces bits10..12, keeps E3FF (tile/priority/flips). Each source word
advances ONLY low source pointer twice; no bank carry in pointer advance; actual
long-indirect word fetch can cross a bank for high byte. Native ROM-only loader
must reject unsupported noncartridge crossings rather than concatenate ROM.
Destination each row+20words, actual rows loop initial DEC then end DEC/BPL.
Valid widths/rows>0 produce exact product; source zero dims would still execute
one pathological row/column, no natural descriptor zero established. Native
extension validation rejects zeros/truncation/oversize instead of replaying
unsupported memory reads. Original descriptors positive≤32, remaps0..7.
Atomic validation tests are native added-content/restore policy, not original
claims source detects malformed memory. No magic RAM map in the native interface.

Single text $3ABC reads DB00 glyph bytes untilFF, writes glyph+2000attr, destination
sequential words, leavesY pointing at separator. Paired entry$3ADC sets VMAIN81:
first doubled glyph atbase, second atbase+20words; glyph ASL/INC is BYTE arithmetic
(7F→FE/FF,80→00/01), next character resets destinationbase+1. Seven rows$3B0E
runs seven calls (counter6..0), skips separators, adds40words per row. Name panel
$3B3D clears11cols×14rows with2000words, stride20, preserving neighbor tiles.
All formats have fixed boundary and termination contracts; malformed extension
streams errors atomically, no host hangs. Original row streams/pointer records
3A43/3A8A/3AA0/3AAC retained from ROM; names/results supply scene-specific uses.

Conditional diagnostics $1116/$1BAB/$1BB7 have unresolved natural entry, E/DB/DP;
not player-facing feature authorization. Under stated native source context,
append high nibble+1 then low nibble+1, byte cursor wraps. Stage fields begins
cursor2, renders each WORD high then low, advances separator WITHOUT writing.
InputsAB12,00FF yield glyphs11,12,2,3 /1,1,16,16 and preserved separators at6/11;
final cursor12. Tests encode conditional domain behavior, not invented UI or
absolute state identity. Wrapper saves/restores registers/status, output effect
only; native need not emulate a CPU register file. No natural caller proof yet.

## HUD, titles, overlays and extensions

HUD $414B copies2000 bytes atword5000, clears4800 through4C00, fills64words
border2DDB at4800 and4B40. Reversed flag gates overlays BEFORE titles; forward
track8/14 does NOT overlay. Track8 descriptor3×3 atword4281 from0D:8C00;
track14 two2×2 at40D9/40CA from0D:D412/D41A. Remap defined by source20bytes.
Title table4305 triples, rectangle pointers4335, destinations43E1, anchors4431/
4441 andbanner destinations4451 decoded separately with16 original bound.
All16 title widths (7,8,9,10,8,13,12,7,11,10,13,7,13,8,7,12), destinations,
anchors independently fixed in test; every source title is one row, common
banner8×4 words uses full64-byte window (including physical prior union).

HUD sprites loop source X15 down to0 into OAM0 upward. Byte anchor additions
wrap, tile15 default, source attrs preserved. Track0 first sprite=(63,179,0x15,0x37)
from relative59,34 +anchor4,145. Tile atOAM36 forced1F and first4 high bytes0;
remaining high bytes retained. Setup clears ONLY all three timer bytes. Nitro
HUD visits trucks3..0, low digit then high in successive sprites+4;+15 tilebase.
Laps likewise3..0 at22,26,2A,2E. Calls actual race::advance_timer every HUD
service; no winner/prepared guard. Saturation and timer authority belong race;
presentation draws high low nibble at3E, middle high nibble3A, middle low32.
Timer(90,12,03)+decimal16→(06,13,03) tiles18,16,18. Saturated(77,99,09)
retains fraction and draws1E/1E/1E. These overlap existing race timer rules but
assert actual OAM mutation/interface rather than duplicate time ownership.

TrackContent now has optional presentation: original None resolves original
source title/overlay/HUD; added/replaced named definitions REQUIRE validated
TrackPresentation with arbitrary title MapImage, banner destination, HUD anchor,
overlay list. This preserves original16 ROM table bounds while catalogue accepts
17+ named tracks. Existing tracks extension fixtures carry explicit valid layout;
added required-presentation test rejects missing/short fields. Paletteremap<=7,
nonzero dimensions, lengths/destination bounds validated transactionally; source
malformed data not silently zeroed. Shop alternate player2 palette4D3D0 is shared
BGR555 data consumed by shop and must stay ROM-backed there (not universal cursor
palette). Frontend/attract background/palette unions similarly consumed by their
scene owner, rendering supplies actual decode/draw composition algorithms.

## Other assigned data, unresolved scope and evidence

Bearing grid1CC7..1DC7 and sine1DC7..1EC7 assigned primary rendering navigation
are actually consumed by ai/contact/terrain/motion: those independently traced
workers' contracts are semantic owners; rendering never regenerates trig tables.
Track background/graphics unions shared with tracks: load_content preserves raw
bytes and presentation installs/decodes them, no extra copies/classification.
Offset-grid1AFB..1B7B values8×8 (row400,col40) supported data but consumer
unresolved, no invented gameplay consumer. Generic sixzeros2EB72,64zeros45300,
29 tileword-like words46260..4629A also remain consumer-unresolved; latter beside
window-shape selector data handed to transitions. No exclusion by pattern or lack
of execution. Existing raw resources preserve these without falsely claiming
native logic; uncertainty remains for stage4 assessment. Cursor24 suffix,
diagnostics and natural entry are explicit conditional supported contracts.

All53 rendering contracts discovered and runnable plus one tracks extension
contract. Rust1.90 locked compile/list pass304 suite tests;53/53 rendering and
1/1 affected tracks failure are explicit pending-unimplemented panics, no
ignored tests, compile errors/hangs/fixture failures. Assets2+Game2 infrastructure
pass. Exact commands/logs checks/contracts/rendering-results.json; initial compile
log retained separately. No implementation audit/gameplay smoke/QA/emulator
experiment, toolkit/reference change, commit/publication. Stage4 must implement
these real owner interfaces and actual composition, not replace with phase mocks.

## Exhaustive primary-source navigation (reviewed, not implementation accounting)

Each row was checked against its instructions or supported data consumers above.
Semantic cross-owners are explicit above; no ledger disposition changed.

| Ledger | Physical half-open range | Source entry/label |
|---|---|---|
| region-341 | $0007B0..$0007BB | clear_sprite_animation_cursors; game.asm:2153 |
| region-132 | $0007BB..$0007FA | animate_sixteen_sprite_slots; game.asm:2172 |
| region-133 | $0007FA..$000804 | hide_finished_sprite_animation; game.asm:2220 |
| region-916 | $000804..$00081C | contact_animation_tile_stream; game.asm:2231 |
| region-1369 | $00081C..$000823 | unselected_contact_animation_tiles; game.asm:2252 |
| region-917 | $000823..$00083B | contact_animation_attribute_stream; game.asm:2264 |
| region-1370 | $00083B..$000842 | unselected_contact_animation_attributes; game.asm:2285 |
| region-27 | $00104C..$0010C2 | upload_four_truck_pose_tiles; game.asm:4058 |
| region-157 | $0010C2..$0010D3 | project_truck_to_screen; game.asm:4122 |
| region-726 | $0010D3..$0010FE | set_track_band_sprite_priority; game.asm:4141 |
| region-158 | $0010FE..$001116 | store_truck_screen_coordinates; game.asm:4171 |
| region-926 | $001116..$001139 | stage_hex_diagnostic_fields; game.asm:4195 |
| region-28 | $001139..$0011B3 | initialize_sprite_arrays_and_pack_oam; game.asm:4224 |
| region-159 | $0011B3..$0011FE | sort_sprites_by_screen_y; game.asm:4302 |
| region-347 | $00196E..$0019C0 | initialize_race_oam_shadow; game.asm:6846 |
| region-3 | $0019F3..$001A07 | initialize_sprite_dma; game.asm:6943 |
| region-29 | $001A07..$001A24 | upload_oam_shadow; game.asm:6961 |
| region-348 | $001A78..$001AFB | load_shared_sprite_palette_and_tiles; game.asm:7059 |
| region-1409 | $001AFB..$001B7B | offset_grid_8x8; game.asm:7122 |
| region-5 | $001B7B..$001BAB | load_font_and_clear_bg; game.asm:7138 |
| region-964 | $001BAB..$001BB7 | append_hex_byte_preserving_registers; game.asm:7169 |
| region-965 | $001BB7..$001BCA | append_hex_byte_glyphs; game.asm:7188 |
| region-30 | $001BCA..$001C07 | service_nmi_video_and_input; game.asm:7214 |
| region-349 | $001C07..$001C0D | fill_vram_with_caller_word; game.asm:7257 |
| region-6 | $001C0D..$001CC7 | clear_vram_and_reset_ppu; game.asm:7272 |
| region-966 | $001CC7..$001CCF | vector_heading_cells_00; game.asm:7359 |
| region-1373 | $001CCF..$001CD0 | vector_heading_x_minus8_y_0; game.asm:7369 |
| region-967 | $001CD0..$001CDF | vector_heading_cells_09; game.asm:7379 |
| region-1374 | $001CDF..$001CE0 | vector_heading_x_minus8_y_1; game.asm:7389 |
| region-968 | $001CE0..$001CEF | vector_heading_cells_19; game.asm:7399 |
| region-1375 | $001CEF..$001CF0 | vector_heading_x_minus8_y_2; game.asm:7409 |
| region-969 | $001CF0..$001CFF | vector_heading_cells_29; game.asm:7419 |
| region-1376 | $001CFF..$001D00 | vector_heading_x_minus8_y_3; game.asm:7429 |
| region-970 | $001D00..$001D0F | vector_heading_cells_39; game.asm:7439 |
| region-1377 | $001D0F..$001D10 | vector_heading_x_minus8_y_4; game.asm:7449 |
| region-971 | $001D10..$001D1F | vector_heading_cells_49; game.asm:7459 |
| region-1378 | $001D1F..$001D20 | vector_heading_x_minus8_y_5; game.asm:7469 |
| region-972 | $001D20..$001D2F | vector_heading_cells_59; game.asm:7479 |
| region-1379 | $001D2F..$001D30 | vector_heading_x_minus8_y_6; game.asm:7489 |
| region-973 | $001D30..$001D3F | vector_heading_cells_69; game.asm:7499 |
| region-1380 | $001D3F..$001D40 | vector_heading_x_minus8_y_7; game.asm:7509 |
| region-974 | $001D40..$001D47 | vector_heading_cells_79; game.asm:7519 |
| region-1381 | $001D47..$001D57 | vector_heading_y_minus8_row; game.asm:7529 |
| region-975 | $001D57..$001D5F | vector_heading_cells_90; game.asm:7539 |
| region-1382 | $001D5F..$001D60 | vector_heading_x_minus8_y_minus7; game.asm:7549 |
| region-976 | $001D60..$001D6F | vector_heading_cells_99; game.asm:7559 |
| region-1383 | $001D6F..$001D70 | vector_heading_x_minus8_y_minus6; game.asm:7569 |
| region-977 | $001D70..$001D7F | vector_heading_cells_a9; game.asm:7579 |
| region-1384 | $001D7F..$001D80 | vector_heading_x_minus8_y_minus5; game.asm:7589 |
| region-978 | $001D80..$001D8F | vector_heading_cells_b9; game.asm:7599 |
| region-1385 | $001D8F..$001D90 | vector_heading_x_minus8_y_minus4; game.asm:7609 |
| region-979 | $001D90..$001D9F | vector_heading_cells_c9; game.asm:7619 |
| region-1386 | $001D9F..$001DA0 | vector_heading_x_minus8_y_minus3; game.asm:7629 |
| region-980 | $001DA0..$001DAF | vector_heading_cells_d9; game.asm:7639 |
| region-1387 | $001DAF..$001DB0 | vector_heading_x_minus8_y_minus2; game.asm:7649 |
| region-981 | $001DB0..$001DBF | vector_heading_cells_e9; game.asm:7659 |
| region-1388 | $001DBF..$001DC0 | vector_heading_x_minus8_y_minus1; game.asm:7669 |
| region-982 | $001DC0..$001DC7 | vector_heading_cells_f9; game.asm:7679 |
| region-983 | $001DC7..$001EC7 | heading_signed_sine; game.asm:7687 |
| region-984 | $001EC7..$002037 | track_cgram_source_prefix; game.asm:7725 |
| region-1003 | $003A43..$003A8A | paired_text_glyphs_3a43; game.asm:13289 |
| region-1004 | $003A8A..$003AA0 | text_glyphs_3a8a; game.asm:13315 |
| region-1005 | $003AA0..$003AAC | paired_text_glyphs_3aa0; game.asm:13338 |
| region-1006 | $003AAC..$003ABC | name_entry_text_records; game.asm:13360 |
| region-36 | $003ABC..$003B0E | draw_terminated_tile_text; game.asm:13389 |
| region-74 | $003B0E..$003B2B | draw_seven_large_text_rows; game.asm:13444 |
| region-37 | $003B2B..$003B3D | begin_clear_name_entry_panel; game.asm:13470 |
| region-109 | $003B3D..$003B5B | clear_name_entry_panel; game.asm:13488 |
| region-398 | $004123..$004137 | track_eight_overlay_descriptor; game.asm:14459 |
| region-399 | $004137..$00414B | track_fourteen_overlay_descriptor; game.asm:14489 |
| region-358 | $00414B..$00419F | initialize_race_hud_tiles; game.asm:14523 |
| region-780 | $00419F..$0041EC | draw_track_specific_overlays; game.asm:14571 |
| region-359 | $0041EC..$004277 | draw_track_title_banner_and_hud_sprites; game.asm:14612 |
| region-198 | $004277..$0042BE | update_lap_and_nitro_hud_digits; game.asm:14690 |
| region-254 | $0042BE..$0042C5 | test_race_timer_saturation; game.asm:14746 |
| region-199 | $0042C5..$004305 | advance_and_draw_race_timer; game.asm:14760 |
| region-375 | $004305..$004485 | track_hud_layout_tables; game.asm:14802 |
| region-310 | $004485..$0044B2 | upload_text_base_palette; game.asm:14844 |
| region-50 | $0044B2..$0044FA | upload_frontend_cursor_tiles_and_palette; game.asm:14873 |
| region-25 | $0044FA..$004560 | draw_remapped_tile_rectangle; game.asm:14918 |
| region-429 | $008000..$015000 | truck_pose_graphics_strips; game.asm:18490 |
| region-378 | $015000..$026C00 | shared_sprites_and_track_asset_union; game.asm:18501 |
| region-54 | $02DAC0..$02E400 | frontend_background_tiles_tail; game.asm:18618 |
| region-55 | $02E400..$02E4E0 | frontend_palette; game.asm:18630 |
| region-1451 | $02EB72..$02EB78 | consumer-unresolved generic data; game.asm:18651 |
| region-380 | $030000..$035400 | track_background_map_union_00_to_06; game.asm:18668 |
| region-53 | $035400..$037980 | frontend_background_tiles; game.asm:18744 |
| region-381 | $03C000..$03D600 | race_hud_graphics_union_prefix; game.asm:18782 |
| region-382 | $03D97E..$03DAB4 | track_title_map_union; game.asm:19084 |
| region-1452 | $045300..$045340 | consumer-unresolved generic data; game.asm:19676 |
| region-52 | $045440..$046240 | frontend_font_sprite_palette_union; game.asm:19723 |
| region-383 | $046240..$046260 | track_banner_map; game.asm:19746 |
| region-1453 | $046260..$04629A | consumer-unresolved generic data; game.asm:19749 |
| region-393 | $04D3D0..$04D3F0 | alternate_player2_shop_palette; game.asm:19926 |
| region-386 | $060000..$068C00 | track_graphics_and_background_union; game.asm:20077 |
| region-436 | $068C00..$068C12 | track_eight_overlay_tiles; game.asm:20130 |
| region-387 | $068C12..$06D412 | track_background_map_union_09_to_14; game.asm:20137 |
| region-437 | $06D412..$06D41A | track_fourteen_overlay_one_tiles; game.asm:20201 |
| region-438 | $06D41A..$06D422 | track_fourteen_overlay_two_tiles; game.asm:20208 |
| region-388 | $06D422..$06EC22 | track_background_map_union_07_and_15; game.asm:20215 |

## Stage4 source alias correction (supersedes suffix uncertainty only)

The Stage3 no-natural-writer interpretation above is retained as superseded.
`truck_effect_cooldowns=$0525` aliases `sprite_animation_cursors=$0519+12`.
Motion contact-effect spawn writes `$18` (24), so slots12..15 naturally select
paired entries24..30; the generic animation service advances once per source
frame. Race keeps each cursor solely in Truck.effect_cooldown and constructs
a transient sixteen-slot view; no independent cooldown decrement or duplicate
animation owner. Integrated race contract now seeds24 and requires25 plus the
actual OAM tile from entry24 after one frame. Diagnostic entry uncertainties
and the unrelated unresolved data remain unchanged.

Stage4 active-window inversion refinement: source scrolling/contracting masks03 have window1 enable(bit1)+invert(bit0), unlike22 noninverse race/expanding. Single Display now retains active inverted-layer bits alongside active masks; native compositor masks inside XORinvert. Window retained config remains its owncaller bytes; disable_masks clears actualmask/invert, notretainedconfig/HDMA. Fixedsynthetic BGred/backdropblue pixelvectors require03insidevisible/outsidehidden and22inverse. Game ready/postrace/pauseclosure disablesHDMA separately andholds finalclosededge pair, preservingTMW/sourceconfiguredmask; these race geometries leave finalvisiblescanlines closed. Existingfixturedefaults0invert onlyaddedfield, no originalpixelassertion weakened.

## Stage5 visible scanline origin correction

Initial original-license capture (original frame181 after cold audio upload) and
native full-bright license (native tick172) had identical pixels after exactly
one row translation: original row y == former native row y+1. The 256x224
framebuffer omits SNES scanline0; visible console scanlines are1..224. render()
now samples console line framebuffer_y+1 for BG/OBJ/window composition. pixel()
remains the explicit console-coordinate primitive. Synthetic row-zero/row-one
contract proves the origin independently. Retained failed captures are in
`debug/artifacts/setup-original-002` and `setup-native-001`; no expected pixels
were learned from native output.

## Stage5 frontend BG3 screen-size correction

Original initial highscore frame806 with source highscore_bg3_scroll=$0132 and
native matched scroll exposed5612 extra native text pixels at the top. Source
initialize_frontend_and_attract (game.asm:7791) writes BG3SC=$4A, whose size bits
select a32x64 tile map. Native composition had hard-coded32x32 for every layer,
wrapping the second vertical page into the first. Layer now retains wide/tall
screen-size flags; frontend BG3 is tall, reset/race BGxSC=$40/$44/$48 are32x32.
The synthetic second-page test independently distinguishes the two source modes.
Failed exact comparisons remain in debug/artifacts/sanity-run-001.

Initial race comparison further distinguishes OBJ from BG scanline origin:
OAM Y=0 starts the first visible row, because object evaluation feeds the next
scanline. render samples BG at row+1 but OBJ at row; the toolkit Object primitive
continues to use direct OAM-relative coordinates. The first scanline correction
was therefore refined rather than applying a global image translation. Independent
synthetic OAM-row0 contract and selected original truck/HUD captures cover it.

Selected ready animation (original1723, extent115/countdown13/delay4) showed its
window silhouette one row too high after BG origin repair: original top edge
row23, native row22. The first HDMA edge pair belongs to the first visible row,
like the OBJ output row rather than BG sampling's console line. Compositor now
uses the visible row for edge-table lookup; synthetic first-pair masking contract
fixes the convention independently. Initial grid's remaining71pixels were solely
pickup state caused by prior nonzero cold RNG policy; config now globally matches
the original reference's zero WRAM, preserving the source's unseeded ambiguity
and superseded assumption, not applying a scenario-specific state edit.

## Iteration3 M5 — native body frames/images

The source rules and original span expectations above are preserved. Source pose
word packing, bank choice and second-row word wrapping now execute only in
presentation/poses.rs while decoding assets or binding an old save word. Motion
uses a decoded native orientation/heading BodyFrame and race stages that frame;
NMI reads its already decoded immutable image. No runtime PoseStrip command list,
CPU address translation or direct ROM span lookup remains for ordinary body
presentation. Version2 serialization still emits the original selector metadata.
Cold loading and atomic save import resolve legacy selections against the bound
resources before live adoption. All65536 words retain the prior native span
policy; no malformed-state original DMA claim is added.

The old transfer-plan test now checks the same eight fixed source byte spans in
the resulting native backend rows instead of asserting obsolete command records.
The former isolated present_boundary wrapper test is replaced by an actual Game
NMI/inhibition regression. Fixed byte expectations and the old staging/order rules
are unchanged. New override coverage ties a nonzero tilt's heading resolution to
its replaced graphics; a later unresolved selection must not partially present
other trucks. Full legacy-selector and save-roundtrip contracts were passed on
R2 before conversion. Natural original VRAM observations and fresh native route
comparisons remain local in modernize/evidence/iteration-003.

## Iteration3 M4 native effect ownership (supersedes cursor implementation above)

`effects::AnimationPlayer` owns Idle or a clip-relative Nitro/Surface/
SurfaceRepeat/Contact phase. The private phase is serialized through an exact
legacy byte adapter, retaining all accepted entries0..30 and old save keys.
Invalid historical bytes remain representable only so existing validation can
reject them atomically. `AnimationClips::decode` compiles paired bytes once into
Hold/Finish/Draw actions with native successors; no FE/FF decoder or flat source
cursor is consulted in normal playback. Overridden terminal cells can still draw
and advance across clips or into an invalid successor, failing on the next pass.

Race validates all16 actual owners before any staging mutation, then services
slots15..0 directly. Trucks own nitro/contact players, world contacts own surface
players and RaceVideo owns the middle group. Racing and transition callers share
this service. Numeric gather/scatter is cfg(test) fixture-only. Powertrain keeps
the same retained-player proposal semantics as before; it does not advance it.
Motion contact checks idle then starts Contact. Nitro starts Nitro. Special
surface checks idle or the terminal Surface/SurfaceRepeat phase before starting
or resuming; these are exactly source hex12/17 (decimal18/23), not decimal12/17.

The source guarantees remain: Hold touches nothing; Finish hides only staged Y
and resets immediately; Draw writes tile/attributes before advancement every
fourth animation frame. Staged-to-visible NMI transfer, coordinates and retained
OAM remain unchanged. Preimplementation contracts cover all31 entries, four
pacing residues,16 slots, all surface gates and legacy owner wire roundtrips.
Fresh original natural nitro observations include all8 tiles, paced predecessor
draws and hide/reset, with persistent replay. Source can write1 and advance2 in
one physical frame, so cursor1 need not appear in a frame-boundary observation.
