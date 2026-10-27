# Native policies and validation limits

The source game rules, frontend, rendering and sound driver are native Rust. Hardware setup is replaced by owned Display/VideoMemory, explicit NMI scheduling and Sound. CPU stack/register preservation and DMA handshakes become typed calls and copies; source-width wrapping arithmetic remains where behavior depends on it. Full state snapshots are native version2 and resource-bound; they are not emulator save states.

Cold power-on contents are not established by the ROM. Native cold state zeroes retained histories, fixed color and otherwise unwritten owner fields; the five-byte RNG is initialized from OS/browser entropy when a game is loaded, rejecting seeds that collapse immediately into the absorbing zero state. Source boot itself still establishes no seed. Reset within that loaded game reuses its random initial history so checkpoints remain replayable; warm entry retains the advancing history. Warm frontend entry preserves source retained fields. The menu/pause shared history can conditionally select a source I/O address outside the four controller words; native scheduling treats this unestablished owner as released and emits retained_history_release rather than invent a new player. Conditional helper semantics remain directly callable and contracted even where no natural caller was established.

Terrain reads beyond the copied source terrain window are rejected when absent; no zero/clamp original-memory claim is made. Added routes/assets are validated to avoid malformed pointers, cyclic control-only routes and invalid indices. Source invalid-memory behavior is not a supported extension format. Original variable-length overlapping route views remain accepted. The all-human AI setup's route-tail write is retained explicitly in RaceState and overwritten by a sufficiently long route copy; it has no supported original route read.

The native NMI model preserves source waits, captured input, staged video transfer and ordered host audio requests at frame boundaries. Subframe busy polling and device input latency remain unmeasured. No instruction-cycle or PPU bus emulation is claimed. Toolkit native DSP uses linear interpolation rather than cycle-exact SNES Gaussian/key-on races; the source song/voice/envelope/BRR/sequence logic and scheduler are implemented. Fast audio flag scheduling uses the deterministic native policy in sound/synthesis.rs. Song0 silent reset uses explicit native envelope policy rather than fabricating low-ARAM configuration bytes.

Original high scores and track records are session-only: the cartridge has no original battery state. Added tracks use separately keyed session records. All records/audio/scheduler/input/event state are included in full snapshots. Raw added resource slots need explicit consumers; arbitrary code patches are outside content replacement support.

Unclassified source bytes retain their original identities/classification. Consumer-specific decisions and assumptions are recorded in uncertainties.json and ledger.json, with source traces. Excluding isolated unentered bytes or allocation fill does not assert universal hardware unreachability. Source-backed fixture corrections and initial failures remain in adjacent traces and checks/implementation/domain-results.json.

Validation on this server consists of source-backed contracts, focused integrated state/audio/frame replay and compile/static checks. No desktop window, browser runtime, physical audio/device test, gameplay smoke or exact-frame QA was run during Stage4. Stage5 records remain separate and preserved.

Stage5 policy correction: Stage4 used fixed nonzero RNG history12/34/56/78/9B. Initial natural original comparison exposed a71pixel initial pickup difference caused solely by that assumption. At that historical QA build the global cold baseline used zero reference WRAM, without per-capture seed edits or masked comparisons. This policy was subsequently superseded by the user-requested random startup fix below. The original seed remains unknown on physical hardware, warm history remains retained, and pure all-zero feedback behavior is unchanged. Superseded policy and failed capture remain in uncertainty history and QA evidence.

Stage5 bounded original comparison resolves six selected frontend/race frames exactly after source-backed BG visible origin, OAM/HDMA row, BG3 map size and shared countdown/banner-word repairs. No graphic mismatch remains in those pairs and no pixel tolerance/mask is used. Original CPU loading/startup handoff crosses physical NMIs while native synchronous loading does not; recipes align named source phase and prepared pickup parity through natural input. Initial music source voice progression matches exactly over the selected30frames, while captured native linear-interpolation waveform/amplitude differs from original DSP output as documented above (review metrics in debug/artifacts/sanity-run-007). This is computational captured-audio review, without physical audition, and does not establish whole-game fidelity.

Follow-up RNG correction: the zero reference baseline disables the original feedback generator and is no longer the player default. Native and browser loading now obtain all five bytes through getrandom (OS randomness / Web Crypto). A headless-only `--rng-seed` option and `Port::load_assets_seeded` support explicit deterministic reference runs, including zero for historical reproduction. Replay identity includes that seed; full saves retain the original reset history alongside their advancing RNG state. Older version2 saves remain readable.

Follow-up terrain edge policy: ordinary driven racing reaches control reads
immediately before the copied terrain ($7EFF7E..$7EFFFF). Native config chooses
zero for those untouched power-on controls, matching the retained zero-memory
reference baseline. This supersedes rejecting those reads, which froze play.
Physical-console initial contents remain unestablished. Neighbor reads now
follow original early-return order; malformed/missing copied data still errors.

Modernization iteration1 retains the same gameplay and version2 save/debug wire values while storing ordinary nitro as a charge count and race time as elapsed source units. Noncanonical decimal bytes accepted by historical saves retain an isolated faithful-port compatibility path rather than being rejected or normalized on import. That path preserves the previous native conditional policy; no new original-hardware invalid-BCD observation is claimed. Name/shop handlers were structurally extracted with the same NMI yield/continuation decisions. Fresh bounded original and faithful-build evidence is tracked separately in modernize/changes.json.

Modernization iteration2 introduces no intentional gameplay divergence. Packed route words decode into immutable nodes with explicit geometry/traversal policies, while original content/JSON encoding, consumer index windows and route-tail effects remain boundaries. Steering's final byte reduction discards coordinate bits14/15; tests verify they do not create an observable bias-carry effect, correcting the candidate menu's overly cautious claim. Acceptance and ranking still retain their distinct masks. Dynamic hazards now use explicit stages and native response quantities, preserving source uses0→one response, contact/presentation order and conditional legacy counters. The AI/contact bearing table has one immutable owner. Natural original attract-mode observations establish track2 response/depletion and lap progression; real-ROM redirect branches remain source-backed domain contracts rather than a claim of natural original traversal on tracks8/13. Existing hardware timing, initialization and audio synthesis limits remain unchanged.

Modernization iteration3 M1 introduces no intentional pickup rule change. Numeric
pickup state is now a save/asset/presentation encoding around explicit lifecycle
and reward values. Original and valid override rewards decode at loading;
nonstandard phases retain their prior deferred projection rejection. Natural
original attract collection and expiry are recorded independently of native
startup parity: their first spawned kinds/collector slots can differ because
loading crosses different source NMIs. Exact starting-build state/frame/PCM
comparisons use the same native recipe, without editing game state or bypassing
checkpoint identity guards. This does not establish whole-game or physical
console fidelity.

Iteration3 R2 is structural refactoring: powertrain inputs, proposed speed/heading
and immediate retained outputs are explicit. It preserves the existing motion
model, fixed-width arithmetic and sequential collision visibility; no new physics
or source representation conversion is claimed. The collision transaction clones
remain responsible for atomic failure and are separate from the removed whole-
Truck powertrain clone.

Iteration3 M5 replaces source pose selectors and per-NMI strip command construction
with decoded body-frame definitions and immutable pose images. The existing
renderer still receives the same two tile rows at the same NMI and retains all
interstrip bytes. Version2 words, including all65536 historical selectors, keep
the faithful native contiguous-ROM span policy for unaligned/crossing words;
this does not establish original DMA behavior for malformed states. The flat/
nonzero-tilt heading rule decodes at the asset boundary, including file overrides.
Original flat/turning and tilted images were independently observed in natural
original VRAM, separate from starting-build differential tests.
