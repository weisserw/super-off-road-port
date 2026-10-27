# Truck motion contracts — independent source trace

Area physics; worker `/root/behavior_contracts/physics`; frozen Round29
`reference/source/game.asm` SHA256
`4e3e1a45e4409ce63a3354740a2cdd3962cd3d9975d88d22ecee40c9d29ded7f`.
`reference/reading.asm`, symbols and local matching `input/game.rom` were consulted.
All physical ranges below are half-open. Bank-zero CPU aliases add $008000;
this translation is not a general LoROM conversion. Assigned source coverage is
listed at the end. Source comments/planning claims were checked against actual
instructions; corrections below leave frozen reference unchanged.

Platform context: toolkit docs/development.md SNES platform row, docs/snes.md
(address/width model) and docs/snes-native.md (independent helpers/limits).
WDC W65C816S datasheet, sections2.4/2.7/2.8 and operation tables5-5:
https://www.westerndesigncenter.com/wdc/documentation/w65c816s.pdf
establish variable accumulator/index width, byte/word sign and overflow flags,
BIT memory N/V, BPL and ROR carry semantics. Local pinned backend
vendor/jgenesis/backend/snes-core/src/ppu/registers.rs write_m7a/write_m7b and
read_mpyl/read_mpym/read_mpyh independently documents signed16×signed8 product
and byte selection. No CPU execution or desktop/emulator experiment was used.

## Native boundaries and calling contracts

`Truck` owns named world position, word speed/heading, controller/upgrades,
fractions, four Wheel records, contact/recovery state and staged screen fields.
No original address-indexed RAM or CPU registers are introduced. Speed is stored
as a word because numerous source operations replace only its low byte, leaving
the high byte unchanged; displacement interprets only the low byte as signed.
Wheel heights, velocities and fractional gravity are bytes with explicit wraps.
World coordinates remain source integer words; no metres/float physics inferred.
MotionTables is immutable game-owned data; all ranges are confined to loading
or independent fixture parsing. Production rule functions are visibly pending.

`steer`, `use_nitro`, `accelerate` expose real domain mutations. Their coordinated
`update_powertrain` establishes call order, not a second test implementation.
`update_trucks` accepts real Terrain/SamplingTables/Projection and four actual
Truck records; its integration contract uses a synthetic interior flat field,
not mock callbacks or an oracle algorithm. MotionPhase observations establish
cross-call order. Stage4 must compose objects/contact domain owners as those
interfaces appear; object animation state and retained transient events must be
included in the eventual owning state/clone/full snapshot. No test here claims
current Port skeleton performs gameplay.

## Per-slot service and shared scheduling

`update_four_trucks`, physical$0CD4..$0D23 (game.asm3391), enters M16/X8,
word-index0 then2,4,6. For each slot it loads controller, heading, world and speed
into working values. Wheel last index is (word index*2)|3, i.e.3,7,11,15.
Calls: acceleration(which calls steering then nitro), displacement, all four
terrain samples, special surface object contact, then blocked response.
These calls are not pure: terrain writes sampled heights, block flags, marker
history and loads the OLD truck_surface_summary. Special-surface logic can
halve working nonnegative speed/start animation/sound before coordinates commit.
The objects area owns that rule; preserve its state mutations on rejection.

If BOTH last-wheel block bytes2 and3 are nonzero at this point, source reloads
COMMITTED world and speed, calls sign-reversing half-speed helper, blocked count,
then contact effect. It skips the integrated-height average; new proposed samples,
marker history and working heading survive. Otherwise it calls contact-transition/
single-axis response, then average_surface. One-axis response calls effect BEFORE
blocked count, unlike full rejection. Project working world, project truck screen
(rendering owns priority adjustment), and construct shadow next. If cooldown is
nonzero, decrement it and skip pair contacts even on1→0. If zero, call pair
contacts; then commit working speed/heading/world. Later slots see earlier commits.
Effects emitted before projection use the previous staged screen coordinates.

`run_race_frame` physical$0198 calls the whole four-slot service BEFORE sixteen
wheel integration, suspension correction and pose selection; OAM sorting/animation
then occur before human control copying/checkpoints, objects/HUD and AI. Thus
current motion consumes retained prior controller words, and current terrain
samples influence this frame's subsequent wheel integration, while surface average
and current shadow run earlier. AI/controller services are not reordered ahead of
movement by a native input convenience. Full integration test checks domain
phases and cooldown/pair alternate branch; race owns global dispatcher contract.

## Steering and powertrain

`update_acceleration_from_upgrades` $1478, `update_rate_limited_steering` internal
$1575 through$15DC: steering uses byte fraction+table[upgrade] and updates angle
only when unsigned carry sets. Rates56,60,64,68,72,76,80. No steering-lock test
occurs here; AI's turn routine owns lock gating. On carry the four independent
controller tests run in order: RIGHT$0100 subtract8; R$0010 subtract4;
LEFT$0200 add8; L$0020 add4. Each wraps heading with$00FF. Multiple bits combine;
all four cancel. Prepared initialization does not suppress this steering update.

Nitro $151B..$1575 returns immediately if prepared_race_frame is nonzero,
without touching history. Otherwise ASL high controller twice leaves Y$4000 in
carry. ROR history inserts current Y into bit7 and previous Y into bit6.
BIT reads N/V from the history itself: proceed only bit7=1,bit6=0. Empty stock
still latches history. Valid packed BCD stock decrements with SED/SEC/SBC1/CLD:
$10→$09; $01→$00; no synthetic saturation of invalid BCD is contracted.
Working speed low byte becomes$7C. If prior projected X high byte is0, stage
boost sprite at prior X/Y and animation cursor1; no +4 clipping here. Otherwise
skip sprite/cursor but still request sound$0202 through send_race_sound_if_human.
That helper $0256 tests ANY nonzero low-byte active-human mask, not this truck's
bit. Attract still consumes stock and boosts with no sound. Held press bit6
suppresses retrigger until release then new press. History sequence0→80→C0→60→B0
is fixed independently from ROR, not learned from native output.

After steering/nitro, acceleration always accumulates table16 + prior fraction16,
stores low8 as a zero-extended word, takes high8 as speed step. Seven words
$0180,$01C0,$0200,$0240,$0280,$02C0,$0300. Starting fraction$F0 gives respective
steps2,2,2,3,3,3,3 and retained$70,$B0,$F0,$30,$70,$B0,$F0. Counter advances even
without B or ground contact. BIT working_controller while A16 tests bit15 B;
nonzero contact marker is additionally required. No throttle/contact gives byte
drag: negative increment, positive decrement, zero unchanged.

Speed selector is speed-upgrade byte unless0: human0 uses table0, AI0 uses
special table8. Nine limits60,64,68,72,76,80,84,101,48. AI steering service explicitly
sets acceleration upgrade6; speed parameter may reach7, needing the eighth limit.
Nitro does not select a separate maximum despite the archived generic comment.
Negative speed adds the fractional step as byte and stores immediately, including
$FF+3→$02. Nonnegative below limit adds step; signed overflow returns unchanged.
If result is still below limit, store. At/over limit or a step reaching it,
animation-even returns with OLD speed, animation-odd stores OLD speed−1.
Thus59+1 against60 gives59 on even and58 on odd, not60 or59 respectively.
Powertrain tests cover nitro normal drag/limit AFTER boost ($7C→$7B when no B,
or on odd frame at cap), and prepared steering/fraction without nitro latch.

## Signed vector displacement and sign-reversing response

`integrate_heading_displacement` physical$0D86 internal: (heading>>3)*2 selects
32 signed word vectors. Source headings naturally byte-masked; only supported
0..255 headings are used by contracts. X vectors begin256,224,...; Y begin0,32,...
Move Y then X by signed16 vector × signed8 speed LOW BYTE, take product bits8..23
via MPYM word and wrapping-add to world. Read of MPYL is discarded by the next
LDA world; its low product byte is not an additional fractional accumulator.
Vector224 ×−1=−224; arithmetic product middle/high gives−1, not0. High speed
word bits are ignored. At heading255, speed8, vectors224/−32 yield X+7,Y−1.

`halve_signed_truck_speed` $1032..$104C is a notable archived-comment error:
negative original speed first becomes magnitude, then LSR. BIT ORIGINAL speed
sets N. BPL branches to EOR/INC (negation) for NONNEGATIVE original, while negative
original stores positive magnitude. It therefore reverses direction and halves,
not preserves sign: +63→−31,$C1(−63)→31,$80(−128)→64. ±1→0; high word untouched.
Contract name reverse_halved_speed explains the actual behavior.

## Wheel and suspension branches

`average_truck_surface_and_begin_suspension` $0AF3 reads INTEGRATED heights,
not sampled targets despite its archived introductory summary. It widens each
byte separately, sums four in16 bits, divides by4 and stores summary byte.
[200,201,202,203]→201; no pairwise byte wrap occurs in this average.

`integrate_sixteen_wheels` internal$0B24 clears all four contact markers then
visits wheel15 down to0. If velocity byte is nonnegative, always gravity.
If negative: target−integrated in8 bits with borrow; target below height→gravity.
Otherwise delta1 sets integrated=target and velocity$FF, fraction$FF, marker$FF,
then common height addition gives target−1. Delta0 stores velocity0 and marker.
Other nonnegative deltas compare against (7−(suspension_upgrade>>1))+1;
less uses delta; equal/above clamps to7−half-upgrade. Store velocity/fraction$FF,
set truck marker, then wrapping-add velocity to height. A positive upward
velocity on the next frame takes gravity, not repeated target comparison.
Gravity subtracts$90 from fraction, decrements velocity iff subtraction borrows,
then wraps height+velocity. No native nonnegative-height clamp is implied.
Tests cover0/1/interior/cap/exact threshold, all seven upgrades, gravity equality/
borrow and high-byte wrap; per-truck marker OR behavior is checked across fleet.

`apply_suspension_speed_correction` internal$0B97 only operates if contact marker
nonzero. Byte sums wheels0+1 and2+3 wrap separately. Byte rear−front sets sign;
PHP retains that sign while magnitude shifts right3 AND3. Restore original sign,
then BYTE speed−signed delta; BVS suppresses store. High speed byte persists.
[40,40,48,48] delta+2; reversed delta−2. Rear−front40 yields (40>>3)&3=1,
not5. At low speed$80 minus2 and$7F minus(−2), signed overflow leaves speed unchanged.

## Body pose and shadow

`select_four_truck_poses` internal$0BDF iterates3→0. Lateral is byte
(w0+w2)−(w1+w3)+$28; longitudinal is (w0+w1)−(w2+w3)+$28. Byte sign clamps negative
to0; otherwise >=$50 clamps$4F. Shift4 gives selectors0..4. The historical clamp
labels name the other axis inconsistently; field uses and operations control.
Heading>>3 gives0..31. Lookup25 word grid at$0CA2 indexes5*longitudinal+lateral.
If selected grid word is nonzero, heading field halves before OR; flat center
word0 uses full heading. Example all40 wheels: selectors2,2 and heading255→31.
[60,40,60,40] gives lateral4,longitudinal2; grid$50 OR15=$5F.
Byte wrapping matters: [100,0,100,0] lateral200+40=$F0 negative→0, not high clamp.

`pack_truck_pose_source` internal$0BFD: combine low3 pose bits shifted6 with
byte-swapped ((pose&$FFF8)>>1). Supported full pose domain0..415 crosses bank:
7→$01C0,8→$0400,255→$7DC0,256→$8000,415→$CDC0. Rendering's upload uses source
high bit to choose bank1/2 and two strips; that area owns DMA and pixels.

`build_truck_shadow_sprite` $0A74..$0AEB consumes prior projected X, attributes,
sampled wheels and projection vertical term. X low byte. Attribute word masked
$31FF then OR$0800. Crucially sampled wheel0+1 and2+3 wrap BYTE separately BEFORE
widening their sum/dividing4. Four200 samples yield pair144+144 /4=72, not200.
Y low byte of ((($2C00−vertical−(average<<3)) wrapping16)>>4)+$2A.
For all200,vertical0:($2C00−576)>>4=668,+42=710→198.
For all40,vertical$2C00:−320→$FEC0>>4=4076,+42→22 low byte.
Packed high OAM clears this truck's X mask only, sets it when projected X high
byte is nonzero, always OR$AA large-size pairs. Mask table all8 bytes retained,
natural word indices use0,2,4,6 masks1,4,16,64. Odd cells contain2,8,32,128;
no natural consumer established, no new gameplay meaning/exclusion invented.
Rendering retains sorting/OAM transfer/full composition and projection priority.

## Blocking, recovery and contact effects

`update_truck_contact_transition` internal$0E0F: transition0 and no contact→1;
transition nonzero and contact→0 plus effect. Otherwise retain. If working speed
bit$80 sets, load flags[0..1] as word, XBA, store flags[2..3]: flags2=old1,
flags3=old0. This is a swap/copy, not clearing old front flags.
After this, flag2 chooses heading−4 masked byte; flag3 chooses heading+4. Selector
uses ORIGINAL heading quadrant, not rotated angle. Table$0E7D [0,1,0,1]:
negative-turn path table1 restores X, table0 restores Y; positive-turn path
inverts that choice. Restored coordinate comes from committed truck. Neither
flag→blocked_frames0, speed unchanged (aside from earlier object/powertrain).
Full rejection bypasses this rotation and contact-transition path as above.

`count_blocked_truck_frames` $0DD4 always sets steering_lock2 and ai_route_bias0,
increments byte blocked count. At>=16 resets0; human retains speed and lock2.
Computer gets lock15 and LOW working and committed speed$81 (upper byte preserved).
This deliberately writes committed speed before final motion commit. AI owns
subsequent steering-lock countdown/route generation; no generic reverse float.

`spawn_truck_contact_effect` internal$0EBC uses old screen projection: cooldown
must0, X high0, X low+4 must NOT carry. Y low+6 wraps. On success cooldown24.
It does not tick cooldown itself; later sprite animation owner must locate and
preserve countdown. Offscreen/carry/cooldown failures leave all state unchanged.
Both-block and one-axis differing call order is documented for composition; the
contact effect/animation owner must preserve staged coordinates and visibility.

## Test evidence, residual limits and handoff

`motion.rs` has36 substantive native tests, independently fixed ROM-derived and
synthetic branch expectations. All compile/discover, none ignored. All36 currently
fail at explicit unimplemented rule entry points, not fixture construction,
compiler errors or timeouts; stage4 implements them. Four pre-existing infrastructure
tests pass. Exact commands/logs: checks/contracts/physics-results.json. Suite110
tests discovered. Initial run caught a fixture construction overflow in `250+delta-7`
(byte intermediate overflow before subtraction); corrected equivalent expected
sample to243+delta, retaining original run under `physics-*-initial.log` and
`physics-initial-results.json`. Recompiled/discovered/reran required area/infra;
final all36 failures are explicit unimplemented entry points. No behavior assertion
was weakened. No behavior implementation, QA, audit, toolkit change, original
asset embedding or frozen reference edit occurred.

Known uncertainties remain those already recorded: source world physical units
are not needed for exact integer behavior; startup RNG and terrain boundary state
retain separate IDs. All supported physics branches have explicit semantics, so
no new unsupported assumption is imposed here. Invalid upgrade selectors and
invalid BCD values are outside established source writers; extension/snapshot
validation must preserve valid finite domains. Headers/track extensions don't
change finite source upgrade table bounds. Stage4 must reconcile whole-service
object/contact state/events with each owner's contracts, include Truck/Wheel and
all transient state in semantic observation/snapshot/clone, preserve source stage
ordering and avoid interpreting passing domain tests as whole-game QA evidence.

## Exhaustive assigned region navigation

This list records independent reviewed source ranges, not ledger disposition.

| Ledger ID | Physical range | Original source |
| --- | --- | --- |
| region-141 | $000A74..$000AEB | game.asm:2833 |
| region-920 | $000AEB..$000AEC | game.asm:2904 |
| region-1443 | $000AEC..$000AED | game.asm:2915 |
| region-921 | $000AED..$000AEE | game.asm:2922 |
| region-1444 | $000AEE..$000AEF | game.asm:2933 |
| region-922 | $000AEF..$000AF0 | game.asm:2940 |
| region-1445 | $000AF0..$000AF1 | game.asm:2951 |
| region-923 | $000AF1..$000AF2 | game.asm:2958 |
| region-1446 | $000AF2..$000AF3 | game.asm:2969 |
| region-142 | $000AF3..$000B69 | game.asm:2980 |
| region-268 | $000B69..$000B6B | game.asm:3055 |
| region-143 | $000B6B..$000BC8 | game.asm:3067 |
| region-144 | $000BC8..$000C33 | game.asm:3133 |
| region-718 | $000C33..$000C35 | game.asm:3209 |
| region-145 | $000C35..$000C39 | game.asm:3220 |
| region-719 | $000C39..$000C3B | game.asm:3233 |
| region-146 | $000C3B..$000C5D | game.asm:3245 |
| region-720 | $000C5D..$000C5F | game.asm:3274 |
| region-147 | $000C5F..$000C63 | game.asm:3285 |
| region-721 | $000C63..$000C65 | game.asm:3298 |
| region-148 | $000C65..$000CA2 | game.asm:3310 |
| region-395 | $000CA2..$000CD4 | game.asm:3353 |
| region-149 | $000CD4..$000D23 | game.asm:3391 |
| region-234 | $000D23..$000D3F | game.asm:3438 |
| region-150 | $000D3F..$000DF1 | game.asm:3461 |
| region-235 | $000DF1..$000E09 | game.asm:3563 |
| region-151 | $000E09..$000E3A | game.asm:3584 |
| region-722 | $000E3A..$000E3F | game.asm:3622 |
| region-152 | $000E3F..$000E7D | game.asm:3636 |
| region-924 | $000E7D..$000E81 | game.asm:3673 |
| region-236 | $000E81..$000EA0 | game.asm:3698 |
| region-153 | $000EA0..$000EF8 | game.asm:3723 |
| region-239 | $001032..$00104C | game.asm:4027 |
| region-160 | $001478..$0014D1 | game.asm:5882 |
| region-727 | $0014D1..$0014D6 | game.asm:5941 |
| region-161 | $0014D6..$0014FC | game.asm:5955 |
| region-728 | $0014FC..$001500 | game.asm:5990 |
| region-162 | $001500..$001504 | game.asm:6003 |
| region-959 | $001504..$00150D | game.asm:6014 |
| region-960 | $00150D..$00151B | game.asm:6036 |
| region-163 | $00151B..$0015A8 | game.asm:6063 |
| region-729 | $0015A8..$0015B5 | game.asm:6150 |
| region-164 | $0015B5..$0015BB | game.asm:6165 |
| region-270 | $0015BB..$0015C8 | game.asm:6178 |
| region-165 | $0015C8..$0015CE | game.asm:6193 |
| region-730 | $0015CE..$0015DB | game.asm:6207 |
| region-166 | $0015DB..$0015DC | game.asm:6222 |
| region-264 | $0015DC..$00161C | game.asm:6230 |
| region-265 | $00161C..$00165C | game.asm:6237 |
| region-961 | $00165C..$001663 | game.asm:6244 |

## Stage 4 composition interface enrichment

The pre-objects update_trucks contract accepted neither persistent contact-animation state nor ROM bearing data, so it could not execute required special-surface/pair effects. Stage4 enriches it with a borrowed actual contact array and immutable ContactTables, returning FleetFrame with phases, sprite/audio effects, shadows and pair visits. Existing phase/state expectations are unchanged; the synthetic test now supplies empty contact state and locally loaded original bearings. The implementation runs real terrain, objects::special_surface and contact::service_contacts before each sequential commit, preserving old committed speed/world while powertrain mutates its proposal.

## Iteration3 R2 — explicit powertrain proposal and retained outputs

The source order/width contracts above are unchanged. Actual steering, nitro and
acceleration rules now live on a compact PowertrainState with immutable inputs
in motion/powertrain.rs. The fleet consumes a PowertrainOutcome, applies only
its five retained stock/history/fraction/animation outputs, and carries proposed
speed/heading through displacement, terrain and contact to the existing commit.
The complete retained result is destructured explicitly, so adding a powertrain
output cannot silently disappear from a handwritten fleet reconciliation list.
Wheels, world/projection and collision state are not copied into that proposal.
Isolated domain entry points retain their existing immediate-commit behavior.

The preimplementation high-byte/retained-field regression was first passed on
completed M1. Existing steering, prepared/nitro, displacement, blocked-response,
current-committed-versus-working contact and sequential fleet contracts remain
unchanged. This is refactoring of the same game model, not modernization of the
physics. Per-target checks and natural comparison evidence are recorded in
modernize/changes.json under iteration3/R2.
