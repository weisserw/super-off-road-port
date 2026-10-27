# Objects independent trace and contract evidence

Worker `/root/behavior_contracts/objects`, frozen Round29
`reference/source/game.asm` SHA256
`4e3e1a45e4409ce63a3354740a2cdd3962cd3d9975d88d22ecee40c9d29ded7f`.
Read all 79 objects-assigned ranges in planning/source-ownership.json and their
callers, data consumers and tails. CPU bank00 aliases are physical+$8000; pointer
words here are bank-zero CPU addresses, not physical offsets. Independent native
interfaces/tests are objects.rs; all rule bodies remain unimplemented. Consulted
toolkit docs/development.md platform reference table, docs/snes.md CPU/address
context and docs/snes-native.md mechanism boundaries. Arithmetic below follows
65816 emitted M/X widths: REP30 words, SEP30 byte indices/accumulator, byte writes
retain adjacent bytes, binary carry/borrow wraps; SED ADC uses decimal carry.
No emulator/QA, CPU execution oracle, original asset embedding, or toolkit edits.

## Count-prefixed placements and initialization

`load_track_object_placements` game.asm:14988, physical4560..45C7 enters SEP30,
clears A high via LDA0/XBA, shifts byte track then REP30. Sixteen pointers5299..
52B9 lead to count-prefixed 6-byte X/Y/height records. Actual counts independently
read from ROM: 4,3,5,2,4,3,2,4,5,2,3,4,3,4,4,4. Each X is tested signed before
any write. Negative X exits without changing `placement_count`; unvisited parallel
arrays retain previous values. Native PlacementList therefore distinguishes
`declared_count` and written `prefix`, and apply_placements preserves existing
tail records. Source starts its body before testing count, so malformed zero count
is not an empty valid list. Native added definitions reject zero/truncation and
excess prefix transactionally; this validation is not an original memory check.
Supported original lists have no negative exits and fit their reserved arrays.

Initialization calls track3 initializer, track0 initializer (which itself services
composites), six dynamic initializer, then **the complete object update**, before
hiding both pickup sprites, clearing pickup_state, both type-count words, and four
pending-award bytes (two word STZs). It does not first clear the prior pickup or
its coordinates/lifetime/nitro stock. A retained collectable pickup can award nitro
in this service before counters/awards clear. `world_initialization...` fixes that
cross-owner fact on actual WorldObjects/Truck state, not a mocked call sequence.

## Pickup service, geometry, awards and sprite pair

Physical45C7..46F3, game.asm:15050..15301: PHP/REP30, state word0 consumes one
RNG and masks lowbyte. **Only zero** enters spawn; nonzero jumps return. Spawn
calculates24*track then consumes a second RNG&3, chooses six-byte record, and
writes coordinates/height BEFORE comparing parity-selected type count>=4. Failure
retains these new fields and prior lifetime, state still0. Success state is
(frame&1)+1, not choice-dependent; project then return with no award timer install.
Original16*4 records46F3..4873 are loaded exhaustively from ROM, including every
world/height field. Fixed RNG fixtures use runtime's independently traced values:
12 34 56 78 9B→DF then6A; 01 FF FF FF FF→00 then0F. The zero-gate service test
asserts two history shifts; nonzero gate asserts exactly one. No RNG oracle exists.

States1/2 test only slot(frame&3), independent of joined human mask. Wrapped X
comparison is `(pickupX-truckX+0180) <0300`; Y is
`(pickupY+00C0-truckY+0280) <0500`. Hence truck-world intervals are
X **(pickupX-0180,pickupX+0180]**, Y **(pickupY-01C0,pickupY+0340]**. Lower
world edge excluded, upper included; wrapping across zero remains binary word
arithmetic. Miss returns without lifetime tick or redraw.

Collection index2*(state-1) reads OLD type count into A, increments memory only
when old<4, then adds base7 for type1 or3 for type2. Old count remains A even
when the increment occurs. Thus natural successive type1 awards state7..10,
pending amount1..4; type2 states3..6, nitro1..4. Count saturates4. Pending award
addition is a byte and wraps. Nitro addition is SED ADC; on carry it **does not
store**, retaining old stock rather than clamping99. 95+4→99;96+4 overflows and
retains96. Sound0502/0602 uses any nonzero low-byte human mask, not whether this
rotating collector is human. Shared send_race_sound_if_human at0256 preserves
command/status-width assumptions; no source-owner sound replacement is invented.

Project pickup4873..48DB uses state*2 into tilewords48F1..4905. Source projection
gets **height lowbyte**, display X is projectedX-0040 masked byte, Y projectedY+28
byte. It hides unrelated122D shadow Y before state>=3 pair handling. Pair centering
SUB0008 then ADD0010 performs **word** arithmetic on packed sprite X/Y; X underflow
borrows Y, next addition can carry Y. Fixed projection fixture at world[1000,2000],
zero variable scales, offsetXFFFF: hardcoded Y-factor34 contributes1088 then>>4=68,
X67→displayX3; projectedY=(2C00>>4) lowbyte=C0→displayY=E8. Pair words E803-8=E7FB,
then+10=E80B. The height0 and source projection constants make this an independent
fixture; an earlier unrecorded drafting assumption zero scales imply zero
projection was corrected before recorded tests.

Conditional direct collection at already capped type1 old4 produces state11
(base7+4), even though natural spawning rejects that count. Primary tile lookup
then reads4905..4907 =08 C2→C208 from adjacent initializer bytes; extra lookup
48EF..48F1 =CA24→24CA. ObjectTables explicitly loads this decoded incidental tile
value, not instructions/native behavior. Natural primary10/extra8 table entries
remain intact. This explains region1448's conditional consumer without asserting
natural state11 execution. No count cap was weakened to hide the branch.

After awarded projection, install lifetime80 and FALL THROUGH immediately into
tick: resulting lifetime79. Every fourth frame subtract0100 from each coordinate
word (Y-- with wrap), then decrement WORD lifetime. Only exact result0 hides both
coordinatesF000 and clears state; initial0 wrapsFFFF and stays active. Tilewords
are retained on hide. States>=3 use expiry path directly without RNG/contact.

## Six sequence-dependent dynamic slots

Physical4905..4E38, game.asm:15548..16468. Full order-slot selects64-pointer table
4984..4A04, not selected track alone. Seventeen fixed groups4A04..4C68 each contain
six records; the first source record fills **native slot5**, then4..0. Every slot
kind cleared first; zero X skips all other writes/RNG/draw (reserved Y/height remain
source data). Nonzero sets coords/height, consumes RNG&7, clears animation/timer,
kind1, projects lowheight and sets tile2520. Tests assert descending consumption,
nonzero-only RNG and preserved inactive fields; all64 pointer selections compared
with independently parsed ROM records. No unsupported18th group inferred.

Service4C68 selects truck(frame&3) and scans slots5..0. Kind0 skips. Contact is
attempted **only animation0 or12 hex (=18)**. The subsequent CMP1E/BCC target
is the same label as fallthrough JMP, so >=1E does not reenable contact. Miss
usually jumps next without shadow or lifetime update; the compare1F miss tail
cannot naturally be reached from the only tested0/12 states. Nonzero animation
states bypass geometry but reach shadow/lifetime. This corrects an interim
message that idle type1 timers always consume uses: **idle0 miss skips lifetime**;
a nonzero animation reaching shadow can underflow the timer without contact.

Geometry X `(objectX-truckX+0100)<0300` gives truck
(objectX-0200,objectX+0100]. Type1/2 Y adjustment01A0 gives
(objectY-00A0,objectY+0260]; type3 adjustment0030 gives
(objectY-0210,objectY+00F0]. Three actual words4DC0..4DC6 are indexed with base
4DBE+2*kind; the preceding4DBE word is the unused type0 position (type0 skips).
No active type0 semantic table entry is fabricated.

Type1 hit installs timer2C (=44), animation22 (=34), clears **full speed word**,
requests0702 through any-human gate. Type2 hit increments kind→3, clears full speed,
animation11 (=17). Type3 requires full speed WORD!=0 (8000 counts, unlike placement
low7 gate), starts animation1D (=29), never clears speed. Later descending slots
see the speed earlier slots changed. All contact mutations operate on actual Truck.

Shadow tile uses animation before its decrement, OR1000 iff timer>=4 unsigned.
Every fourth frame nonzero animation--. Then only kind1 performs WORD timer--:
nonnegative selects reaction delta table by decremented timer0..43, adds signed
byte to spriteY, and forces animation22 if current<1F; else retains it. On negative
timer, animation==1D returns with timerFFFF and uses unchanged. Otherwise reset
animation0,timer0,tile2520, then uses--: negative means exhausted, set tile256E and
kind++. Thus initial uses0 still permits the response before exhaustion (not zero
uses meaning never collide). Tests cover timer0 underflow paths, special post-tick
animation29, palette threshold3/4, old tile/new animation, y44→43 deltaFC and
word speed/slot ordering. Tables4DC6..4DF2 have44 signed bytes,4DF2..4E38 have35
word tiles covering state0..22. No game identity is invented for these type states.

## Terrain marker special contact and track-three fixed sprites

`test_special_surface_contact`4E38..4E9C called from update_four_trucks immediately
after proposed terrain sampling and before both-block rejection. It reads logical
slot=current wordindex/2; require nonzero Truck.contact_marker and
(terrain_history&F0)==F0. History is the actual persistent probe shift register,
including rejected probes and prior terrain helper effects, not averaged height.
Cursor is `sprite_animation_cursors+4` alias contact_effect_animation_cursors.
Cursor0 requests0802 (any-human gate), halves low proposed speed only if sign clear,
copies **staged previous** truck screen X/Y low bytes, writes cursor0A (=10).
Cursor12/17 (18/23 decimal) does the same nonnegative halve/position without sound,
writes cursor13 (=19). Every other cursor leaves speed/state untouched. Upper
speed byte retained. `position_contact_effect_sprite`4E85 uses current index again
and leaves X logicalslot; following cursor write thus targets correct slot. It
stores Y at OAM160+4*slot+1 and X at+0. Speed change can subsequently be discarded
when physics both-block reloads committed speed, while cursor/sound/sprite persist;
this integration boundary is handed to physics/stage4. It is not the reverse/half
helper used elsewhere. All gates/cursors/signed lowbyte/status distinctions tested.

Track3 initializer4E9C..4EFF compares byte selected_track==3, otherwise no writes.
Three parallel X bytesA4,2C,B8 and Y bytesA0,98,48 expand into sortable staging
(tile228C) and fixed OAM group(tile208A,Y-10 byte). The two groups have separate
priority/attributes. Tests fix full groups and preservation when disabled; added
content explicitly chooses this special rather than relying on an invented ID.

## Track-zero composites: inclusive loop and concrete adjacent-field effects

Initializer4EFF..4F45 gated track0, pointer514E→515E count4, four6byte records, clears
four reaction words, then calls service. RepeatedD15E pointer words5150..515E have
no established extra selector because gate track0 only; retained raw storage,
no invented additional tracks/table extent. Negative X returns before initial
service and leaves declared count, same retained-field issue as general placements.

Service4F45..502E copies composite_count to scratch, starts record0, draws three
tilewords (state*6) from48 frames502E..514E then writes three X bytes+0,+10,+20 and
same Y+28. Byte X additions wrap without Y carry, contrasting pickup packed-word
centering. Nonzero reaction decrements every fourth frame **after drawing**;
reaching0 can immediately retrigger47. Movement gate is speed&007F !=0. Geometry
X `(objectX+0200-truckX+0300)<0600` gives
(objectX-0100,objectX+0500]; Y `(objectY-0100-truckY+0080)<01C0` gives
(objectY-0240,objectY-0080]. A truck at exactly object world Y is NOT inside.

**Source correction:** tail `DEC scratch_count; BMI return; JMP CF5B` is inclusive,
not count iterations. With count4 it executes5 records. CF5B starts at recordX
read, does not reset indices/scratchcount. Actual four parallel arrays begin6A1
(X),6A9(Y),6B1(height),6B9(reaction),6C1(count). Fifth derived record therefore
has X=first.Y, Y=first.height, height=first.reaction, animation=count. On a fourth
frame its reaction decrement writes count4→3. Scratch count was copied before
this write so this call still draws5; later count3 draws4. Native `composite_tail`
exposes this concrete derived view, and service_composites updates count explicitly,
without reproducing addresses/a RAM image. Initial service on animation%4==0
makes count3 before general object's second composite call that initialization.
Superseded planning description ordinary count-bounded was corrected here.

If a conditional/native restore puts count>4 or triggers the extra derived
reaction to47, further source reads cross other owners' fields. No natural such
entry established; record uncertainty and require supported native-state
validation until a real consumer needs an explicit domain effect. It is **not**
a basis to omit the observed fifth/count side effect or zero adjacent fields.
Frame composition/snapshot must include count/timers and retained extra sprite
staging: source shorter later updates do not clear prior OAM slots. `sprites`
reports this service's writes; presentation retains untouched prior OAM records.

## General placed-object update and owner integration

5178..5229 calls gated composites, pickup, dynamics, then placements in ascending
record order. This matters: a pickup awards the selected truck even if a following
hazard stops it, and placement motion tests then see its stopped speed. Actual
WorldObjects/Truck service contract asserts pending money1, pickupstate7, hazard
kind3, speed0, placement animation0 and unchanged RNG, plus semantic phase order.
Phases alone cannot satisfy this contract. Placement X/Y/height project read-only;
word X is projectedX-40 (do not indiscriminately mask tobyte), Y+28 byte and tile
from **old animation** table5229..5299. Every fourth frame nonzero animation--;
if now0, movement gate speed&007F and rotating truck rectangle
X(objectX-0140,objectX+0140],Y(objectY-0140,objectY+0080] starts55. Retrigger on
1→0 is same frame, while displayed tile remains the old state's tile. Source
loop checks count after body (zero unsupported as above). Original capacities
remain validated; named added track count is independent of16 originals.

TrackContent now carries optional ObjectContent: original `None` retains ROM
track/full-order resolution via load_original_content; a file-backed added or
replaced track must supply explicit validated `objects` (placements declared_count
and prefix world/height records, four pickups, six native-order dynamic records,
optional four composites, track_three_sprites boolean). No silently zero-filled
missing object definitions or fixed native catalogue ceiling. Existing extension
fixtures expanded; a native tracks test requires missing/invalid objects rejection.
Title/banner definitions remain presentation/race owners. Future Game/RaceState
must own WorldObjects (single pending-award authority shared results/economy),
include mutable object/contact/retained sprite state in snapshots/observations,
and pass real immutable definitions/tables into orchestration, not callback mocks.
The tests currently call actual pending world/domain interfaces; no behavior body
or mock implementation is supplied in stage3.

## Recorded tests and limits

Forty objects unit tests cover these branches plus one adjacent tracks extension
contract. Pinned Rust1.90 locked compile/no-run and discovery pass;250 total suite
tests. All40 objects and1 extension test fail at explicit pending unimplemented
functions, not fixtures/compile errors; four infrastructure tests pass separately.
`checks/contracts/objects-results.json` gives exact commands/logs. Draft fixture
projection/RNG/state11 tile arithmetic corrected from primary source before final
handoff; initial recorded evidence retained when final affected RNG fixture rerun
was necessary. Passing tests would still not establish original runtime behavior.
No ignore/disable, source behavior implementation, gameplay smoke test, audit,
original-runtime capture, source modification or toolkit change occurred.

## Exhaustive primary review index

All entries below independently read with consumers as detailed above; ownership
is navigation, not implementation accounting. Original ledger remains pending.

| ID | Physical range (exclusive end) | Source entry |
| --- | --- | --- |
| region-360 | $004560..$0045C7 | load_track_object_placements |
| region-200 | $0045C7..$0045CF | dispatch_random_pickup_state |
| region-278 | $0045CF..$0045D7 | gate_random_pickup_spawn |
| region-781 | $0045D7..$0045DA | skip_inactive_pickup |
| region-279 | $0045DA..$00461F | choose_track_pickup_record |
| region-361 | $00461F..$00462B | activate_random_pickup |
| region-280 | $00462B..$00462E | return_after_pickup_spawn_attempt |
| region-255 | $00462E..$004631 | dispatch_pickup_expiry |
| region-201 | $004631..$004670 | test_pickup_near_one_truck |
| region-782 | $004670..$0046BF | award_collected_pickup |
| region-818 | $0046BF..$0046C5 | set_pickup_lifetime_eighty |
| region-256 | $0046C5..$0046F1 | animate_and_expire_pickup |
| region-202 | $0046F1..$0046F3 | return_pickup_service |
| region-400 | $0046F3..$004873 | pickup_spawn_records |
| region-362 | $004873..$0048BC | project_pickup_sprite |
| region-783 | $0048BC..$0048D9 | draw_extra_pickup_tile |
| region-363 | $0048D9..$0048DB | return_pickup_projection |
| region-401 | $0048DB..$0048EF | pickup_award_bases_and_tiles |
| region-1448 | $0048EF..$0048F1 | unlabelled data: consumer review above |
| region-402 | $0048F1..$004905 | pickup_state_tilewords |
| region-364 | $004905..$004921 | initialize_sequence_objects |
| region-784 | $004921..$004978 | initialize_dynamic_object_record |
| region-365 | $004978..$004984 | advance_sequence_object_record |
| region-403 | $004984..$004A04 | dynamic_object_record_pointers |
| region-404 | $004A04..$004A28 | dynamic_object_group_00 |
| region-405 | $004A28..$004A4C | dynamic_object_group_01 |
| region-406 | $004A4C..$004A70 | dynamic_object_group_02 |
| region-407 | $004A70..$004A94 | dynamic_object_group_03 |
| region-408 | $004A94..$004AB8 | dynamic_object_group_04 |
| region-409 | $004AB8..$004ADC | dynamic_object_group_05 |
| region-410 | $004ADC..$004B00 | dynamic_object_group_06 |
| region-411 | $004B00..$004B24 | dynamic_object_group_07 |
| region-412 | $004B24..$004B48 | dynamic_object_group_08 |
| region-413 | $004B48..$004B6C | dynamic_object_group_09 |
| region-414 | $004B6C..$004B90 | dynamic_object_group_10 |
| region-415 | $004B90..$004BB4 | dynamic_object_group_11 |
| region-416 | $004BB4..$004BD8 | dynamic_object_group_12 |
| region-417 | $004BD8..$004BFC | dynamic_object_group_13 |
| region-418 | $004BFC..$004C20 | dynamic_object_group_14 |
| region-419 | $004C20..$004C44 | dynamic_object_group_15 |
| region-420 | $004C44..$004C68 | dynamic_object_group_16 |
| region-203 | $004C68..$004C7E | scan_dynamic_objects_against_truck |
| region-257 | $004C7E..$004D5D | animate_dynamic_object_and_resolve_contact |
| region-258 | $004D5D..$004DB5 | update_type_one_object_lifetime |
| region-204 | $004DB5..$004DBE | advance_dynamic_object_scan |
| region-1449 | $004DBE..$004DC0 | unlabelled data: consumer review above |
| region-421 | $004DC0..$004DC6 | dynamic_object_height_adjustments |
| region-422 | $004DC6..$004DF2 | dynamic_object_reaction_y_deltas |
| region-423 | $004DF2..$004E38 | dynamic_object_animation_tiles |
| region-205 | $004E38..$004E4C | test_special_surface_contact |
| region-785 | $004E4C..$004E64 | test_pickup_contact_animation |
| region-819 | $004E64..$004E6B | set_contact_effect_animation_nineteen |
| region-786 | $004E6B..$004E7E | sound_and_slow_pickup_contact |
| region-820 | $004E7E..$004E83 | set_contact_effect_animation_ten |
| region-206 | $004E83..$004E85 | return_special_surface_test |
| region-787 | $004E85..$004E9C | position_contact_effect_sprite |
| region-366 | $004E9C..$004EA5 | initialize_track_three_objects |
| region-788 | $004EA5..$004EF7 | initialize_track_three_sprites |
| region-367 | $004EF7..$004EF9 | return_track_three_initializer |
| region-424 | $004EF9..$004EFF | track_three_sprite_coordinates |
| region-368 | $004EFF..$004F09 | initialize_track_zero_objects |
| region-789 | $004F09..$004F43 | load_composite_object_placements |
| region-369 | $004F43..$004F45 | return_track_zero_initializer |
| region-207 | $004F45..$004F4E | test_track_zero_object_service |
| region-259 | $004F4E..$004F97 | project_track_zero_composite_objects |
| region-790 | $004F97..$00502E | update_composite_object_contact |
| region-425 | $00502E..$00514E | composite_object_tile_triplets |
| region-426 | $00514E..$005150 | track_zero_composite_placement_pointer |
| region-1450 | $005150..$00515E | unlabelled data: consumer review above |
| region-427 | $00515E..$005178 | track_zero_composite_placements |
| region-208 | $005178..$0051D1 | update_and_project_track_objects |
| region-281 | $0051D1..$0051D5 | tick_placement_object_animation |
| region-209 | $0051D5..$0051DA | test_placement_object_still_animating |
| region-282 | $0051DA..$0051DD | skip_placement_object_collision |
| region-210 | $0051DD..$005206 | test_moving_truck_near_placement_object |
| region-283 | $005206..$00521B | trigger_placement_object_animation |
| region-211 | $00521B..$005229 | advance_placement_object_scan |
| region-1010 | $005229..$005299 | placement_reaction_tilewords |
| region-376 | $005299..$005429 | track_object_placement_bundle |

## Stage4 composite boundary fixture correction

The composite X-boundary test originally put the other axis at objectY, although its preceding assertion and source CF97 Y calculation establish that objectY lies outside the shifted Y interval. Source requires (objectY-0100-worldY+0080)<01C0. The allegedly accepted X-upper case therefore correctly failed after implementation. Both X-boundary fixtures now use worldY=objectY-0100, an interior Y, retaining the original lower-reject/upper-accept assertions. Initial39pass/1fail preserved in checks/implementation/domain-results.json; all Stage3 evidence remains. No source rectangle or expectation was loosened.

Stage4 full-state validation: conditional pickup state11 remains accepted. Type1 dynamic timer initializes44 (DYNAMIC_RESPONSE_TICKS) then decrements before indexing the44-byte Y-delta table at physical4DC6; nonnegative restored timers above44 have no supported table lifetime and are rejected before mutation. Negative source completion sentinels remain accepted.
