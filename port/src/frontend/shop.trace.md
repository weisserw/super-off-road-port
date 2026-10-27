# Economy independent source trace

Reference: frozen Round29 `reference/source/game.asm`, SHA256
4e3e1a45e4409ce63a3354740a2cdd3962cd3d9975d88d22ecee40c9d29ded7f.
Physical bank-zero offsets below execute at CPU $008000+offset, DB00/DP0000
where the source has an established natural caller. Encoding widths are explicit
65816 M/X widths, not a generic integer model. Toolkit SNES/platform guides
`/root/romatlas/docs/development.md`, `snes.md`, `snes-native.md` consulted.
Executable contracts are `frontend/shop/tests.rs`; functions in shop.rs are
explicit pending stubs, not implementations or reference oracles.

## Shared owners and actual callers

`attract::Session` is the single wallet/selected-wallet/session-record owner.
`Shop.selection` reuses the existing names Selection/Editor retained history and
repeat fields by moving the value into shop, rather than clearing on entry.
Truck owns stock/upgrades; WorldObjects owns pending awards. Results will call
wallet addition while settling that same pending array, not a second counter.
Records colocated with money initializer are owned results; all six default
arrays and copy loops were inspected, not treated as economy implementation.
Warm frontend initialization resets wallets and stock but preserves records;
cold Game reset is a separate domain operation.

Natural callers: attract entry calls money initializer; attract overlay calls
normalization; shop refresh normalizes both visible-page wallets before sampling.
Results podium adds rank award to digit3; pending settlement adds nonzero pending
award to digit3 then clears the same original array. High-score insertion also
normalizes each wallet before comparison. Digit4/5 addition routines have no
established natural references in source, so their contracts are conditional
entry semantics only. No player-accessible features are added.

## Wallet layout, initializer and addition ($361F, $36E8..$3774)

Initializer REP30 zeroes twelve words at offsets22,20..0, hence all24 wallet
bytes. It clears selected_wallet and writes1 into offsets2,8,14,20: each record
is [0,0,1,0,0,0]. Record copy is a separate internal entry; wallet init returns
before it. Tests preserve SessionRecords and unrelated frontend fields.

Record offsets table is [0,6,12,18]. Ordinary add uses `$0563+offset`, stored
digit3, NOT digit5. Conditional entries use $0564/$0565. Each saves X at incoming
width, PHP/SEP30 forces byte body, CLC ADC and store wraps byte; normalization is
separate. PLP restores width before PLX; PLX replaces N/Z and SEP30 discards Y
high, so these helpers are not general flag/Y-preserving functions. Native APIs
express the value effect rather than registers. Decimal flag is clear in actual
results calls. Conditional public addition has that explicit precondition.

Displayed six digits have two appended decimal zeros: digit3 is a $10000 unit,
digit4 $1000 and digit5 $100. `AwardDigit` therefore names TenThousands,
Thousands and Hundreds. An early draft used ambiguous Thousands for digit3;
corrected from display arithmetic before final handoff, original execution logs
preserved. Initial $001000 stored digits display $100000.

## Byte normalization and highest borrow ($3733..$3774)

Select record once, start digit5 and walk to0. CMP #10 establishes C and N:
A<10 stores; otherwise BPL uses the N of A-10, NOT the sign bit of A.
Thus $0A..$89 subtract10/repeat, but $8A..$FF add10/borrow/repeat. A=$80
subtracts ten12 times→8 and increments previous digit12; A=$89→7,+13.
A=$8A adds ten12 times modulo256→2 and decrements previous digit12.
Fixed low vectors with original digit2=1:
$80→[0,0,1,1,2,8], $89→[0,0,1,1,3,7],
$8A→[0,0,0,8,8,2]. These are manually derived source fixtures, not computed
by a second algorithm in the tests.

For carry, CPY #1 suppresses preceding increment only at highest digit0: highest
10→0, discarded overflow, no arbitrary precision saturation. Borrow has no such
guard: `DEC selected_money_truck,X` is exactly preceding byte because selected
address055F is one byte before wallets0560. On record0 highest borrow decrements
selected_wallet; later records decrement previous record's digit5, which this
pass does not revisit. Native function accepts Session owners, not a memory map.
Contract examples highest FF→9 and selected0→FF; wallet1 highestFF→9 and previous
wallet5 zero→FF. Natural valid affordability prevents negative total; malformed
or conditional snapshots can expose it. Explicit residual uncertainty retained;
no claim this is naturally player reachable, no silent clamp or zero substitute.
Rejected invalid selected-wallet index is transactional native validation.

## Inactive conversion ($2C5C..$2CA7)

PHP/SEP30. Visit trucks3→0 with active-mask8→1, skip any joined slot. Build
record offset6*truck in byte arithmetic. OR ONLY stored digits0..3; ignored4/5
can remain nonzero when conversion stops. Crucially DEC digit3 occurs BEFORE
nitro CMP80. Already capped truck loses one raw digit3 unit, returns without
normalization or selected_wallet change. With nonzero upper balance and digit3
zero, it leaves FF in digit3. Source comment saying refresh display is wrong:
call is normalization only, no draw. If stock<80, SED CLC ADC1 increments BCD,
CLD, selects truck and normalizes, then repeats. Stock79/balance3 produces80
with balance1: first unit buys stock then next iteration decrements before cap
exit. Stock09/balance2 produces11 and zero. Descending completion leaves last
actually normalized truck selected. PHP/PLP preserves caller status, not extra
frontend state. No wrapping conversion of the last two digits is invented.

## Cost, stock and affordability ($2E88..$2ED2, $2FDB..$3024)

Item index=2*row+column,5 branches to done before tables. Source item array stride
4 from nitro061B is **nitro, acceleration, steering, speed, suspension**, per
symbols061F,0623,0627,062B. Costs[1,8,4,10,6], caps[99BCD,6,6,6,6]. An initial
draft's intuitive steering/speed ordering was wrong; fixed all item/bar fixtures
and enum from actual address consumers, retained economy-initial logs/results.

Page0 local players0/1 map truck0/1 and wallets0/1. Nonzero page chooses base+2
trucks and +12 wallet bytes; source supports two frontend pages, not a new menu.
For funds, OR digits0..2; any nonzero means sufficient, else compare digit3>=cost.
Digits4/5 do not participate. Cap rejection is unsigned >= and does not debit.
Successful stock byte increments under SED, then CLD; subtract cost under SEC
binary from digit3, storing raw byte, with no immediate normalization. E.g.
upper balance1/digit3zero purchasing speed10 storesF6, later refresh borrows.
Exact cost8 acceleration stores0; nitro09→10. Other fields remain unchanged.
No purchase audio command appears in either handler; source only entry/completion
commands are emitted. Invalid local player is transactional native validation.

## Input, clamped geometry and shared history ($2DF0..$304F)

Done player returns before mutating repeat or history. HIGH controller byte zero
clears repeat; low-byte buttons alone count as released. Any nonzero high byte
consumes countdown even when no direction. Repeat0 reloads8 and moves if low
nibble!=0; eight following calls decrement without movement, next moves:9 calls.
Use original DirectionTables; opposing combinations are not normalized axes.

Despite routine labels/comments saying wrap, inspect branches:
column sum negative→$3034 LDA0; column>=2 falls $3030 LDA1. Row sum negative
branches $3049 LDA0; row>=3 falls $3045 LDA2. Thus actual clamp0..1/0..2,
NOT modulo wrap. Conditional byte values still use encoded signed branch order.

After movement ALWAYS perform B edge: ASL input-high puts B in carry; ROR shared
history; BIT history sets N/V from history7/6. Require N=1,V=0. Start does not
choose done; only B at item5. Retained history can suppress initial held B.
First done path waits NMI BEFORE drawing GET READY at4903, setting bit1 and
hiding nine Ys. Second path immediately draws4913/setsbit2/hides, no extra wait.
NMI-resumable first wait must retain sampled second input and skip resampling.

Geometry index=5*row+column into original names grid tables (12 bytes suffice
for cells0,1,5,6,10,11); x=3*tableX+10 or90, y=2*tableY+60, all byte wrapping.
Nine offsets source order8→0 map to destination records8→0, preserve tile bytes.
Second OAM group base24; coordinates wrapping is independent byte X/Y, not
packed word XY. Finalrow/col1 anchors(64,160)/(192,160), last record adds32.

## Page, redraw and displayed money

Page setup increments inhibit, disables window masks, calls real driver tiles/
heading/palettes helper (names contracts own variants), fonts/clear. It clears
columns/rows and sampled HIGH bytes only, keeps LOW inputs, histories/repeats,
sets done=joined_pages[page] XOR3. BG12NBA40 means BG1tilebase0,BG2byte8000.
Map clears4800/4400 inclusive401words, headings4904 use appropriate page stream
25bytes throughFF. Cursor9 tilewords written both groups. Refresh money then
six preincrement redraw calls (source counter5 down0), clear inhibit, audio0301,
real fadein. Actual install tests require VRAM/OAM/bar/session mutation, not a
phase-only mock.

Redraw cursor is WORD, increments then if>=5 sets0; FFFF→0 before comparison.
Each local not-done player reads selected field from correct truckpair. Zero
stock leaves stale bar unchanged; cap>=7 to6 for render only. Two units emit283A
then destination-=20word and count-=2; odd tail283B. Five yields pair,pair,odd;
six gives3pairs. No clearing of unused rows occurs.

Refresh normalizes both page*2 and page*2+1 regardless joined mask, leaves second
selected. Draw only joined bits at4964/4974. Draw_money first clears9words2000.
Leading zeros make NO VRAM write or advance, so significant digits compact left.
Once significant, digit+1 with attr20; then two2001 zeros. Allzero field emits
just00 atfirsttwo positions; rest remains blank. This is not right alignment.

## Full NMI orchestration and transition handoff

Loop waits then refreshmoney→onebar→sample JOY1/2 via controls→firsthandler→
secondhandler→donecompare. Thus successful purchase's debit/stock renders on
next frame, not same frame, and both frontend pages still sample JOY1/2. Contract
requires actual Truck/Session/VideoMemory updates with predebit money tiles.
Doneboth hides frontend sprites, waits one frame, increments inhibit, command
E803, expands/bounces actual frontend window. After return ACK0101 then byte
page++: less OR equal page_limit reenters setup; greater returns. Source sets
SEP20 before comparison: no invented adjacent-byte limit. Real Fade/Window
optional state lives in Shop for resumability; full Game must compose immutable
transition assets and all owner snapshots in stage4, not replace with phase logs.

No implementation, emulator experiment, runtime QA or toolkit edit performed.
No exclusions or source-ledger dispositions changed. Known code/data coverage
below is navigation to independently inspected primary entries, not proof that
pending implementation exists. Conditional finer-awards/borrow need honest
source-informed implementation policy and remain explicitly unverified runtime.

## Exhaustive primary navigation
- region-304: physical $002C5C..$002CA7, game.asm:10362: convert_inactive_truck_money_to_nitro.
- region-16: physical $002CA7..$002CB9, game.asm:10414: prepare_player_shop_frontend.
- region-321: physical $002CB9..$002CF6, game.asm:10433: initialize_shop_players_and_cursors.
- region-756: physical $002CF6..$002CFB, game.asm:10468: select_second_shop_page.
- region-322: physical $002CFB..$002D18, game.asm:10480: clear_shop_maps_and_choose_heading.
- region-757: physical $002D18..$002D1E, game.asm:10502: select_alternate_shop_rectangle.
- region-323: physical $002D1E..$002D57, game.asm:10515: draw_shop_display_and_fade_in.
- region-355: physical $002D57..$002D59, game.asm:10554: set_shop_loop_widths.
- region-92: physical $002D59..$002D74, game.asm:10565: run_shop_input_frame.
- region-184: physical $002D74..$002D87, game.asm:10585: leave_completed_shop_page.
- region-356: physical $002D87..$002D9E, game.asm:10602: advance_shop_page.
- region-390: physical $002D9E..$002DA1, game.asm:10621: reload_next_shop_page.
- region-357: physical $002DA1..$002DA2, game.asm:10632: return_completed_shop.
- region-997: physical $002DA2..$002DB4, game.asm:10640: shop_continue_cursor_tilewords.
- region-998: physical $002DB4..$002DF0, game.asm:10663: text_glyphs_2db4.
- region-93: physical $002DF0..$002DFB, game.asm:10691: test_player_one_shop_done.
- region-758: physical $002DFB..$002DFC, game.asm:10707: return_shop_handler.
- region-94: physical $002DFC..$002E1C, game.asm:10719: repeat_player_one_shop_direction.
- region-185: physical $002E1C..$002E4B, game.asm:10745: move_player_one_shop_cursor.
- region-95: physical $002E4B..$002E66, game.asm:10779: detect_player_one_shop_confirm_edge.
- region-186: physical $002E66..$002E88, game.asm:10802: finish_player_one_shop_selection.
- region-96: physical $002E88..$002E90, game.asm:10829: return_or_select_player_one_money_record.
- region-759: physical $002E90..$002E92, game.asm:10845: select_second_wallet.
- region-97: physical $002E92..$002E9D, game.asm:10856: test_player_one_purchase_funds.
- region-760: physical $002E9D..$002EA6, game.asm:10871: check_low_wallet_digits.
- region-98: physical $002EA6..$002EB0, game.asm:10885: select_player_one_upgrade_slot.
- region-761: physical $002EB0..$002EB3, game.asm:10902: advance_shop_redraw_field.
- region-99: physical $002EB3..$002ED1, game.asm:10915: buy_player_one_shop_item.
- region-248: physical $002ED1..$002ED2, game.asm:10940: return_rejected_player_one_purchase.
- region-187: physical $002ED2..$002EFE, game.asm:10951: expand_first_shop_cursor_sprites.
- region-762: physical $002EFE..$002F2A, game.asm:10989: build_shop_cursor_sprites.
- region-999: physical $002F2A..$002F3C, game.asm:11023: cursor_coordinate_deltas.
- region-1000: physical $002F3C..$002F46, game.asm:11048: shop_item_costs_and_caps.
- region-100: physical $002F46..$002F52, game.asm:11076: test_player_two_shop_done.
- region-249: physical $002F52..$002F8F, game.asm:11093: repeat_and_move_player_two_shop_direction.
- region-763: physical $002F8F..$002FA1, game.asm:11134: position_second_shop_cursor.
- region-250: physical $002FA1..$002FC9, game.asm:11152: detect_player_two_confirm_and_finish_slot.
- region-764: physical $002FC9..$002FDB, game.asm:11181: finish_second_shop_selection.
- region-251: physical $002FDB..$003024, game.asm:11203: buy_player_two_shop_item.
- region-188: physical $003024..$003030, game.asm:11254: update_shop_column_with_wrap.
- region-765: physical $003030..$003034, game.asm:11271: set_first_player_done_bit.
- region-252: physical $003034..$003036, game.asm:11283: wrap_shop_column_to_zero.
- region-189: physical $003036..$003045, game.asm:11294: update_shop_row_with_wrap.
- region-253: physical $003045..$00304B, game.asm:11311: choose_shop_row_wrap_value.
- region-190: physical $00304B..$00304F, game.asm:11325: store_shop_row_and_return.
- region-101: physical $00304F..$00307D, game.asm:11338: cycle_shop_upgrade_display_field.
- region-766: physical $00307D..$00307F, game.asm:11372: advance_upgrade_field.
- region-102: physical $00307F..$003097, game.asm:11384: choose_shop_upgrade_bar_destination.
- region-767: physical $003097..$00309A, game.asm:11406: cap_upgrade_bar_six.
- region-103: physical $00309A..$0030A2, game.asm:11417: test_shop_upgrade_bar_count.
- region-768: physical $0030A2..$0030B2, game.asm:11431: draw_upgrade_bar_pair.
- region-104: physical $0030B2..$0030C0, game.asm:11449: draw_shop_bar_end_and_test_player_two.
- region-769: physical $0030C0..$00310B, game.asm:11466: draw_second_page_upgrade_bar.
- region-105: physical $00310B..$00310D, game.asm:11515: return_shop_field_redraw.
- region-1001: physical $00310D..$003121, game.asm:11524: shop_upgrade_bar_destinations.
- region-23: physical $00361F..$003688, game.asm:12564: initialize_money_and_record_defaults.
- region-59: physical $003688..$0036B8, game.asm:12628: default_track_record_names.
- region-60: physical $0036B8..$0036D8, game.asm:12635: default_track_record_times.
- region-64: physical $0036D8..$0036E4, game.asm:12642: default_truck_names.
- region-1002: physical $0036E4..$0036E8, game.asm:12649: truck_money_record_offsets.
- region-1314: physical $0036E8..$003701, game.asm:12674: add_award_to_money_digit_five.
- region-1315: physical $003701..$00371A, game.asm:12699: add_award_to_money_digit_four.
- region-294: physical $00371A..$003733, game.asm:12724: add_award_to_money_digits.
- region-35: physical $003733..$003774, game.asm:12749: normalize_six_money_digits.
- region-106: physical $003774..$003791, game.asm:12800: refresh_two_players_money.
- region-774: physical $003791..$003795, game.asm:12823: advance_placement_record_pair.
- region-107: physical $003795..$0037B4, game.asm:12835: draw_money_for_joined_players.
- region-775: physical $0037B4..$0037C4, game.asm:12860: draw_second_placement_record.
- region-108: physical $0037C4..$003815, game.asm:12878: draw_six_digit_money_field.
- region-61: physical $003815..$003851, game.asm:12927: default_high_score_digits.
- region-62: physical $003851..$00386F, game.asm:12934: default_high_score_names.
- region-63: physical $00386F..$003879, game.asm:12941: default_high_score_order.

## Stage4 source-backed fixture corrections
Initial41/43: synthetic SharedGraphics cursor_palette supplied16colors, while original upload_frontend_cursor_tiles_and_palette game.asm14873..14904 loops0040bytes (32colors). Enriched fixture32, preserving strict source span and assertions. Bar second-player fixture used4A1D,49DD,49BD; actual SBC0020 loop game.asm11491..11502 yields4A1D,49FD,49DD. Corrected only latter address expectations, retaining six-unit cap/three pair assertions; implementation unchanged. Names resources added original frontend shape and explicit cloneable continuation fields to existing synthetic fixtures, with original behavior assertions retained.
