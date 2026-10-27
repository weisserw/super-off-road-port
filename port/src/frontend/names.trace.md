# Names, joining and shared driver panels — independent contract trace

Reference: frozen Round29 reference/source/game.asm / reading.asm. Original SHA
identity is workspace.json; no adopted research supplement. Main CPU E=0,
DB=DP=0 in natural frontend context. Physical bank-zero offsets map to CPU
$008000+offset; bank09 assets below use physical$048000+(CPU-$098000).
This is a Stage3 trace; every behavior interface remains explicitly pending.
Contracts are frontend/names/tests.rs, not an implementation oracle.

## Entry, widths, waits and shared ownership

$BB5B physical3B5B saves P and SEP30; INC video-inhibit wraps byte, clears current
page and both joined-page bytes. It calls fadeout M16, resets BG1 horizontal and
vertical latches and active window masks. The display helper does not disable
HDMA itself. $BB81 sends APU word0501, then initializes pages.

The page loop runs page zero and advances until page>limit, inclusively. Although
its textual LDA/CMP have .w ADDRESS operands, operand data width is M8 here:
initialize_name_entry_page eventually falls through $BCB7 E803, contracting
window, REP20, $BCC2 LDA0101/STA APU, SEP20/RTS. Thus INC page/LDA page/CMP
page_limit read byte data after that return. Do not read adjacent menu-row or
page bytes as a word based on the assembler suffix. Boot sets page_limit=0;
page1 routines remain conditional supported two-player-page behavior. Source
page values>1 would run beyond the two joined bytes; there is no evidence of a
natural such configuration or a player-count menu. Native frontend support
covers the established two pages without inventing a selector.

$BB99 uploads cursor tiles/palette then waits NMI. It resets the active page's
join bits, BG3 vertical scroll, both row/column positions, both name cursors and
done mask, sets countdown5F, and copies ten defaults to each temporary buffer
in descending9..0 order. Importantly it does NOT reset repeat or button-history
bytes. These are retained by source callers (including shop/pause shared history),
not reinitialized for convenience at every page. Frontend Session remains the
single owner of joined masks, records/truck_names, inhibit, layer bits and audio
commands. Selection owns transient page/editor state, not a second session copy.

Video setup enables layer mask15, clears4800 and4000 using rendering's inclusive
401-word fill, installs standard driver graphics/palette/panels, heading map at
4020, both fonts and text palette, four paired-text records in offsets12,8,4,0
order. Common instructions at4904/4914 use BA8A/BA93. Nonzero page adds BA9C at
490B and BA9E at491B. It initializes/hides/repacks sprites, clears video inhibit,
fades in, and enters per-frame service.

Frame sequence atBC48: wait; redraw joined second name at4B16 then first at4B05;
update countdown; sample JOY1/2 (both page branches identical); pack OAM; first
join, second join, first edit, second edit; require done==03 exactly. A join may
wait and draw grid before returning to the SAME sampled input iteration. Clears
of completed panels also wait and must resume without repolling input or rerunning
countdown. Native advance therefore resumes at actual source wait boundaries,
not a blocking host loop or only an effect-list mock.

Completion commits names, hides frontend sprites, waits one final NMI, increments
inhibit, clears window Start-abort-enable, sends E803, executes contraction,
sends0101 and returns with M8 to inclusive page iteration. Selection audio0501
belongs page loop entry; join or ordinary character acceptance has no own audio
command in this source. Attract calls names then shop before joined-word gate.
Cold Game reset and warm frontend reentry are distinct from these session/page
resets; snapshots must preserve every pending wait/input/history/display owner.

## Join edge, shared ROR history and ordering

$BD96/$BDED tests (joined_page OR done_mask)&ownbit before updating history. A
joined/done player returns with history untouched. Otherwise A=input-high;
three ASLs move Select bit5 into carry, then ROR history inserts current Select
in bit7 and shifts previous bit7 to bit6. BIT history sets N,V independently
of A; accept only N=1,V=0. Literal history sequence0→80→C0→60→B0 establishes
newpress,held,release,newpress. It sets only own join bit, waits frame, draws seven
paired rows at4943/4954 and installs cursor tilewords2026/2028. Initial cursor
X words15 or9D, Ys48/58; unlike subsequent movement these are full word stores.

B confirmation uses the SAME history but one ASL from high bit7. A simultaneous
Select+B joining from history0 first produces80 then editor producesC0 and
suppresses confirmation. Already joined guards skip Select history updates;
subsequent editor polling tracks B alone. Tests retain this coupled history,
not two invented edge detectors. B/Y/Select/Start influence repeat as high-byte
nonzero; low-byte A/X/L/R alone do not.

Countdown expiry runs BEFORE poll/join: an absent player's fresh Select on final
countdown boundary cannot join because done bit is set first. Both local players
sample hardware JOY1/JOY2 even on page1; joined records move pages, devices do not.

## Editor, coordinates, repeat and preview

$BE44/$BF16 guards joined and not done. If input HIGH byte==0, repeat=0. Else
repeat>0 decrements and skips movement. Repeat==0 reloads8 then calls source
16-entry direction tables. This is movement on call1; eight subsequent
calls consume8→7→6→5→4→3→2→1→0; next movement is call10, a NINE-call interval.
The tests explicitly use initial movement, eight countdown-only calls, movement.
A held action with zero direction still consumes/reloads this delay. Controls
owns lookup quirks (no generic opposing-direction cancellation).

Column byte+signed delta wraps negative→4 and >=5→0. Row wraps negative→6 and
>=7→0. Cell=5*row+column; all35 cells must be kept. Source cell-table X/Y byte
adds origin15/9D and48, second spriteY+10. Movement stores only X LOW byte,
retaining previous word high; test initial1AA and row5col2 (tableX24 hex) gives
139/1C1, Ys98/A8. Initial join writes full X words and normally clears high.

Glyph preview is unconditional for a valid editor: write2A at current byte,
replace with nonnegative selected glyph. Negative token80 and FF retain2A until
confirmation. It occurs even with no B, so editing is live preview. Grid row5
contains glyph24,27,0,0,80; row6 all five FF. B rising edge ordinary glyph advances
cursor by TWO while cursor<06. This permits cursor0,2,4,6: a FOURTH preview slot,
not a strict three-character editing bound. Cursor6 cannot advance further.
Negative token other thanFF backs up TWO unless cursor0, without clearing old
preview. FF sets own done bit via TSB and hides every completed panel. Neither
negative token is stored into name data by preview. Actual interleaved odd bytes
remain source-default data; they are not recomputed as semantic padding.

Commit $BCCB copies temporary bytes0,2,4 only, first local player to truck_names
base and second to base+3. Page0 base0, every nonzero supported page base6. Thus
fourth preview is discarded in final three-byte names; no fourth final character
is invented. Character spellings remain ROM glyph IDs.

## Countdown and completed panels

$BCFF returns immediately if countdown0, even if some done bits unset. Otherwise
writes temporary_player1 offsets7=01 and8=(OLD countdown>>4)+1; FF at9 terminates.
Only absent player's countdown is drawn at4A67/4A77 using paired glyph rendering.
5F yields glyph1 then6 (tilewords2002/2003 vertical and200C/200D at next column).
It decrements only when animation_frame&03==0; 95 qualifying boundaries expire.
DEC reaching0 computes done |= joined_page XOR03, not a guessed inactive mask.
The native supported page mask is low2 bits. It calls hide_completed_panels;
source passes A03 but routine reloads done mask, clearing ALL done panels.

Panel one clear4941, panel two4954. Rendering clear draws eleven columns and
fourteen rows,32-word stride, word2000, after its NMI wait. It writes corresponding
cursor pair YsF0 and retains other sprite fields/other unfinished pair. A done
player's editor guard then leaves preview/history/repeat unchanged.

## Driver graphics, palettes and all original consumers

$B257 graphics wrapper clears4000 (401words), disables HDMA and uploads4DA0 bytes
from CPU098000 to VRAM byte0, physical48000..4CDA0. Palette wrapper uploads100
bytes from09D410 to CGRAM0 (128 colors). Nonzero page overwrites color40..5F
with40bytes from09D158 (32 colors); upper CGRAM80..FF remains untouched.

Standard-panel wrapper B426 always installs same standard map09CF60 twice at
4121/4131 via B3EA/B412 descriptors. Heading wrapper used shop additionally draws
09CDA0 at4020 viaB3C2. Heading32x7 map, standard14x18 maps (1F8 bytes). The second
standard descriptor remaps source palette4→5, so loader preserves descriptors,
not only raw map bytes. All calls use native rendering rectangle semantics.

Conditional joined variants from B2DB: first joined bit picks continued09D198
map withB3D6 at4121 and16-color override at CGRAM40:09D390 page0,09D3D0 nonzero.
Second joined bit picks same map withB3FE at4131,16colors at CGRAM50:09D3B0 page0,
09D3F0 nonzero. Clear4800 inclusive401words follows. All four joined-bit combinations
and both page palette choices have contracts. Physical4D3D0..4D3F0 is an actual
consumer even though the primary inventory assigns its region elsewhere; keep
cross-owner loading, do not omit due archival area boundary.

$B457 inline driver heading is conditional entry only (no natural incoming call
established), sets X16 and renders FF-terminated twelve-byte stream physical3463
atVRAM408A, status-preserving text callee. It authorizes no player diagnostic UI;
it remains a source implementation/conditional rendering requirement. Shared
mechanism is presentation::memory::draw_text, not a separate font renderer.

Bounds: loader must take complete maps/palette/tile/default/grid/text/record spans
and reject truncation transactionally for native overrides; source assumes ROM
is valid. These are explicit native validation differences, not discovered source
error-handling branches. Original bytes remain runtime ROM-backed, no original
asset payload embedded in authored code. Tests use local ROM plus literal expected
branch/coordinate/format results. Synthetic malformed assets exercise interfaces.

## Coverage, cross-area and limits

Controls owns direction lookup and device sample translation; names owns repeat,
join/B history and stateful service. Rendering owns font/map/pairedtext/OAM/clear
mechanisms and visual staging; names owns consumers, palette variants and UI
composition. Transitions owns fades/window contraction; names retains exact call
ordering and suspended continuation. SessionRecords.truck_names is reused, not a
second records state. Upcoming shop/results must share this Session and history
lifetime; source history2 also doubles as pause owner. Implemented native rules
must preserve those transitions without introducing an emulated RAM map.

No source runtime/QA has been executed. No original source changed. Source does
not explicitly initialize history/repeat at page setup; deterministic cold-state
choice remains an implementation assumption, while warm retention is established.
Page limits beyond1 have no established consumers and are not an invented
extension interface. Native track additions remain unrelated to frontend count.
All test behavior remains pending, with actual compile/list/run outcomes recorded
in checks/contracts/names-results.json. Initial fixture match-arm type mismatch
was corrected without changing expected values before the recorded successful
compile. No knowingly failing behavior is claimed passed.

## All primary review ranges

- region-18 physical$003257..$00325A; begin_driver_graphics_upload; game.asm:11708.
- region-32 physical$00325A..$003296; upload_driver_frontend_tiles; game.asm:11720.
- region-19 physical$003296..$0032AE; configure_driver_palette_dma; game.asm:11755.
- region-33 physical$0032AE..$0032DB; upload_driver_frontend_palette; game.asm:11775.
- region-20 physical$0032DB..$0032E4; prepare_shop_graphics_and_heading; game.asm:11807.
- region-324 physical$0032E4..$0032EC; select_driver_heading_map; game.asm:11821.
- region-325 physical$0032EC..$003304; draw_driver_heading_and_test_first_panel; game.asm:11834.
- region-770 physical$003304..$003319; draw_alternate_frontend_rectangle; game.asm:11855.
- region-326 physical$003319..$003347; draw_continued_first_driver_panel; game.asm:11873.
- region-391 physical$003347..$003357; upload_alternate_continued_panel_palette; game.asm:11902.
- region-327 physical$003357..$00337A; draw_second_driver_panel; game.asm:11919.
- region-392 physical$00337A..$00338C; draw_continued_second_driver_panel; game.asm:11943.
- region-771 physical$00338C..$0033B8; load_second_page_palette; game.asm:11961.
- region-328 physical$0033B8..$0033C2; clear_frontend_text_map_and_return; game.asm:11991.
- region-69 physical$0033C2..$0033D6; driver_heading_rectangle_descriptor; game.asm:12004.
- region-330 physical$0033D6..$0033EA; continued_driver_rectangle; game.asm:12020.
- region-71 physical$0033EA..$0033FE; driver1_panel_rectangle_descriptor; game.asm:12036.
- region-394 physical$0033FE..$003412; alternate_player2_shop_rectangle; game.asm:12052.
- region-73 physical$003412..$003426; driver2_panel_rectangle_descriptor; game.asm:12068.
- region-21 physical$003426..$003431; prepare_standard_driver_panels; game.asm:12087.
- region-34 physical$003431..$003457; draw_two_standard_driver_panels; game.asm:12102.
- region-1316 physical$003457..$003463; draw_inline_driver_panel_heading; game.asm:12128.
- region-1317 physical$003463..$00346F; inline_driver_panel_heading_tiles; game.asm:12141.
- region-24 physical$003B5B..$003B81; begin_player_selection; game.asm:13516.
- region-38 physical$003B81..$003C3F; initialize_name_entry_pages; game.asm:13543.
- region-1007 physical$003C3F..$003C48; draw_alternate_name_page_extra_text; game.asm:13633.
- region-39 physical$003C48..$003CB7; run_name_entry_frame; game.asm:13647.
- region-776 physical$003CB7..$003CC2; send_frontend_sound_e8; game.asm:13703.
- region-816 physical$003CC2..$003CCB; send_name_entry_audio_ack; game.asm:13717.
- region-40 physical$003CCB..$003D75; commit_names_and_tick_join_countdown; game.asm:13732.
- region-110 physical$003D75..$003D7D; hide_first_name_cursor_pair; game.asm:13821.
- region-41 physical$003D7D..$003D8C; clear_completed_second_name_panel; game.asm:13834.
- region-111 physical$003D8C..$003D94; hide_second_name_cursor_pair; game.asm:13850.
- region-42 physical$003D94..$003DC2; test_first_player_join_edge; game.asm:13864.
- region-75 physical$003DC2..$003DEC; show_first_joined_name_grid; game.asm:13896.
- region-43 physical$003DEC..$003E19; test_second_player_join_edge; game.asm:13924.
- region-777 physical$003E19..$003E22; draw_second_name_page; game.asm:13956.
- region-817 physical$003E22..$003E43; initialize_name_entry_cursor_sprites; game.asm:13969.
- region-44 physical$003E43..$003E7F; handle_first_player_name_input; game.asm:13994.
- region-76 physical$003E7F..$003EA1; wrap_first_name_grid_cursor; game.asm:14035.
- region-45 physical$003EA1..$003ECA; position_first_name_grid_cursor; game.asm:14065.
- region-77 physical$003ECA..$003F16; edit_first_player_name; game.asm:14096.
- region-46 physical$003F16..$003F51; handle_second_player_name_input; game.asm:14141.
- region-778 physical$003F51..$003F73; wrap_second_name_grid; game.asm:14181.
- region-47 physical$003F73..$003F9C; position_second_name_grid_cursor; game.asm:14211.
- region-779 physical$003F9C..$003FE8; edit_second_player_name; game.asm:14242.
- region-1008 physical$003FE8..$003FF2; temporary_name_defaults; game.asm:14284.
- region-1009 physical$003FF2..$00405B; name_editor_cell_tables; game.asm:14306.
- region-56 physical$048000..$04CDA0; driver_frontend_tiles; game.asm:19846.
- region-70 physical$04CDA0..$04CF60; driver_heading_tilemap; game.asm:19855.
- region-72 physical$04CF60..$04D158; driver1_panel_tilemap; game.asm:19864.
- region-58 physical$04D158..$04D198; alternate_driver_palette; game.asm:19873.
- region-331 physical$04D198..$04D390; continued_driver_tilemap; game.asm:19882.
- region-329 physical$04D390..$04D3B0; continued_driver_palette; game.asm:19891.
- region-434 physical$04D3B0..$04D3D0; second_page_palette_zero; game.asm:19898.
- region-435 physical$04D3F0..$04D410; second_page_palette_one; game.asm:19933.
- region-57 physical$04D410..$04D510; driver_frontend_palette; game.asm:19961.

Stage4 implementation: actual named ROM assets/editor/driver variants plus cloneable real fade/window and sampled-input continuation owners. Existing synthetic shop fixtures enriched for added fields (assertions unchanged). Countdown draws its established two glyphs without requiring unrelated synthetic temporary tail initialization; original natural defaults provide FF, while semantic supported counter API retains tail bytes. Initial31/5 implementation failures corrected; final36/36 passes. Game orchestration still pending.

Stage4 shared clear correction: continued driver wrapper33B8 (game.asm11994) calls1C0D, whose7272 LDA0000/0401word loop proves4800..4C00 zeros, not2000. Corrected native copy and the two fixture literal expectations; previous36pass under erroneous literal retained in result history. Explicit name-panel clear3B3D still writes2000 and is unchanged.
