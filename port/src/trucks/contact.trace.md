# Independent collision trace and executable contracts

Reference: frozen Round29 `reference/source/game.asm`, SHA256
`4e3e1a45e4409ce63a3354740a2cdd3962cd3d9975d88d22ecee40c9d29ded7f`.
Also independently read symbols.asm, terrain neighborhood/classifier/interpolation,
shared vector quantization, human-gated sound and motion callers. Primary ownership
is all thirteen regions below, not only the named entry in the planning inventory.
No original source/reference bytes or ledger dispositions changed.

CPU execution aliases below are bank00, physical+8000. DP0000/DB00 and binary
arithmetic apply to natural race caller; M/X changes are followed explicitly.
The toolkit platform-reference table, docs/snes.md and snes-native.md were read.
WDC [W65C816S datasheet](https://www.westerndesigncenter.com/wdc/documentation/w65c816s.pdf)
sections2.4/2.7/2.8, operation tables and accumulator switching caveats establish
low-byte preservation/high-byte hiding, width-dependent arithmetic and flags.
Source multiplication uses signed16 M7A * signed8 M7B; distance reads product
low16, separation reads bits8..23. Native rules must use named quantities rather
than a CPU/register or WRAM emulator. Tests use synthetic native data/local ROM,
not captured port outputs or a second response implementation.

## Exhaustive ownership and callers

| Ledger region | Physical half-open | Source entry/line | Contract coverage |
|---|---|---|---|
|region-117|028B..02C7|check_other_truck_contacts:446|service scans, distance entry|
|region-118|02C7..030E|measure_coarse_truck_distance:492|absolute/coarse/sentinel|
|region-224|030E..0312|return_noncontact_distance:538|axis cutoff sentinel|
|region-119|0312..036A|classify_truck_contact_angles:554|bearing/sector packing|
|region-701|036A..039C|reverse_contact_speeds:614|negated exchange and sound|
|region-120|039C..03A4|test_parallel_contact_sectors:653|00/22 dispatch|
|region-225|03A4..03E1|exchange_contact_motion:668|exchange/parity/byte delta|
|region-1281|03E1..0409|deflect_current_truck_contact:713|conditional isolated entry|
|region-121|0409..0456|deflect_oblique_truck_contacts:746|11/13/31/33 headings|
|region-226|0456..0492|resolve_asymmetric_contact:798|remaining cases/impact gate|
|region-122|0492..0545|separate_contact_pair_against_terrain:840|midpoint/probes/stores|
|region-227|0545..0546|return_after_blocked_separation:935|blocked other keeps world|
|region-123|0546..055F|quantize_relative_heading_sector:947|all boundary endpoints|

Natural caller game.asm:3452..3492 finishes projection and shadow BEFORE pair
contacts, then commits current working speed, heading and world words AFTER the
scan. Existing projection/OAM are not refreshed by positional separation here;
this late adjustment is intentionally visible next service frame, with motion's
projection/staging owning that ordering. Pair distance/normal/midpoint read
committed current arrays, not proposed working world. Working current heading
and speed feed response. Other speed/heading/accepted coordinates mutate its
committed record immediately; later slot processing sees these modifications.

## Scan and cooldown

Caller converts source word-index to logical slot for byte cooldown. Nonzero
cooldown decrements once and entirely bypasses check_other_truck_contacts; zero
enters scan. Scan preserves Y/status via PHP/PHY with explicit X16 save/restore,
then X8 visits word-indices6,4,2,0, skipping current. Distance A16 is compared
unsigned to40: >=40 skips, <40 resolves. After EACH near pair current cooldown is
set6. This does not stop the scan; other trucks' cooldowns are never inspected.
Only outer motion commits current working state after scan. Other record changes
are immediate. Tests `zero_cooldown_scan...`, `every_nonzero_cooldown...`,
`current_slot_is_excluded...`, `contact_scan_rejects_exact_metric40...` exercise
actual shared Truck/WorkingMotion and Terrain interfaces, not only event mocks.
The synthetic all-zero direction table isolates exact sector0 responses and is
an immutable lookup fixture, not a fake algorithm.

Three coincident other trucks with speeds30,20,10 visited3,2,1 exchange against
working1: final other speeds1,30,20 and working10. Current committed speed/world
remain untouched except cooldown6. Each separation still uses original committed
current midpoint, so working world after all three is117E,1080. Six unmarked
terrain probes shift history80 to02. A metric40 pair (dx180,dy80 =>36+4) is
excluded;37 (dx180,dy40 =>36+1) is included. Other far records remain unchanged.

## Distance

M16 subtraction wraps first. If sign bit set, EOR FFFF/INC produces wrapping
absolute magnitude; 8000 remains8000. Either absolute axis >=0A00 returns41.
Otherwise ASL twice then XBA yields floor(axis/64) as byte. M7A low byte is that
value, high zero; M7B same byte, so returned MPYL is its square. Add two squares
unsigned16. Largest permitted axis09FF =>39²=1521. Threshold40 is squared/coarse,
not a coordinate distance. Tests exercise wrapped coordinate seam FFFF→0000,
subcell quantization, each axis cutoff and signed minimum.

## Bearing, shared quantization and sectors

Classifier M16 subtracts committed current-other, logical-shifts four, then uses
LOW BYTE. For Y, byte EOR FF/INC reverses sign; X is low scratch byte. Shared
quantize_signed_vector_to_lookup_index (physical0842, game.asm:2305) stores byte
components, forms byte magnitudes and jointly halves both until each<8. Restores
negative component signs into nibbles and packs Y-high/X-low. This is the same
helper needed by AI, which owns the reusable implementation; contact_angles
must call that shared interface, not introduce float atan2 or a duplicate model.
Read direction[packed] from physical1CC7..1DC7, preserving all256 cells, including
unresolved natural access cells. No naming of stored cells as dead based on this
consumer. +16,-16 committed separation gives (+1,+1) after Y reversal, index11,
ROM headingE0. Working heading10-other20 yields relative30/40 =>sectors1/1 =>11.
Working world2000 does NOT change that result. Other fixed bearings use ff/04/0c/
40/c0 indices and ROM5F/FF/7F/BF/40, with magnitude16 jointly scaled to4.

Relative headings subtract normal byte-wrapping. quantize_relative_heading_sector
preserves Y: [00,20) OR [E0,100) =>0; [20,60)=>1; [60,A0)=>2;
[A0,E0)=>3. Pack other sector high nibble/current low. Ten explicit endpoint
assertions and the full16 dispatch fixture establish limits independently.

## Response cases and width-sensitive sound

All ordinary speed/heading stores below are BYTE stores preserving destination
upper bytes. Current values come from WorkingMotion; other from Truck. Every
case proceeds to shared positional separation, whether sound is suppressed or
not. `respond_motion` only contracts speed/headings/sound selection; actual
`service_contacts` composes it with terrain separation.

* Case02: assembly XBA sequence writes current=-OLD OTHER, other=-OLD CURRENT,
  not each truck's own negation. Byte two's-complement wraps and80 remains80.
  Absolute byte magnitudes of new speeds >=20 select impact sound; otherwise
  separate quietly. Tests distinguish cross exchange,31/32,80 and zero.
* Cases00/22: exchange unchanged low speed bytes. Other heading adds10 if
  animation parity even, subtracts10 if odd, wrapping low byte. Sound threshold
  uses absolute BYTE-WRAPPING original(other-current) >=20. E.g7F-80 wrapsFF,
  absolute1, quiet;00-20 wrapsE0, absolute20, impact. Do not widen difference.
* Odd cases11/13/31/33: current sector1 subtract4, sector3 add4; OTHER sector3
  subtract4, sector1 add4. This branch never requests impact despite high speeds.
  The trailing LDA03 is discarded when separation begins; not a response ID.
* All remaining cases: LSR packed selects low current-sector parity. Odd current
  sector turns current heading+10, leaves speeds, compares OTHER speed unsigned
  against20 for sound. Even sector quarters current speed via two memory LSRs
  only if current byte is nonnegative; negative byte stays unchanged. Other
  heading always+08. Sound compares resulting CURRENT speed unsigned>=20, so
  negative80/FF are loud; positive7F quarters to1F and is quiet. LDA05/LDA04 are
  immediately overwritten, not state, multipliers or independent response modes.

request_truck_impact_sound changes M16, command0302, calls
send_race_sound_if_human physical0256 (game.asm:376). This helper preserves the
command on stack and gates by ANY nonzero low byte active-human mask, not current
truck ownership or low4bits only. It changes no owner state beyond APU command.
Separating code explicitly sets its widths afterward; no assumption of M8 return.
Tests separately cover all16 cases, threshold/parity/wrap/unsigned alternatives,
other high-byte preservation and mask00/08/80.

## Conditional entry $83E1

All natural case branches skip physical03E1..0409: parallel ends JMP separation,
odd/asymmetric dispatch starts0409. No source branch/call reference to
`deflect_current_truck_contact` was found beyond declaration. Its bytes remain
in scope; lack of caller does not authorize dropping it or routing ordinary
cases through it. Conditional M8, decimal-clear, DP0000/DB00 contract: speed<60
adds20; speed>=60 unchanged (unsigned). Current heading +/-08 on even/odd frame,
with odd path BRA preventing accidental second addition; then common separation.
No stack imbalance. Test covers5F/60/FF, byte heading wrap/parity and preserved
high bytes. `collision-conditional-current-entry` records unresolved natural
entry/context; this is conditional behavior evidence, not reachability proof.

## Positional separation and terrain cross-effects

Midpoint uses committed positions with CLC ADC16 then LSR logical16. Carry out
is discarded BEFORE shift: F000+F100 wrapsE100, midpoint7080, not F080. Normal
sine table physical1DC7..1EC7, phase offsetsY=normal, X=normal+40. M7A0200 * signed
sine byte, reading MPYM gives signed2*sine as word. Add midpoint wrapping16 for
current candidate. Other uses normal+80/C0. All phase increments wrap byte.
Eight-octant fixed fixtures use sine0=0,64=127,32=91 and their signed mirrors,
thus offsets254/182. Source scale0200 is not a claimed metre/pixel distance.

First terrain sample M16 caller status saved; neighborhood resets scratch widths,
X but classifier restores M16/carry outcome. It does NOT preserve marker history:
selected control bit shifts into bit7 of terrain_control_shift for each sample,
even blocked. Neighborhood/scratch/MPY registers are clobbered; second Y product
is saved in scratch54 before changing M7B for X, then transferred into probeY
BEFORE second sample clobbers scratch. contact_midpoint words CC/CE and normal
0529 do not overlap these scratch fields. First sample runs before second; marked
current followed by unmarked other turns history00→80→40. Both unmarked80→20.
Terrain's original-track8 height adjustment does not change accept/reject branch.

Carry-clear current candidate writes WORKING coordinates only; carry-set leaves
old working proposal, even if that differs from committed. Other candidate always
computed and sampled, independent of first result. After restoring original other
Y, carry-clear writes committed other position words; carry-set RTS preserves its
old coordinates. No automatic surface-summary/wheel/pose refresh or opposite-truck
cooldown write occurs. Four accept/block combinations and all octants exercise
real Terrain/SamplingTables with fixed cell controls. Native malformed terrain
or invalid logical slot returns Result error rather than unchecked read; this is
explicit native validation, not an original path or invented zero-filled terrain.
Boundary-world behavior still follows existing terrain-window-boundary-state
uncertainty; the wrap-midpoint fixture remains within checked row/index bounds.

## Execution evidence and handoff

24 substantive native tests in contact.rs. Compile/list successful with Rust1.90.0
and Cargo.lock (134 total suite tests). All24 area failures are exact explicit
unimplemented pending bodies; no fixture panic, ignored/disabled test or hang.
The original four infrastructure tests remain green. Exact commands/exit codes:
checks/contracts/collision-results.json; per-operation logs adjacent. Initial
compile found six integer-inference errors in literal fixture tuples (From<i32>
for u16); fixed casts without changing expected values, retained in
collision-initial-compile-note.txt. No behavior implementation or oracle added.

AI worker: establish shared signed-byte vector quantizer with joint unsigned
magnitude scaling then nibble-sign restoration, preserving these fixed contact
bearing fixtures. Motion caller: projection/shadow precede contact; working commit
follows; nonzero current cooldown ticks only once; do not erase other immediate
mutations. Objects/terrain: retain marker history from BOTH contact separation
probes, including blocked branches, before subsequent owners consume history.
Full snapshot/debug state must include contact cooldown and pending working owner
state as stage4 composes actual domains. No gameplay QA, source mutation, toolkit
edit, commit or publication occurred.

## Modernization iteration3 M6: native pair-response decisions

`ContactAngles` now carries a `ContactResponse` chosen directly from two native
relative-heading bands. The runtime no longer packs sector nibbles, compares
encoded case bytes or extracts turn policy from their bits. Responses name the
actual operations: reverse exchange, parallel exchange, paired oblique turns,
current deflection, or current braking with peer deflection. The four angular
bands keep the same byte-wrapped +20/quarter-turn quantization. Oblique signed
heading deltas are selected once; applying them still wraps only the low byte.

The source case table and all numeric expectations above remain authoritative.
Encoded test cases use a test-only adapter to select the same relative headings;
no packed compatibility dispatch remains in production. No snapshot format or
public semantic observation changes. Transactional clones, descending peer
visits, immediate accepted peer changes, cooldowns, normal bearing, separation,
terrain-history order and sound request gating remain unchanged. The isolated
conditional deflection entry remains independent of normal response selection.
