# Results independent source trace

Stage 3 worker `/root/behavior_contracts/results`, frozen Round29 source
`reference/source/game.asm` and `reference/reading.asm`. Adopted identity remains
workspace.json's frozen hashes; no source supplement. Primary ROM SHA256
24b687f95bb9737d223738f13c00aeaa7d697fa9e2fd50597b81d0cfa2160daa.
Planning scopes were navigation only. This worker reread all62 assigned ranges,
shared helpers/callers, physical data and source symbol addresses independently.
All domain bodies remain explicit unimplemented stubs: contracts are runnable,
not claims of implementation or original-runtime validation.

65816 context is DB00/DP0000 for these natural callers; CPU00:Axxx maps to
physical002xxx, CPU00:Bxxx to003xxx. SEP20/REP20 selects byte/word accumulator;
SEP10 truncates index high bytes; PHP/PLP restores widths but subsequent PLX
changes N/Z. Referenced toolkit platform guides: docs/development.md SNES table,
docs/snes.md and docs/snes-native.md. Drawing consumes SNES VRAM WORD addresses,
converted to byte offsets only at native storage boundary. Native types do not
emulate RAM/registers. Record/session ownership is shared with attract/names/shop.

## Entry, checkpoints and podium setup

`begin_postrace_presentation` game.asm9137, physical2694: PHP, HUD service THEN
wait_next_nmi_frame. The HUD helper advances the BCD timer even with a winner;
this extra update is retained by the full begin_postrace contract. The first
ordinary NMI returns before continuation bookkeeping. The low order byte AND7
must be zero, and low continuation byte below3, to increment it. INC is M8;
Session.continuation_progress high byte is retained. Contracts include0→1,
2→3, capped3, non-eighth slots and nonzero preserved high byte.

Disable HDMA, INC video inhibit, transfer postrace map from07:DAD2 length0DC0
to VRAM word55C8, then STZ inhibit (not restore old value) before
initialize_race_countdown_window. Map physical3DAD2..3E892,1760 words; final
word5CA7, neighboring5CA8 unchanged. This transition is the existing postrace
window owner, not a host blocking loop. Only after it returns normalize all four
checkpoint bytes descending3..0. CMP count BCC/BEQ leaves <=count unchanged;
otherwise SBC count ONCE, not modulo. Fixed [3,4,5,255] with count4 gives
[3,4,1,251]. No lap decrement accompanies normalization. Call AI ranker after
normalization, then fade out. AI owns ranking calculation and race owns drivers.

`show_postrace_awards`2705 increments inhibit, clears CGWSEL/CGADSUB, selects
layers05, clears BG3 vertical scroll latch, forceblanks, loads podium, patches
rank palettes and draws awards/names. Sends audio0401, animation counter0,
fade-in, THEN wait timer0190. Full orchestration tests assert actual wallets,
VRAM/palette, Display math/scroll, Session audio/layers and deferred NMI state;
a trace-only list cannot satisfy those tests. Subordinate Fade/Window waits
remain resumable under the existing transition owner. The stateful Game must
compose these operations and retain their mutable state in snapshots.

## Exact resource consumers

`load_podium_resources`3121: clear inclusive401 words at4000, disable HDMA.
Graphics07:8000 length4000→VRAM byte0 (physical38000..3C000).
Palette07:B480 length0200→all256 CGRAM colors
(physical3B480..3B680), not just a named prefix.
Map07:AE40 length0640→word4040..435F (physical3AE40..3B480).
The graphics/map/palette overlap physical source windows by design, separate
consumers establish the spans. Contracts compare runtime-loaded ROM spans and
synthetic install destinations/bounds; no original asset bytes are embedded.

`color_podium_by_ranked_truck`2A7A visits rank2,1,0, destination color bytes
[35,25,45] and eight-byte palette selected by (truckID&3)*8 at07:B5A0.
Each copy writes four native colors, preserving adjacent colors. Contract
rank[7,4,6,1] deliberately establishes mask semantics and separate destinations.

`load_wipeout_graphics`28A9: inhibit++, HDMA off, one NMI, clear at4000;
copy09:D510 length2A80 toVRAM byte0, then0A:DABA length12E0 toword1540
(byte2A80), making contiguous destination0..3D60. Physical sources
4D510..4FF90 and55ABA..56D9A; long ROM reads stay banked/address-translated.
Copy08:E59A lengthC0 to CGRAM color20..7F. Loser0 skips replacement;
loser1/2/3 selects offsets[0,60,C0] from physical2A3D then48 colors from
08:EC1A+offset into color29..58. Palette outside these windows remains intact.
Map08:E65A uses rectangle at2A62,32x23, remap enabled, destination4020.
Physical4665A..46C1A has736 words; last destination42FF.
Shared font/base palette/cursor uploads and clear4800 follow; source owner
rendering provides these mechanisms. Send audio1001. Human mask low byte chooses
interactive continue, zero chooses gameover prompt/timer. install_loss isolates
these resource writes; full Game must additionally compose the shared uploads,
waits, layer enable/fade and mode-dependent prompt before rendering.

## Rank award versus pickup award

`draw_podium_awards_and_names`2AA5 calls font/clear then rank2→0. Awards
[15,10,5] are ordinary money helper digit3 ($10000 displayed units per source
six-digit+00 format), credited IMMEDIATELY. Helper saves/restores X (truck ID)
while Y high is truncated by SEP30. Rank scratch is preserved on stack and
restored; no assumption that helper leaves flags/scratch untouched is made.

Displayed amount is truck_pending_awards[X], NOT the rank award just credited.
If pending>=10 subtract10 ONCE and use leading tile2, otherwise leading tile0.
Remainder+1 second tile; trailing tile1. Attributes24/28/2C by rank. Thus
pending25 emits [2,16,1] rather than a general decimal conversion. Three names
come from truckID*3 and use2800 attributes. Rank[3,2,0,1] credits wallet digits
[5,0,10,15], draws corresponding pending9/10/25 and names, leaves pickup array
unchanged. Selected wallet ends at winner rank0. Source body has no award
normalization here; raw byte addition/wrap remains until later normalization.

`settle_awards_and_test_losing_human`27CB first disables Start abort and contracts
window. Then loops3→0, selects each wallet, adds nonzero pending amount using
digit3 helper, clears that byte, continues. This is the SINGLE mutable
WorldObjects.pending_awards array borrowed into results, not a copied results
counter. Tests include byte250+15→9 without normalization and repeat settlement
which credits nothing twice. Last selection0 even if every amount zero.
Only then test participation bit of last rank and choose loss/shop handling.

## Record comparison, drawing, blink and dismissal

`draw_podium_times_and_records` compares M8 race high component to stored first
byte, then race middle component to second only when high ties. Fraction is
ignored. Strict less sets by INC new_track_record_pending, not assignment1;
nonrecord clears. Pending255 faster→0 and7 faster→8 fixtures preserve exact
byte behavior. This flag is distinct from the pickup array despite the nearby
old summary calling it accounting count. Its low byte alone gates blink service.

`draw_winner_time_and_stored_record`2B63: winner names at4869..486B attributes28;
time at4872..4875 from high&0F, middle>>4, separator2827, middle&0F, each digit
+2801. Stored holder names at4889..488B; stored time4892..4895. Track names stride3,
times stride2, no fraction. Fixed fixtures include highF1 and storedA2 to prove
high nibble ignored for drawing though comparison uses full bytes.

Awards timer starts400 after fade-in. Ordinary loop waits NMI before processing.
If pending zero, poll ANY bit of first TWO whole controller words; ports3/4 do
not dismiss. If no input, DEC WORD timer; negative result dismisses. Timer0→FFFF
is final boundary; a held low-byte R on port2 qualifies without edge detection.

IMPORTANT correction to broad old prose: timer>250 with pending record does NOT
poll controller input. CMP250 BCC goes normal polling; BNE goes blink. Blink
chooses blank when animation bit10 clear, winner-time redraw when set. Both
branch `finalize_awards_dismissal` alias00:A7C1 INSIDE dismissal routine at its
REP20/DEC timer. This bypasses JOY reads above it atA7B1..A7BF. A held button
cannot dismiss the first150 pending-record frames. Fixed timer400 heldB→399
Waiting and251 heldStart→250 Waiting expose the cross-label entry semantics.
Blank loop X=0C down through0 writes13 words4869..4875 inclusive, not12.

At EXACT250: M8/index8, selected original track*2 times, track*3 names; save
race high/middle and rank0 truck's three-name bytes. Then extra wait_next_nmi_frame,
redraw stored row followed winner row, THEN ordinary input/decrement path.
advance_awards commits and suspends CommitWait without polling even if heldB;
finish_record_wait resumes on the extra boundary and can dismiss with timer250
unchanged. Without input nexttimer249, and <250 never commits again. Pending
flag is not cleared by commit. Final runtime must include this phase in clone/
snapshot and cannot merge two NMI waits into one simulation tick.

Original record bounds remain16 and original SessionRecords is sole owner.
AddedRecords contains ONLY arbitrary named added tracks. Tests write24 added
keys alongside original15, require missing-key/invalid16 transactional errors,
and preserve original arrays. Added record initial defaults/identity remain a
native extension policy: use supplied named content defaults, never clamp IDs
into the original array or limit total catalogue size. No battery hooks are
introduced: original records are session RAM.

## High-score insertion and rows

Defaults copy atB63F is separate from wallet initializerB61F. Copies exact
48 track-name,32 track-time,60 score-digit,30 score-name,10 order,12 truck-name
bytes. Warm entry does not invoke copy; attract owns signature gating, results
owns record behavior. Existing six arrays are reused, not independent copies.

`insert_truck_totals_into_high_scores`3879 normalizes ALL trucks3→0, human or
not. Selected helper scans displayed rank9→0, records by orderID*6 and six-byte
lexicographic comparison, strictgreater only. Unlike an intuitive descending
leaderboard, order0 stores LOWEST, order9 HIGHEST. First qualifying rank r reuses
storage order0, copies six digits and corresponding truck three-name bytes
(source moneyoffset/2 and scoreoffset/2 establish name strides). Shift order1..r
to0..r-1, insert reused storageID at r. Source destination alias
high_score_names+1D+X == high_score_order+(X-1); this is an indirect ordering
operation, not corruption to a second name array.

Fixture scores0..9, new5: rank9..6 lower,rank5 equal skips,rank4 qualifies;
new order[1,2,3,4,0,5,6,7,8,9], score storage0 now5. New9 inserts below equal9
at8, so duplicates/ties are NOT globally deduplicated. All equal0 does not insert.
New1000000 in most-significant position outranks009000 despite digit sums.
Four-truck service fixture normalizes last-byte10/11 then inserts descending;
this requires actual economy helper calls and one shared Session selection.
Native invalid storage IDs must fail transactionally before indexing/writes.

`draw_highscore_rows`391E uploads paired font, heading480B, then rows9→0 using
separate ROM destination words at3A1B/3A2F. Stage six digits MSF; leading zeros
emit vertical tile pair0/1, all-zero row remains those blank-font tiles. First
nonzero d emits byte(2*(d+1)) /next; subsequent zeros emit2/3. VMAIN81 means
second pair at+32words and next character base+1. Appends2/3 borders atbase6 and7.
Names use byte(char<<1),nexttile; character80 wraps→0/1. Fixture establishes
indirection row0→storage9, leading blanks, interiorzero, borders and name bytewrap.
Original highscore destinations/layout resource data remain ROM-backed.

## Losing-human flow, penalty, continue and gameover

After pickup settlement, nonhuman last-ranked truck immediately goes shop then
inactive conversion/return. Losing human counts LOW FOUR active bits using four
RORs; entering carry cannot affect those four consumed original bits. Preserve
loser/count across inactive-money conversion and high-score insertion, which run
BEFORE penalty or participation removal. All-human/upper-mask state does not
change that low-four count. One human DEC low continuation byte; negative result
clears it, toggles loser out, loads wipeout, then unwinds saved frontend returns
into reentry. Multiple humans skip decrement. Native LossRoute makes nonlocal
unwind explicit, not emulated stack operations. Source alternate no-human NO path
jumps00:A826 into common PLP/PLA/PLA unwind after toggle already cleared progress.

Penalty four upgrade arrays order speed,acceleration,steering,suspension:
DEC twice byte, BPL keeps result, negative sets0. Ordinary0/1→0,2→0,6→4;
conditional80→7E,81→7F,82→0,FF→0. Nitro is not reduced. Clear six wallet bytes
for losing truck, X advances to6*(loser+1). IMPORTANT corrected old summary:
STA frontend_player_one_input_lo,X ($055C+X) aliases $0562+6*loser = SAME
wallet digit2, NOT a continuation/input flag. It sets startercash[0,0,1,0,0,0].
No RAM map is needed; native penalty resets that wallet and preserves others.

`toggle_human_participation`288E uses XOR bit[1,2,4,8], not unconditional clear.
Page0=mask&3, page1=mask>>2 WITHOUT masking; continuation low STZ, high retained.
85 toggletruck0→84 yields pages[0,21], continuationAB03→AB00. Natural losing
human call precondition guarantees selected bit set; conditional source interface
still preserves XOR when absent. Continue YES choice0 goes shop, no mask change;
NO nonzero toggles loser, zero resulting WHOLE mask means reentry, else shop.

Human prompt at2975 draws paired text4ACB and4B06; resets sprite arrays, installs
nine tilewords at low OAM2+4i and second copy26+4i, waits, releases video inhibit,
layers15. First cursor originAE24 then expands nine sprites from existing shop
coordinate deltas. Choice0,fade-in. Only losing slot's JOY high byte is read.
Held B bit80 dismisses immediately, before directions, no edge/release wait.
Leftbit02 takes priority even when choice already0 (right ignored), switches1→0
and X24; otherwise rightbit01 switches0→1 and XA8. No repeat, timeout or Start
acceptance in human loop. Redraw nine first cursor coordinate bytes only;
second copy tilewords remain source staging. Dismiss hides frontend sprites and
contracts window, returns choice; Game composes those actual subordinate owners.

No-human prompt2A32 draws GAMEOVER glyph stream at4B0B,wait/layers15/fade-in,
X16 countdown012C. Each NMI DEX first, BMI dismiss; otherwise OR all four HIGH
controller bytes. Low-byte A does not dismiss, port4 Select does. 300 noninput
waits leave0,301st→FFFF dismisses. This is source timed terminal/attract return,
not an invented final victory screen or player menu.

## Contract evidence and remaining scope

42 source-derived native contracts in results/tests.rs. Recorded pinned
Rust1.90 --locked compile/list pass509 total suite tests; results0pass/42 failures
all explicit `results ... pending` unimplemented panics;0 ignored. All four
infrastructure tests separately pass. Exact commands/logs results-results.json.
No implementation, QA, emulator experiment, reference or toolkit edits.

Stage4 must implement ALL interfaces and actual Game/session/display/transition/
audio composition; helper tests do not replace stateful full-loop behavior.
Return effects/phase state must preserve waits, draw order, normalization/score
prelude and one pending award owner. Record compare INC wrap and conditional
upgrade/mask inputs have no independently observed natural entry beyond source
preconditions; contracts intentionally preserve them. Added named track records
have native validation policy separate original16 lookup bounds. Cartridge
battery status remains unchanged. Runtime snapshots must include added records,
results phases/extra-NMI timer, choices, SessionRecords and mutable presentation.

Assigned threshold data region987/988/989 at2687..2694 was independently
checked against AI rank consumers; it is overlapping80 threshold windows, whose
behavior contracts already live in AI. Results does not create a second RNG/rank
owner. Unnamed2C2B adjacent byte is outside three-byte award lookup: bounded rank
2..0 consumer does not read it; no unsupported fourth award is invented. Its
original identity/classification remains unchanged for implementation accounting.

## Primary range navigation (reviewed, not implementation accounting)
- region-987: physical$002687..$00268C, game.asm:9067: ai_parameter_random_threshold_prefix.
- region-988: physical$00268C..$00268F, game.asm:9089: ai_and_human_rank_random_thresholds.
- region-989: physical$00268F..$002694, game.asm:9111: human_rank_random_threshold_tail.
- region-247: physical$002694..$0026CB, game.asm:9137: begin_postrace_presentation.
- region-743: physical$0026CB..$0026E9, game.asm:9172: transfer_race_entry_map.
- region-274: physical$0026E9..$0026F6, game.asm:9193: normalize_postrace_checkpoint_indices.
- region-744: physical$0026F6..$0026FC, game.asm:9211: wrap_finished_checkpoint.
- region-275: physical$0026FC..$002705, game.asm:9224: rank_finished_race_and_fade_out.
- region-288: physical$002705..$002740, game.asm:9239: show_postrace_awards.
- region-276: physical$002740..$00274B, game.asm:9272: wait_awards_frame_and_test_pending_count.
- region-745: physical$00274B..$0027B1, game.asm:9287: commit_or_flash_track_record.
- region-277: physical$0027B1..$0027CB, game.asm:9348: dismiss_awards_on_input_or_timeout.
- region-298: physical$0027CB..$0027F7, game.asm:9369: settle_awards_and_test_losing_human.
- region-746: physical$0027F7..$0027FA, game.asm:9400: jump_race_end_continuation.
- region-299: physical$0027FA..$00281D, game.asm:9412: count_active_humans_for_continue.
- region-747: physical$00281D..$00282C, game.asm:9444: restart_after_continue_loss.
- region-300: physical$00282C..$002877, game.asm:9461: penalize_losing_truck_upgrades.
- region-316: physical$002877..$00287C, game.asm:9509: test_continue_choice.
- region-748: physical$00287C..$002886, game.asm:9522: remove_last_ranked_human.
- region-317: physical$002886..$002889, game.asm:9536: resume_shop_after_continue.
- region-354: physical$002889..$00288E, game.asm:9547: settle_inactive_truck_stock_and_return.
- region-749: physical$00288E..$0028A9, game.asm:9561: toggle_human_participation.
- region-301: physical$0028A9..$002910, game.asm:9587: load_wipeout_graphics.
- region-750: physical$002910..$00293C, game.asm:9642: load_result_palette.
- region-302: physical$00293C..$002972, game.asm:9675: draw_wipeout_background.
- region-751: physical$002972..$002975, game.asm:9706: jump_continue_prompt_setup.
- region-303: physical$002975..$0029C1, game.asm:9718: show_continue_prompt_and_cursor.
- region-318: physical$0029C1..$0029D2, game.asm:9761: poll_continue_controller.
- region-752: physical$0029D2..$0029DE, game.asm:9780: select_continue_left.
- region-319: physical$0029DE..$0029E2, game.asm:9795: test_continue_right_input.
- region-753: physical$0029E2..$0029F6, game.asm:9808: select_continue_right.
- region-320: physical$0029F6..$0029FE, game.asm:9827: dismiss_continue_prompt.
- region-754: physical$0029FE..$002A19, game.asm:9842: draw_continue_prompt.
- region-810: physical$002A19..$002A32, game.asm:9864: wait_frontend_controller_or_timeout.
- region-990: physical$002A32..$002A3D, game.asm:9882: paired_text_glyphs_2a32.
- region-991: physical$002A3D..$002A43, game.asm:9904: result_palette_offsets.
- region-992: physical$002A43..$002A62, game.asm:9926: paired_text_glyphs_2a43.
- region-314: physical$002A62..$002A76, game.asm:9950: continue_picture_rectangle.
- region-993: physical$002A76..$002A7A, game.asm:9965: result_truck_participation_bits.
- region-289: physical$002A7A..$002AA2, game.asm:9990: color_podium_by_ranked_truck.
- region-994: physical$002AA2..$002AA5, game.asm:10022: podium_cgram_destinations.
- region-290: physical$002AA5..$002B53, game.asm:10049: draw_podium_awards_and_names.
- region-755: physical$002B53..$002B5B, game.asm:10149: compare_record_time_low_words.
- region-291: physical$002B5B..$002B5F, game.asm:10162: clear_new_track_record_flag.
- region-389: physical$002B5F..$002B63, game.asm:10174: set_new_track_record_flag.
- region-292: physical$002B63..$002C28, game.asm:10187: draw_winner_time_and_stored_record.
- region-995: physical$002C28..$002C2B, game.asm:10287: podium_award_values.
- region-1447: physical$002C2B..$002C2C, game.asm:10298: unnamed data.
- region-996: physical$002C2C..$002C38, game.asm:10305: podium_vram_destination_pairs.
- region-293: physical$003121..$0031B5, game.asm:11553: load_podium_resources.
- region-309: physical$003879..$00391E, game.asm:12953: insert_truck_totals_into_high_scores.
- region-812: physical$00391E..$003A11, game.asm:13069: draw_highscore_rows.
- region-813: physical$003A11..$003A1B, game.asm:13218: highscore_heading_tiles.
- region-814: physical$003A1B..$003A2F, game.asm:13242: highscore_score_row_destinations.
- region-815: physical$003A2F..$003A43, game.asm:13266: highscore_name_row_destinations.
- region-295: physical$038000..$03C000, game.asm:18763: podium_display_resource_bundle.
- region-1011: physical$03DAD2..$03E892, game.asm:19175: post_race_vram_transfer.
- region-313: physical$04659A..$04665A, game.asm:19779: continue_picture_palette.
- region-315: physical$04665A..$046C1A, game.asm:19788: continue_picture_tilemap.
- region-1012: physical$046C1A..$046D3A, game.asm:19795: result_palette_replacements.
- region-311: physical$04D510..$04FF90, game.asm:19970: continue_picture_tiles_a.
- region-312: physical$055ABA..$056D9A, game.asm:20017: continue_picture_tiles_b.

## Stage4 source correction retained alongside superseded fixture interpretation
Initial40/42: default score-order permutation validation missing in implementation, added. Podium fixture expected2000 for clear4000/4360 but source load_podium_resources3121 (game.asm11553) calls clear_vram_and_reset_ppu1C0D (7272), explicitly LDA0000 then0401 stores and RTS; reset_video_registers is a separate entry. Corrected these literals to0. Same source zero-clear at28A9 corrects loss neighbor4300 expectation to0; implementation corrected accordingly. Podium helper STZ HDMAEN does not writeTMW, so retained main_window_layers1F asserted instead of0. Full Game must separately disable Window.hdma_enabled, not erase configured masks. Postrace API receives actual trucks for all HUD digits and retains actual Route for source AI ranking; count alone cannot fabricate a valid ranking. Existing fixtures adapted with source-state fields only, original behavior assertions retained except explicitly erroneous literals.

Stage4 source wait integration: loss prelude converts inactive stock, inserts records and chooses/applies penalty before its leading wait. Game resumes actual loss resource upload at the following NMI, stages continue/game-over text/cursor while still inhibited, then releases layers/inhibit only after the next NMI. Source prompt placement24,AE becomes visible through the following normal OAM boundary. Focused Game integration verifies these separate boundaries, one LOSS sound request and pending-fade snapshot/frame/PCM fork replay; all42domain assertions remain unchanged.

## Modernization iteration3 M7: native leaderboard entries and score quantities

Session records now own a `Leaderboard` of score/name entries with the original
stable storage identities and rank order. Canonical six-digit rows decode to a
native monetary score (stored units are hundreds); normal insertion compares
quantities rather than display slices. The name stays in the shared three-letter
game alphabet and pairs with its score in one entry. Rendering alone converts the
quantity to six leading-zero-aware digits and appends the original two zeros.

`LegacySessionRecords` preserves the exact snapshot keys and fixed arrays.
ROM defaults, wallet reads and restored digit rows decode at their boundaries.
Nondecimal rows retain a lossless compatibility value; mixed comparisons use the
source lexicographic rule. They are not silently normalized or numerically summed.
The strict comparison, reverse rank search, lowest-storage reuse, duplicate-ID
aliasing in direct domain calls and prevalidation-before-mutation remain intact.
Full snapshots and asset defaults still require a rank permutation. Wallet
normalization and descending all-truck insertion keep their previous order.

Existing encoded fixture construction is explicit and test-only; all original
numeric expectations remain unchanged. Two additional contracts characterize
duplicate-ID insertion and nondecimal candidate/row preservation, including the
old serialized shape. They passed on the starting implementation before edits.
Independent original observations show natural storage0 reuse at rank8 below an
equal highest score and unchanged rows/names/order through warm entry. The native
warm-entry scenario lets startup install its signature before joining; the older
early-Start recipe is retained but does not certify that retention contract.
