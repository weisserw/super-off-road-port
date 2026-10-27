# Source behavior inventory

Stage 3 behavior contracts complete for all sixteen areas. Stage4 native implementation is present for all sixteen areas; original contract evidence remains preserved below. Current native execution passes620 tests, all six gates, with bounded Stage5 QA in debug/report.md and checks/implementation/stage5-final-check.json. Historical execution evidence is checks/implementation/domain-results.json.

Historical Stage3 suite discovered606 behavior contracts plus four infrastructure tests; current suite totals620 passing tests.
Source traces, worker identities, exact test names and execution evidence are indexed in
`checks/contracts/stage3-handoff.json`; combined compile/discovery checks passed.
Historical Stage3 pending failures are preserved; final Stage4 execution passes all615 native tests (including every original contract), indexed in checks/implementation/final-check.json.
This inventory is navigation and proposed ownership, not independent branch tracing.
The adopted source is frozen Round29 `reference/source/game.asm` and
`reference/reading.asm`; workspace.json contains its SHA identity. No research
supplement exists. CPU $008000..$00E09B maps to physical $000000..$00609B
for bank-zero routines. Banked asset CPU pointers must use LoROM translation,
not this single-bank shortcut. Uploaded SPC code executes in ARAM, not on the
65816. All entry labels below are in game.asm; exact line/range/ledger links for
every named unit are retained in port/source-identifiers.json and
planning/source-ownership.json.

## Preserved Stage3 contract assignment and exhaustive cross-check

Each area below requires its **own sequential worker** in stage 3, with the full
prompts/contract.md plus this area and its source context. Suggested order follows
the table; workers may add minimum declarations/interfaces to compile contracts.
No workers have been launched in planning. Proposed files may be refined with
source-backed reasons; create adjacent `<module>.trace.md` evidence and native
`#[cfg(test)]` modules. All sixteen workers independently traced source and compiled/listed/executed their contracts;
see individual execution records and adjacent traces. These tests do not implement the game.

| ID | Worker scope | Proposed implementation/contract module | Trace / worker / implementation |
| --- | --- | --- | --- |
| runtime | Reset, cadence, random state and persistence | port/src/runtime.rs | port/src/runtime.trace.md / complete (13 contracts) / implemented |
| tracks | Track catalogue, order, routes and start grids | port/src/world/tracks.rs | port/src/world/tracks.trace.md / complete (19 contracts) / implemented |
| terrain | Terrain sampling, interpolation and projection | port/src/world/terrain.rs | port/src/world/terrain.trace.md / complete (25 contracts) / implemented |
| controls | Controller sampling and direction decoding | port/src/controls.rs | port/src/controls.trace.md / complete (15 contracts) / implemented |
| physics | Truck motion, upgrades, nitro, wheels and suspension | port/src/trucks/motion.rs | port/src/trucks/motion.trace.md / complete (36 contracts) / implemented |
| collision | Truck pair contact and separation | port/src/trucks/contact.rs | port/src/trucks/contact.trace.md / complete (24 contracts) / implemented |
| ai | Computer driving, ranking and difficulty | port/src/trucks/ai.rs | port/src/trucks/ai.trace.md / complete (37 contracts) / implemented |
| race | Race lifecycle, checkpoints, lap/time and progression | port/src/race.rs | port/src/race.trace.md / complete (38 contracts) / implemented |
| objects | Track props, dynamic hazards, pickups and effects | port/src/world/objects.rs | port/src/world/objects.trace.md / complete (40 contracts) / implemented |
| rendering | Graphics assets, sprites, text, HUD and composition | port/src/presentation/mod.rs | port/src/presentation/rendering.trace.md / complete (53 contracts) / implemented |
| transitions | Fades, scrolling windows, ready countdown and pause | port/src/presentation/transitions.rs | port/src/presentation/transitions.trace.md / complete (53 contracts, including source window-inversion regression) / implemented |
| attract | Licensed/software splashes, attract cycle and Start reentry | port/src/frontend/attract.rs | port/src/frontend/attract.trace.md / complete (32 contracts) / implemented |
| names | Human joining, driver panels and name entry | port/src/frontend/names.rs | port/src/frontend/names.trace.md / complete (36 contracts) / implemented |
| economy | Wallets, upgrades, speed shop and stock | port/src/frontend/shop.rs | port/src/frontend/shop.trace.md / complete (43 contracts) / implemented |
| results | Podium, records, high scores, wipeout and continue | port/src/frontend/results.rs | port/src/frontend/results.trace.md / complete (42 contracts) / implemented |
| audio | ROM audio resources, host commands and native SPC music/effects | port/src/sound/mod.rs | port/src/sound/audio.trace.md / complete (101 contracts) / implemented |

The machine-readable primary-review index assigns all **1,527 regions** (458
Code, 1,055 Data, 10 Padding, four Unknown) and every existing unit exactly once.
An assignment is a requirement to review the original, not proof of semantic
coverage or a permission to exclude. Shared physical asset unions require every
consumer; source call/dependency links override physical proximity. No code or
unknown region is excluded. Runtime owns padding/header review; original explicit
FF bank-fill evidence may justify later exclusion, without modifying identity.
All ledger dispositions and all four source uncertainties remain pending.
The unknown IDs source-unknown-0001fb/source-unknown-0019c0 are assigned to
race/runtime and source-unknown-070149/source-unknown-0704ff to audio; each
worker must follow neighboring consumers and record an honest impact assessment.
No natural-execution claim is inferred from a source label or a planning test idea.

## runtime — Reset, cadence, random state and persistence

**Scope/state owner:** Own deterministic native reset, per-frame counters/audio heartbeat and source RNG; distinguish warm frontend record signature from cartridge battery saving.

**Known entries:** `reset_and_boot` (game.asm:19); `next_pseudorandom_byte` (game.asm:6900); `service_nmi_video_and_input` (game.asm:7214).

**Storage/ledger navigation:** 14 primary regions, 2 Code; region-1 ($000000..$0000AA), region-174 ($0019C2..$0019F3); complete IDs, ranges and original source links: planning/source-ownership.json filtered by `area=runtime`.

**Dependencies/update effects:** race,attract,rendering,audio. Follow both callers and callees; cross-area effects are required contracts, not pure-call assumptions.

**Required contract branches/interfaces:** Reset defaults versus warm frontend reentry; RNG seed/update/wrap; NMI inhibit still advances counters/audio heartbeat; clone/full-snapshot all owners; reject malformed restores atomically.

**Assets/identifiers/observations:** Use source-labelled tables/resources referenced by these entries and every assigned Data unit in the ownership index. Expose named owner state and bounded semantic transition/award/sound/contact events; no RAM map. Proposed module/contract: `port/src/runtime.rs`, adjacent source trace; runtime integration contracts additionally use `game::Port` and debug observations.

**Facts, leads and unresolved questions:** ROM header reports no SRAM; records are session RAM, not a battery save. Unknown $0001FB and $0019C0 bytes need consumer review, not invented state.

**Status:** worker `/root/behavior_contracts/runtime` complete (traced/tested, not implemented).
Trace: `port/src/runtime.trace.md`; contracts/interfaces: `port/src/runtime.rs`
(`runtime::tests::*`, 13 tests). Locked Rust1.90 compile and discovery pass;
area run: 1 pass /12 expected pending failures /0 ignored (10 unimplemented,
2 missing Game observations). All4 original infrastructure tests pass separately.
Evidence: `checks/contracts/runtime-results.json` and referenced logs.
Cross-area: NMI presentation ownership retained; runtime fixes counter/heartbeat
ordering only. Attract/results own five-byte signature/default-record decisions.
RNG caller-width and transient overlapping-store derivation corrected; explicit
power-on seed uncertainty added without selecting an unsupported original seed.
Implementation remains pending.

## tracks — Track catalogue, order, routes and start grids

**Scope/state owner:** Decode 16 original descriptors, 64 direction/order entries, terrain/background/graphics pointers, route streams and grid setup; add validated named file-backed tracks.

**Known entries:** `select_track_descriptor` (game.asm:345); `clear_wheel_state_and_select_route` (game.asm:975); `load_route_pointer` (game.asm:985); `initialize_start_grid` (game.asm:1041); `load_track_assets` (game.asm:4451).

**Storage/ledger navigation:** 81 primary regions, 13 Code; region-2 ($000200..$000216), region-336 ($00055F..$00057C), region-702 ($00057C..$000580), region-337 ($000580..$0005BE); complete IDs, ranges and original source links: planning/source-ownership.json filtered by `area=tracks`.

**Dependencies/update effects:** terrain,race,ai,rendering,objects. Follow both callers and callees; cross-area effects are required contracts, not pure-call assumptions.

**Required contract branches/interfaces:** All 16 descriptors and both direction variants where encoded; sequence wrap 63→0; reverse pointer/grid effects; mixed byte/word/bank table strides; route termination/bounds and added slots; cross-bank ROM graphics reads.

**Assets/identifiers/observations:** Use source-labelled tables/resources referenced by these entries and every assigned Data unit in the ownership index. Expose named owner state and bounded semantic transition/award/sound/contact events; no RAM map. Proposed module/contract: `port/src/world/tracks.rs`, adjacent source trace; runtime integration contracts additionally use `game::Port` and debug observations.

**Facts, leads and unresolved questions:** Reverse track 09 absent from natural order does not authorize dropping its data. World fixed point units require independent trace. Shared asset unions overlap physically; decode by consumers, not disjoint archive region boundaries.

**Status:** worker `/root/behavior_contracts/tracks` complete (traced/tested, implementation pending).
Contracts: `world::tracks::tests::*` in `port/src/world/tracks.rs` (17 tests);
source/branch derivations: `port/src/world/tracks.trace.md`.
Pinned Rust1.90 locked compile/list pass (34 suite tests); tracks 0 pass/17 expected
unimplemented failures/0 ignored. All4 infrastructure tests pass separately.
Evidence: `checks/contracts/tracks-results.json` and referenced logs.
Corrections: heading-zero, not reversed flag, selects negative grid step and X
column swap; odd order independently swaps Y. Terrain copy includes endpoint
($16B1 bytes). All32 routes retained including overlapping streams and reverse09.
TrackContent stores raw maps: presentation owns BG2 high-byte OR20, palette,
VRAM transfer/clear semantics. Grid.initial_surface_origin is the world origin
before terrain's inherited-heading wheel offset, not the final probe. Route
checkpoint initial1 and sequence wrap belong to race. Extension contracts cover
replacement, mixed names/order and >16 slots; object/title definitions must be
reconciled with later areas before complete playable extension implementation.

## terrain — Terrain sampling, interpolation and projection

**Scope/state owner:** Decode height/control neighborhoods, four wheel probes, triangles and cardinal/diagonal blocking, then project world X/Y/height using per-track scales.

**Known entries:** `sample_wheels_and_project_world_position` (game.asm:6271); `finish_projection_and_classify_terrain` (game.asm:6387); `normalize_terrain_triangle_and_interpolate` (game.asm:6567); `project_working_world_position` (game.asm:6305).

**Storage/ledger navigation:** 16 primary regions, 12 Code; region-167 ($001663..$001724), region-168 ($001724..$001781), region-240 ($001781..$00178F), region-169 ($00178F..$0017A2); complete IDs, ranges and original source links: planning/source-ownership.json filtered by `area=terrain`.

**Dependencies/update effects:** tracks,physics,collision,objects,rendering. Follow both callers and callees; cross-area effects are required contracts, not pure-call assumptions.

**Required contract branches/interfaces:** Each triangle orientation, signed endpoints, boundary neighborhoods, blocked cardinal/diagonal probes, fractional rounding/carry and projection offsets; terrain source length and descriptor bounds.

**Assets/identifiers/observations:** Use source-labelled tables/resources referenced by these entries and every assigned Data unit in the ownership index. Expose named owner state and bounded semantic transition/award/sound/contact events; no RAM map. Proposed module/contract: `port/src/world/terrain.rs`, adjacent source trace; runtime integration contracts additionally use `game::Port` and debug observations.

**Facts, leads and unresolved questions:** Mode 7 registers are used as integer multipliers; source display BGMODE=$09 is Mode 1 with BG3 priority, not a Mode 7 race renderer.

**Status:** worker `/root/behavior_contracts/terrain` complete; independent trace `port/src/world/terrain.trace.md`; 25 substantive native contracts in `world::terrain::tests` (`port/src/world/terrain.rs`). Rust1.90 locked compile/discovery pass (59 total suite tests); terrain0 pass/25 expected unimplemented failures/0 ignored; all4 infrastructure pass. Exact commands/logs `checks/contracts/terrain-results.json`. Implementation pending.

**Traced corrections/cross-area effects:** quadrant selects compare+sum carry; negative fractions use ones complement, not negation; base overlaps unsigned cornerA word; signed Mode7 truncations are distinct. Four-wheel scan preserves blocked prior heights and reloads old truck surface summary, without averaging. Persistent descending marker history feeds objects special-contact high nibble; physics must preserve this after rejected proposals. Grid origin applies inherited-pose signed-sine offsets before sampling. Native supported-window boundary validation is separate from unresolved outside-copy WRAM behavior (`terrain-window-boundary-state`); no invented zero/clamp rule.

## controls — Controller sampling and direction decoding

**Scope/state owner:** Own controller word translation, frontend page sampling and direction lookup interfaces. Source edges/repeat timers remain with names/shop/pause owners.

**Known entries:** `poll_frontend_controllers` (game.asm:14344); `decode_frontend_direction` (game.asm:14370); `decode_controller_direction_words` (game.asm:4545); `route_rectangle_and_controller_helpers` (game.asm:2019).

**Storage/ledger navigation:** 67 primary regions, 3 Code; region-1280 ($00141F..$001438), region-48 ($00405B..$00409A), region-49 ($00409A..$004123); complete IDs, ranges and original source links: planning/source-ownership.json filtered by `area=controls`.

**Dependencies/update effects:** names,economy,race,physics,transitions. Follow both callers and callees; cross-area effects are required contracts, not pure-call assumptions.

**Required contract branches/interfaces:** All 16 direction nibbles including opposing directions; joined-mask controller copying; actual alternate frontend page reads; word-direction helper conditional entry semantics; recorded input latency across race dispatcher.

**Assets/identifiers/observations:** Use source-labelled tables/resources referenced by these entries and every assigned Data unit in the ownership index. Expose named owner state and bounded semantic transition/award/sound/contact events; no RAM map. Proposed module/contract: `port/src/controls.rs`, adjacent source trace; runtime integration contracts additionally use `game::Port` and debug observations.

**Facts, leads and unresolved questions:** Both frontend pages read JOY1/JOY2 in source; do not silently substitute JOY3/JOY4. Pause fourth candidate reads JOY3 high again; preserve traced asymmetry.

**Status:** contract worker complete `/root/behavior_contracts/controls`; implementation pending. Independent trace `port/src/controls.trace.md`, native tests `port/src/controls.rs` (`controls::tests::*`,15 tests). Rust1.90 locked compile/list pass (74 total suite tests); area0 pass/15 expected unimplemented failures/0 ignored; all4 infrastructure pass. Exact evidence `checks/contracts/controls-results.json`. All67 assigned regions reviewed including both paired table bytes; mixed region49 hide/font tails explicitly handed to rendering for executable coverage. Race owns actual phase orchestration and packed rectangles; controls contracts two-frame retained/human-copy/AI latency. Conditional word helper entry uncertainty in uncertainties.json; no reachability/exclusion claim. Source-defined opposite combinations suppress both axes; both frontend pages read JOY1/2.

## physics — Truck motion, upgrades, nitro, wheels and suspension

**Scope/state owner:** Own four truck motion records, speed/fractions, headings, nitro edge history, four wheels each, suspension/pose and blocked-motion recovery; commit in source slot order.

**Known entries:** `update_four_trucks` (game.asm:3391); `update_acceleration_from_upgrades` (game.asm:5882); `consume_nitro_on_press_edge` (game.asm:6063); `integrate_heading_displacement` (game.asm:3498); `integrate_sixteen_wheels` (game.asm:3009); `select_four_truck_poses` (game.asm:3150).

**Storage/ledger navigation:** 50 primary regions, 35 Code; region-141 ($000A74..$000AEB), region-142 ($000AF3..$000B69), region-268 ($000B69..$000B6B), region-143 ($000B6B..$000BC8); complete IDs, ranges and original source links: planning/source-ownership.json filtered by `area=physics`.

**Dependencies/update effects:** controls,terrain,collision,objects,ai,rendering. Follow both callers and callees; cross-area effects are required contracts, not pure-call assumptions.

**Required contract branches/interfaces:** Upgrade-dependent acceleration/drag/speed caps; signed reverse drag; BCD nitro edge/empty/held; steering wrap/pose lag; displacement multiply/truncation; grounded/airborne wheel integration/clamps; rejected motion and AI recovery.

**Assets/identifiers/observations:** Use source-labelled tables/resources referenced by these entries and every assigned Data unit in the ownership index. Expose named owner state and bounded semantic transition/award/sound/contact events; no RAM map. Proposed module/contract: `port/src/trucks/motion.rs`, adjacent source trace; runtime integration contracts additionally use `game::Port` and debug observations.

**Facts, leads and unresolved questions:** 16-bit wrapping and signed shifts must be traced; a generic float vehicle model is not a substitute. Physics entry calls object contact before coordinate commit.

**Status:** worker complete `/root/behavior_contracts/physics`; independent source trace `port/src/trucks/motion.trace.md`, native contracts `port/src/trucks/motion.rs` (`trucks::motion::tests::*`,36 tests). Rust1.90 locked compile/discovery pass (110 suite tests); physics0 pass/36 expected explicit unimplemented failures/0 ignored; all4 infrastructure tests pass. Exact commands/logs: `checks/contracts/physics-results.json`. Implementation remains pending.

**Trace corrections/handoff:** Source half-speed helper reverses sign, suspension correction stores low byte with signed-overflow guard, average surface reads integrated wheels while shadow averages wrapping sampled byte-pair sums. Both-block rejection preserves proposed samples/history/heading, restores committed speed/world and bypasses average; one-axis effect→recovery versus both-axis recovery→effect. Controller/AI latency and sequential per-slot commits preserved. Shared `Truck`/`Wheel`/`Upgrades` interfaces in motion.rs are available to collision/AI/race; objects/rendering owners retain special-surface/animation/priority/upload algorithms and must compose state/events into full update service. No game behavior implemented; no new unsupported assumption.

## collision — Truck pair contact and separation

**Scope/state owner:** Own truck-pair coarse threshold, relative sectors, speed exchange/deflection and terrain-constrained separation with per-truck cooldown effects.

**Known entries:** `check_other_truck_contacts` (game.asm:446); `measure_coarse_truck_distance` (game.asm:492); `classify_truck_contact_angles` (game.asm:554); `reverse_contact_speeds` (game.asm:614); `exchange_contact_motion` (game.asm:668); `separate_contact_pair_against_terrain` (game.asm:840).

**Storage/ledger navigation:** 13 primary regions, 13 Code; region-117 ($00028B..$0002C7), region-118 ($0002C7..$00030E), region-224 ($00030E..$000312), region-119 ($000312..$00036A); complete IDs, ranges and original source links: planning/source-ownership.json filtered by `area=collision`.

**Dependencies/update effects:** physics,terrain. Follow both callers and callees; cross-area effects are required contracts, not pure-call assumptions.

**Required contract branches/interfaces:** Below/equal/above squared-distance threshold; heading-sector parallel/oblique/asymmetric branches; signed speed reversals; each blocked separation direction; cooldown six-tick behavior and slot-order side effects.

**Assets/identifiers/observations:** Use source-labelled tables/resources referenced by these entries and every assigned Data unit in the ownership index. Expose named owner state and bounded semantic transition/award/sound/contact events; no RAM map. Proposed module/contract: `port/src/trucks/contact.rs`, adjacent source trace; runtime integration contracts additionally use `game::Port` and debug observations.

**Facts, leads and unresolved questions:** deflect_current_truck_contact has corrected conditional source context; independently verify carry/flags across helper calls rather than using labels as an oracle.

**Status:** worker `/root/behavior_contracts/collision` complete; independent trace `port/src/trucks/contact.trace.md`, executable contracts `port/src/trucks/contact.rs` (24 tests), implementation pending. Rust1.90 locked compile/list pass (134 suite tests); area0 pass/24 expected explicit unimplemented failures/0ignored; all4 infrastructure pass. Commands/results `checks/contracts/collision-results.json`. Covers all13 assigned regions, all16 sector cases, signed/byte-width thresholds, conditional isolated entry, eight separation octants, terrain accept/block combinations, marker-history order and descending scan/cooldown. AI must provide shared vector quantization; motion projects before pair contacts and commits after; objects must retain both probe-history mutations. Conditional-entry natural caller remains uncertainty `collision-conditional-current-entry`.

## ai — Computer driving, ranking and difficulty

**Scope/state owner:** Own AI route bias/timeouts/locks, generated controller words, checkpoint redirects, race ranking and human-upgrade/rank based difficulty adjustments.

**Known entries:** `steer_computer_trucks` (game.asm:2383); `quantize_signed_vector_to_lookup_index` (game.asm:2305); `derive_ai_parameters_from_human_upgrades` (game.asm:8785); `rank_trucks_by_race_progress` (game.asm:8601); `adjust_ai_state_using_race_rank` (game.asm:8920).

**Storage/ledger navigation:** 38 primary regions, 34 Code; region-134 ($000842..$00088D), region-710 ($00088D..$000890), region-135 ($000890..$000950), region-711 ($000950..$000953); complete IDs, ranges and original source links: planning/source-ownership.json filtered by `area=ai`.

**Dependencies/update effects:** tracks,runtime,physics,race,results. Follow both callers and callees; cross-area effects are required contracts, not pure-call assumptions.

**Required contract branches/interfaces:** Skip joined humans; all signed vector quadrants; steering lock expiry; checkpoint timeout/rewind/redirect/restart; fifth-lap winner side effect; minimum/capped upgrades and random rank adjustments; ranking checkpoint and Y tie breaks.

**Assets/identifiers/observations:** Use source-labelled tables/resources referenced by these entries and every assigned Data unit in the ownership index. Expose named owner state and bounded semantic transition/award/sound/contact events; no RAM map. Proposed module/contract: `port/src/trucks/ai.rs`, adjacent source trace; runtime integration contracts additionally use `game::Port` and debug observations.

**Facts, leads and unresolved questions:** AI writes acceleration upgrade each update and supplies controller state after current frame movement; distinguish its route/lap update from human checkpoint service.

**Status:** worker `/root/behavior_contracts/ai` complete; independent trace `port/src/trucks/ai.trace.md` covers all38 assigned regions and cross-call data. Native domain interfaces +37 substantive tests in `port/src/trucks/ai.rs`; locked Rust1.90 compile/discovery pass (171 suite tests), area0 pass/37 exact pending-unimplemented failures/0 ignored; all4 infrastructure tests pass. Exact commands/results/logs `checks/contracts/ai-results.json`. Implementation pending, no QA.

**Contract/source corrections:** shared `quantize_vector` is collision dependency; target service packs reducedY/X then adds quarterturn, not ideal bearings. Timeout rewinds index while cached old rectangle remains selected. FD zero RNG redirects, nonzero skips. Parameter override scans highest computer slot, all-human underflow tail remains outside original route windows (uncertainty `ai-all-human-parameter-tail`). Leader lap4 indexes equality threshold by lap4, rank1 adds38 byte; original rank one/two thresholds both80 make second branch conditional only. Tests fix separate original/synthetic data paths, all RNG call ordering, no implementation oracle. Race owns AI-after-HUD integration and winner/banner handoff; all driver state must join full snapshots in stage4.

## race — Race lifecycle, checkpoints, lap/time and progression

**Scope/state owner:** Own race state, initial prepared frame, human/attract dispatch, checkpoints/laps/winner, postrace invocation and next-order progression. HUD timer semantics are shared with rendering.

**Known entries:** `initialize_selected_race` (game.asm:110); `run_race_frame` (game.asm:227); `poll_controls_and_check_routes` (game.asm:1857); `declare_fifth_lap_winner` (game.asm:1976); `finish_race_and_increment_order` (game.asm:309); `initialize_track_countdown_display` (game.asm:3783).

**Storage/ledger navigation:** 42 primary regions, 38 Code; region-333 ($0000AA..$00017E), region-883 ($00017E..$000198), region-112 ($000198..$0001CF), region-334 ($0001CF..$0001D0); complete IDs, ranges and original source links: planning/source-ownership.json filtered by `area=race`.

**Dependencies/update effects:** runtime,tracks,physics,ai,objects,rendering,transitions,results,audio. Follow both callers and callees; cross-area effects are required contracts, not pure-call assumptions.

**Required contract branches/interfaces:** Initial lap 1 and winner at 5; control/skip checkpoint markers and limits; input copied before human route tests; prepared frame return versus active frame; attract Start unwind modeled as transition; winner dispatch/order wrap; BCD timer saturation/freeze and last-lap banner triggers.

**Assets/identifiers/observations:** Use source-labelled tables/resources referenced by these entries and every assigned Data unit in the ownership index. Expose named owner state and bounded semantic transition/award/sound/contact events; no RAM map. Proposed module/contract: `port/src/race.rs`, adjacent source trace; runtime integration contracts additionally use `game::Port` and debug observations.

**Facts, leads and unresolved questions:** No finite ending inferred: explicit sequence wraps modulo64. Trace winner precedence if multiple trucks complete within one dispatcher update. Countdown timings owned by transitions.

**Status:** worker `/root/behavior_contracts/race` complete; implementation pending. Independent `port/src/race.trace.md` reviews all42 assigned regions/cross-calls; `port/src/race.rs` has38 native `race::tests::*` contracts. Rust1.90 locked compile/list pass209 suite tests; area0pass/38 expected explicit-unimplemented failures/0ignored;4 infrastructure pass. Exact `checks/contracts/race-results.json` plus preserved race-initial-* results. Corrected AI rectangle boundary expectations/source trace in place (lower world edge excluded, upper included), retained original AI logs and reran affected test separately (expected pending). Progress is single race owner, RaceAi alias; banner state word width, lastlap low-byte store. Source corrections: winner freezes primary decrement only, secondary still animates; timer advances prepared/winner/postrace HUD calls, no such freeze guard; HIGHSCORE_DIGITS operand aliases active_human_mask. Grid heading byte preserves high. Transitions owns windows/countdown delay/pause, rendering owns OAM/HUD digit writes, results invokes extra HUD time update. Missing/cyclic malformed extension checks explicit native validation uncertainty, not original behavior. No gameplay or QA.

## objects — Track props, dynamic hazards, pickups and effects

**Scope/state owner:** Own count-prefixed placements, six dynamic sequence slots, random pickup state/awards/lifetime, track-zero composites, track-three specials and moving-truck contact effects.

**Known entries:** `load_track_object_placements` (game.asm:14988); `dispatch_random_pickup_state` (game.asm:15050); `initialize_sequence_objects` (game.asm:15548); `animate_dynamic_object_and_resolve_contact` (game.asm:16205); `test_special_surface_contact` (game.asm:16475); `update_and_project_track_objects` (game.asm:17050).

**Storage/ledger navigation:** 79 primary regions, 46 Code; region-360 ($004560..$0045C7), region-200 ($0045C7..$0045CF), region-278 ($0045CF..$0045D7), region-781 ($0045D7..$0045DA); complete IDs, ranges and original source links: planning/source-ownership.json filtered by `area=objects`.

**Dependencies/update effects:** tracks,runtime,physics,terrain,economy,rendering,audio. Follow both callers and callees; cross-area effects are required contracts, not pure-call assumptions.

**Required contract branches/interfaces:** Negative sentinel/count bounds; one-truck-per-frame rotation; spawn random zero path/slot/type parity; pickup rectangle boundaries, award-count saturation/nitro overflow; eighty-tick lifetime; per-type hazard use exhaustion/reaction; track0/3 gates and effect animation cursors.

**Assets/identifiers/observations:** Use source-labelled tables/resources referenced by these entries and every assigned Data unit in the ownership index. Expose named owner state and bounded semantic transition/award/sound/contact events; no RAM map. Proposed module/contract: `port/src/world/objects.rs`, adjacent source trace; runtime integration contracts additionally use `game::Port` and debug observations.

**Facts, leads and unresolved questions:** Object type identities remain provisional until data/picture consumers justify names. Sequence object source is indexed by full order slot, not only track ID.

**Status:** worker `/root/behavior_contracts/objects` complete; independent source trace `port/src/world/objects.trace.md` covers all79 assigned regions and cross-calls.40 substantive native contracts in `objects::tests` plus1 tracks added-object extension contract; Rust1.90 locked compile/list pass250 suite tests; objects0pass/40 expected pending-unimplemented failures/0ignored, extension1 expected pending failure;4 infrastructure tests pass. Exact tests/commands/results `checks/contracts/objects-results.json`, `objects-list.log`. No behavior implementation/QA. Initial execution logs retained as `objects-initial-*` after independently correcting the zero-gate RNG expected second value0F.

**Cross-area source corrections:** old pickup count forms award; collection lifetime80 ticks immediately79; packed pickup XY centering carries/borrows Y, composite X is byte-only. Dynamic contacts only animation0/12; idle miss skips lifetime, nonzero reached-shadow underflow can consume uses. Full truck speed word zeroed; descending hazards see prior stops. Special contact requires high historyF and marker, cursor0→10 with sound vs12/17→19 silent, only nonnegative low speed halves. Track0 inclusive count4 executes5 with derived first-field tail/count reaction; fourth-frame count→3. Explicit uncertainties retain conditional tail growth and capped pickup state11 consumers. Native ObjectContent now part of optional TrackContent: originals resolve ROM/full-order when None; added/replaced tracks require explicit validated definitions. Physics must compose real contact state before rejected proposals; race/results/economy share single WorldObjects pending-award owner. Presentation retains untouched OAM when later composite writes shorten; object sprites are staging writes, not an instruction to clear. Implementation remains pending.


## rendering — Graphics assets, sprites, text, HUD and composition

**Scope/state owner:** Own planar ROM graphics/palettes/tilemaps/fonts, display staging, source OAM packing/Y sorting/priority, truck pose uploads, sprite animation, HUD digits and read-only Mode1 composition.

**Known entries:** `upload_four_truck_pose_tiles` (game.asm:4058); `sort_sprites_by_screen_y` (game.asm:4302); `pack_sorted_sprites_to_oam` (game.asm:4240); `draw_remapped_tile_rectangle` (game.asm:14918); `draw_terminated_tile_text` (game.asm:13389); `draw_seven_large_text_rows` (game.asm:13444); `initialize_race_hud_tiles` (game.asm:14523).

**Storage/ledger navigation:** 101 primary regions, 33 Code; region-341 ($0007B0..$0007BB), region-132 ($0007BB..$0007FA), region-133 ($0007FA..$000804), region-27 ($00104C..$0010C2); complete IDs, ranges and original source links: planning/source-ownership.json filtered by `area=rendering`.

**Dependencies/update effects:** tracks,terrain,physics,objects,race,transitions,attract,names,economy,results. Follow both callers and callees; cross-area effects are required contracts, not pure-call assumptions.

**Required contract branches/interfaces:** ROM palette/planar decoding and VRAM word→byte units; inclusive $401-word clear; rectangle width flags/remap/row stride; terminated and paired glyph formats; OAM high bits/flips/sizes/Y sort ties; animation sentinel timing; per-track overlays/titles, BCD HUD digits; layer/window/color math/brightness.

**Assets/identifiers/observations:** Use source-labelled tables/resources referenced by these entries and every assigned Data unit in the ownership index. Expose named owner state and bounded semantic transition/award/sound/contact events; no RAM map. Proposed module/contract: `port/src/presentation/mod.rs`, adjacent source trace; runtime integration contracts additionally use `game::Port` and debug observations.

**Facts, leads and unresolved questions:** Hex diagnostic helpers are present but natural callers/access unresolved: trace before classifying; do not add player diagnostic UI. NMI uploads staged prior state, not newly mutated simulation output. Toolkit supplies composition mechanisms only.

**Status:** worker `/root/behavior_contracts/rendering` complete; independent trace `port/src/presentation/rendering.trace.md` covers all101 primary ranges/cross-consumers. Native tests `port/src/presentation/tests.rs`:53 substantive rendering contracts; compile/list304 total pass,53 expected pending-unimplemented failures/0 ignored; four infrastructure tests pass. Extra tracks extension required-presentation contract compiled/discovered/runs with expected pending failure. Exact evidence `checks/contracts/rendering-results.json`. Actual video/Display/SpriteArrays/MapImage/TrackPresentation interfaces remain pending implementation; no gameplay/QA. Source corrections: shared first DMA wrapped tail wholly overwritten by second before display (superseded domain-tail hypothesis retained); full512-byte palette consumer; COLDATA0 preserves fixed color; byte-paired glyph wrapping; OAM hide then repack. Cross-owner: transitions supplies window edges/brightness and owns timing; attract/names/shop/results install scene-specific assets, alternate shop palette remains shop ROM consumer; generic zero/grid/window-neighbor data consumers unresolved in uncertainties. Added named tracks require validated title/HUD/overlay layout, originals None resolve source. Actual Game/Race must compose video/objects/race and snapshot all retained mutable state; no phase-only substitutes.

## transitions — Fades, scrolling windows, ready countdown and pause

**Scope/state owner:** Own frame-resumable fade/wait/HDMA window effects, attract scrolling/bounce, race ready/get-ready/countdown and pause release/new-press transition.

**Known entries:** `fade_out_fade_in_and_wait_frame` (game.asm:6986); `animate_scrolling_window` (game.asm:8243); `setup_hdma_window_and_build_scanlines` (game.asm:12183); `begin_race_get_ready_window` (game.asm:17499); `draw_track_countdown_frame` (game.asm:17718); `begin_human_race_pause_control` (game.asm:17894).

**Storage/ledger navigation:** 96 primary regions, 58 Code; region-4 ($001A24..$001A6F), region-963 ($001A6F..$001A78), region-31 ($002275..$002287), region-736 ($002287..$002293); complete IDs, ranges and original source links: planning/source-ownership.json filtered by `area=transitions`.

**Dependencies/update effects:** runtime,controls,race,rendering,attract,names,economy,results,audio. Follow both callers and callees; cross-area effects are required contracts, not pure-call assumptions.

**Required contract branches/interfaces:** Fade brightness endpoints/waits; window shape scaling/signed clipping/closed rows and double buffer; frontend grow/shrink max(extent/16,1), race shrink max(extent/4,1); countdown delay underflow inclusive six calls; abort-enabled Start versus ignored transition input; pause owner selection/release/new press and fourth-slot asymmetry.

**Assets/identifiers/observations:** Use source-labelled tables/resources referenced by these entries and every assigned Data unit in the ownership index. Expose named owner state and bounded semantic transition/award/sound/contact events; no RAM map. Proposed module/contract: `port/src/presentation/transitions.rs`, adjacent source trace; runtime integration contracts additionally use `game::Port` and debug observations.

**Facts, leads and unresolved questions:** Do not represent blocking assembly waits as host blocking loops. Natural pause was later observed in archive gameplay notes despite older per-region summaries saying unobserved; source contracts remain independently traced.

**Status:** worker `/root/behavior_contracts/transitions` complete; independent trace
`port/src/presentation/transitions.trace.md` covers all96 assigned ranges/crosscalls.
52 substantive native contracts in `port/src/presentation/transitions/tests.rs`;
locked Rust1.90 compile/list pass356 suite tests; area0pass/52 expected explicit
unimplemented failures/0ignored, all4 infrastructure pass. Exact commands/logs:
`checks/contracts/transitions-results.json`. Original49-test logs preserved
`transitions-*-initial49.log` and results-initial49.json before source/fixture
corrections +3 full scheduling/capacity contracts. No implementation or QA.

**Corrections/handoffs:** frontend contraction divides16, race divides4; enabled
Start abort skips normal mask/blank cleanup; fade counter uses captured prior
value; initial countdown delay6 draws7, later5 draws6; opposite active/build
buffer and untouched staging tail retained. `disable_masks` clears actual Display
selection only, not retained Window.masks or HDMA. Ready M16 gate names active+
shifted human mask word. Top-level pause contracts include actual banner/inhibit
and2priming waits before growth, JOY3-triggered fourthowner waits actualJOY4.
TrackPresentation now includes explicit CountdownLayout so added named tracks
need no original index. Attract integrates overlay8→economy normalize→15→all4
Start/timer poll, plus wrapper final poll; race retains sole banner state and
actual shared calls. Source unknown046260..04629A not shape consumer, retained
rendering uncertainty. Frame/snapshot integration and behavior implementation pending.

## attract — Licensed/software splashes, attract cycle and Start reentry

**Scope/state owner:** Own initialized-RAM signature, boot splashes/timeouts, attract graphics/records panel, rotating attract order slots and Start transition into selection.

**Known entries:** `initialize_frontend_and_attract` (game.asm:7775); `show_licensed_splash` (game.asm:7969); `wait_and_replace_startup_splashes` (game.asm:8012); `prepare_attract_panel` (game.asm:8156); `show_scrolling_highscores` (game.asm:8550); `poll_start_or_splash_timeout` (game.asm:10334).

**Storage/ledger navigation:** 23 primary regions, 14 Code; region-7 ($002037..$0020DE), region-8 ($0020E2..$00210A), region-734 ($00210A..$002125), region-9 ($002125..$00212A); complete IDs, ranges and original source links: planning/source-ownership.json filtered by `area=attract`.

**Dependencies/update effects:** runtime,transitions,rendering,names,economy,results,race,audio. Follow both callers and callees; cross-area effects are required contracts, not pure-call assumptions.

**Required contract branches/interfaces:** Cold versus valid-signature path; 140/110 wait behavior and early Start; attract order slot rotation; human mask clearing and AI BCD nitro99; Start reentry bypass; nobody-joined return to attract; source title/scroll/fade assets.

**Assets/identifiers/observations:** Use source-labelled tables/resources referenced by these entries and every assigned Data unit in the ownership index. Expose named owner state and bounded semantic transition/award/sound/contact events; no RAM map. Proposed module/contract: `port/src/frontend/attract.rs`, adjacent source trace; runtime integration contracts additionally use `game::Port` and debug observations.

**Facts, leads and unresolved questions:** Source title identity should come from actual graphics/text, not invented menu screens; records rendering data owned by results.

**Status:** worker complete `/root/behavior_contracts/attract`; independent trace `port/src/frontend/attract.trace.md` covers all23 assigned ranges and crosscalls. Contracts `port/src/frontend/attract/tests.rs`,32 substantive native tests, minimal pending interfaces `frontend/attract.rs`; locked Rust1.90 compile/list pass388 suite; area0pass/32 explicit unimplemented failures/0ignored, all4 infrastructure pass. Exact evidence `checks/contracts/attract-results.json`. Implementation pending. Source corrections: cold record defaults AFTER first completed splash; full5-byte signature only entry, poll firstbyte only; waits141/111/501 inclusive polls; warm entry resets24 wallet+20stock/upgrade bytes while preserving records; highscore preincrement scroll; palette partial writes and inclusive clears. Panel uses actual wallet/Truck/Display/window/VRAM state, not effect-only mocks. Shared SessionRecords/wallet interface for results/economy reconciliation; no second race-state owner. Names/shop actual calls precede joined WORD test even nobody joined. No implementation or QA.

## names — Human joining, driver panels and name entry

**Scope/state owner:** Own two-player page join masks, Select edges, character-grid cursor/repeat/edit/EXIT rules, temporary paired glyph names, final three-character names and page completion.

**Known entries:** `begin_player_selection` (game.asm:13516); `initialize_name_entry_pages` (game.asm:13543); `run_name_entry_frame` (game.asm:13647); `commit_names_and_tick_join_countdown` (game.asm:13732); `test_first_player_join_edge` (game.asm:13864); `edit_first_player_name` (game.asm:14096); `edit_second_player_name` (game.asm:14242).

**Storage/ledger navigation:** 57 primary regions, 40 Code; region-18 ($003257..$00325A), region-32 ($00325A..$003296), region-19 ($003296..$0032AE), region-33 ($0032AE..$0032DB); complete IDs, ranges and original source links: planning/source-ownership.json filtered by `area=names`.

**Dependencies/update effects:** controls,rendering,transitions,attract,economy,audio. Follow both callers and callees; cross-area effects are required contracts, not pure-call assumptions.

**Required contract branches/interfaces:** Join edge/done precedence; seven-row grid wrap and each special cell (erase/EXIT); four-slot live-preview/three-byte commit boundaries; repeat cadence; countdown ticks every fourth frame and absent-player done mask; two pages/controller asymmetry; final name copy and sound/transition order.

**Assets/identifiers/observations:** Use source-labelled tables/resources referenced by these entries and every assigned Data unit in the ownership index. Expose named owner state and bounded semantic transition/award/sound/contact events; no RAM map. Proposed module/contract: `port/src/frontend/names.rs`, adjacent source trace; runtime integration contracts additionally use `game::Port` and debug observations.

**Facts, leads and unresolved questions:** frontend_page_limit is initialized zero in observed boot code; extra-page routines still require support when reachable through source state. Do not invent a player-count menu.

**Status:** worker `/root/behavior_contracts/names` complete; independent trace `port/src/frontend/names.trace.md` covers all57 primary ranges and crosscall consumers;36 substantive native contracts `frontend::names::tests::*` in `port/src/frontend/names/tests.rs`. Rust1.90 locked compile/list424 suite pass; area0pass/36 expected explicit pending-unimplemented failures/0ignored; all4 infrastructure pass. Exact `checks/contracts/names-results.json`. Implementation pending. Source corrections: cursor0/2/4/6 permits fourth preview, commits only0/2/4; repeat reload8 gives9-call interval; retained shared Select/B history and repeats not reset at page init; countdown expiry before input blocks late joins; page compare M8 after final ACK SEP20; joined and standard driver variant full palettes/maps, source palette4→5 remap for second panel. Actual Session/VideoMemory/SpriteArrays staging contracts, no mock-only flow. Upcoming shop/results reuse SessionRecords and retained history; conditional inline heading has no established natural caller or added UI.

## economy — Wallets, upgrades, speed shop and stock

**Scope/state owner:** Own six unpacked decimal wallet digits, nitro/upgrade stock, shop pages/cursors/purchase validation and field redraw intents; initialize session money independently of record defaults.

**Known entries:** `initialize_money_and_record_defaults` (game.asm:12564); `add_award_to_money_digits` (game.asm:12724); `normalize_six_money_digits` (game.asm:12749); `convert_inactive_truck_money_to_nitro` (game.asm:10362); `prepare_player_shop_frontend` (game.asm:10414); `run_shop_input_frame` (game.asm:10565); `buy_player_one_shop_item` (game.asm:10915).

**Storage/ledger navigation:** 72 primary regions, 60 Code; region-304 ($002C5C..$002CA7), region-16 ($002CA7..$002CB9), region-321 ($002CB9..$002CF6), region-756 ($002CF6..$002CFB); complete IDs, ranges and original source links: planning/source-ownership.json filtered by `area=economy`.

**Dependencies/update effects:** controls,names,results,physics,rendering,transitions,audio. Follow both callers and callees; cross-area effects are required contracts, not pure-call assumptions.

**Required contract branches/interfaces:** Decimal carry/borrow/high-digit guard; insufficient/equal funds and item-specific caps/costs; purchase B edge and repeat reload8 giving9-call movement; clamped row/column and source Start Next Race cell done selection; second player/page wallet mapping; inactive cash→nitro and losing upgrade stock settlement.

**Assets/identifiers/observations:** Use source-labelled tables/resources referenced by these entries and every assigned Data unit in the ownership index. Expose named owner state and bounded semantic transition/award/sound/contact events; no RAM map. Proposed module/contract: `port/src/frontend/shop.rs`, adjacent source trace; runtime integration contracts additionally use `game::Port` and debug observations.

**Facts, leads and unresolved questions:** Track record/high-score default copy is co-located in money initializer but owned by results. Trace money additions at digit4/5 separately: input/carry contracts are not interchangeable.

**Status:** worker `/root/behavior_contracts/economy` complete; independent trace `port/src/frontend/shop.trace.md` covers all72 primary ranges and crosscalls;43 substantive native contracts `frontend::shop::tests::*` in `port/src/frontend/shop/tests.rs`. Rust1.90 locked compile/list467 suite pass; area0pass/43 explicit pending-unimplemented failures/0ignored, all4 infrastructure pass. Evidence `checks/contracts/economy-results.json`, superseded pre-item-order-correction logs/results retained as economy-initial-*. Implementation pending. Source corrections: grid CLAMPS despite wrap labels; repeat reload8 yields9calls; B at cell5 completes, Start does not; first done extra NMI. Ordinary award/debit is digit3 ($10000 displayed units), finer4/5 additions conditional; CMP-N borrow threshold8A not80; highest borrow previous-wallet tail/selected-byte source effect explicitly conditional. Inactive conversion decrements BEFORE80 cap, checks only0..3 and does not draw. Stock order nitro/acceleration/steering/speed/suspension. UI significant money digits compact left then00, bars cap6 and leave stale whenzero, redrawpreincrement and sixprimes. Shared Session wallets/Selection histories/Truck stock remain single owners; results must settle WorldObjects pending awards once. Actual Shop/Display/VRAM/Window/Fade composition contracts, no phase-only mock. No implementation or QA.

## results — Podium, records, high scores, wipeout and continue

**Scope/state owner:** Own final rank awards/time/records, high-score insertion/order, timed podium, last-ranked human penalty, continue cursor, participant removal and game-over/frontend return.

**Known entries:** `begin_postrace_presentation` (game.asm:9137); `draw_podium_awards_and_names` (game.asm:10049); `commit_or_flash_track_record` (game.asm:9287); `count_active_humans_for_continue` (game.asm:9412); `penalize_losing_truck_upgrades` (game.asm:9461); `show_continue_prompt_and_cursor` (game.asm:9718); `insert_truck_totals_into_high_scores` (game.asm:12953).

**Storage/ledger navigation:** 62 primary regions, 40 Code; region-247 ($002694..$0026CB), region-743 ($0026CB..$0026E9), region-274 ($0026E9..$0026F6), region-744 ($0026F6..$0026FC); complete IDs, ranges and original source links: planning/source-ownership.json filtered by `area=results`.

**Dependencies/update effects:** ai,race,economy,names,rendering,transitions,attract,audio. Follow both callers and callees; cross-area effects are required contracts, not pure-call assumptions.

**Required contract branches/interfaces:** Rank award table/name assignment; record less/equal comparisons and timer250 commit/blink; any controller1/2 dismiss or timeout; pending awards settlement once; single/multi-human continuation underflow; four upgrades minus2 clamp0; YES/NO participant/cash/shop transition; sorted scores ties and session default preservation.

**Assets/identifiers/observations:** Use source-labelled tables/resources referenced by these entries and every assigned Data unit in the ownership index. Expose named owner state and bounded semantic transition/award/sound/contact events; no RAM map. Proposed module/contract: `port/src/frontend/results.rs`, adjacent source trace; runtime integration contracts additionally use `game::Port` and debug observations.

**Facts, leads and unresolved questions:** No battery persistence: records survive only native session/reset semantics supported by source signature. Ending is loop/loss/return flow, not an invented final victory screen.

**Status:** worker `/root/behavior_contracts/results` complete; independent trace `port/src/frontend/results.trace.md` covers all62 assigned ranges, defaults/resources/shared calls;42 substantive native contracts `frontend::results::tests::*` in `port/src/frontend/results/tests.rs`. Rust1.90 locked compile/list509 suite pass; area0pass/42 verified explicit pending-unimplemented failures/0ignored, all4 infrastructure pass. Exact `checks/contracts/results-results.json`. Implementation pending. Source corrections: pending record timer>250 bypasses JOY polling;250 commit extra NMI then input; record flag INC wraps, fraction ignored. Podium credits rank money now while displayed pickup count settles separately once. Highscore order0 lowest/9highest, strict lexicographic insertion may duplicate equal score below existing tie. Penalty alias resets same losing wallet digit2 startercash, byte DEC/BPL upgrades. Continue heldB/noedge/left priority/loser port; nohuman301wait high-byte timeout. Original SessionRecords single owner, AddedRecords only arbitrary added named slots, no fixed total count. Actual Session/Progress/VRAM/Display/full postrace mutations, no phase-only mock. No implementation or QA.

## audio — ROM audio resources, host commands and native SPC music/effects

**Scope/state owner:** Own upload-derived immutable audio definitions, host command/ack/heartbeat timing, eight native sequencer voices, effect arbitration, control-flow stacks/loops, duration/note/envelope/pitch/modulation/noise/fade and BRR/DSP mixer state.

**Known entries:** `upload_audio_block_streams` (game.asm:18362); `spc_initialize_and_poll_host` (game.asm:20262); `spc_start_priority_effect_voice` (game.asm:20388); `spc_load_song_and_process_voice` (game.asm:20574); `spc_sequence_command_handlers` (game.asm:22042); `spc_pitch_envelope_and_modulation_helpers` (game.asm:20702); `spc_write_voice_dsp_and_initialize_timer` (game.asm:22363).

**Storage/ledger navigation:** 717 primary regions, 17 Code; region-115 ($000256..$000263), region-222 ($000263..$000265), region-26 ($005FFE..$00609C), region-78 ($070004..$0700EB); complete IDs, ranges and original source links: planning/source-ownership.json filtered by `area=audio`.

**Dependencies/update effects:** runtime,attract,names,economy,race,objects,transitions,results. Follow both callers and callees; cross-area effects are required contracts, not pure-call assumptions.

**Required contract branches/interfaces:** Upload block metadata gaps/overwrite lifetime and split sample20; sixteen song pointer/config columns and141-byte copy overlap; command operand lengths including8E/BA/B6/B8; nested call/loop returns; priority effect reserved voices/reset persistence; first note/rest ordering, AC pitch snap, interpolation zero duration/release; signed fades/slow divider; BRR loops/envelopes/key-on/off and deterministic sample-clock snapshots.

**Assets/identifiers/observations:** Use source-labelled tables/resources referenced by these entries and every assigned Data unit in the ownership index. Expose named owner state and bounded semantic transition/award/sound/contact events; no RAM map. Proposed module/contract: `port/src/sound/mod.rs`, adjacent source trace; runtime integration contracts additionally use `game::Port` and debug observations.

**Facts, leads and unresolved questions:** Race music host selectors6..13 become songs5..12; do not assign unsupported musical titles. Main ROM $070004→ARAM$0400; second upload $078004→ARAM$8200. $070149/$0704FF unknown bytes and unsupported patterned tails require consumer assessment. Implement domain sequence interpreter, not an SPC CPU emulator.

**Status:** worker `/root/behavior_contracts/audio` complete. Independent detailed trace:
`port/src/sound/audio.trace.md` (all717 original range links, cross-call/flag/data
lifetime derivations). Native contracts in `sound/{definitions,sequencer,voice,host,synthesis}.rs`:
101 substantive discovered tests; Rust1.90 locked compile/list610 suite pass,
area0pass/101 expected explicit pending-unimplemented failures/0ignored;
all4 infrastructure tests pass. Exact commands/logs `checks/contracts/audio-results.json`.
Implementation remains pending; no original audio or gameplay QA executed.

**Source corrections and handoff:** Follow actual B6/B8 vectors, not label order;
zero-rise returns without stage advance, fall/release use duration-tick-1;
literal positive-fade BMI branch overrides generic clamp prose. Note initialization
precedes AC target stepping; new KON delayed to next driver update, KOFF is current.
Retain live141-byte envelope overlap across songs; song0 config-FF low-ARAM reads
remain a documented silent-start policy/uncertainty. Native clock second update264
samples is ordered deterministic policy, not cycle-exact SPC timing. All33 BRR
entries, split sample20, instrument33 effective reads and conditional commands
remain supported. Immutable data is not behavior. Game must own one complete Sound
shared across frontends, consume existing session audio_commands in order, expose
semantic voice/command state and include mixer/predictor/clock/resampler/live
configuration in clone/full snapshots; possible toolkit mixer serialization change
must follow toolkit instructions and library-changes.md. No SPC CPU/RAM map.
Original unknown070149/0704FF identities preserved with source consumer assessment;
no exclusions or confirmed new classifications declared.

## Cross-area source-backed scheduling and coverage decisions

The inspected `run_race_frame` / `update_race_frame_services` calls wait/input/pause,
then truck motion, wheel integration, suspension speed correction, pose selection,
sprite Y sorting/OAM packing, sixteen sprite animations, controller copy and route
checks, world objects, banner/HUD and finally AI. Initial preparation runs one
service frame under prepared_race_frame. Preserve this latency/order until detailed
contracts prove a different entry context. NMI separately uploads OAM/pose tiles
when uninhibited and always increments frame/animation/audio heartbeat counters.
Presentation must retain appropriate staging so a pure render does not accidentally
advance or expose a future simulation frame. Window/fade/pause blocking waits
become explicit resumable states at console cadence.

All sixteen original tracks, reverse data, 64 order slots, alternate pages,
multiplayer masks, attract/demo, podium/record/high-score, continue/game-over,
track-zero/three specials and secondary SPC modes remain in scope. Source order
wraps; no final ending scene or battery save has been established. Conditional
helpers and diagnostic text routines require trace/relevance decisions, not an
unrequested player-facing debug feature. No exclusion follows from a historical
runtime coverage gap. Archive gameplay notes are leads for later bounded QA,
not test expectations or permission to execute broad gameplay during planning.

Assets require semantic decoding and validators in their domain owners; raw
regions alone do not provide valid tracks, pictures or sound. Source metadata,
physical overlap and uploaded lifetimes must be preserved. Extension manifests
must combine original and added named track slots with no global fixed count;
original source tables retain their sixteen-track decode bounds. Original font,
HUD, truck pose strips, frontend/continue/podium graphics, title/countdown maps,
window shapes, terrain/routes/placements, SPC descriptors/streams/instruments/BRR
all have explicit owners. Source banks and offsets remain in named loader
recipes/definitions, not executable rules.

## Stage 3 combined handoff

All sixteen assigned workers completed sequentially; no additional independent behavior area
was discovered beyond the inventory. Cross-area consumers remain linked in adjacent traces.
The final native suite compiles and lists 610 tests (606 domain contracts, four infrastructure).
Combined assets and Game infrastructure each pass two tests. Domain assertions retain expected
pending failures: implementation has not begun. Primary source ownership preserves all 1,527
original region identities/ranges and 1,408 units, and all dispositions remain pending.
`checks/contracts/stage3-handoff.json` records every discovered test and its area/worker/trace,
with individual execution manifests and preserved superseded evidence.

Resolved contract integration: sole race Progress with RaceAi alias; shared signed-vector helper;
source-backed rectangle lower-excluded/upper-included correction; shared Session/Selection
wallets/names/retained input history; single WorldObjects pending award owner; original and added
TrackContent terrain/routes/objects/presentation/countdown definitions; actual staged Display
and Sound interfaces. Stage 4 must compose these owners in Game, not replace implementation
with phase/effect logs. Runtime debug remains the honest skeleton until real state is wired.

Stage5 bounded evidence: natural startup/name/shop/initial Racing and46ticks of throttle/steer/nitro passed with reviewed captures and active PCM. Six independently aligned original full-frame pairs are exact (license/software/two highscore scroll phases/grid/countdown), including the shared $077D countdown/banner word. Initial30frame captured music has exact8voice sequence/duration/DSPpitch/sample/gain endpoint progression; native interpolation waveform/amplitude differences remain explicit. No full-game traversal, physical audio playback, desktop window, or expanded gameplay coverage is claimed. See debug/report.md and debug/qa.json.

## Modernization iteration 1 representation links

The source rules and expectations in this document remain authoritative. Native nitro inventory and race elapsed units now live in `port/src/trucks/nitro.rs` and `port/src/race/time.rs`. Their source asset/save/display encodings remain at boundaries, including lossless compatibility for exceptional old save bytes. Name/shop scene extraction preserves NMI waits, window/history ownership and sound order. See [architecture.md](architecture.md#modernization-iteration-1) and [modernize/changes.json](modernize/changes.json) for attributable changes and fresh validation; earlier QA records still describe their recorded builds.

## Modernization iteration 2 representation links

Source route and hazard rules retain their existing expectations. `port/src/world/tracks/route.rs` decodes node geometry and traversal actions at the asset boundary; separate acceptance, steering, ranking and grid views preserve source distinctions. `port/src/world/objects/hazard.rs` expresses lifecycle stages, remaining response ticks and remaining complete reactions while preserving legacy wire fields and conditional states. `port/src/trucks/bearings.rs` consolidates the identical AI/contact lookup resource. Source traces remain unchanged historical evidence. See [architecture.md](architecture.md#modernization-iteration-2), the iteration2 source-backed contracts in `port/src/game/iteration_two_tests.rs`, and [modernize/changes.json](modernize/changes.json).
