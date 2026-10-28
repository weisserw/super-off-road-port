# super-off-road native Rust port

**[Play the game in your browser](https://weisserw.github.io/super-off-road-port/)**

The shared native game implements the original frontend, four-truck races, terrain/contact/AI, track objects, results/records/shop, Mode1 presentation and native audio. It uses the matching ROM as data; no emulator runs the game rules. Defaults are 256×224 at SNES NTSC cadence, displayed at 4:3.

Build instructions and platform limits are in [BUILDING.md](BUILDING.md). `python3 run.py desktop` starts desktop on a machine with SDL2; `python3 run.py web` builds and serves the browser frontend. F1 resumes, F2 pauses the frontend runner, F3 resets, F4 saves a full state, F5 restores it and F6 opens player settings. Gameplay controls use the shared SNES mappings in settings: left/right steer (L/R give the source smaller steering step), B accelerates, Y uses nitro and Start joins/pauses according to the current scene. Name entry uses directions, Select and B. Each player's mapping is configurable in F6.

Starting from attract mode opens a mode menu before name entry. Use Up/Down (or Select) to choose **Arcade (Random)** or **Arcade (Classic)**, then A, B, or Start to confirm. Random visits each track once in a shuffled cycle, chooses its direction independently each race, and varies compatible original hay-bale layouts and which bales appear. Classic keeps the original track progression, directions, and objects. Full snapshots retain the selected mode and random setup.

The executable accepts `--rom`, `--assets`, `--settings`, `--headless` and the shared debug options. Supply the ROM SHA256 in port/src/config.rs. Records last for the session and full snapshots; the original cartridge has no battery-backed save. Desktop and browser consume the same Game APIs and MemoryAssets validation.

Internal modernization keeps the existing controls, timing and save format. Nitro inventory, race time and hazard responses use native quantities; routes use decoded waypoint geometry and traversal actions. Source encodings remain at display, asset and compatibility boundaries. The iteration's scope and validation are recorded in [modernize/changes.json](modernize/changes.json).

## GitHub Pages

The [Pages workflow](.github/workflows/pages.yml) builds and publishes the browser
app on every push to `main`, and can also be run manually from Actions. In the
repository's **Settings → Pages**, select **GitHub Actions** as the publishing
source. The deployment job reports the site URL.

Like the DuckTales workflow, it checks out a pinned ROMAtlas revision beside this
repository, installs Rust 1.90.0 and wasm-bindgen 0.2.129, then packages `web-dist/`
with relative URLs, the configured ROM (verified against `workspace.json`), and
the title “Super Off Road”. Only that generated directory is uploaded to Pages.
Update the ROMAtlas commit in the workflow when adopting toolkit changes.

To build the same static site locally with the sibling `/root/romatlas` checkout:

```sh
python3 scripts/build-pages.py
```

## Content replacements and added tracks

An asset folder may contain `resources.json` and/or `tracks.json`, using version1 manifests with `entries` (`id`, `file`) and optional `order`. Files are relative safe paths. Empty manifests in examples/original-only/ are working original-content-only defaults: all16 tracks and the original64-slot forward/reverse sequence remain selected. No ROM bytes are distributed there.

Raw replacement IDs and exact lengths are listed in port/src/resource_definitions.rs. A replacement must match its original Data-region size. The loader overlays explicit replacements into immutable ROM data before semantic decoding; unsupported/malformed decoded structures fail loading. Added raw slots remain available to resource tools but require a native semantic consumer to affect play. Editing raw code bytes is unsupported; game rules live in the Rust modules.

To add a track, put a complete version1 `TrackFile` JSON file under the asset folder and add its named ID to tracks.json. The file schema is defined by `TrackFile` in port/src/world/tracks.rs: `version: 1`, projection offset/scale, terrain (0x16B1 bytes), two background maps (0x600 bytes each), planar graphics (0x8000 bytes), two packed routes, two headings, explicit object definitions and explicit title/HUD/overlay/countdown presentation. Each `routes` entry is an array of u16 packed words, including its terminal FFxx word. The terminal low byte supplies checkpoint_count, the final checkpoint index, so count+1 complete pairs are required. The native `TrackContent`/`Route` types are decoded representations, not the JSON wire format. Redirects must address complete records and a human checkpoint must be reachable. Object and presentation structures are in world/objects.rs and presentation/mod.rs. Replacing a named original track uses the same complete schema.

Without an explicit nonempty order, added IDs follow the original64 directional slots. With an explicit order, named IDs define an arbitrary-length forward sequence; no16-track ceiling is imposed. Original ROM data supplies the shared graphics/sounds/rules unless replaced through a supported raw data consumer. Added-track records are keyed by track ID in separate session state. Snapshots bind the complete effective content and order, rejecting different assets.

Game rules are shared in runtime.rs, controls.rs, trucks/, world/, race.rs, frontend/, presentation/ and sound/. architecture.md describes ownership; differences.md records assumptions and hardware limits. Stage5 QA is a separate coordinator step: read debug/goal.md; implementation does not launch it automatically.

Normal desktop/browser startup randomizes the five-byte source RNG history using OS/browser entropy. Reset reuses that game’s initial seed; new loads get fresh randomness. Headless diagnostics can reproduce a run with `--rng-seed 123456789b`; seed identity is checked on replay.
