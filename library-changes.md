# Library changes

## Follow-up: completed one-shot audio save failure

Reproduced `invalid voice envelope/phase` with a nonlooping BRR sound at
pitch0x3fff. Its final source advance stopped the voice while whole phase steps
remained. `Voice::next` now clears the phase when the source ends, keeping the
inactive cursor valid without weakening snapshot validation. Playback before
completion is unchanged; a subsequent key-on already resets the cursor.

Added a regression saving/restoring at every sample across that endpoint and
checking the restored next sample. It failed with the user's exact error before
the fix and passed afterward. Full locked release desktop-feature toolkit tests
PASS81 tests; desktop-feature all-target Clippy with warnings denied PASS.
No window opened. Toolkit changes remain uncommitted; unrelated edits preserved.
This explains the save failure; it does not by itself establish the cause of the
reported gameplay freeze. Game follow-up checks are recorded separately.

Preparation baseline: no port-driven changes had been recorded. Stage4 changes and evidence follow; Stage5 owns runtime comparisons.

## Stage4 native audio snapshots (in progress, uncommitted)

Requirement: full Game snapshot must fork/restore actual native synthesis,
including private BRR predictor/source cursors, direct/ADSR envelopes, noise,
clock, interpolation phase and optional stereo FIR echo delay/history. Clone
alone cannot provide browser JSON persistence.

Toolkit instructions read: /root/romatlas/AGENTS.md, project.md,
docs/development.md native runtime check section. Pre-existing modifications
preserved: .cargo/config.toml, AGENTS.md, docs/development.md,
scripts/port-template/BUILDING.md, scripts/port.py, plus untracked
scripts/port-template/cargo-config.toml. No edits to these files.

Changed /root/romatlas/crates/romatlas-port/src/snes/audio.rs: complete typed
serde state on native mixer records; Mixer::snapshot_json and
Mixer::restore_snapshot_json version1; unknown-field/version/size/cursor/source/
predictor/envelope/noise/echo validation before replacing the live value.
Immutable game ROM/resource binding and host/resampler state remain game-owned.
No CPU/emulator dependency or original bytes added. Library edits remain uncommitted.

Actual focused check: cargo +1.90.0 test --release --locked -p romatlas-port
--features desktop snes::audio::snapshot_tests:: PASS1/1 (26filtered), including
nonzero active filtered BRR, fractional source advancement, noise generator,
feedback echo, exact subsequent91samples/state, isolation after keyoff, invalid
voice cursor and unsupportedversion atomic rejection. No desktop window opened.
Preparation status (superseded by the completed checks below): required native desktop-feature test/clippy checks were pending. Fidelity limits remain
native linear interpolation, no cycle-exact DSP bus/register races or echo ARAM.

Toolkit required checks completed: release --locked desktop-feature Clippy all-targets with -D warnings PASS; full release --locked -p romatlas-port --features desktop test PASS:27unit +3browser_saves+4genesis+2gg+4nes+5pce+15runtime+3sms+3snapshots+9snes+1brightness+4objlimits=80tests,0failures,0ignored; doctests0. Headless only. No other toolkit file changed.

Stage5 QA made no further toolkit edits. The pre-existing six-file baseline and Stage4 SNES audio Mixer snapshot change remain preserved and uncommitted. QA used the existing explicit-platform original server and native JSON-RPC harness; game-local rendering/scheduler/RNG repairs and their checks are recorded in debug/repairs.json. Stage4 toolkit focused/full80tests/Clippy results above remain unchanged; no toolkit-wide confidence rerun was added.

## Follow-up: faster development checks

Added `run.py check-fast` in toolkit scripts/port.py: sequential development
format/shared-library Clippy/unit tests; optional Rust test-name substring;
optional single desktop or Wasm development build. Full release check commands
and their checks/latest.* results remain separate; fast checks write checks/fast.*
and identify themselves as development-only. Invalid options/empty filters reject.
Generated workspace instructions, template BUILDING.md, template Cargo profile
config, toolkit AGENTS.md and development/porting docs describe the two loops.
Existing toolkit changes were preserved; all toolkit edits remain uncommitted.

This workspace explicitly adopts the template profile config under .cargo/:
dev/test opt-level0, incremental enabled and debug symbols disabled; release
incremental/debug settings retained. AGENTS.md and BUILDING.md document usage.
A checkpoint test's replay request now supplies a30-second test budget instead
of relying on the production debugger's1-second default, which the unoptimized
suite exceeded; production settings and replay assertions are unchanged.

Validation so far: focused terrain development check PASS30tests; full
unoptimized game suite PASS627tests/0failures/0ignored; warm focused check
(format+lint+30tests) measured2.369seconds. Records: checks/follow-up/dev-all.*,
dev-warm-timing.json. Synthetic scripts/test-port.py PASS, including actual shared/filtered,
desktop-only and Wasm-only fast builds, CLI rejection and unchanged acceptance
records. Final game release checks PASS all six gates,627tests/0failures/0ignored,
desktop/Wasm lint and builds. Records: checks/follow-up/dev-final-check.*. Superseded timeout
failure retained at checks/follow-up/dev-before-timeout-fix.*. No graphics launched,
no publication, no changes to gameplay rules or commercial assets.

## Browser display title

Toolkit scripts/port_web.py now uses optional workspace.json title for the
browser heading/tab, retaining the directory-name fallback. This workspace
sets title to Super Off Road. Verified the real HTTP config endpoint via an
ephemeral loopback server; no Rust rebuild needed. Toolkit edits uncommitted.

## Follow-up: controller binding display order

Shared settings.rs now derives a button/system/D-pad order from actual action
names: SNES A B X Y L R Start Select Up Right Down Left; corresponding GB/NES,
Genesis, SMS, GG and PCE controls are filtered from the same order. Custom names
follow alphabetically. Desktop labels, rebind selection and clear selection use
this order. Browser shell.js uses the identical ordering independently of saved
JSON insertion order, including pre-existing persisted settings. No binding or
settings schema changes. Template browser builds consume this shell and all
native builds use the shared menu; this workspace adopts both automatically.

Added a runtime regression across seven platform defaults plus persisted
settings, custom actions and actual menu rebind/clear selection. Browser
regressions check actual settings rows after reloading reversed persisted
platform action maps. Toolkit release desktop tests PASS82; desktop-feature all-target Clippy PASS.
Pinned headless Chromium suite PASS including all persisted platform orders.
Full workspace check PASS all six gates and627tests; browser bundle rebuilt
and existing server refreshed. Actual Super Off Road browser settings verified
with exact SNES row order. Records: checks/follow-up/button-order-*. No desktop graphics launched. All toolkit
edits remain uncommitted; prior changes and records preserved.
