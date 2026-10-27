# Source-backed transition contracts

Stage3 worker `/root/behavior_contracts/transitions`; no behavior implemented.
Reference frozen Round29 `reference/source/game.asm`, SHA256
`4e3e1a45e4409ce63a3354740a2cdd3962cd3d9975d88d22ecee40c9d29ded7f`.
Physical bank0 offsets below map CPU `$008000+offset`; data-bank pointers use
LoROM translation. Tests live `transitions/tests.rs`, pending interfaces in
`transitions.rs`. Planning descriptions were independently checked against
instructions, real ROM tables and all cross-call contracts.

## Platform and ownership

Read toolkit docs/development.md platform table, docs/snes.md and
snes-native.md. Hardware specification consulted:
https://problemkaputt.de/fullsnes.htm (I/O MPYM, Mode7 signed multiplication,
indirect HDMA, byte/word 65816 widths). M7A is signed16, M7B signed8, product
signed24; this source reads MPYM only, bits8..15, never an unrestricted scaled
integer. Arithmetic before byte stores uses current M width, not operand suffix
alone. PHP/PLP restores width/status; SEP index8 truncates index high bytes.
SNES window bounds are inclusive; [129,128] closes a line. HDMA `$FF` describes
127 repeated transfers, not256; two entries cover254 lines, exceeding visible224.
No emulator/GUI/QA used. Native scheduling resumes at source NMI boundaries;
NMI counters/heartbeat/audio continue during fades/windows/pause, race motion
only runs in its actual race dispatcher. Runtime and Game own scheduler/reset/
snapshot integration; transition mutable buffers/phase/history must clone fully.

Presentation owns actual Display/VideoMemory. Race owns Progress/banner/HUD;
rendering owns rectangle copy/animations. `banner_calls`/`animation_calls` are
non-draining scheduling observations, never a replacement for actual shared
owner invocation in stage4. Tests require actual window, countdown VRAM,
inhibit/display and phase mutation rather than only mocked phase reports.
`TrackPresentation.countdown` carries explicit center/destination for added
named tracks; original-layout table bounds16 do not bound the catalogue.

## Fades and waits ($001A24..$001A78)

`fade_out_fade_in_and_wait_frame`, `fade_display_in`, `wait_next_nmi_frame`
use byte counters. Each fade first waits for independent NMI counter to change,
then sets animation counter0. Inner loop captures OLD animation count and waits
for a different value; brightness uses the captured value, not new NMI count.
Fadeout captures0..15 => brightness15..0 then writes80 forcedblank. There are
16 update boundaries plus leading wait, not16 total. Fadein installs enabled
mask in both TM/TMW after leading wait, captures0..30 => floor(count/2)&15,
finishes at15 on captured30. Thus31 updates plus leading wait32. No busy host
wait is permitted; Fade stores primed/complete/counter. Tests cover both endpoints,
forcedblank and mask setup. NMI-disabled source cannot complete a wait; native
scheduler must not manufacture a frame boundary.

Conditional `wait_x_frame_boundaries_preserving_status` waits then DEX/BNE,
so nonzero count waits exactly that many; zero wraps256 in X8 or65536 in X16.
Incoming width/context/natural caller unresolved. API names CountedWaitWidth;
no new natural gameplay route inferred.

## Frontend window frame families ($002275..$002433)

`draw_attract_overlay_each_frame` writes brightness8, normalizes the SELECTED
six-digit wallet via economy, writes15, then polls Start/timeout. Its BPL loop
uses poll's returned sign (timer word decrement). Tests normalize [0,0,0,0,0,15]
→[0,0,0,0,1,5], check brightness event8→15 and timer0→FFFF. Poll at `$002C38`
ORs Start from ALL4 devices, but only signature FIRST BYTE nonzero triggers
stack-unwind frontend reentry; otherwise even Start decrements timer. Attract
worker owns actual outer signature/poll/scene flow; economy owns digit overflow.
`allow_window_start_abort` sets flag1, expands, polls again; preserve final poll
when frontend/attract composes this helper.

Scrolling init (`initialize_frontend_window_motion`) sets center[E0,80], masks
[03,00,00]. `animate_scrolling_window` timer100, extent0, setup, BG1V zero,
brightness15, velocityFE(-2). Each frame wait→scroll from OLD centers→optional
JOY1/2 Start abort→build OLD extent→grow max(extent>>4,1)→X-- byte→Y+=velocity.
Scroll is BYTE negative of (center+offset): horizontal offset7C, vertical78;
high scroll write zero. Example centerE0/80 => scrollA4/08. Bounce only equality
Y40/A0; negate velocity byte, no inequality clamp. X0→FF wraps. Extent high>=4
clears actual masks but does not blank or disable HDMA. Timer is not decremented
inside this loop; later poll owns it.

Expanding init center[80,B0], masks[22,22,22], velocityFE, extent0, shape0.
Per frame wait→BUILD→optional Start→grow max(extent>>4,1), Y--, X+=velocity;
X40/B0 equality bounces. Stop high>=4→forcedblank→clear masks.
Frontend contracting init center[80,70], masks[03,03,00], extent0400, shape0.
Wait→optional Start→build→subtract max(extent>>4,1). **Divides16**, unlike race
closure divide4. STORE subtraction evenFFFF then BPL high; negative finishes
forcedblank/mask clear. Example0100→00F0 and0000→FFFF.

Important source correction: all THREE enabled Start aborts jump directly RTS,
SKIPPING normal blank/mask cleanup. Scrolling abort precedes build, contracting
precedes build, expanding follows build. Tests assert retained masks/HDMA,
brightness, center/extent and selector ordering. Only JOY1/2 Start is used here;
JOY3/4 do not abort these loops. Frontend setup schedules two NMI waits and no
ordinary geometry growth until primed.

`disable_window_masks` at B46F clears hardware W12SEL/W34SEL/WOBJSEL only.
It does NOT clear retained configuration bytes0552..4, disable HDMA, blank,
modify geometry or scroll. Initial fixture assumption that Window.masks cleared
was corrected before final evidence: these are source CONFIGURATION, active
Display.masked_layers alone clears. No implementation output informed fixtures.

## Shape loading and ping-pong geometry ($00347D..$00361F)

Directory B616 has THREE records bank+LE address, stride3:08:E29A/E39A/E49A,
physical04629A/04639A/04649A, each256 bytes=128 signed edge pairs. Byte directory
is not LE24 in address/bank order. Selector0/1/2 from inspected frontend/race/
pause callers proves ownership; unrelated preceding046260..04629A tilewords
are not consumed by shape loading. No role invented for those29 words; rendering
uncertainty remains pending. ROM fixtures assert actual first/last/middle pairs.

Setup resets selector0; channel7 indirect mode41, port26, pointer bank0,
indirect bank7E. Wait, build twice, wait, install retained masks, HDMA80.
Each build toggles selector=(old+1)&1. Selector1 chooses descriptor+7 which
DISPLAYS8208 but initializes/writes8000. Selector0 displays8000 but writes8208.
Thus current build is opposite displayed buffer: one-build latency. Setup two
builds primes both before enable. Native buffer index0=8000,index1=8208.
Initialization writes256 closed pairs[81,80] backward from pair255. Remaining
native260-row tail is retained until actually written. Initial fixture full260
buffer equality corrected: setup equality required only initialized256, tails
may remain different. Descriptors contain [FF,8000,FF,80FE,00] and
[FF,8208,FF,8306,00], not ROM pointers or256-line counts.

Rows0..127 consume paired bytes. Distance=abs(row-64), multiply signed extent
by distance; use MPYM byte. Row<64 subtract from centerY and skip on BORROW;
row>=64 add and skip on CARRY. Example extent0100 centerY112 maps48..175;
centerY0 clips top, centerY255 clips bottom, no wrapped vertical writes. Extent0
collapses all rows at center. Extent0400*distance40 produces010000, MPYM00,
not256: tests lock wrap. Each accepted row repeats edges over4 rows; later
source pairs overwrite overlaps. After last output, closes following3rows,
so only last sample's first row survives. Last accepted row remains X when
later samples skipped. At y255, repeated tail writes may reach258, not overrun
native260 rows. Wrong-size native buffers reject before partial mutation.

Positive edge sample: MPYM+centerX, clampFF on carry. Negative: byte negate,
MPYM from product with that byte, centerX-MPYM. Left underflow clamps1, right
underflow0. Example center2,extent0100,sample-16=>[1,0]; center250,+16=>FF.
Fraction truncation occurs BEFORE subtraction (extent80,-3=>center-1).
Special -128 remains80 after negate, still signed -128 at M7B; extent80 yields
productFFC000, MPYMC0, center80-C0 underflows→[1,0]. No unsigned128 multiply.
Pure helper tests use fixed independent vectors, not second oracle loops.

## Countdown drawing and layout ($0055EE..$0056E1)

All16 centers/destinations are source table fixtures. State selects stride3
records at D667: address little endian THEN bank, unlike shape directory.34
records, including state11..14 sameD7F6. Load full98-byte7x7word consumers,
including frame0..5 overlapping earlier HUD union (regions not independent
consumer boundaries), not just assigned short prefixes. Descriptor8007/0007
bypasses remapping; pause5x3 descriptor normal remapping. Synthetic tests use
actual MapImage words and verify VRAM destination.

Draw CURRENT state first, DEC delay WORD, BPL return; else delay=5,state++.
Initial6 produces SEVEN draws, subsequent5 SIX draws. Source summaries saying
six are correct only after first reload. Delay8000→7FFF does NOT advance.
Invalid native state/map/layout must error atomically (supported extension
validation, not claim original rejected corrupt memory). Added named layouts
carry explicit center/destination without original table index.

## Ready/postrace resumable control ($005429..$0055EE)

Both select track center, masks[22,0,22],shape1,extent0,color math53 =>
math layers13/half=true. Ready leading wait writes80 then0F brightness;
state0,delay6. Human audio0E01 gate reads WORD DP06+07 while M16: named API
human_gate_word comprises active_human_mask(low),shifted_human_mask(high), not
RAM dump. Nonzero high alone therefore still requests ready audio. Normal
caller zero/width conditions are traced separately, no broad high-byte claim.
Ready preserves inhibit. SEP30 X19 DEX/BPL waits26 boundaries BEFORE setup2
waits, then extra wait draws countdown/banner once. Grow loop wait→build→if
extent<70 addmax(extent>>4,1); countdown/banner does NOT advance during growth.
Can overshoot threshold (109+6=115), no cap. Once reached, another waiting loop
draws countdown/banner until state>=11 (17); then close. Close each frame draws
only while state<12 (18), builds, subtractsmax(extent>>2,1). If result negative,
DO NOT STORE (extent0 stays0), wait→clear inhibit+HDMA→wait→return. Tests lock
state threshold, actual map, old buffer build, divisor and two final boundaries.

Postrace leading wait sets math53, center/masks, clears inhibit, audio0F01
UNCONDITIONALLY; state12(18),delay6,shape1,extent0. Setup2 waits, then first
wait/draw, second wait/draw/banner/16animations. Each growth continuation uses
that draw/banner/animation path; state>=22(34) wraps to18. Build old extent,
grow divide16 until>=70 then set timer0100. Hold waits/draw/banner/animations,
wraps state, DEC timer WORD until0 (zero initial would wrap). Then closure
waits/draw/banner with same wrap, NO sprite animation, build/shrink divide4.
Negative result not stored; final wait→HDMAoff→wait. Unlike ready closure this
exit does NOT clear inhibit (entry already cleared it). Tests distinguish kinds
and preserve actual display/memory/window effects. Race owns extra HUD timer
service before results presentation; transition must not fabricate timer guard.

## Human pause ($0056E1..$0057FA)

Running poll first handles existing owner (saved one-based byte shared with
frontend second-player history): read actual owner JOY high, held Start returns
without reentering pause. Release clears byte then scans candidates SAME call.
Joined pages bits1/2 map logical0/1 then2/3. Priority1→2→3→4, devicesJOY1/2/3,
FOURTH ALSO readsJOY3. Once owner4 saved, its wait readsJOY4, not triggering3.
No invented symmetrical fix. Source natural race caller established in race
poll; older archive 'unobserved' note does not exclude pause.

New owner saved→helper leading wait→math53,brightness80→0F,track center/masks,
INC BYTE inhibit (FF→00 source wrap), draw pause banner at countdown destination
+41 WORDS,shape2,extent0,setup2 waits. Each expansion frame wait/build; if>=60
return elseadd8. Thus12 increments then13th frame builds/checks60 before waiting
release/newpress. Controller owner index uses(owner-1)*2 into JOY1H. Busy loop
requires release THEN new press; unrelated Start doesn't resume. Resume waits
once, clears inhibit and HDMA, waits again then returns WITHOUT clearing owner.
Next ordinary poll keeps owner if stillheld, clears onrelease. Pending full
service interface tests require leading wait before real banner/inhibit mutation,
2 priming waits before growth, actual owner release/newpress and2 resume waits.
Invalid native owner5 rejects atomically, not source malformed-state behavior.

## Cross-area obligations and residual limits

Attract/economy must integrate overlay normalize+poll and final poll after abort
wrapper; names/results supply their own window callers. Rendering implements
actual masks/Mode1/rectangle decode and trace all shape-neighbor unknowns by
consumer. Stage4 must wire real banner and sprite-animation calls, not merely
increment observation counters. Game snapshots must include Fade/Window buffers/
phase/source histories. Source frame alignment QA remains pending. Header/toolkit
records unchanged; no asset bytes embedded. No behavior/ledger completion claimed.
Native malformed data and temporal mapping between busy polling and NMI remain
explicit source-informed domain validation/scheduling limits, not exclusions.

## Exhaustive assigned-region navigation

The following all96 regions were read with their callers and consumer tables;
this index alone is navigation, detailed derivations and tests above provide
behavior evidence. Ranges are physical half-open; source identities preserved.
- region-4 $001A24..$001A6F: fade_out_fade_in_and_wait_frame; game.asm:6986
- region-963 $001A6F..$001A78: wait_x_frame_boundaries_preserving_status; game.asm:7040
- region-31 $002275..$002287: draw_attract_overlay_each_frame; game.asm:8188
- region-736 $002287..$002293: allow_window_start_abort; game.asm:8206
- region-13 $002293..$0022AC: initialize_frontend_window_motion; game.asm:8222
- region-808 $0022AC..$002346: animate_scrolling_window; game.asm:8243
- region-175 $002346..$00237F: expand_and_bounce_frontend_window; game.asm:8329
- region-243 $00237F..$002389: allow_start_to_abort_window; game.asm:8362
- region-176 $002389..$0023B3: grow_window_extent_and_move_center; game.asm:8377
- region-244 $0023B3..$0023BC: reverse_window_horizontal_velocity; game.asm:8409
- region-177 $0023BC..$0023C3: test_window_expansion_limit; game.asm:8423
- region-245 $0023C3..$0023CE: blank_and_cleanup_window; game.asm:8436
- region-296 $0023CE..$0023FD: begin_contracting_window; game.asm:8453
- region-737 $0023FD..$002407: test_first_two_start_buttons; game.asm:8483
- region-297 $002407..$002433: contract_window_until_closed; game.asm:8498
- region-22 $00346F..$00347D: disable_window_masks; game.asm:12164
- region-191 $00347D..$003550: setup_hdma_window_and_build_scanlines; game.asm:12183
- region-305 $003550..$003554: skip_window_row_above_screen; game.asm:12289
- region-192 $003554..$00355D: place_window_row_below_center; game.asm:12301
- region-306 $00355D..$003561: skip_window_row_below_screen; game.asm:12315
- region-193 $003561..$00358E: scale_left_window_boundary; game.asm:12328
- region-772 $00358E..$003592: select_negative_window_edge; game.asm:12362
- region-194 $003592..$0035A1: scale_negative_left_boundary; game.asm:12374
- region-307 $0035A1..$0035A3: clamp_left_window_boundary; game.asm:12391
- region-195 $0035A3..$0035C4: store_left_and_scale_right_boundary; game.asm:12402
- region-308 $0035C4..$0035C8: clamp_right_window_boundary; game.asm:12424
- region-196 $0035C8..$0035D7: scale_negative_right_boundary; game.asm:12436
- region-773 $0035D7..$0035D9: select_zero_window_edge; game.asm:12454
- region-197 $0035D9..$003608: finish_window_boundary_pair; game.asm:12466
- region-396 $003608..$003616: window_hdma_descriptor_pair; game.asm:12491
- region-397 $003616..$00361F: window_shape_source_records; game.asm:12527
- region-791 $005429..$00548A: initialize_race_countdown_window; game.asm:17322
- region-821 $00548A..$0054B8: grow_countdown_window; game.asm:17368
- region-284 $0054B8..$0054CC: animate_postrace_countdown_frame; game.asm:17404
- region-792 $0054CC..$0054D2: restart_countdown_state_one; game.asm:17422
- region-285 $0054D2..$0054E8: wait_countdown_hold_and_begin_closure; game.asm:17434
- region-793 $0054E8..$0054EE: restart_countdown_state_two; game.asm:17454
- region-286 $0054EE..$005518: close_postrace_window; game.asm:17467
- region-370 $005518..$005580: begin_race_get_ready_window; game.asm:17499
- region-212 $005580..$005596: expand_race_start_window; game.asm:17551
- region-371 $005596..$005597: minimum_start_window_growth; game.asm:17572
- region-213 $005597..$0055B1: commit_start_window_growth_and_wait_countdown; game.asm:17583
- region-794 $0055B1..$0055EE: contract_race_window; game.asm:17605
- region-822 $0055EE..$0055FE: countdown_track_window_x; game.asm:17641
- region-823 $0055FE..$00560E: countdown_track_window_y; game.asm:17665
- region-824 $00560E..$00562E: countdown_track_vram_destinations; game.asm:17689
- region-214 $00562E..$00565C: draw_track_countdown_frame; game.asm:17718
- region-287 $00565C..$005665: advance_track_countdown_frame; game.asm:17749
- region-215 $005665..$005667: return_countdown_draw; game.asm:17762
- region-825 $005667..$0056CD: countdown_rectangle_sources; game.asm:17771
- region-826 $0056CD..$0056E1: countdown_rectangle_descriptor; game.asm:17862
- region-216 $0056E1..$0056E9: begin_human_race_pause_control; game.asm:17894
- region-795 $0056E9..$0056F6: test_pause_owner_release; game.asm:17909
- region-217 $0056F6..$00570E: find_pause_start_in_first_two_trucks; game.asm:17926
- region-796 $00570E..$005715: test_player_two_pause_start; game.asm:17949
- region-218 $005715..$00571D: test_third_pause_candidate; game.asm:17962
- region-797 $00571D..$005724: test_player_three_pause_start; game.asm:17977
- region-219 $005724..$00572C: test_fourth_pause_candidate; game.asm:17990
- region-798 $00572C..$005733: test_shared_pause_start_path; game.asm:18005
- region-260 $005733..$005742: save_pause_owner_and_read_controller; game.asm:18018
- region-799 $005742..$00575A: wait_pause_release_then_press; game.asm:18036
- region-220 $00575A..$00575C: return_human_race_control; game.asm:18057
- region-800 $00575C..$005780: draw_pause_banner_rectangle; game.asm:18070
- region-428 $005780..$005794: pause_banner_descriptor; game.asm:18095
- region-261 $005794..$00579A: wait_pause_transition_frame; game.asm:18128
- region-801 $00579A..$0057FA: expand_pause_window; game.asm:18142
- region-827 $03D600..$03D60C: countdown_tilemap_03d600; game.asm:18789
- region-828 $03D60C..$03D66E: countdown_tilemap_03d60c; game.asm:18813
- region-829 $03D66E..$03D6D0: countdown_tilemap_03d66e; game.asm:18843
- region-830 $03D6D0..$03D732: countdown_tilemap_03d6d0; game.asm:18873
- region-831 $03D732..$03D794: countdown_tilemap_03d732; game.asm:18903
- region-832 $03D794..$03D7F6: countdown_tilemap_03d794; game.asm:18933
- region-833 $03D7F6..$03D858: countdown_tilemap_03d7f6; game.asm:18963
- region-834 $03D858..$03D8BA: countdown_tilemap_03d858; game.asm:18993
- region-835 $03D8BA..$03D91C: countdown_tilemap_03d8ba; game.asm:19023
- region-836 $03D91C..$03D97E: countdown_tilemap_03d91c; game.asm:19053
- region-430 $03DAB4..$03DAD2: pause_banner_tile_rectangle; game.asm:19168
- region-837 $03E892..$03E8F4: countdown_tilemap_03e892; game.asm:19181
- region-838 $03E8F4..$03E956: countdown_tilemap_03e8f4; game.asm:19211
- region-839 $03E956..$03E9B8: countdown_tilemap_03e956; game.asm:19241
- region-840 $03E9B8..$03EA1A: countdown_tilemap_03e9b8; game.asm:19271
- region-841 $03EA1A..$03EA7C: countdown_tilemap_03ea1a; game.asm:19301
- region-842 $03EA7C..$03EADE: countdown_tilemap_03ea7c; game.asm:19331
- region-843 $03EADE..$03EB40: countdown_tilemap_03eade; game.asm:19361
- region-844 $03EB40..$03EBA2: countdown_tilemap_03eb40; game.asm:19391
- region-845 $03EBA2..$03EC04: countdown_tilemap_03eba2; game.asm:19421
- region-846 $03EC04..$03EC66: countdown_tilemap_03ec04; game.asm:19451
- region-847 $03EC66..$03ECC8: countdown_tilemap_03ec66; game.asm:19481
- region-848 $03ECC8..$03ED2A: countdown_tilemap_03ecc8; game.asm:19511
- region-849 $03ED2A..$03ED8C: countdown_tilemap_03ed2a; game.asm:19541
- region-850 $03ED8C..$03EDEE: countdown_tilemap_03ed8c; game.asm:19571
- region-851 $03EDEE..$03EE50: countdown_tilemap_03edee; game.asm:19601
- region-852 $03EE50..$03EEB2: countdown_tilemap_03ee50; game.asm:19631
- region-431 $04629A..$04639A: window_shape_samples_0; game.asm:19756
- region-432 $04639A..$04649A: window_shape_samples_1; game.asm:19763
- region-433 $04649A..$04659A: window_shape_samples_2; game.asm:19770
