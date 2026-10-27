# Race contracts — independently traced Stage 3

Source: frozen Round29 reference/source/game.asm / reference/reading.asm, SHA256
4e3e1a45e4409ce63a3354740a2cdd3962cd3d9975d88d22ecee40c9d29ded7f.
Bank00 execution aliases CPU008000+physical for the listed routines, native
65816 DB00/DP0000 established by boot. All42 primary race regions below were
read independently, with data, source consumers and relevant callees. Navigation
index is not evidence; original source remains unchanged. No behavior implemented.

Consulted toolkit docs/development.md platform row, docs/snes.md address/width
context, docs/snes-native.md native boundary, and WDC W65C816S March2024 datasheet
sections2.4/2.7/2.8, operation tables5, caveats7.20/7.23:
https://www.westerndesigncenter.com/wdc/documentation/w65c816s.pdf . M and X select
independent operand/index widths; SEP X8 clears index high bytes. XBA moves both
accumulator bytes even in M8. CMP carry means unsigned greater/equal, not signed
ordering. SEC/SBC then CLC/ADC here are explicitly binary except the HUD SED
chain. Byte memory stores retain neighboring bytes; byte INC wraps; word memory
DEC affects both bytes. No generic idealized rectangle/timer model is the oracle.

## Initialization and actual retained state

initialize_selected_race game.asm110 physical00AA..017E first blanks/inhibits,
sets Mode1/BG3 priority and same layer tile/map bases as boot, then loads font/
clears background. Its initial words are tile attributes2000,2202,2404,2606;
headings0040, speeds0000, working speed0000. Checkpoint bytes all0, lap bytes
all1, scratch39 word0. It calls derive_ai_parameters_from_human_upgrades BEFORE
setting four timeoutsC8. Consequently parameter/suspension changes from AI setup
survive timeout reset. Neither early initializer nor the timeout loop clears
acceleration fractions, steering fractions/locks, retained input/nitro history,
upgrades or all other truck state. Later helpers own their explicitly traced
writes; do not clear a Truck wholesale and inadvertently change race-to-race
history. initialize_race_records and reset_route_timeouts isolate these boundaries.

Then: clear16 wheel sample/integrated/velocity bytes; clear16 animation cursors;
load route; load track assets; initialize grid; shared sprite palette; shared
sprite tiles; race OAM; sprite arrays/OAM pack; HUD setup (including clear all
three timer bytes); track object placement; initialize banner. load_route_pointer
(game.asm985 physical056A..05BE) finally sets checkpoint bytes1. Grid heading
writes are M8 to each heading word LOW byte, not a full word assignment. Natural
initializer has already cleared heading highs via0040, so normal result is0080
etc; conditional apply_route_grid must retain supplied high bytes. First drafting
fixture incorrectly expected full-word overwrite; independently rereading the
STA widths corrected it BEFORE final contract record. Grid positions/odd-order
row/heading-dependent X swaps and initial probe are tracks/terrain owners, not a
second grid algorithm here. Initial sampling applies integrated heights and truck
surface summary; wheel sample heights remain separately owned. Full source
cross-effects documented by tracks.trace.md/terrain.trace.md.

Set prepared1, winner0, enable NMI+automatic joypad, video inhibit0, layers1F.
Run update_race_frame_services directly WITHOUT a wait/input-pause dispatch. It
still executes every service below, including timer/input/AI, then returns only
because prepared is nonzero. DEC prepared (natural1→0), fade in, invoke ready
window in BOTH human/attract paths (the BEQ targets the following instruction).
Ready/window resumable timing belongs transitions; no blocking host waits.
After ready wait one boundary; read DP06 (named HIGHSCORE_DIGITS by one equate,
but same physical semantic field as active_human_mask). Nonzero emits host word
01|(((order&7)+6)<<8); SPC decrements selector, songs5..12. No high-score consumer
or score condition is established by that unfortunate equate name. Attract sends
nothing. Sound owner interprets host/DSP state.

prepare_selected_race contracts real RaceState, real immutable terrain/tables/
route/grid, and actual one-service mutations in addition to named phase evidence.
Future objects/presentation owners must add actual state in composition; an array
of phases alone cannot satisfy timer/input/truck/progress assertions. The test
stops at the ready handoff; transitions supplies resumable windows/waits, not new
player controls. Source render mode88=2 before each service.

## Frame ordering and branch exits

run_race_frame game.asm227 physical0198: wait NMI; poll race/attract Start; then
update_four_trucks → integrate_sixteen_wheels → suspension correction → poses →
sprite Y sort → OAM pack → sprite animation → human input copy/checkpoints →
objects → banner → lap/nitro HUD (and time) → computers. Prepared check is LAST.
Thus movement consumes retained words, human checkpoint code sees new automatic
words, and computer acceleration/throttle/heading are written after HUD for the
next movement frame. Terrain rejection/history and pair-projection/commit details
remain exact motion/contact contracts, not reordered by a scheduler convenience.

Nonzero prepared returns regardless human mask/winner. Otherwise mask nonzero
(any byte bit, not only low4) chooses human branch: no winner continue, winner
invoke postrace. Attract no winner continue, winner fades out then boot frontend
restart (which chooses next attract order); it does NOT increment ordinary track
order or show human results. Input dispatch: mask nonzero calls human pause owner;
mask0 ORs all four full auto-controller words and tests Start1000. Start increments
frontend_reentry BYTE, discards the race return address in source and transfers to
boot frontend loop. Native returns FrontendRestart intent, never runs remaining
race services on that branch. Full reentry reset/masks/order/signature belongs
runtime/attract; cold reset is distinct. No partial native stack emulator.

Human finish calls begin_postrace_presentation before reading/incrementing slot.
Read slot AFTER result returns; results may have changed session state or unwound
instead of returning. INC byte then AND3F; preliminary AND0F selected-track write
is overwritten by select_track_descriptor, which reads actual order/direction.
Tests use a nonidentity order fixture to distinguish this. At255 increment wraps0,
at63 masks0. Original order mask applies original64 slots; added catalogue slots
remain extensible and need native extension sequencing (not a global16 count).

## Human route interpreter and single progress owner

poll_controls_and_check_routes game.asm1857 physical06CB: copy enabled full words
from automatic controllers first (controls owner); then human scanner SEP M8/X8.
Shift mask left4, decrement byte slot4→3→2→1→0, ASL carry selects human slots.
High4 mask bits discard; disabled slots retain input and Driver state. Both
high-byte-only nonzero mask and truly zero mask produce no visited slots here.
For each human, byte checkpoint*4 wraps before TAY. Load HIGH of first packed
word: FE/FD skip by increment/reinterpret immediately, WITHOUT AI redirect/RNG.
FF resets index0 and rereads. Otherwise bit40 of that high byte (packed bit14)
also skips. Marker tests precede rectangle reads, so unused second control words
may contain arbitrary redirect-looking values without becoming human pointers.
Finite native extension validation detects missing records/control-only cycles
transactionally; source could hang or read unrelated memory for invalid inputs.
This error policy is a native supported-input boundary, not a confirmed original
behavior or permission to omit an established consumer.

Ordinary record converts slot to word-array offset, rectangle helper returns
carry. Rejected: next truck, preserving any preceding marker skip effects.
Accepted: checkpoint INC byte. New value1 increments lap (start-line crossing),
requests last-lap display then compares lap exactly5; if equal store slot+1 as
winner. This path always jumps scanner, BYPASSING checkpoint-limit comparison.
Thus source count1 start acceptance retains index1. Nonstart accepted index
compares unsigned route_count: >=count resets0, below retains. No lap here.
Lap255 wraps0; lap6 is not winning5. There is no existing-winner guard; all four
simultaneous finishers scanned3→0 leave winner1. AI runs later and may overwrite
winner again; source does not stop at first observed finish. Drivers/timeouts/
parameters remain same native shared state used by AI. Progress is now defined
in race.rs and trucks::ai::RaceAi is a public alias; banner is source word width.
Do not maintain copied human/AI lap or winner authorities in stage4.

### Rectangle direction correction and fixed boundaries

Shared helper game.asm2019 physical0744..0782, also duplicate inline sequence in
AI test_ai_waypoint_progress physical09AD, explicitly computes:
extent=(packed&001F)<<7; center=packed&BFE0;
unsigned word-wrap(center - world + extent) < extent*2 for BOTH X and Y.
The earlier AI trace/contract inverted edge prose/expectations by thinking of
world-center. Source SBC is center-world. Fixed independent example center2000,
extent0100: world1F00→0200 CMP0200 rejects, 1F01→01FF accepts,2100→0000 accepts,
2101→FFFF rejects. Y same. Thus lower WORLD edge excluded, upper included. Zero
extent rejects even exact center. Word subtraction/add wraps atFFFF; bit14 clears
but bit15 retained. Examples center0000 extent0080: worldFFF0 and0080 accept,
FF80 rejects. These are source-fixed literals, no implementation oracle.

Corrected AI test name and three edge literals in place with full superseded
interpretation/correction note in ai.trace.md. Existing checks/contracts/ai-* logs
remain unchanged. Affected test rerun recorded race-ai-rectangle-correction.log,
still fails explicit pending stub, no claim it passes. inside_checkpoint is the
single canonical helper interface; inside_waypoint must delegate in stage4,
not implement another divergent comparison. Original routes never hit checkpoint64
but supplied conditional state contracts BYTE ASL twice aliasing64→record0 before
acceptance increments to65; trace labels this conditional, not natural gameplay.

## BCD time and HUD ownership

HUD title setup game.asm14676 clears fraction/middle/high bytes. update HUD
physical4277..4305 (rendering primary owner) writes lap/nitro tiles then timer.
Source has NO winner/prepared test. High<09 advances unconditionally; else
middle<99 advances; only high>=09 AND middle>=99 freezes all three bytes exactly,
retaining arbitrary previous fraction. SED M8 CLC ADC16 to fraction, ADC00 middle,
ADC00 high, CLD. Valid decimal fixture 00:09:84 +16 =00:10:00;08:99:99→09:00:15.
At09:98:90→09:99:06 then next call freezes. Never force fraction99 or reset it at
saturation. These bytes are encoded source time, not asserted milliseconds.
Visible tiles are high.low, middle.high, middle.low plus15. Rendering applies
sprite positions/remaining lap/nitro writes; race owns clock arithmetic. It must
call this once per HUD service, including initial prepared frame and the EXTRA
HUD call at begin_postrace_presentation game.asm9137. Final AI winner occurs after
HUD this frame; human winner before HUD; both continue timing through this call.
Pause/window waits don't run race HUD unless explicit source caller does.

## Banner word counters and source sprite staging

Physical0EF8..1032 game.asm3783..4016. Original layout16 packed coordinates from
ROM, main tile table35 word entries, effect6 word entries. Loading uses named
ROM ranges and checks complete words/bounds. Added tracks may supply BannerLayout
instead of indexing beyond original16. Initialize main coord fromtrack,attrs3584;
STZ word primary and secondary state. It does not zero secondary coords/tile or
extra pair on other tracks. Track8 installs extra coord2760/tile3180 and2770/3182.
request_last_lap sets M8/X8, preserves byte slot, lap>=4 and banner LOW<13 sets
LOW1E, preserving banner high. Negative/high values aren't signed comparisons.
Natural initialized high remains0; conditional AB02→AB1E fixture fixes byte-store.

Banner service PHP/REP M16/X16: if primary<19 draw directly; >=19 test winner,
if winner nonzero ALSO branches to draw, NOT return. The old comment at0F7B says
return; instruction target0F8C proves otherwise. Only nonwinner phase(animation&3)
==0 decrements primary WORD before tile lookup. No decay below19, allowing ready
source selector to grow via draw_track_countdown_frame (transitions owner).
State19 phase0 becomes18 and draws3584; state30 phase0→29 tile35A8; winner state30
stays30 tile35A6 but secondary still animates. Primary0/18 unchanged by service.

Write primary tile from indexed table. If secondary state0: write packed hidden
coordF0F0, then if primary15 initialize secondary coord=primarycoord+4 WORD-wrap,
state5; FALL THROUGH into secondary animation SAME CALL. Phase0 immediately DEC
secondary5→4 then chooses effect tile3B05; phase1 retains5 but moves coord-0100.
Existing nonzero secondary: phase0 DEC, phase1/3 no DEC; saved A=(animation&3)
then AND1 controls odd vertical movement. Thus packed Y decrements modulo byte
with X unchanged (0002→FF02). State1 phase0 becomes0 and still draws tile3B09;
only next zero-state call hides/rearms. Tile valid states0..34 / secondary0..5;
unsupported restored/added values return native transactional validation errors,
not out-of-bounds synthetic ROM interpretation. The source ready producer reaches
0..18 and lastlap/postrace 19..34; no required naturally supported state excluded.

Transitions owns countdown map source strides/delay/window/fades/pause. Rendering
owns actual OAM array composition and presentation; BannerDisplay holds semantic
staged sprite records and rules above, not a second PPU/RAM image. All counters,
coordinates, drivers, input histories, timer/prepared flags and pending resumable
phase must enter full snapshots/clone/observations at implementation.

## Unclassified gap and limits

Original unknown-0001fb physical01FB..0200 bytes01 02 04 00 00 lie after unconditional
JMP initialize_selected_race and before separate selection entry. No inspected
branch/call/data consumer establishes a role. Preserve classification/identity;
no padding or unreachable exclusion, no new native behavior inferred. Runtime
owns broader record/cold-reset signature semantics, results records/wallet/loss,
objects dynamic effects, presentation assets, audio host sequencer. Those owners
must independently trace their own data/cross-calls. No original-runtime QA or
emulator experiment was run, no toolkit/reference/ledger disposition changed.

## Exhaustive primary source review index

- region-333 physical$0000AA..$00017E; game.asm:110; initialize_selected_race
- region-883 physical$00017E..$000198; game.asm:202; send_track_order_race_music
- region-112 physical$000198..$0001CF; game.asm:227; run_race_frame
- region-334 physical$0001CF..$0001D0; game.asm:258; return_prepared_race_frame
- region-113 physical$0001D0..$0001D4; game.asm:269; test_human_race_mode
- region-700 physical$0001D4..$0001DE; game.asm:282; dispatch_finished_race
- region-114 physical$0001DE..$0001E5; game.asm:296; repeat_until_race_winner
- region-221 physical$0001E5..$0001ED; game.asm:309; finish_race_and_increment_order
- region-335 physical$0001ED..$0001FB; game.asm:324; wrap_track_order_and_reload
- unknown-0001fb physical$0001FB..$000200; game.asm:333; 
- region-116 physical$000265..$00026E; game.asm:405; poll_race_or_attract_start
- region-223 physical$00026E..$00028B; game.asm:420; leave_attract_on_any_start
- region-124 physical$0006CB..$000703; game.asm:1857; poll_controls_and_check_routes
- region-228 physical$000703..$000708; game.asm:1902; wrap_route_checkpoint_index
- region-125 physical$000708..$00070C; game.asm:1914; test_skippable_route_marker
- region-229 physical$00070C..$000711; game.asm:1927; skip_route_control_record
- region-126 physical$000711..$000727; game.asm:1940; test_truck_checkpoint_rectangle
- region-230 physical$000727..$000730; game.asm:1962; increment_truck_lap
- region-706 physical$000730..$00073A; game.asm:1976; declare_fifth_lap_winner
- region-127 physical$00073A..$00073E; game.asm:1992; test_route_checkpoint_limit
- region-231 physical$00073E..$000743; game.asm:2005; reset_route_index_at_limit
- region-128 physical$000743..$000797; game.asm:2019; route_rectangle_and_controller_helpers
- region-707 physical$000797..$00079D; game.asm:2080; copy_controller_two
- region-129 physical$00079D..$0007A0; game.asm:2092; test_controller_three_enabled
- region-708 physical$0007A0..$0007A6; game.asm:2105; copy_controller_three
- region-130 physical$0007A6..$0007A9; game.asm:2117; test_controller_four_enabled
- region-709 physical$0007A9..$0007AF; game.asm:2130; copy_controller_four
- region-131 physical$0007AF..$0007B0; game.asm:2142; return_after_controller_poll
- region-342 physical$000EF8..$000F1E; game.asm:3783; initialize_track_countdown_display
- region-723 physical$000F1E..$000F36; game.asm:3810; initialize_countdown_pair
- region-343 physical$000F36..$000F38; game.asm:3830; return_track_countdown_initialization
- region-925 physical$000F38..$000F58; game.asm:3839; track_countdown_oam_coordinates
- region-237 physical$000F58..$000F70; game.asm:3866; request_last_lap_banner
- region-154 physical$000F70..$000F7B; game.asm:3889; test_race_banner_counter
- region-269 physical$000F7B..$000F82; game.asm:3904; suppress_banner_after_winner
- region-724 physical$000F82..$000F8C; game.asm:3918; pace_countdown_animation
- region-155 physical$000F8C..$000FAA; game.asm:3933; update_banner_tile_and_secondary_sprite
- region-725 physical$000FAA..$000FBA; game.asm:3956; advance_countdown_secondary_sprite
- region-238 physical$000FBA..$000FDE; game.asm:3973; animate_banner_secondary_sprite
- region-156 physical$000FDE..$000FE0; game.asm:4000; return_banner_service
- region-262 physical$000FE0..$000FEC; game.asm:4009; countdown_effect_tiles
- region-263 physical$000FEC..$001032; game.asm:4016; race_countdown_tiles

## Executable coverage and actual outcomes

Native `race::tests::*` contains38 substantive contracts. Test names below map
the trace sections to branch/boundary/integration evidence; expectations were
derived above, not from pending stubs. All38 execute and fail ONLY explicit
unimplemented panics. Rust1.90 locked compile/list pass209 total tests,0ignored.
Corrected AI rectangle test executes/fails pending. Both assets2+game2
infrastructure tests pass. Commands/log paths: checks/contracts/race-results.json.
Initial36-test run is preserved separately as race-initial-*; final adds explicit
timeout boundary/conditional index-width tests and separates timeout ownership.
No further implementation audit or broad verification pass was run.

- `race::tests::banner_rom_loader_preserves_all_track_and_word_tile_entries_and_bounds`
- `race::tests::original_banner_layout_covers_all_tracks_and_only_eight_installs_extra_pair`
- `race::tests::initial_banner_clears_word_states_but_retains_unwritten_secondary_and_fixed_slots`
- `race::tests::track_eight_banner_extra_pair_has_sixteen_pixel_separation_and_source_tilewords`
- `race::tests::last_lap_request_thresholds_preserve_word_high_byte_and_do_not_overwrite_active_banner`
- `race::tests::last_lap_invalid_slot_is_rejected_without_mutating_shared_progress`
- `race::tests::primary_banner_decays_only_at_four_frame_phase_and_only_at_or_above_nineteen`
- `race::tests::winner_freezes_primary_decay_but_still_updates_tiles_and_secondary_animation`
- `race::tests::secondary_trigger_falls_through_to_same_call_decrement_and_tile_lookup`
- `race::tests::secondary_moves_on_odd_parity_without_decrement_and_wraps_packed_y`
- `race::tests::secondary_final_tile_is_visible_for_one_call_before_zero_state_hides_it`
- `race::tests::invalid_banner_selector_returns_error_without_partial_sprite_or_counter_mutation`
- `race::tests::packed_rectangle_excludes_lower_world_edge_includes_upper_and_rejects_zero_extent`
- `race::tests::rectangle_masks_control_bit_fourteen_preserves_bit_fifteen_and_wraps_world_subtraction`
- `race::tests::enabled_humans_scan_descending_copy_new_full_words_and_preserve_disabled_ai_words`
- `race::tests::no_human_low_bits_neither_change_progress_nor_retained_inputs`
- `race::tests::human_fd_fe_and_bit_fourteen_records_skip_without_reading_redirect_targets`
- `race::tests::human_terminator_wraps_immediately_then_start_acceptance_increments_lap`
- `race::tests::rejected_human_checkpoint_keeps_index_but_preceding_control_skips_still_persist`
- `race::tests::accepted_nonstart_checkpoint_at_limit_wraps_without_incrementing_lap`
- `race::tests::start_acceptance_bypasses_checkpoint_limit_test_even_when_limit_is_one`
- `race::tests::simultaneous_fifth_lap_humans_overwrite_winner_in_descending_scan_and_raise_banner`
- `race::tests::lap_winner_requires_exactly_five_and_byte_lap_overflow_does_not_finish`
- `race::tests::missing_or_control_only_cyclic_extension_route_returns_error_without_partial_copy_or_progress`
- `race::tests::timer_adds_decimal_sixteen_with_fraction_and_middle_carry`
- `race::tests::timer_freezes_only_when_both_high_and_middle_thresholds_reached_retaining_fraction`
- `race::tests::timer_crossing_saturation_reaches_first_middle_99_then_freezes_on_following_call`
- `race::tests::race_record_initialization_writes_only_explicit_truck_fields_and_preserves_powertrain_history`
- `race::tests::route_grid_application_sets_checkpoint_one_and_source_byte_headings_preserving_unwritten_high_bytes`
- `race::tests::prepared_return_has_priority_over_human_attract_and_winner_dispatch`
- `race::tests::race_music_selectors_are_five_through_twelve_after_spc_decrement_and_human_gate`
- `race::tests::attract_start_checks_all_four_words_and_increments_reentry_byte_without_running_human_pause`
- `race::tests::original_progression_masks_after_increment_and_uses_order_table_not_low_nibble_track`
- `race::tests::prepared_frame_executes_all_owners_in_source_order_and_increments_timer_before_return`
- `race::tests::winner_frame_still_advances_hud_timer_and_copies_input_before_postrace_dispatch`
- `race::tests::selected_race_preparation_orders_real_owner_setup_and_one_service_before_ready`
- `race::tests::timeout_reset_follows_ai_setup_and_preserves_all_other_driver_state`
- `race::tests::conditional_human_record_index_multiplies_at_byte_width_before_route_access`
