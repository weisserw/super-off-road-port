# Computer driver contracts — independent Stage 3 trace

Source: frozen Round29 `reference/source/game.asm` SHA256
`4e3e1a45e4409ce63a3354740a2cdd3962cd3d9975d88d22ecee40c9d29ded7f`.
CPU bank00 mapped $8000+physical, DB00/DP0000. Independently read every assigned
range below, original ROM bytes, callers/callees and symbol aliases; planning is
navigation, not an expectation source. Source is unchanged. No rule implemented.

Platform context consulted: toolkit docs/development.md SNES row, docs/snes.md
address/width context and docs/snes-native.md domain boundary. WDC W65C816S
March2024 datasheet sections2.4/2.7/2.8,5 operation table and7.20/7.23:
https://www.westerndesigncenter.com/wdc/documentation/w65c816s.pdf . M/X control
independent operand widths; XBA swaps accumulator bytes even with M8, SEP X8
clears upper index bytes, byte stores retain adjacent byte; CMP sets carry for
unsigned >=, BPL tests sign not signed overflow. PHP/PLP preserve saved branch
sign across later arithmetic. Arithmetic here binary; no SED in callers.

## Shared vector quantizer and target heading

`quantize_signed_vector_to_lookup_index` game.asm2305 physical0842..0885:
PHP, REP M16 stores original lowX/highY then SEP M8/X8. Absolute byte magnitudes
use EOR FF/INC; -128 has unsigned magnitude128. Both magnitudes shift together
until each<8. Restore original signs to nibbles then Y<<4|X; when a small
negative component shifts to0, its negative-zero nibble remains0. Extremes:
(-128,127) magnitude(128,127)→(4,3), packed3C; (127,-128)→C3.
(8,7)→(4,3)→34, (7,8)→43. Both negative7→99. Tests use fixed vectors and nine
small-grid literals, not another quantizer. Contact owns committed difference
reduction/Y-negation; it must call this same domain helper, not atan2.

`update_computer_trucks`0885..088D checks exactly mask0F then RTS; other masks
shift4 and scan3→0, consuming original low4 human bits, high bits discarded.
For each computer: acceleration=6, turn step table[slot] (03/04/04/04), signed
bias converted by ASL/ROR into a signed word bias*128 (arithmetic half-cell).
Checkpoint*4 is computed M16 here; cache route record before later timeout.
Set throttle word8000 at word-array index slot*2. Each packed coordinate keeps
BFE0 after optional bias, suppressing bias independently when that coordinate's
low5 extent==1. Word subtraction wraps then logical>>6. Assembly XBA/LDA/XBA
packs **reducedY low into quantizer X and reducedX low into quantizer Y**. Lookup
bearing then adds quarterturn40 with byte wrap. Table values are source, not
ideal trig: index10=BF,01=FF,F0=40,0F=7F,00=FF,CC=5F. Thus fixed axis target
expectations FF/3F/80/BF, zero3F, bias(-4,-4)→indexCC→9F.

## Steering delay and lock

`turn_ai_heading` internal093E stores desired low and incoming word-array index;
current_truck_index selects byte arrays. Nonzero bias_delay decrements; exactly
expiry clears route_bias. This still occurs while locked. Nonzero steering_lock
decrements and returns even when it becomes0; next call can turn. Otherwise
byte delta=desired-heading; PHP saves delta sign; (delta+4) wrapping CMP8 skips
for -4..+3 inclusive. Restore delta sign for ±slot turn step, low byte store only.
Zero delay does not clear an existing bias. Fixed high-byte fixtures protect word
heading from accidental full-word mutation. No hard angle snap is supported.

## Timeout, cached rectangles, next targets and lap effects

`tick_ai_route_timeout`098D: only odd animation counter calls DEC timeout. Zero
underflowsFF, not expiry. On1→0 decrement checkpoint; sign set wraps to
route_checkpoint_count, then timeout100. It preserves caller word-array X.
`test_ai_waypoint_progress`09AD uses cached route offset Y from BEFORE timeout;
never reread the rectangle using mutated checkpoint. Both axes extent=(low5)<<7,
center=word&BFE0; accepted iff wrapped(center-world+extent)<extent*2. Lower world edge
excluded, upper world edge included (corrected by race worker; see note below). Extent0 empty. Rank distance instead preserves bit14.

On accepted rectangle, rank/adjust BEFORE reading parameter and reloading timeout
from exact8 entries FF FF FF C8 C8 C8 96 96. If current checkpoint==0 increment
lap, call race-owned last-lap helper (lap>=4 and banner<19→30), set winner slot+1
only when resulting lap==5, then advance target. checkpoint==route_count resets
checkpoint0 and returns (no lap/no bias clearing). Other checkpoint advances.
Source banner helper changes M/X to8 but preserves byte X; native owner returns
named effects/state rather than a register file. Winner can be overwritten by
later finishing slot in same descending service: no existing-winner guard here.

`advance_ai_route_index`0A1B clears bias, increments byte checkpoint and computes
record offset in BYTE width (ASL twice). Natural record counts remain bounded;
malformed added routes must return errors, not synthesize coordinates. If OR of
two coordinate low bytes &1F >=4, consume RNG and set bias=(draw&7)-4,delay100;
otherwise retain old delay and no draw. Then inspect new record first word HIGH:
FE redirects to second word LOW unconditionally. FD consumes a further draw:
(draw&3)==0 redirects; nonzero increments current checkpoint once more. Thus
FD redirect is one of four outcomes, not the archive comment's inverted prose.
Bias draw precedes FD draw and bias/delay remain even after redirect/skip.
Other bytes including FF simply return; source subsequent route behavior remains
race/tracks composition, not a fabricated special next-target branch.

RNG fixtures use actual runtime::RandomState, no duplicate oracle:
12/34/56/78/9B gives DF then6A, history after draws fixed from runtime instruction
trace. DF&7-4=3; DF&3=3 skip;6A&3=2 skip. State12/00/00/00/00 gives first0.

## Ranking and human-relative difficulty

`rank_trucks_by_race_progress`249D..2587 builds candidate lap+1 bytes; negative
candidate laps are skipped (normal laps1..5); selected candidates markedFF.
For each rank scan3→0. Greater unsigned lap then greater unsigned(checkpoint-1)
wins; checkpoint0 comparesFF and therefore precedes checkpoint1 at equal lap.
For equal progress choose absolute WRAPPING word difference from next route
coordinate. X selected only if X extent==1, else Y; center maskFFE0 (bit14 kept).
Compare current-best distance with candidate distance: equal replaces best, so
lowest slot wins exact ties in descending scan. Difference8000 remains8000 after
two's-complement absolute and compares unsigned. Malformed missing route data is
native error; no zeros/clamps invented. Tests fix both coordinate branches and ties.

`adjust_ai_state_using_race_rank`25FB preserves word X/Y/status. Find current slot
in ranking and subtract preceding computer count: result is preceding human count,
not raw four-truck rank. RNG first draw compares rank threshold_one. If >=, compare
speed upgrade with driver parameter: below→increment, above→decrement nonzero,
equal→special override. If first draw<threshold_one, compare SAME draw to
threshold_two: >= decrements nonzero; below does nothing. Original one/two tables
both128 in all four cells, so this second decrease arm is conditional data behavior,
not naturally selected by original tables. Synthetic threshold fixture separately
contracts it without pretending it ran naturally.

Equal speed: human-rank0 requires lap==4 or skips second draw. On that path LDA lap
leaves A=4 so parameter_threshold index is **lap4**, not current speed. Other ranks
use equal speed as index. Human-rank1 adds38 to threshold BYTE wrapping. Consume
second RNG; if draw<threshold write BOTH controller bytesFF; else retain original.
Fixed DF/6A, parameter4 threshold50 +38=88 forcesFFFF; leader original threshold50
rejects6A, distinct fixture70 demonstrates lap-index4 vs parameter1 threshold30.
Parameters5..7 share physical threshold bytes with rank window at268C..268F;
load as overlapping windows, not exclusive ledger regions. All writes happen after
movement; controllerFFFF affects subsequent motion/edges, not this frame's motion.

## Parameter setup and source cross-owner underflow

`derive_ai_parameters_from_human_upgrades`258B: active humans supply max speed
(start0) and min speed(start6). Every computer gets suspension6. All parameters
seed max-1 except max0 stays0; slot2 overwritten max(min-1,0). Scan X3→0 for first
computer and override that slot with min(max+2,7), not always slot3. Attract no
humans therefore params[0,0,5,2], all suspension6. High masks use slot masks1/2/4/8.

All four human bits set: scan decrements X0→FF, BPL fails but falls through override
and STA parameter_base,X. Byte XFF yields0797+FF=0896, route_words079F offsetF7.
This is NOT a fifth native driver. The race initializer calls setup at game.asm156,
THEN load_route_pointer at164 and grid/assets. The copy stops at its FF terminal;
all32 original route windows are30..126 bytes, so tail offsetF7 is NOT overwritten
and is outside every supported route read window.
`SetupEffects.unused_route_tail_byte` retains this known side-effect boundary
without emulated RAM or invented error. Non-natural direct setup caller/long added
route lifetime is explicitly residual uncertainty; do not port a destructive route
write into added content or silently claim source loop is bounds checked. All-human params remain
seeded/min values; test validates report and suspension preservation.

## Full-domain service and cross-area handoff

service_computers mutates shared Truck plus Driver/RaceAi with real Route/AiTables
and runtime RNG; not a test-only mock. Fixed service phases accompany actual
controller/acceleration/heading/world assertions. It visits computer3→0, writes6
and8000 before steering, then waypoint/rank; humans untouched, no RNG on rejected
rectangles, no movement this service. Source run_race_frame/update_race_frame_services
executes AI after controller/checkpoint, objects and HUD. Race worker owns that
integration order and finish transitions; rendering owns banner staging; motion
owns steering_lock fields/recovery; contact reuses quantize_vector; tracks owns
loaded route validity and unused setup-tail lifetime. No area implementation here.

Pending boundary hypotheses: unnatural AI parameters>7, lap overflow>=127 and
route record indices beyond original content have no supported native behavior;
validate added content and full snapshots, don't invent source wrapping reads.
Physical0A6C..0A74 eight64 bytes have no established AI consumer; adjacent timeout
table exactly8 cells indexed0..7. No exclusion/disposition is made from proximity.

## Exhaustive primary area navigation reviewed

- region-134 physical$000842..$00088D; game.asm:2305; quantize_signed_vector_to_lookup_index
- region-710 physical$00088D..$000890; game.asm:2371; skip_human_ai_update
- region-135 physical$000890..$000950; game.asm:2383; steer_computer_trucks
- region-711 physical$000950..$000953; game.asm:2503; clear_ai_route_bias
- region-136 physical$000953..$000958; game.asm:2514; test_ai_steering_lock
- region-712 physical$000958..$00095E; game.asm:2527; expire_steering_lock
- region-137 physical$00095E..$000989; game.asm:2541; turn_ai_toward_target
- region-918 physical$000989..$00098D; game.asm:2574; ai_truck_turn_steps
- region-138 physical$00098D..$00099C; game.asm:2599; tick_ai_route_timeout
- region-713 physical$00099C..$0009AB; game.asm:2618; rewind_ai_checkpoint
- region-139 physical$0009AB..$000A05; game.asm:2636; advance_ai_checkpoint_inside_rectangle
- region-714 physical$000A05..$000A09; game.asm:2694; restart_ai_route
- region-232 physical$000A09..$000A12; game.asm:2706; increment_ai_lap
- region-715 physical$000A12..$000A1A; game.asm:2720; record_ai_finish
- region-140 physical$000A1A..$000A52; game.asm:2736; select_next_ai_route_target
- region-716 physical$000A52..$000A59; game.asm:2775; randomize_ai_waypoint_advance
- region-233 physical$000A59..$000A60; game.asm:2788; apply_ai_route_redirect
- region-717 physical$000A60..$000A64; game.asm:2802; advance_ai_checkpoint
- region-919 physical$000A64..$000A6C; game.asm:2811; ai_parameter_waypoint_timeouts
- region-1442 physical$000A6C..$000A74; game.asm:2822; unresolved stored data
- region-178 physical$00249D..$002522; game.asm:8601; rank_trucks_by_race_progress
- region-246 physical$002522..$002555; game.asm:8684; break_ranking_tie_with_y_distance
- region-179 physical$002555..$002587; game.asm:8721; store_next_ranked_truck
- region-986 physical$002587..$00258B; game.asm:8759; truck_participation_bits
- region-350 physical$00258B..$0025DB; game.asm:8785; derive_ai_parameters_from_human_upgrades
- region-738 physical$0025DB..$0025DD; game.asm:8840; clear_ranking_candidate
- region-351 physical$0025DD..$0025E9; game.asm:8851; store_minimum_ai_parameter
- region-739 physical$0025E9..$0025EC; game.asm:8867; advance_rank_scan_index
- region-352 physical$0025EC..$0025F4; game.asm:8879; raise_ai_parameter_with_cap
- region-740 physical$0025F4..$0025F6; game.asm:8895; set_attract_ai_parameter
- region-353 physical$0025F6..$0025FB; game.asm:8906; return_after_ai_parameter_setup
- region-180 physical$0025FB..$00261F; game.asm:8920; adjust_ai_state_using_race_rank
- region-273 physical$00261F..$002621; game.asm:8950; count_preceding_computer_truck
- region-181 physical$002621..$002634; game.asm:8962; test_rank_based_random_adjustment
- region-741 physical$002634..$002674; game.asm:8985; adjust_ai_speed_upgrade
- region-182 physical$002674..$002679; game.asm:9029; test_second_rank_adjustment_threshold
- region-742 physical$002679..$002681; game.asm:9042; decrease_ai_speed_upgrade
- region-183 physical$002681..$002687; game.asm:9055; return_rank_adjustment_helper

### Superseded lifetime interpretation

During initial tracing, setup-before-load suggested the all-human spill would be
overwritten by route loading. That hypothesis was rejected by rereading the
copy's FF-termination rule and independently measuring all32 ROM streams:
30..126bytes, never the spill at247. The supported conclusion is unused tail,
not overwritten data. Native extensions retain explicit validated Route content;
they must not materialize the source's stray tail as a route record. No reference
classification, ledger disposition or natural runtime claim follows from this.

## Race-worker source correction: rectangle edge orientation

The original trace/contract called the lower world edge inclusive and upper
exclusive. Independent race-worker reread of game.asm test_ai_waypoint_progress
and test_packed_checkpoint_rectangle establishes subtraction center-world, not
world-center. With center2000 and extent0100, world1F00 produces0200 and CMP0200
rejects; world2100 produces0000 and accepts. Thus lower world edge excluded,
upper included. The affected fixed-vector test is corrected in place without
implementation; old ai-area logs remain preserved, race reconciliation logs
record the still-pending result. Other centered fixtures remain unchanged.
Race now owns Progress; RaceAi is a public alias preserving AI contract API,
and banner_state becomes source word width. No second lap/winner authority.

## Stage 4 synthetic distant-record correction

Two Stage3 fixtures called 0x6002/0x6002 distant from world0x2000, but the actual source target/rectangle mask is BFE0, so 0x6002 & BFE0 = 0x2000. Once implemented, those fixtures correctly accepted the supposedly distant rectangle (and the short service route then lacked a next Y word). The fixtures now use 0x7002/0x7002, whose center after the same mask is0x3000. This retains their existing rejection/no-RNG assertions and tests the intended source branch; no implementation mask or source expectation was weakened. Initial outcome35pass/2fail preserved in checks/implementation/domain-results.json and prior Stage3 logs remain unchanged.

Stage4 Game ownership: setup_parameters.unused_route_tail_byte is consumed into RaceState.route_tail_byte, exported in semantic observations and complete snapshots. Original32route windows30..126bytes do not overwrite or read F7. A long file-backed added route copy replaces that byte from its corresponding word before subsequent route service, following source copy order as an extension lifetime policy; no original long-route claim is made.
