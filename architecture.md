# Native port architecture

Stage4 implements the complete native game. Historical planning follows the current description; it is preserved as a record, not the current runtime specification.

`game::Port` implements the common ROMAtlas `Game` API for desktop, browser and headless tools. Immutable `Arc<Resources>` binds checked ROM/file data, named track catalogue, original directional order, graphics and audio definitions. Clones share only immutable data. `game/state.rs` contains owned simulation and continuation state; `game/scheduler.rs` sequences source NMI boundaries and calls actual domain owners. Rendering and observation are read-only.

| Authority | Responsibilities |
| --- | --- |
| Session | Human participation, joining, wallets, names, retained frontend state and original session records |
| RaceState / Progress | Four trucks, canonical animation frame, route progress, lap/winner/time, native video staging and sound requests |
| WorldObjects | Pickups, pending awards, hazards, placements, composite records and contact animations |
| Display / VideoMemory | Layers, active window enable/inversion, scanlines, palettes, VRAM and staged/visible OAM |
| Sound | Host commands, sequence/voice state, ordered DSP writes, native mixer, PCM clock and resampler |
| Scheduler continuation | Splashes, names/shop waits, fades, window/countdown/pause, postrace/records/loss/continue and sampled input |

NMI first transfers eligible staged video and automatic input, advances counters/audio heartbeat, then resumes the suspended source operation. Video inhibition retains staged state. Source sound queues drain once into Sound; ready and race service calls operate on the same world/truck/video owners. Frontend entry suspends boot's subsequent reset/track selection until it returns. Loss penalty, loss upload and prompt release occupy separate NMI boundaries. Source aliases are represented by one owner: truck effect cursors are the suffix of generic animation cursors; the adapter advances them once, and menu/pause retained history transfers explicitly.

`game/snapshots.rs` exports version2 resource-bound snapshots. Restore parses and validates a complete candidate (including permutations, continuation presence, nested capacities, routes, masks and audio), rebuilds immutable selected content and terrain, then atomically replaces live state. Sound snapshots include all native sequence, voice, mixer and resampler state. Toolkit Mixer snapshot version1 covers BRR predictors, envelope, phase, key/noise/echo state. No CPU instruction interpreter or original RAM mirror is used. See differences.md for deliberate policies and remaining hardware validation limits.

Headless semantic observations expose scene, counters, canonical actors/progression, timers, window/menu/pause state, world, sound and bounded event journal. Debug transport adds deterministic frame/audio hashes and checkpoint recipes described in debug/api.md. Original records are session state; this cartridge has no battery save.

## Preserved Stage1/2 architecture record

# Native port architecture

Stage 2 behavior planning complete; stage 1 infrastructure checks passed with Rust 1.90.0
and Cargo.lock. Preserved results: checks/skeleton.json and checks/skeleton.log. No source
gameplay, frontend sequence, renderer or audio interpretation is implemented.
The original ledger and uncertainty classifications remain pending.

## Modules and ownership

`port/src/lib.rs` exposes the same `config`, `assets`, `identifiers` and `game`
modules to native tests, desktop and browser. `main.rs` contains native CLI/debug
and desktop entrypoints; the Wasm `web` wrapper stays in `lib.rs`. Both frontends
use `Port::load_assets`, Game lifecycle/snapshot hooks and toolkit settings.

`game::Port` owns only a simulation tick counter at this stage and an immutable
`Arc<Assets>`. Clone forks the mutable counter by value. No mutable resource,
callback, audio voice or event journal is shared. Future gameplay owners belong
in cohesive domain modules, not additional address-indexed fields on Port:

| Proposed owner | Source responsibilities and entry points |
| --- | --- |
| frontend | attract/frontend dispatcher, player joining, name entry, speed shop, records, continue/game over |
| race | initialize_race, ready/countdown, lap/winner resolution, track-order progression, pause |
| trucks | four native truck records, input/AI, steering/acceleration/nitro/upgrades, collisions and terrain response |
| world | named track slots, terrain/route decoding, object placement and animation |
| presentation | source background/OAM/font/palette loading, HUD, windows/fades and composition |
| sound | host APU commands, uploaded SPC sequence interpretation, sample/voice/envelope state |

The next planning stage refines these owners and their shared interfaces from
source. This table proposes boundaries; it does not establish traced semantics.
All stateful behavior must be included in clone, full snapshots and semantic
observations when added. Rendering and observation remain read-only.

## Identifiers and resource definitions

The frozen source is Round29 `reference/source/game.asm`, with physical storage
`org` and execution aliases `base`; SHA identities are in workspace.json.
`port/source-identifiers.json` inventories every named ledger unit, preserving
its original ID, name, physical range, source reference and source description.
This covers routines, states/tables, graphics, tracks, music/effects and samples
without inventing names for unresolved content. It is a navigation artifact,
not detailed source tracing or proof of behavior.

`identifiers.rs` declares four TruckId variants, Scene boundaries, Upgrade choices
and semantic event categories. Scene::Pending alone describes the current
skeleton. License/title/menu/race scene boundaries are provisional until tracing.
Track slots, graphics and sound resources use descriptive string IDs, allowing
file-backed additions without a fixed level-count restriction.

`assets.rs` owns the `resources.json` manifest. `resource_definitions.rs` contains
one named raw recipe per Data region (1,055), grouped in source physical order;
its comments retain ledger region IDs. Recipe names come from the region's first
named source unit; unit subdivisions remain indexed in source-identifiers.json.
Unresolved data regions use explicit unresolved IDs. No Code, Padding or Unknown
range is declared as an implemented native consumer. Loading these raw bytes does
not classify or implement any behavior. Source-specific decompression, palette,
map, route, animation and audio interpretation are deferred to contracts and
implementation. That stage should introduce semantic collections/validators for
added tracks and assets while retaining supported original slots.

ROM identity is checked both at the Rom boundary and game asset boundary. All
recipe reads use checked physical offsets. Existing raw replacements require the
exact original storage length; added named slots require nonempty bounded bytes.
Toolkit collection validation owns ID/path/duplicate/order/size checks. This
structural validation is deliberately not a claim of semantic file validity.
Desktop loads only resources.json; fetched MemoryAssets uses the same collection.

## Scheduling, platform helpers and persistence

`config.rs` owns ROM identity, 256x224 output, SNES_NTSC TickRate, 4:3 aspect and
shared default settings. Host input/audio/presentation scheduling uses toolkit
Game frontends. The skeleton counts tick calls without interpreting input and
renders black without changing state. Default audio is silence, explicitly
pending. Toolkit F1–F6 lifecycle/settings controls remain available.

SNES native helper documentation (`/root/romatlas/docs/snes-native.md`) establishes
byte-addressed PPU helpers and a cloneable 32-kHz BRR/voice mixer. Helpers are
mechanisms, not a complete PPU/SPC or this game's rules. Later source tracing must
honor 65816 widths/banks/flags and SPC execution/upload lifetimes. In particular,
upload metadata is not sample payload and CPU aliases are not physical offsets.

The LoROM header at physical $7FD8 declares SRAM size zero. Original battery
save hooks retain defaults. Version 1 full native skeleton snapshots include
ROM identity and simulation ticks; restore validates before mutation. Immutable
assets remain attached to the loaded game; frontend/driver snapshot envelopes
handle driver state separately. Full snapshot formats must expand to include
all mutable game/audio state as those systems are implemented.

Debug schema and replay/checkpoint contracts: debug/api.md. Stage 1 tests exercise
matching-ROM bounds/identity, all raw recipes, override extension validation,
atomic full snapshots, clone isolation and actual debug checkpoint/branch replay.
No gameplay correctness claim follows from infrastructure checks.

## Stage 2 source-informed domain design

Behavior planning now assigns sixteen independent trace/contract workers in
behaviors.md. planning/source-ownership.json covers all 1,527 original regions
and 1,408 units without altering ledger identities/dispositions. The six
skeleton boundaries above expand into these cohesive owners:

| Owner/module | Mutable state and interface |
| --- | --- |
| runtime.rs | Reset/session identity, console tick/animation counters, source RNG; drives deterministic domain phases |
| world/tracks.rs | Immutable decoded named TrackDefinition catalogue; selected slot/order/direction and route/grid preparation |
| world/terrain.rs | Immutable terrain samples/control cells and projection definitions; pure probe/project queries |
| controls.rs | Sampled four controller words and translation; owner-specific edges stay with frontend/race/motion |
| trucks/motion.rs | Four Truck records containing position/speed/fractions/heading, upgrades/nitro history, four wheel states, suspension/pose and recovery counters |
| trucks/contact.rs | Pair resolution operates explicitly on current proposal and other committed trucks; contact cooldown lives in Truck |
| trucks/ai.rs | AI route bias/timeouts/locks and generated controller intents; ranking returns explicit ordered TruckIds |
| race.rs | Prepared/ready/running/finished phase, selected track/order/direction, lap/checkpoint records, winner, BCD time/banner state |
| world/objects.rs | Placements, six dynamic objects, pickup/count/lifetime, composite/special props and contact-effect state |
| presentation/mod.rs | Immutable decoded tiles/font/maps/palette; owned mutable source-equivalent display staging, sprite animation/OAM, HUD and scene layout |
| presentation/transitions.rs | Brightness/fade progress, window center/extent/velocity/shape, scanline mask buffers, countdown/banner delays, pause owner/release state |
| frontend/attract.rs | Splash/signature/attract selection and timed scene state; frontend_start_reentry becomes explicit transition intent |
| frontend/names.rs | Two-player page/join/done masks, grid cursors/repeat/Select histories, temporary and final names |
| frontend/shop.rs | Four six-digit wallets, per-page shop cursor/repeat/confirm/done states; purchase mutates explicit truck stock/upgrades |
| frontend/results.rs | Result awards, record times/names, score entries/order, podium timeout/new-record blink, loss and continue state |
| sound/mod.rs | Host command/ack/heartbeat queue/state, native sequencer voice/stack/loop/envelope/pitch state, native sample clock and cloneable mixer |

Proposed modules are not created as empty cosmetic files in planning. Contract
workers establish minimal domain interfaces before implementation. Immutable ROM
assets may share Arc; every mutable owner, staging buffer, audio decoder/history,
input edge and retained event must fork by value on snapshots. Avoid callbacks
capturing shared mutable journals. Port coordinates owners through typed inputs,
results and transition/audio/award intents; domain rules must not call frontends.
Do not place all rules in game.rs or introduce an emulated CPU/RAM/register map.

### Update and presentation order

The actual source dispatcher at physical $000198 is authoritative: wait/input/
pause → four truck proposals/terrain/contact/commit in source slot order → sixteen
wheels → suspension speed correction → poses → Y sort/OAM pack → sprite animation
→ controller copy/human checkpoint tests → track objects → banner/HUD → AI.
Thus controls/AI writes occur after current movement and feed subsequent movement;
use explicit sampled/generated state rather than silently moving input ahead.
The race initializer runs one prepared service frame before fade/ready behavior.
NMI at physical $001BCA separately uploads staged OAM and pose data if video is
not inhibited, then always advances frame/animation/audio-heartbeat counters.
A detailed contract must settle which staged frame becomes visible; render itself
must be read-only. No frontend refresh-rate branch changes simulation cadence.

Source blocking fades/countdown/pause/frontend waits become resumable native
phase states. Track progression increments then masks the order slot to 63;
its descriptor has reverse bit7 and track bits0..6. All sixteen original track
definitions remain bounded while catalogue/added content uses named extensible
slots. Attract has separate no-human state and sound suppression. Source warm
RAM signature preserves records within the session; header SRAM is zero, so do
not manufacture original battery saves. Reset/full native snapshot behavior must
be defined separately from frontend reentry, and full snapshots expand beyond
the skeleton's tick-only format before gameplay is stateful.

### Numerical and asset contracts to trace

Truck positions, speed and wheel/height arithmetic use source integer words;
name domain wrappers for world coordinates, heights, heading and screen pixels
once each fixed-point scale is independently derived. Heading vector lookup and
terrain projection use SNES signed multiplication behavior; do not assume float
trigonometry or a universal scale. Money uses six unpacked decimal digits while
nitro and race timer are packed BCD; conversions and overflow rules are explicit
interfaces, not one generalized money type. Truck IDs use logical slots0..3;
assembly byte/word indices are confined to loaders/source traces.

BGMODE=$09 selects Mode1 with high BG3 priority. Mode7 arithmetic registers serve
multiplication and do not imply a Mode7 display. Use toolkit SNES planar/palette,
BG/OBJ/composition and BRR/mixer mechanisms; game-owned code supplies source
layout/layer priorities/window masks/sequence timing. VRAM register words become
byte offsets at helper boundaries. Domain loaders resolve banked pointers and
shared physical unions; graphics DMA bus wrapping and font/text terminators need
contracts. Raw manifest structural validity is not semantic decoded validity.

Audio upload reconstructs the two source payload lifetimes rather than treating
physical ROM as ARAM: first physical $070004→ARAM$0400, second $078004→ARAM$8200.
Sample20 crosses upload metadata/storage gaps. Native sequencer definitions keep
song/effect/instrument IDs provisional when musical identity is unknown. Song
configuration copies141 bytes and overlaps initial envelope workspace; model its
observable effect explicitly. Sequence stacks/commands and DSP mixer are domain
algorithms; no SPC700 instruction execution is part of the native game.

### Semantic observations required by contracts

Extend debug/api.md during implementation with scene/phase, selected named track,
original order/direction, joined human slots and frontend cursor/name/shop state;
truck world/screen positions, speed/heading/pose/wheels/upgrades/nitro, route/laps,
AI lock/recovery; pickup/object states; winner/ranking/wallet/record/continue;
brightness/window/countdown/pause owner/timers; source audio command/music/effect/
voice state and native sample clock. Include bounded non-draining event sequence
numbers for transient scene/sound/contact/award effects and clone retention.
The current skeleton schema remains honest until these owners exist. Read-only
observation, render, snapshot restore and replay must preserve all owner state.

No source behavior tests or game logic were added in planning. Source hypotheses,
conditional helpers, unknown/patterned storage and precise carry/timing derivations
remain the independent workers' responsibility. No source behavior is excluded
because runtime evidence is absent; optional diagnostic code is reviewed without
adding player-facing menus. QA/modernization records remain untouched.

### Stage 3 tracks interface refinement

`world::tracks` now declares pending source-derived Descriptor, Selection, Route,
TrackContent, NamedTrack/Catalogue and Grid interfaces. Descriptor pointers are
CPU addresses confined to loaders. Route preserves packed words including its
terminal; race/AI own consumer decoding. Grid origin precedes terrain's wheel
probe offset. Grid X swap follows heading-zero negative displacement; order low
bit controls Y swap independently. TrackContent owns immutable raw windows,
projection pair, both routes and heading variants. Presentation applies map
attributes/transfers separately. Named extension format is a native contract,
with object/title additions to reconcile once those domain interfaces exist.
See world/tracks.trace.md for exact source ranges, branches and contract evidence.

### Stage 3 terrain interface refinement

`world::terrain` declares SamplingTables, checked Terrain windows/Neighborhood, explicit AdjacentControls, Surface/BlockDirection, WheelSamples and Projection/ScreenPosition. Endpoints unsigned byte, slopes signed word, world coordinates wrapping source words; projection X isword/Y isbyte. Four-wheel sampling retains control-marker history and reloads old surface summary; averaging remains physics. Data before/after copied window remains an explicit source-consumer uncertainty, not an emulated RAM image or invented default. See terrain.trace.md and 25 pending executable contracts.

### Controls contract handoff

controls.rs owns ROM direction tables, held-action→controller-word translation, frontend pair sampling and enabled race copies. Frontend uses signed low-byte deltas; conditional word helper uses full signed words, with natural entry unresolved. Native input must preserve opposing directions for the source lookup (conflicting nibble3/7/B/C..F maps both deltas zero). Both frontend pages sample JOY1/2; page only selects participant records. Race must consume retained words for motion before human copies/checkpoint checks, with AI writes last. Retained input plus owner-specific edge/repeat state must all enter full snapshots. controls.trace.md hands region49 sprite/font tails explicitly to presentation; primary index assignment never excludes those rendering effects.

### Stage3 physics interface reconciliation

`trucks::motion::{Truck,Wheel,Upgrades,WorkingMotion,MotionTables}` declares native
shared truck ownership for collision/AI/race. Word speed/heading preserve source
byte-store behavior; wheel arithmetic is explicit byte wrapping. All rule bodies
remain pending. Motion service domain tests use real synthetic Terrain and
SamplingTables, not callbacks implementing an oracle. Composition with objects'
contact animation and collision domain state is required as those contracts land.
`motion.trace.md` corrects sign-reversing contact half-speed, integrated-height
surface average versus sampled byte-pair shadow average, and rejected-proposal
side effects. Full snapshot/semantic observation coverage expands in stage4.

### Collision contract reconciliation

`trucks::contact` reuses Truck/WorkingMotion and real terrain/sampling interfaces. Pair proximity/bearing/midpoint use committed current world while response uses working current speed/heading; accepted current correction changes working only, other changes committed immediately. Caller projection/shadow precedes contacts and commit follows. Shared AI signed-vector quantization dependency is explicit, with fixed ROM bearing fixtures. Marker history shifts on both terrain tests including blocked returns. Conditional $83E1 entry is isolated, natural routing unresolved; no fabricated case dispatch. See contact.trace.md and24 pending contracts.

### AI contract handoff

`trucks::ai::{Driver,RaceAi,AiTables}` own checkpoint/lap/timeout/delay/parameter
plus race banner/winner view while shared Truck retains physical heading/lock/bias.
Stage4 reconciles race-owned common fields, avoiding duplicate mutable authorities.
`quantize_vector` is the single bearing-grid helper used by contact and AI; AI packs
Y/X reduced axes and applies quarterturn. Timeout changes checkpoint but accepts
against cached prior record. Parameter setup's all-human underflow reports an unused
route-tail byte outside original30..126byte windows; does not corrupt native added
routes. See ai.trace.md and uncertainty ai-all-human-parameter-tail. Snapshot owners
include every Driver/RaceAi/RNG/Truck field; service occurs after HUD, never movement.

### Race contract reconciliation

`race::Progress` is the single mutable owner of Driver checkpoint/lap/timeout/
parameter state plus human mask/animation/banner/winner. `trucks::ai::RaceAi` is
an alias to preserve its public contract API. Banner is word-width, last-lap
request stores low byte only. `race::inside_checkpoint` is canonical packed
rectangle acceptance; AI delegates in stage4. The earlier AI boundary fixture
was source-corrected (center-world subtraction: lower excluded, upper included),
retaining source/old-log evidence. `RaceState`/FrameEnvironment use real domain
trucks/terrain/routes/tables/time/sprite state; phase evidence never replaces
actual service mutations. Presentation/objects must add actual owner state to
composed service at implementation, not effect-only mocks.

Timer updates on every explicit HUD call including prepared and postrace entry;
winning does not freeze it. Banner winner freezes primary decay but still draws
and animates secondary. Track8 extra sprites, source byte grid headings, no
blanket reset of retained truck history are contracted. Transitions retains
resumable fade/ready/window/pause ownership; results owns postrace flow and must
call clock service once at entry. Every owner joins full snapshots/clone/debug
observations in stage4. See race.trace.md and38 pending contracts.

## Objects contract ownership refinement

WorldObjects owns random pickup/counts/lifetime, four pending-award bytes shared
with results/economy, six descending dynamic slots, general placements, optional
track0 composites and four special-surface contact cursors. Immutable ObjectContent
is optional on original TrackContent (None resolves original full-order ROM group),
required for added/replaced named track definitions. Full source trace and40 tests:
port/src/world/objects.trace.md / objects.rs; one additional tracks extension test.
Game/RaceState must compose actual WorldObjects with trucks and immutable tables,
then include all retained object/contact/OAM state in snapshot/observations. No
behavior is implemented yet. Track0's inclusive extra iteration is explicit derived
first-record tail/count effect, not RAM emulation; original fourth-frame count4→3.
Pickup packed XY centering can alter Y through X borrow/carry; composite X additions
are bytes. Object service order composite→pickup→dynamic→placements changes actual
awards/speed/animation in that order; source init services retained pickup before
clearing counters/awards. Physics proposed-surface response consumes persistent
history before blocked reload; its animation/sound survives discarded slowdown.
Presentation retains untouched OAM across shorter updates and owns generic cursor
advancement; objects owns which source cursor values trigger responses.

## Rendering contract ownership refinement

Presentation owns actual VideoMemory staged/visible OAM+VRAM+CGRAM, native Display
layer/main-sub/window/math/brightness config, SpriteArrays/animation cursors and
immutable semantic graphics/HUD/TrackPresentation. Pure pixel/render consume
visible state; NMI publishes prior staged OAM and current pose strips unless
inhibited. Source field/byte/word aliases stay in loaders/traces, no CPU register
or RAM emulator. Game/Race must compose these actual owners with WorldObjects,
clock/banner/trucks/transitions and full snapshot/debug state. Source53 contracts
and all101 range/cross-owner trace: presentation/tests.rs, rendering.trace.md.

TrackContent.presentation optional originals None resolves source16-table layout;
added/replaced named tracks require validated arbitrary title/banner/HUD/overlays,
no native catalogue16 ceiling. Existing extension fixtures updated; missing/short
layout rejects. Shared sprite DMA02:D000 bank-wrapped C00 tail is wholly overwritten
by immediate1040bytes toEFC0..FFFF before video activation; final native graphics
copy preserves surviving2BC0 ROM prefix plus1040 continuation, without inventing
RAM tail. Trace preserves superseded hypothesis and exact lifetime proof. Full
race palette consumer512bytes extends368-byte source prefix; do not truncate.
COLDATA0 retains fixed color; power-on choice remains recorded assumption.

Transitions owns scanline generation/double buffering/fades/countdown/pause;
presentation consumes inclusive edges/masks and source Mode1 composition.
Frontend area workers own when their bundle/layout is installed, with shared
planar/palette/text/rectangle/OAM mechanisms here; no UI diagnostics added.
Bearing/sine table semantic consumers remain AI/terrain/motion; unresolved grid/
generic zero/tileword blocks remain explicit pending consumer assessment.

### Stage3 transitions refinement

Source-backed transitions use native Fade/Window/FrontendWindow/RaceTransition/
Pause state, not blocking host loops. Opposite displayed/build buffer selection
retains one-build staging; masks configuration is distinct from active Display
selection, and HDMA remains enabled after masks-only clear. Source shape math
reads only signed-product middle byte before asymmetric edge clamps. Keep full
phase/buffer/history state in Game snapshots; NMI/audio continues while race
motion stops. Added TrackPresentation.countdown explicitly supplies custom ready/
pause center and VRAM destination without original track-table index. Actual
banner/sprite calls remain shared owners; observational call counts cannot replace
their execution. Detailed boundaries/order/tests: presentation/transitions.trace.md.

### Stage3 attract/session refinement

`frontend::attract::Session` owns frontend signature/entry intent and shared session-record/wallet proposal; upcoming results/economy workers refine/reuse these values rather than copy independent records. Source full5-byte signature gates initial splash/default-copy only; poll firstbyte nonzero permits Start. Valid record signature preserves records while common entry resets wallets and all20 stock/upgrade bytes. Source stack unwind becomes explicit nonlocal reentry; cold Game reset is separate from warm frontend entry. Cloneable Splash/Fade, Panel/Window and Highscores retain real display/timer/geometry state; render reads it only. ROM-backed resource install copies exact source spans/partial palettes/inclusive clears. Full stateful Game integration must include these owners and actual names→shop→joined-word gate, standard panels/results rows/fade/window calls, not substitute phase logs. Detailed source boundaries: frontend/attract.trace.md.

### Stage3 names ownership refinement

`frontend::names::Selection` owns transient local page/cursors/countdown/temporary glyph buffers and suspended waits; shared `attract::Session` retains joined pages, records/truck names, wallets, inhibit and audio. Both UI pages sample JOY1/2. Names implements driver-panel asset consumers/variants through shared presentation mechanisms, reusable by shop/attract; source maps/palettes remain ROM-backed. The live cursor reaches byte6 (fourth preview) while final commit uses0/2/4 only. Repeat8 reload produces9-call spacing; Select and B share history retained across page initialization. Stage4 must compose this retained history with shop and pause-owner lifetime, never silently clear on each page. Final ACK restores M8 before inclusive page comparison. Natural boot page_limit0 is not a new player-count menu.

### Economy contract ownership and source corrections

`frontend::shop::Shop` retains the existing names Selection/Editor values across
frontend entry, including repeat/button history; session wallets/selected wallet
remain `attract::Session`, Truck owns stock, WorldObjects owns pending awards.
Actual Game must move/compose these shared owners and snapshot suspended Fade/Window
state. Money digits are most-significant first with two appended display zeros;
ordinary awards/debits affect stored digit3 ($10000), finer4/5 helpers remain
conditional. Normalize source CMP-N threshold8A and highest-borrow adjacent-domain
effects; never substitute generic integer clamp. Source shop cursors clamp2x3,
repeat8 reload gives9calls, stock order nitro/acceleration/steering/speed/suspension.
UI refresh precedes input; purchases display next frame, zero bars preserve old
tiles, first done waits an extra NMI before second handler. Driver panels/fonts
reuse names/presentation; real Fade/Window service rather than phase-only output.
Detailed branch/ROM contracts: frontend/shop.trace.md. Implementation pending.

### Results contract handoff

`frontend::results` reuses Session.records/wallets/selected and race::Progress;
AddedRecords stores only additional named-track records, never duplicate original
arrays. WorldObjects.pending_awards is borrowed once into settlement. Podium rank
awards credit immediately; displayed pickup counts settle on dismissal. Original
record timer>250 blink bypasses controller poll, exact250 commits then requires
an extra NMI before redraw/input; snapshots retain this resumable phase. Score
order is ascending indirect storage (lowest0), strictgreater insertion may create
a duplicate below equal existing score. Penalty byte-alias resets losing wallet
to source startercash rather than an invented input flag. Continue losing-port
heldB and priority-left differ from menu edge/repeat policy. Source high/low
continuation bytes remain explicit; byte writes preserve the high byte. The real
Game must compose postrace HUD→wait→window→rank→fade→podium→record/dismiss→contract
→settlement→loss/highscores→continue/shop/inactive conversion, including actual
Display/font/window/audio state, not return-effect lists. All bodies pending;
42 executable contracts and source derivations: frontend/results.trace.md.


### Audio contract ownership and source corrections

`sound::Sound` is the single cloneable mutable audio owner: native Driver/Voice/
Sequence stack, HostState pending second update, live EnvelopeDefinitions,
shared immutable AudioDefinitions, native DSP Mixer/predictor/envelopes/noise and
resampler/native clocks. Existing Session.audio_commands must feed this owner in
source order; headless/desktop/browser use the shared Game audio hook. Immutable
DataAddress is source data identity, not a CPU/RAM map; original code is never
interpreted as SPC instructions. Sound contracts cover all34 commands,16 songs,
16 effect descriptors,33 sample directory entries and source out-of-allocation
instrument33 effective reads. Live141-byte config writes must not reset unrelated
envelope rows on song reload. Song0 low-ARAM config read remains explicit silent
startup assumption; deterministic264-sample second heartbeat update is a native
scheduling policy, not instruction-cycle fidelity. Actual B6/B8 vectors, positive
fade BMI and zero/one duration envelope branches supersede old prose. DSP writes
KON-before-voice/KOFF-after preserve new-note key-on delay. Native toolkit linear
interpolation and missing cycle-level DSP races remain explicit limits; complete
mixer serialization may require logged toolkit extension during implementation.
101 contracts and all717 source range links: sound/audio.trace.md. Stage4 must
compose actual sound synthesis/state, not preserve raw bytes or return event lists.

## Stage 4 implemented foundation

Runtime source RNG/boot writes/NMI counter effects/header facts, controller direction/sampling/held-action translation and track order/parallel descriptors/routes/start-grid/resource/catalogue loaders are now native implementations. Terrain sampling implements byte-width addressing, marker/block classification, reflected triangle interpolation, wheel probes and exact signed/truncated projection. The single-point sample_at interface shares these algorithms with motion/contact. Typed tracks.json decoding validates original/file resources through MemoryAssets::collection; explicit added object/presentation/countdown schemas have no global16-slot cap. Shared Game orchestration, other domain modules, full snapshots and ledger completion remain pending. Focused actual results: checks/implementation/domain-results.json.

QA source ownership correction: source countdown_state $077D is a 16-bit word shared by ready/countdown drawing and banner animation. Progress.banner_state owns that word; RaceTransition.countdown.state is its continuation view, synchronized at source call boundaries, including banner mutations. Snapshot validation requires equality. Countdown delay remains the independent word $077F. This supersedes the earlier duplicated live owners found by the exact ready13/4 frame comparison (21 differing banner pixels).
