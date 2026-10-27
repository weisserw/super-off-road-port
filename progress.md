Follow-up: reproduced and repaired two independent failures: completed one-shot audio left an invalid save cursor; eager terrain-neighbor reads stopped driven racing at the track edge. All627 game tests/six check gates and81 toolkit tests/Clippy pass; see checks/follow-up/audio-freeze.json and audio-check.json. Preceding terrain controls use an explicit zero power-on policy; physical-console values remain unestablished. Historical pipeline and QA records below remain preserved.


# Port progress

All five stages complete. Coordinator completion audit PASS: current source/build identities, preserved original ledger/reference identities, complete implementation accounting, reviewed bounded QA, and both administrative audits verified. See checks/implementation/coordinator-completion.json.
Current native check: all six required gates PASS,620 tests/0failures/0ignored. One reviewed QA pass covers startup/46initial-race ticks, six selected exact original frames and30frame captured-audio comparison; no whole-game or device QA. Source accounting preserves1527regions/1408units/22assessed uncertainties. See debug/report.md, debug/qa.json and checks/implementation/stage5-final-check.json. Toolkit edits remain uncommitted; no publication/check-in.

Coordinator: /root. Skeleton subagent: /root/skeleton. Stage 1 was edited by the skeleton agent; stage 2 documentation by
/root/behavior_planning. The original Stage1 record below is historical; current Stage4 milestones follow.

| Stage | Status / handoff |
| --- | --- |
| 1 Skeleton | Complete; all six infrastructure checks passed; handoff below |
| 2 Behavior planning | Complete; /root/behavior_planning; sixteen sequential worker scopes in behaviors.md |
| 3 Behavior contracts | Complete; /root/behavior_contracts and sixteen sequential area workers; 606 domain contracts, 610 total tests |
| 4 Whole-game implementation | Complete; /root/implementation; final615tests/six checks/audit pass; handoff below |
| 5 Short smoke / exact-frame QA | Complete; one reviewed bounded pass,620tests/sixgates/both audits PASS |

Stage 1 artifacts: architecture.md (owners, provisional source-linked identity
inventory and scheduling), debug/api.md (version 1 semantic state and persistence
contracts), port/source-identifiers.json (all named ledger units with original
IDs/ranges/source descriptions), port/src/resource_definitions.rs (1,055 named
raw Data-region recipes), assets.rs (matching-ROM loading and raw overrides),
identifiers.rs (closed domain identities/events), game.rs (shared blank/pending
Game implementation and full skeleton snapshots). Main/browser now share native
library modules; behavior tests will be discovered by native cargo test.

`cargo +1.90.0 test --locked --manifest-path port/Cargo.toml` passed all four
infrastructure tests: all named ROM recipes and identity/bounds; raw replacement
size/added-slot validation; atomic full snapshots and clone isolation; actual
debug session branch restore and replay-checkpoint import with transport hash
validation. No ignored tests. Full `python3 run.py check` **passed** all six checks: format, desktop Clippy
with warnings denied, native unit tests, desktop build, Wasm Clippy with warnings
denied, locked Wasm release build. Rust toolchain: 1.90.0. Preserved stage results
are checks/skeleton.json and checks/skeleton.log; checks/latest.json and
checks/latest.log also contain this successful run. Desktop runtime,
browser runtime and audio devices are untested. No gameplay smoke test, emulator
experiment, publication, check-in or toolkit edit has occurred.

Ledger: all original 1,527 regions remain pending; original identities/ranges
preserved. Four uncertainty entries remain pending. Raw storage loading is not
implementation accounting. research/current.json is absent. QA plan/results and
acceptance records are unchanged; no modernization was invoked.

Stage 1 handoff: no concrete blockers. The coordinator should launch stage 2
in its own subagent. Exact resumption: read prompts/plan.md and actual
source to replace behaviors.md with complete stable behavior areas and their
future worker scopes. Planning must refine provisional scene/asset/module names,
check all dispatch/data consumers and record dependencies; no gameplay logic or
contract worker launches belong to planning. Semantic asset decoding/validators
and sound/music identity remain explicitly deferred to source contracts and
implementation.


## Stage 2 handoff

Behavior-planning subagent /root/behavior_planning completed a source survey of
startup/frame dispatch, track/route/asset pointer tables, truck/terrain/collision/
AI, frontend selection/name/shop, postrace/continue/high-score, windows/pause,
HUD/sprite/text and uploaded SPC command/sequence/sample consumers. Artifacts:
behaviors.md (sixteen stable cohesive areas, dependencies, source entry lines,
branch/fixture requirements, proposed interfaces and unresolved questions),
planning/source-ownership.json (all 1,527 regions and 1,408 units assigned once),
architecture.md (native state owners, actual phase order, asset/numerical rules
and required observations). Primary assignments are navigation only, not traces.
No ledger/uncertainty disposition was changed; no reference or toolkit edits.

Source-backed findings include controller/AI updates after current-frame motion,
NMI display staging, 64-slot repeating progression over sixteen original tracks,
Mode1 display despite Mode7 multiplication use, original zero-SRAM/session-only
records, alternate-page/pause controller asymmetries and split uploaded sample20.
Carry/branch/format edge derivations, original diagnostic relevance and unknown
consumer impact remain explicitly assigned to independent contract workers.

Planning validation evidence: planning/validation.json. The ownership index has exactly 1,527 distinct original region
IDs, 1,408 original unit IDs, no missing/extra/duplicate assignments; all sixteen
areas have entries and proposed contract paths. No code changed and no behavior
contracts were compiled or run. The prior complete skeleton check remains valid;
no passing build is asserted as behavior proof. QA not started; earlier evidence
and records preserved, no gameplay experiment/publication/check-in performed.

Exact resumption: coordinator launches the stage 3 subagent with full
prompts/contract.md and this handoff. That coordinator launches one separate
worker sequentially per behaviors.md area: runtime, tracks, terrain, controls,
physics, collision, ai, race, objects, rendering, transitions, attract, names,
economy, results, audio. Each receives full contract prompt, source revision,
its area and dependencies, minimal-interface instructions and current skeleton.
Record pending→active→completed worker evidence/test compile/list/run outcomes.
Reconcile cross-area state/calls as workers finish; new discoveries receive their
own worker. Compiling substantive behavior tests may fail until implementation;
do not weaken them. No known concrete tooling blocker.

## Stage 3 live worker record

Coordinator /root/behavior_contracts. Workers run sequentially with exclusive edits.
Runtime is first; all sixteen contracts and implementations initially pending.

Runtime worker **active**: /root/behavior_contracts/runtime. Independently traced boot/RNG/NMI and frontend persistence boundary; writing source-derived contracts. No behavior implemented.

Runtime worker **complete**: /root/behavior_contracts/runtime. Source trace in
port/src/runtime.trace.md; minimal pending interfaces and13 substantive contracts
in port/src/runtime.rs, native-registered in lib.rs. Rust1.90 locked test compile
and discovery pass (17 total); runtime1 pass/12 expected pending failures/0 ignored;
all4 infrastructure tests separately pass. Exact commands/logs:
checks/contracts/runtime-results.json. No behavior implementation or QA.
Source correction: RNG final state same for byte/word X despite overlapping STX;
word A returns adjacent history in high byte. Power-on seed unresolved and logged
in uncertainties.json; no arbitrary default was imposed. Runtime counts/cadence
contract NMI cross-area effects while presentation retains upload ownership.
Exact next worker: tracks, then terrain per behaviors.md; coordinator reconciles
interfaces and launches each sequentially.

Tracks worker `/root/behavior_contracts/tracks` active: independently tracing selection, all parallel tables/routes, source grid branch/word semantics and loading; exclusive editor.

Tracks worker **complete**: /root/behavior_contracts/tracks. Source trace
port/src/world/tracks.trace.md; domain interfaces plus17 native contracts in
port/src/world/tracks.rs. Compile/list pass (34 total), tracks0 pass/17 expected
unimplemented failures/0 ignored; all4 infrastructure pass. Commands/evidence:
checks/contracts/tracks-results.json. No behavior implementation/QA/toolkit/reference edits.
All16 parallel descriptors and32 route views, selection, grid, wheels, content
bounds and named replacement/extensions contracted. Heading-zero grid branch,
inclusive terrain length and overlapping route stream corrections recorded.
Next worker: terrain; use Descriptor projection fields/TrackContent terrain and
Grid.initial_surface_origin, and independently trace actual wheel offset/probe
helper side effects. Do not treat origin as computed surface coordinates.

Terrain worker `/root/behavior_contracts/terrain` active: independently tracing neighborhood addressing, triangle/block paths, probe offsets and signed Mode7 multiplication/projection. Exclusive editor; contracts pending.

Terrain worker **complete**: `/root/behavior_contracts/terrain`. `port/src/world/terrain.rs` + adjacent trace cover every assigned region and cross-calls; 25 contracts, Rust1.90 locked compile/list pass (59 suite tests), area0 pass/25 expected unimplemented failures/0 ignored; assets2+game2 infrastructure pass. Exact evidence `checks/contracts/terrain-results.json`. No implementation/QA/toolkit/reference edits. Corrections: ones-complement reflection/corner swaps, signed word slopes and overlapping base, quadrant compare+carry, prioritized BIT-overlap block tails, signed sine doubled probe, signed/truncated Mode7 projection. Objects must retain marker-history high nibble effects, physics retains sampled/old-summary ordering; tracks grid uses inherited pose before final probe. Boundary-state uncertainty retained explicitly, no clamp/zero fiction. Next worker controls.

Controls worker `/root/behavior_contracts/controls` active: independently tracing both direction helpers/tables, controller masking, frontend page reads and race dispatcher latency. Exclusive editor; contracts pending.

Controls worker **complete**: `/root/behavior_contracts/controls`. Native interfaces+15 contracts `port/src/controls.rs`, independent `controls.trace.md`. Rust1.90 locked compile/list pass74 total, area0 pass/15 expected unimplemented failures/0 ignored; all4 infrastructure pass. `checks/contracts/controls-results.json` retains exact evidence. Full67 assigned-region review; paired word high bytes consumed, opposite-direction combinations source-table-specific, every frontend page samples JOY1/2, race preserves disabled AI words and movement-before-new-human-copy latency. No behavior implementation/QA/toolkit/reference edits. Conditional word helper entry remains honestly unresolved in uncertainties.json. Rendering must contract mixed region49 hide sprites (both OAM first+Y byte prepack,16 sprite Ys,32 high bytes then repack) and font uploads (two0800 spans then inclusive clear helper); race packed rectangle and transitions pause asymmetry remain their owners. Next worker physics.

Physics worker `/root/behavior_contracts/physics` active: independent trace of all50 assigned regions, source byte/word arithmetic, wheel/suspension/pose and motion-crosscall effects; exclusive editor. No behavior implementation.

Physics worker **complete**: `/root/behavior_contracts/physics`. All50 primary
regions traced with cross-call source review; `port/src/trucks/motion.trace.md`
records branches, ranges and independently fixed expectations. `motion.rs` adds
shared Truck/Wheel/Upgrades/MotionTables native interfaces and36 substantive
contracts; no behavior implementation. Rust1.90 locked compile/list pass (110 suite
tests); physics0 pass/36 failures all at explicit pending stubs/0 ignored;
assets2+game2 infrastructure pass. Exact evidence checks/contracts/physics-results.json.
Source corrections: half-speed helper reverses sign; suspension is byte-wide;
shadow's sampled pair sums wrap before widening; normal average uses integrated
heights. Proposal rejection retains terrain samples/history/working heading;
full block skips average and orders recovery before effect; single-axis reverses
that effect/recovery order. Objects must compose special-surface animation state;
rendering retains screen priority/OAM transfer. Collision/AI/race reuse native
Truck state and preserve sequential commit/cooldown order. Existing uncertainty
IDs retained, no new unsupported semantic assumption. Next worker: collision.

Collision worker `/root/behavior_contracts/collision` active: independently tracing all thirteen contact regions, committed-versus-working coordinates, width/flag-sensitive response cases and terrain-tested pair separation. Exclusive editor.

Collision worker **complete**: `/root/behavior_contracts/collision`. All13 assigned regions independently traced in `port/src/trucks/contact.trace.md`;24 substantive contracts in contact.rs. Rust1.90 locked compile/list pass134 total; area0pass/24 expected unimplemented failures/0ignored; all4 infrastructure pass. Evidence `checks/contracts/collision-results.json`; initial integer fixture compile errors preserved in adjacent note and fixed without expectation change. No behavior implementation/QA. Corrected negated speed exchange, committed-vs-working midpoint/distance, word-wrapped midpoint, unsigned negative-speed impact branch, descending scan/cooldown continuation and both probe-history effects. Conditional current-deflection natural entry unresolved in uncertainty record. Next worker ai: shared signed-vector helper must satisfy fixed contact bearings.

AI worker `/root/behavior_contracts/ai` active: independently tracing full AI route/steering/rank/difficulty code and shared quantizer; sole editor. Source-specific tests pending, no implementation.

AI worker **complete**: `/root/behavior_contracts/ai`. `port/src/trucks/ai.rs` +
`ai.trace.md` independently cover38 assigned regions/crosscalls, shared quantizer,
steering, all route controls/timeouts/laps, ranking and setup/random difficulty.
37 substantive native contracts; Rust1.90 locked compile/list pass (171 total),
area0 pass/37 exact pending-unimplemented failures/0 ignored; all4 infrastructure
tests pass. Evidence `checks/contracts/ai-results.json`; no behavior/QA/toolkit/
reference edits. Literal bearings corrected from actual ROM during fixture review,
not learned from port output. All-human setup tail not overwritten: original route
windows end before offsetF7, unused tail effect explicitly recorded and uncertainty
preserved. Cross-effects: timeout tests cached rectangle; FD zero redirects; rank
leader lap4 threshold index is lap4; RNG draw order retained; AI runs after HUD.
Next worker race; reconcile RaceAi banner/winner fields with single race state owner.

Race worker `/root/behavior_contracts/race` active: independently tracing race preparation, dispatcher, human routes, BCD timer/banner and progression. Sole editor; no implementation.

Race worker **complete**: `/root/behavior_contracts/race`. All42 assigned regions
and cross-calls independently traced in `port/src/race.trace.md`;38 substantive
contracts in race.rs. Rust1.90 locked compile/list pass209 suite; area0pass/38
explicit pending-unimplemented failures/0ignored, all4infra pass. Exact evidence
`checks/contracts/race-results.json`; initial36-test result preserved race-initial-*.
Reconciled shared Progress authority/RaceAi alias, source word banner width and
canonical packed rectangles. Corrected earlier AI edge fixture/trace retaining
superseded interpretation and original logs; affected test rerun fails pending
as expected. No behavior implementation/QA/toolkit/reference edits.
Source corrections: timer has no prepared/winner guard; postrace entry extra
HUD call increments it. Winner freezes banner decrement only, secondary still
animates. Grid headings low-byte only. Music gate DP06 active-human alias.
Handoffs: objects supplies actual state/service composition and retains terrain
marker/probe effects; rendering actual staged OAM/HUD glyphs; transitions source
ready/countdown/fade/pause waits; results extra clock update then order after
normal return. New uncertainties preserve originals; malformed extension policy
is explicit native validation, not original memory behavior. Next worker objects.

Objects worker `/root/behavior_contracts/objects` active: independently tracing all placement/pickup/dynamic/contact/composite source and cross-owner effects; exclusive editor, no behavior implementation.

Objects worker **complete**: `/root/behavior_contracts/objects`. All79 assigned
regions/crosscalls independently reviewed in port/src/world/objects.trace.md;40
substantive native contracts plus1 tracks object-extension contract. Rust1.90 locked
compile/list250 suite pass;40+1 expected pending-unimplemented failures/0ignored;
all4 infrastructure tests pass. checks/contracts/objects-results.json records exact
commands/logs; superseded initial logs retained after fixed RNG fixture correction.
No implementation/QA/toolkit/reference edits or ledger disposition changes.
Actual WorldObjects service interfaces cover awards, speed, hazard/placement state
and ordering; TrackContent optional original objects and explicit added definitions
reconciled. Key source corrections old-count awards/immediate lifetime tick, packed
XY borrow, dynamic eligible0/12 only, idle miss skips timer, track0 inclusive fifth
alias count4→3. Conditional unsupported tail/state11 uncertainties recorded.
Next worker rendering: preserve object staging writes/untouched OAM, tile/cursor
source semantics and special track3 priorities, generic sixteen cursor updates;
compose actual world owner in race/Game during implementation, no phase-only mock.

Rendering worker `/root/behavior_contracts/rendering` active: independently tracing all101 primary ranges, transfer/format consumers, OAM/text/HUD/animation and read-only composition; sole editor, no implementation.

Rendering worker **complete**: `/root/behavior_contracts/rendering`. Presentation
mod.rs/memory.rs/sprites.rs pending native interfaces +tests.rs53 substantive
contracts; detailed rendering.trace.md covers all101 assigned ranges and source
cross-consumers. Rust1.90 locked compile/list304 suite pass;53 rendering +1 tracks
required-presentation expected unimplemented failures/0ignored, all4 infrastructure
pass. Exact commands/logs checks/contracts/rendering-results.json; initial compile
log retained. No behavior implementation/QA/toolkit/reference edits. Extension
TrackContent.presentation optional-original/required-added validated title/HUD/
overlay definitions. Source-correct DMA tail is fully overwritten before display
(superseded hypothesis retained), complete512-byte palette, COLDATA no-component
preservation, native Mode1 composition and staged/visible OAM boundaries traced.
Actual Game/Race composition/snapshot ownership remains implementation work.
Next worker transitions: consume real Display window edges/brightness, preserve
source timing/double buffering; inspect generic46260..4629A near shape selector
without inventing a consumer; all unresolved identities retained in uncertainties.

Transitions worker `/root/behavior_contracts/transitions` active: independently tracing all fade/window/countdown/pause routines and data, exclusive editor; no behavior implementation.

Transitions worker **complete**: `/root/behavior_contracts/transitions`. All96
assigned regions/crosscalls independently traced in presentation/transitions.trace.md;
52 executable native contracts in transitions/tests.rs. Rust1.90 locked compile/list
pass356 suite; area0pass/52 explicit pending-unimplemented failures/0ignored;
assets2+game2 infrastructure pass. `checks/contracts/transitions-results.json`.
Initial49 logs/results preserved; corrected retained mask vs active display and
unused buffer-tail fixtures, then added3 actual frontend/pause setup/capacity tests.
No behavior/QA/toolkit/reference edits. Added TrackPresentation.countdown layout
for arbitrary named tracks; original table16 remains only original decode bound.
Source corrections: frontend shrink/16 vs race/4; abort skips cleanup; displayed
buffer opposite newbuild, clear256rows only; initial countdown7 vs reload6 draws;
ready word human gate and paused countdown duringgrowth; pause owner4 triggerJOY3
but actual waitsJOY4. Attract handoff: normalize selected wallet at brightness8→15,
then all4 Start/signature-first-byte timer poll, wrapper polls once after expansion.
Next worker attract. All implementations remain pending.

Attract worker `/root/behavior_contracts/attract` active: independently tracing cold/warm signature, splashes/polls, attract assets/order and Start reentry; sole editor, no implementation.

Attract worker **complete**: `/root/behavior_contracts/attract`.32 substantive contracts, locked Rust1.90 compile/list388 suite pass;32 exact expected pending-unimplemented failures/0ignored,4infra pass. `checks/contracts/attract-results.json`; `port/src/frontend/attract.trace.md` all23 assigned ranges/crosscalls. Real Session/record/wallet/Truck/Display/maps/window/splash/scroll contracts, no behavior implementation/QA. Fullsignature-vs-firstbyte poll, cold record copy AFTER first splash, warm money+stock reset, inclusive waits, real partial palette/VRAM copies and highscore preincrement derived independently. Shared SessionRecords/wallet proposal must reconcile upcoming names/economy/results owners; no second race owner. Next worker names, then economy per inventory.

Names worker `/root/behavior_contracts/names` active: independent name/join/countdown/editor and driver-panel data/call tracing; exclusive editor, behavior remains pending.

Names worker **complete**: `/root/behavior_contracts/names`.36 substantive contracts, locked Rust1.90 compile/list424 suite pass;36 exact pending-unimplemented failures/0ignored,4infra pass. `checks/contracts/names-results.json`; detailed `port/src/frontend/names.trace.md` all57 assigned source ranges and crosscall consumers. Actual Session/Selection/VideoMemory/SpriteArrays contracts and shared DriverAssets consumers; no implementation/QA. Source4th preview vs3byte commit, repeat9, shared retained Select/B history, expiry-before-join, page-loop M8 finalACK and full alternate/continued driver palettes traced. Initial synthetic match-arm type mismatch fixed before recorded compile, no expectation change. Next economy worker must reuse Session wallets/records and retained menu history/repeat; driver panels shared names installer, no duplicate player count menu.

Economy worker `/root/behavior_contracts/economy` active: independently tracing all72 shop/wallet ranges, byte decimal carry/borrow and shared session/display inputs; sole editor. No implementation.

Economy worker **complete**: `/root/behavior_contracts/economy`. All72 primary
ranges and crosscall consumers independently traced in frontend/shop.trace.md;
43 substantive native contracts in frontend/shop/tests.rs, pending real domain
interfaces in shop.rs. Rust1.90 locked compile/list467 suite pass; area0pass/43
failures all verified exact economy-unimplemented panics/0ignored; assets2+game2
infrastructure pass. Exact commands/logs checks/contracts/economy-results.json.
Superseded economy-initial-* logs/results preserved after source-symbol crosscheck
corrected intuitive item order to nitro/acceleration/steering/speed/suspension and
award digit3 to displayed$10000 units. No expectation learned from implementation.
Source quirks: clamped grid, repeat9, raw debitdigit3, CMPnegative threshold8A,
highest borrow adjacent domain byte, inactive decrement-before-cap and leftcompact
money display. Source finer-award entries/highest-borrow natural reachability
remain explicit uncertainties. No behavior implementation/QA/toolkit/reference
edits or source-ledger dispositions. Sole editing finished. Next worker results:
reuse SessionRecords/wallets/Selection histories, settle SINGLE WorldObjects
pending-award array once, apply actual shop/display/transition service.

Results worker `/root/behavior_contracts/results` active: independently tracing all62 primary ranges plus record defaults, podium/loss resources and high-score consumers; exclusive editor, no implementation.

Results worker **complete**: `/root/behavior_contracts/results`. All62 assigned
ranges independently reviewed with shared defaults/score/render/transition calls;
detailed `port/src/frontend/results.trace.md`,42 substantive native contracts in
results/tests.rs and explicit pending interfaces. Locked Rust1.90 compile/list509
suite pass;42 results failures all exact pending-unimplemented,0ignored; all4
infrastructure pass. Evidence `checks/contracts/results-results.json`. Initial
synthetic unused helper removed before recorded run; no expectation weakening.
Source corrections: record>250 bypasses input and250 extraNMI commit; INC flag;
ascending indirect highscores/ties; penalty resets same losing-wallet startercash;
heldB/leftpriority/loser controller and gameover301highbyte waits. Shared Session
record/wallet+single WorldObjects pending award owner retained; AddedRecords only
added named slots. Full postrace contracts require actual state/VRAM/Display and
source-ordered composition, not phase logs. No implementation/QA/toolkit/reference
edits. Exact next worker audio.

Audio worker `/root/behavior_contracts/audio` active: independently tracing host upload, song/sequence/voice helpers and immutable audio resources; sole editor, no implementation.


Audio worker **complete**: `/root/behavior_contracts/audio`. All717 assigned
source ranges/cross-consumers indexed in `port/src/sound/audio.trace.md` (~1100
lines including717-row appendix). Five cohesive modules contain101 substantive
contracts for definitions, native sequence/voice/host and synthesis/snapshots.
Rust1.90 locked compile/list610 suite pass; all101 area failures verified exact
`not implemented: audio ...`,0 ignored; all4 infrastructure tests pass. Evidence
`checks/contracts/audio-results.json`. Initial compile/run logs and later
live-envelope fixture correction logs/results retained, not replaced silently.
No behavior implementation, original/device/emulator QA, toolkit edit or reference
change. Original unknown070149/0704FF IDs/ranges retained with consumer assessment;
new song0-config/timing/native-DSP/conditional-resource assumptions explicit.

Audio corrections: B6/B8 vector target order differs nearby labels; positive fade
BMI differs generic clamp prose; rise0 returns unchanged, rise1 uses documented
DIV0 FF, fall/release subtract extra1; duration initialization aliases first3
current envelopes to1.141-byte config stays live across songs and overlaps20
complete rows plus row21 timer, not immutable defaults. Source song0 configFF
low-ARAM reads remain silent-start policy/residual uncertainty, not a fabricated
RAM image. New note KON commits following successful driver update, KOFF current;
master fade precedes slow divider. Both uploads, split sample20 and all33 directory
entries/16 songs/34 commands/effect arbitration represented by actual native domain
interfaces, not raw preservation or phase-only mocks.

Stage3 coordinator next: reconcile combined610 contracts, preserve expected
behavior failures, compile/list combined suite and run required infrastructure as
needed, then hand off Stage4 only when every area is traced/tested. Stage4 must
own one complete Sound in shared Game, feed Session.audio_commands, implement
native timing/sequence/mixer/resampler and full atomic snapshots/semantic debug.
Native toolkit Mixer currently Clone but lacks public complete serialization;
any needed reusable toolkit change follows its AGENTS and library-change logging.
No further audio worker edits are pending; exclusive editor released.

## Stage 3 final coordinator handoff

All sixteen workers completed sequentially, with source-linked executable contracts and
adjacent detailed traces. The current source ownership index preserves every original
1,527 region ID/range and 1,408 unit IDs, source SHA unchanged, all ledger dispositions
pending. No additional independent area was discovered; cross-area consumers and
conditional helper questions are recorded within the existing coherent owners.

Authoritative combined map: `checks/contracts/stage3-handoff.json` lists worker identity,
trace, per-area execution manifest, every current native test and source-review count.
The pinned Rust 1.90.0 locked combined test suite compiles and discovers **610 tests**:
606 behavior contracts and four infrastructure tests. Combined compile/list and
assets2/Game2 infrastructure all passed; exact commands/logs in
`checks/contracts/combined-results.json`. Area executions retain expected unimplemented/
missing-observation failures; runtime's existing source-metadata fact test passes.
No ignored tests, no game implementation, no implementation audit, no gameplay QA.
Superseded source interpretations, fixtures and execution records remain preserved.

Cross-area reconciliation is source-backed: race Progress sole owner/RaceAi alias;
shared vector quantizer; AI checkpoint rectangle correction; Session/Selection shared
records/wallet/names/history; single WorldObjects pending awards; TrackContent custom
objects/presentation/countdown without a global fixed count; native Display staging
and Sound ownership. Full snapshots/debug integration is still an implementation
requirement, not proven by pending contracts or skeleton tests. Source ambiguities
remain documented in uncertainties.json and adjacent traces, not promoted to facts.

Exact resumption: coordinator launches Stage 4 in its own subagent per prompts/start.md.
Implement every inventoried behavior using original source plus tests/traces. Wire
actual stateful Game orchestration (all scenes, physics/world objects, presentation,
sound, controls, records/extensions) and full snapshots/debug observations; pending
stubs or lists of effects are not completion. Reconcile native Sound mixer snapshot
serialization in the toolkit if needed under its own instructions. Then complete
ledger/uncertainty accounting, docs and bounded maintainability pass; require
python3 run.py check and audit before Stage 5. QA remains not started. No concrete
Stage 3 blockers and no toolkit/reference edits, check-in or publication occurred.

## Stage 4 live implementation

Exclusive editor `/root/implementation`. Implementing native domain rules from original source and adjacent contracts, starting runtime/controllers/track loaders, then terrain/trucks/race/world, presentation/frontend/audio and complete shared Game orchestration. No gameplay QA, toolkit/reference edits or implementation completion claimed. All source ledger identities remain pending until equivalents are implemented.

Stage4 foundation milestone: controls15/15, tracks19/19, terrain25/25 pass locked Rust1.90 tests. Runtime11/13 pass; remaining2 require real Game runtime/snapshot integration. Implemented actual domain algorithms/loaders, not only stubs. Added serde typed native track files (Cargo.lock updated offline); preserved all Stage3 evidence. Synthetic countdown-field omission corrected with trace note. No QA/audit/reference/toolkit edits; all other systems and whole-game integration remain in progress. Next: native truck powertrain/wheels/motion/contact/AI, then race/world and full frontend/presentation/sound integration.

Stage4 driving milestone: motion36/36, contact24/24 and AI37/37 pass locked Rust1.90. Fleet service now borrows the actual world contact array plus immutable bearings and returns native sprite/audio/contact outcomes, executing real proposal→terrain→surface→blocked response→projection/shadow→pair→commit in source order. AI shares canonical rectangles/Progress and real RNG. Two synthetic distant-record fixtures were corrected from6002 to7002 because source BFE0 masks6002 to origin2000; original assertion intent retained and prior35/2 result preserved. Details in ai.trace.md/motion.trace.md and checks/implementation/domain-results.json. Race helpers in progress; whole shared Game, other systems, accounting and QA remain pending.

Stage4 world milestone: world objects40/40 pass. Actual source loaders/pickup/hazard/surface/placement/composite state, one pending-award array and source-ordered init/service implemented; initializer returns actual sound requests for Game to consume. Source-boundary fixture correction recorded adjacent, initial39/1 retained. Race35/38 pass,3real preparation/frame integration contracts pending. Presentation implementation next; no QA or whole-game completion claimed.

Stage4 objects milestone:40/40 pass; actual pickups/awards/dynamic hazards/placement/composite state and full initialization/service implemented. Native initializer returns source sound requests for integration. Source-derived composite X-boundary fixture repair retained initial39/1 and trace. Presentation memory/sprite/Mode1/HUD implementations written; targeted53-test run now active. Full race preparation/frame owners still pending until these systems integrate.

Presentation53/53 targeted locked tests PASS. Actual native memory, text, rectangle, asset DMA, sprite/pose staging, HUD/title and pure Mode1 composition implemented using toolkit primitives. Race integration now enriching resources and mutable video/world owners. Newly verified alias: truck_effect_cooldowns0525 is generic animation cursors0519+12; spawn writes24, establishing natural contact-animation suffix24..30 access contrary to prior trace uncertainty. Frame adapter must map these single Truck-owned fields, not decrement an independent cooldown or copy a second owner. Source correction record to follow.

Race38/38 PASS with actual immutable resources/mutable video/world owners, source preparation and ordered frame services. Strengthened assertions prove shared effect cursor24→25 and entry24 OAM tile after one advancement, real initialization VRAM/world state, pending fade continuation. Retained hidden sprite views seeded after OAM reset; surface coordinates only staged on actual source writes. Transitions52/52 PASS on first targeted run: source ROM shapes/layouts/countdown/pause maps, pingpong geometry, fades, frontend windows, countdown/ready/postrace/pause resumable scheduling. Actual Game continuation/banner/animation/audio consumption remains pending; frontend/audio and whole-game integration next. Alias correction retained original summary as superseded in uncertainties and rendering trace. No QA/audit/full completion claimed.

Attract32/32 PASS first targeted run: actual ROM spans/maps/partial palettes, full5-byte warm signature versus first-byte Start poll, cold record/default boundary, session reset/rotation/join completion, resumable splash fades/highscore scroll/panel. Shared six-digit normalization implements source CMP10 sign branch and adjacent owner borrow; panel uses Session directly, overlay shares helper/poll. Names selection/editor/resources and resumable join/clear/fade/window continuations now implemented; targeted run active. Full shared Game and other frontend/audio/accounting remain pending.

Names36/36 PASS: all35 cells/livepreview/coupled Select+B history/repeat/paired-name commit/countdown/page/palette variants, actual ROM driver/map/font staging and pending join/clear/input/fade/window continuation state. First31/5 failures corrected in countdown draw handling; valid contract assertions unchanged, fixture added fields only. Shop implementation next; full Game/audio/ledger/docs/check/audit/QA pending.

Shop43/43 PASS final targeted: actual wallet arithmetic/adjacent borrow/inactive conversion/purchase/cap/source clamped cursor/UI/bar/real installed page/fade/first-done sampled-input wait/window/audio sequence. Source-backed fixture fixes preserve initial41/2 and strict palette32colors/three bar-pair intent. Results actual state/assets/records/loss code next; full Game/audio/snapshots/source ledger/check/audit pending.

Results42/42 PASS: actual ROM assets/podium/wallet/pickup settlement, strict records/extra NMI blinkcommit/indirect ascending score list/loss/penalty/continue/gameover/postrace owner mutations. Added actual Truck HUD input and retained realRoute for source ranking; full Game drives transition/input owners. Source zero-clear/TMW fixture correction preserves initial40/2; shared names continued-map zero-fill narrow contract1/1 PASS, prior36 history retained. Audio immutable definitions/loaders implementation started; all sound driver/synthesis and shared Game/accounting/check/audit/QA remain pending.

Audio foundations PASS definitions15/15,voice28/28,sequencer29/29,host16/16 targeted. Actual upload/data/table/BRR consumers,34 sequencecommands+call/loop,allvoice/pitch/envelope/stage/auxiliary arithmetic,host arbitration/fade/heartbeatdelayedupdate. Initialdefinitions14/1 andvoice27/1 source-backed fixture corrections retained in audio.trace. Ordered host-generated DSP writes queue in same Driver, pending sole Sound consumption. Full synthesis/snapshots and shared Game still pending. Reusable toolkit mixer snapshot serialization needed; applicable toolkit instructions read, unrelated sixexistingchanges untouched, edits uncommitted/librarylog forthcoming.

Audio101/101 targeted PASS including synthesis13/13 first run; full native PCM/resampler/driver/scheduler/atomic sound snapshot (resourceSHA256 bound). Reusable toolkit Mixer snapshots implement validated version1 complete predictor/source/envelope/noise/echo state. Focused1/1 and required desktop-feature releaseClippy/full80tests PASS,0failures; library-changes.md records exactfile/baseline/API/checks/limits, editsuncommitted. Whole shared Game integration now next; domain success does not mark completion or ledgeraccounting.

Integrated Game5/5 PASS: shared native scene scheduler/render/audio/tick_players/reset/snapshots/debug; actual continuations/owners/effects, original64directional sequence or custom namedorder, enabledfetched file manifests. Focused contracts prove pendingjoin originalsample retention, activePCM+frame+semantic fork/restore, malformed nested restore atomicity and actualadded17slotcontent selected/prepared throughGameAPI with assetbinding. Originalcheckpoint1000ms timeout preserved; sourceequivalent rendererfastpaths fixedinitial1/2, priorGame4/1 incompleteaddedroute exposed count+1fullrecord loaderhole andsource-backed fixture correction. Expandedvalidatedversion2 snapshots include world/video/sound/input/scheduler/events andsourcecontinuations. Runtimepending2 andaffectedtracks checks next; newly verified windowinversion03vs22 adds sameDisplay activeinvertmask beforefinalchecks. Source accounting/docs/boundedcleanup/check/audit stillpending; QA notstarted.

Runtime13/13 PASS including previouslypending2 actualGame counter/reset/RNG/pendingtransition atomicreplay contracts. Windowinverse03/22 singleDisplayfield andfixedpixelcontract implemented; affectedtracks/presentation/race checks runningnext. Game currentnativefivefocusedcontracts passed beforewindowchange. No QA initiated.

Affected regression milestone: tracks19/19, presentation106/106 (rendering53 + transitions53), race38/38, results42/42 and full Game6/6 PASS,0ignored. Loss source ordering now retains penalty before first NMI, uploads on next NMI, releases prompt on following NMI; new integration contract also replays pending fade/frame/PCM from snapshot. Initial tracks17/2 incomplete expectation correction preserved in domain-results. Total615 tests discovered. Bounded maintainability, source accounting and final docs/check/audit now active; no QA launched.


Stage4 final handoff: full source-backed native runtime/controllers/tracks/terrain/physics/contact/AI/race/objects/presentation/windows/frontend/results/audio are implemented under shared Game APIs. Actual scheduler owns source NMI continuations, captured input and single-consumption sound/award effects; one owner per Session/Progress/WorldObjects/Display/Sound. Full version2 snapshots include every mutable owner and versioned native Mixer/Sound/resampler state; malformed restore is transactional and effective resources are bound. Original64 directional sequence and arbitrary named custom catalogue/order use real MemoryAssets loading. Original cartridge records remain session-only, with no battery hook invented.

Final required gate: Rust1.90.0/Cargo.lock; format, desktop release Clippy -Dwarnings,615native tests, desktop release build, Wasm release Clippy -Dwarnings and locked Wasm release build all PASS. `python3 run.py audit` PASS:1527regions,1408units,22uncertainties; accounting_complete=true, semantic_correctness_assessed=false, release_ready=false. First Clippy failure, successful pre-final gate and first audit missing semantic-row span metadata remain preserved beside final records. No ignored/disabled contracts. Toolkit focused snapshot1test, required full80tests and desktop Clippy previously PASS; library-changes.md records exact one-file edit and preserved six-file baseline.

Source ledger classifications/IDs/ranges and frozen references remain intact. Unknown gaps have bounded consumer-specific no-impact assessments; unclassified data retention does not assert native behavior. Source-backed fixture corrections preserve initial failure evidence; native assumptions (cold RNG/history/fixed color, conditional pause I/O owner, unsupported terrain boundaries, song0/native DSP clock) are explicit in uncertainties.json/differences.md. Bounded maintainability pass formatted Rust, clarified source constants/owners and corrected stale current docs. README/BUILDING/debug API/current architecture/behavior links and working original-only manifests are complete.

Coordinator resumption: read checks/implementation/stage4-handoff.json and qa-entry-recipes.md, then invoke Stage5 with debug/goal.md. Recreate checkpoints from deterministic recipes on this exact source/assets build; do not reuse skeleton checkpoints or infer expected original pixels/audio from native output. Preserve existing QA/superseded records and run only the authorized short smoke plus initial exact-frame suite. Implementation worker stops here and releases exclusive edit ownership. No QA agent launched by this worker.

## Stage5 live QA

Exclusive worker `/root/qa`; bounded two-check plan ready in debug/plan.json, one pass active. Native short startup/input smoke and separate initial original-fidelity rendering/scroll/animation/audio sanity only. No prior captures or checkpoint evidence exists. Preserve implementation accounting and stable check IDs during repairs.

Stage5 current evidence: native smoke harness machine assertions pass in debug/artifacts/smoke-run-001 (natural startup/name/shop/Racing1354→1400, bounded throttle/left/nitro and active PCM; selected capture review pending record). Separate sanity-run-001 passed exact full RGB license/software after visible-line origin repair; highscore0132 failed5612pixels. Source BG3SC4A requires32x64 vertical map; native hardcoded32x32, size flags/second-page contract now added. Source original snapshots through natural name/shop NextRace atframe1415 retained for initialrace counterpart. Source audio comparison remains pending. Build and reference probe SIGTERM143 attempts retained; fresh affected build/test and snapshot-resumed original comparison active. No full pass restart or broad traversal.

Stage5 repair milestone: four full frontend pairs (license/software/highscore0132/0142) exact0 mismatches in sanity-run-005; initial grid exact0 in sanity-run-006 after global deterministic zero cold-RNG policy and independently source-validated prepared pickup parity. Countdown13/4 still failed21 pixels, localized banner tile. Source $077D word is shared countdown/banner state, while delay is separate $077F word; scheduler now synchronizes the continuation view with canonical Progress before/after draw/banner service and snapshots reject divergence. Focused source-order contract1/1 PASS (620 total discovered); release rebuild active. Preserve all earlier71/189pickup, window-origin and21banner failures; no masks or lowered exact criteria. Original/native audio-alignment002 retains real0-drop WAVs and exact8voice sequence/duration/DSPpitch/sample/gain at both30frame endpoints; final bounded review pending.

Stage5 bounded suite resolved/reviewed: smoke-run-003 natural route plus46initial-race ticks PASS; sanity-run-007 six selected full256x224 RGB pairs exact0, actual original/native30frame audio endpoint sequence/duration/DSPpitch/sample/gain exact all8voices,0drops. Signal review documents interpolation waveform/amplitude limit without claiming physical audition. Fresh current-build Ready13/4 checkpoint export/import preserves semantic/frame/audio hashes. Four unaffected frontend pairs retained from sanity-run-005 with explicit producer impact/SHA review; every earlier failed attempt remains. debug/qa.json records exactly one reviewed pass, separate sanity-results enabled/reason retained. Final run.py check all6gates/620tests PASS, currentbuildSHA c5a985da6cb5214f7ff11d4321cdd535d75c9e544f7549800ca58926e85fea2b unchanged; toolkit QAaudit onepass/twochecks PASS. Required implementation audit active; no further traversal/tests planned beyond its mandated built-in check.

Stage5 final required gates complete: `python3 run.py audit` PASS all six checks/620tests,1527regions/1408units/22uncertainties; accounting_complete=true, semantic_correctness_assessed=false, release_ready=false. Toolkit `port_debug.py audit` PASS onepass/twochecks. Exact final records stage5-final-check.*, stage5-final-audit.*, stage5-final-qa-audit.json and stage5-handoff.json. Worker stops and releases exclusive editing for coordinator completion audit; no further QA/traversal, publication or check-in.


Coordinator completion audit: PASS. Verified all74 current source identity hashes and829 frozen reference/review files, unchanged1527 original region IDs/ranges/classifications and1408 unit IDs, all16 independent contract-worker traces/handoffs, complete22 uncertainty assessments, final620 passing tests/six gates, one reviewed two-check QA pass/six exact frames and current checkpoint semantic/frame/audio replay. QA binary hash and plan hash match current records. Both final administrative audits pass; implementation/QA handoffs and BUILDING/current docs are present. Historical Stage4 ledger completion notes and615-test handoff remain preserved as historical accounting, with final620-test QA evidence separate. No required work remains in the fixed pipeline; no further traversal, modernization, check-in or publication follows. Device/runtime and native-audio limits remain in debug/report.md and differences.md.


## Follow-up: randomized startup and reported race-finish lockup

The prior global zero comparison seed was a gameplay defect: its feedback remains zero. Player-facing loads now use getrandom OS/browser entropy for all five bytes, rejecting a seed that immediately collapses to zero. Each loaded game retains its initial random history for deterministic reset/replay; full snapshots retain both initial and advancing history. Explicit seeded constructors and headless-only --rng-seed support reproducible diagnostics, with checkpoint identity bound to the seed. Browser/desktop use the same Game rules. Historical zero-reference captures are reproducible only with the explicit zero seed.

Finish investigation: added tests exercise a natural computer finish, the final human checkpoint, human/AI winner podium and automatic award timeout, the losing-human continue path, normal shop input and the next race, with audio and visible podium checks. These paths currently pass without a fault or stall; no race-finish code fix is claimed. A concise user question requests the stop location to reproduce the remaining report. No additional game traversal or publication is authorized.

Follow-up check complete: Rust1.90.0/Cargo.lock, all624 tests/0failures/0ignored, desktop and Wasm Clippy/build plus formatting PASS (checks/follow-up/check.json). Native CLI startup sampled distinct nonzero histories across independent loads, reset preserved each initial history and explicit seeded replay matched. Browser entropy backend compiled successfully; no desktop window/browser-device runtime launched. Race-finish tests pass the tested human/AI/podium/continue/shop/next-race paths; the user-reported lockup is NOT marked fixed or dismissed, pending the observed stop screen or a reproducible saved state.
