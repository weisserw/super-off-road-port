//! Atomic versioned game snapshots. Immutable resource identity is checked first.
use super::*;
use anyhow::ensure;
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    version: u32,
    rom_sha256: String,
    asset_identity: String,
    state: State,
    sound: String,
    // Older version2 saves have no separate reset seed. Keep the receiving
    // game's seed in that case; restore their current RNG state exactly.
    #[serde(default)]
    initial_random_history: Option<crate::runtime::RandomState>,
}
impl Port {
    pub(super) fn export_snapshot(&self) -> Result<String> {
        Ok(serde_json::to_string(&Snapshot {
            version: 2,
            rom_sha256: crate::config::ROM_SHA256.into(),
            asset_identity: self.resources.identity.clone(),
            state: self.state.clone(),
            sound: self.sound.snapshot_json()?,
            initial_random_history: Some(self.initial_random_history),
        })?)
    }
    pub(super) fn import_snapshot(&mut self, json: &str) -> Result<()> {
        ensure!(json.len() <= 16 * 1024 * 1024, "game snapshot too large");
        let mut saved: Snapshot = serde_json::from_str(json)?;
        ensure!(
            saved.version == 2
                && saved.rom_sha256 == crate::config::ROM_SHA256
                && saved.asset_identity == self.resources.identity,
            "game snapshot identity/version mismatch"
        );
        validate(&saved.state, &self.resources)?;
        self.resources
            .poses
            .resolve(&mut saved.state.race.video.poses)?;
        let selected = Arc::new(
            self.resources
                .race(saved.state.sequence_index, saved.state.session.order_slot)?,
        );
        let terrain = Arc::new(crate::world::terrain::decode_terrain(
            &selected.track.terrain,
        )?);
        let mut sound = self.sound.clone();
        sound.restore_snapshot_json(&saved.sound)?;
        self.state = saved.state;
        self.selected = selected;
        self.terrain = terrain;
        self.sound = sound;
        if let Some(history) = saved.initial_random_history {
            self.initial_random_history = history;
        }
        Ok(())
    }
}
fn validate(state: &State, resources: &Resources) -> Result<()> {
    use crate::{
        frontend::{names, results, shop},
        presentation::transitions as t,
    };
    use state::{Menu, Scene};
    ensure!(
        state.sequence_index < resources.sequence.len()
            && state.session.page_limit <= 1
            && state.session.selected_wallet < 4,
        "invalid snapshot selection"
    );
    ensure!(
        state.session.joined_pages.iter().all(|&mask| mask <= 3),
        "invalid frontend joined mask"
    );
    let video = &state.race.video;
    let mut sprite_order = video.sprites.order;
    sprite_order.sort_unstable();
    ensure!(
        sprite_order == std::array::from_fn(|i| i as u8),
        "invalid sorted sprite permutation"
    );
    ensure!(
        video.display.brightness <= 15
            && video.display.masked_layers <= 31
            && video.display.window_inverted_layers <= 31
            && (224..=260).contains(&video.display.window_edges.len())
            && video
                .display
                .layers
                .iter()
                .all(|layer| matches!(layer.bits_per_pixel, 2 | 4 | 8)),
        "invalid display snapshot"
    );
    ensure!(
        state
            .race
            .trucks
            .iter()
            .all(|truck| truck.nitro_effect.valid()
                && truck.contact_effect_player.valid()
                && truck.upgrades.steering < 7
                && truck.upgrades.acceleration < 7
                && truck.upgrades.speed < 8)
            && state
                .race
                .world
                .contacts
                .iter()
                .all(|contact| contact.player.valid()),
        "invalid truck/animation snapshot"
    );
    ensure!(
        state.race.progress.banner_state < 35
            && state.race.banner.secondary_state <= 5
            && state.race.progress.winner_plus_one <= 4
            && state
                .race
                .progress
                .drivers
                .iter()
                .all(|driver| driver.parameter <= 7),
        "invalid race progress snapshot"
    );
    // collect_pickup can conditionally produce state11. Type1 hazards start
    // at DYNAMIC_RESPONSE_TICKS=44, decrement before indexing the44-byte
    // dynamic_y_deltas table, and retain negative completion sentinels.
    ensure!(
        state.race.world.pickup.phase.supported()
            && state.race.world.placements.len() <= 255
            && state
                .race
                .world
                .placements
                .iter()
                .all(|object| object.animation < 56)
            && state
                .race
                .world
                .dynamic
                .iter()
                .all(|object| object.stage.supported()
                    && object.animation < 35
                    && object.response.valid_snapshot()),
        "invalid world snapshot"
    );
    if let Some(composites) = &state.race.world.composites {
        ensure!(
            composites.count <= 4
                && composites.sprites.len() <= 5
                && composites
                    .records
                    .iter()
                    .all(|record| record.animation < 48),
            "invalid composite snapshot"
        );
    }
    let mut score_order = state.session.records.scores.order;
    score_order.sort_unstable();
    ensure!(
        score_order == std::array::from_fn(|i| i as u8),
        "invalid score permutation"
    );
    ensure!(
        state.pause.owner_plus_one <= 4 || state.pause.phase == t::PausePhase::Running,
        "invalid active pause owner"
    );
    if state.pause.phase != t::PausePhase::Running {
        ensure!(
            state.pause.owner_plus_one != 0 && state.scene == Scene::Racing,
            "pause continuation outside racing"
        );
    }
    ensure!(
        state.session.active_humans == state.race.progress.active_humans
            && state.session.inhibit_video == state.race.video_inhibit
            && state.session.enabled_layers == state.race.enabled_layers,
        "divergent shared race views"
    );
    ensure!(
        matches!(state.race.entry, crate::race::RaceEntry::Running) && state.race.working_pose < 32,
        "invalid delegated race entry"
    );
    let selection = state.menu.selection();
    ensure!(
        selection.page <= 2
            && selection.done_mask <= 3
            && selection
                .editors
                .iter()
                .all(|editor| editor.cursor_byte <= 6 && editor.cursor_byte % 2 == 0),
        "invalid menu cursor"
    );
    ensure!(
        match selection.resume {
            names::Resume::Join(player) | names::Resume::Edit(player) => player <= 2,
            _ => true,
        },
        "invalid menu resume point"
    );
    ensure!(
        match selection.phase {
            names::Phase::JoinDraw(player) | names::Phase::ClearPanel(player) => player < 2,
            _ => true,
        },
        "invalid menu panel continuation"
    );
    let mut window_count = usize::from(state.window.is_some());
    if let Some(window) = &state.window {
        window_valid(window)?;
    }
    if let Some(panel) = &state.panel {
        window_valid(&panel.window)?;
        motion_valid(&panel.motion)?;
        window_count += 1;
    }
    if let Some(sequence) = &selection.sequence {
        match sequence {
            names::Sequence::Window { window, motion } => {
                window_valid(window)?;
                motion_valid(motion)?;
                window_count += 1;
            }
            names::Sequence::Fade(fade) => fade_valid(fade)?,
        }
    }
    match &state.menu {
        Menu::Shop(shop) => {
            ensure!(
                shop.selection
                    .editors
                    .iter()
                    .all(|editor| editor.column < 2 && editor.row < 3),
                "invalid shop grid"
            );
            if let Some(window) = &shop.window {
                window_valid(window)?;
                window_count += 1;
            }
            if let Some(motion) = &shop.window_motion {
                motion_valid(motion)?;
            }
            if let Some(fade) = &shop.fade {
                fade_valid(fade)?;
            }
            ensure!(
                shop.phase != shop::Phase::FadeIn || shop.fade.is_some(),
                "missing shop fade"
            );
            ensure!(
                shop.phase != shop::Phase::Expand
                    || (shop.window.is_some() && shop.window_motion.is_some()),
                "missing shop window"
            );
            ensure!(state.scene == Scene::Shop, "active shop outside shop scene");
        }
        Menu::Names(selection) => {
            ensure!(
                state.scene == Scene::Names
                    && selection.page < 2
                    && selection
                        .editors
                        .iter()
                        .all(|editor| editor.column < 5 && editor.row < 7),
                "invalid active name page"
            );
            ensure!(
                !matches!(
                    selection.phase,
                    names::Phase::EntryFade | names::Phase::FadeIn
                ) || matches!(selection.sequence, Some(names::Sequence::Fade(_))),
                "missing name fade"
            );
            ensure!(
                selection.phase != names::Phase::ContractWindow
                    || matches!(selection.sequence, Some(names::Sequence::Window { .. })),
                "missing name window"
            );
            if selection.phase == names::Phase::WaitFrame {
                for player in 0..2 {
                    if state.session.joined_pages[usize::from(selection.page)] & (1 << player) != 0
                    {
                        ensure!(
                            selection.temporary[player].contains(&0xff),
                            "unterminated live temporary name"
                        );
                    }
                }
            }
        }
        Menu::Idle(selection) => {
            ensure!(
                selection
                    .editors
                    .iter()
                    .all(|editor| editor.column < 5 && editor.row < 7),
                "invalid retained menu grid"
            );
        }
    }
    ensure!(
        window_count == 1,
        "snapshot must have one source window owner"
    );
    if let Some(fade) = &state.fade {
        fade_valid(fade)?;
    }
    if let Some(motion) = &state.frontend_motion {
        motion_valid(motion)?;
        ensure!(state.window.is_some(), "frontend geometry lost window");
    }
    if let Some(transition) = &state.transition {
        ensure!(
            transition.countdown.state == state.race.progress.banner_state
                && transition.countdown.state < 34
                && transition.audio_commands.len() <= 16
                && state.window.is_some(),
            "invalid race window continuation"
        );
    }
    if let Some(prompt) = &state.prompt {
        ensure!(
            prompt.loser < 4 && prompt.choice <= 1,
            "invalid continue prompt"
        );
    }
    if let Some(postrace) = &state.postrace {
        let mut rank = postrace.rank;
        rank.sort_unstable();
        ensure!(rank == [0, 1, 2, 3], "invalid postrace rank");
        let choice = &resources.sequence[state.sequence_index];
        let content = &resources
            .tracks
            .tracks
            .iter()
            .find(|track| track.id == choice.id)
            .context("snapshot track missing")?
            .content;
        ensure!(
            postrace.route.as_ref() == Some(&content.routes[usize::from(choice.reversed)])
                && postrace.route_count
                    == content.routes[usize::from(choice.reversed)].checkpoint_count,
            "postrace route resource mismatch"
        );
        ensure!(
            !matches!(
                postrace.phase,
                results::PostracePhase::FadeOut | results::PostracePhase::FadeIn
            ) || state.fade.is_some(),
            "missing postrace fade"
        );
        ensure!(
            postrace.phase != results::PostracePhase::Contract
                || (state.frontend_motion.is_some() && state.window.is_some()),
            "missing result contraction"
        );
    }
    match state.scene {
        Scene::AttractPanel => ensure!(state.panel.is_some(), "missing attract panel"),
        Scene::HighscoreFade | Scene::RaceFade | Scene::AttractFade | Scene::LossFade => {
            ensure!(state.fade.is_some(), "missing scene fade")
        }
        Scene::Highscores => ensure!(state.highscores.is_some(), "missing highscore scroll"),
        Scene::HighscoreClose => ensure!(
            state.highscores.is_some() && state.frontend_motion.is_some() && state.window.is_some(),
            "missing highscore closure"
        ),
        Scene::Names => ensure!(matches!(state.menu, Menu::Names(_)), "missing names owner"),
        Scene::Shop => ensure!(matches!(state.menu, Menu::Shop(_)), "missing shop owner"),
        Scene::Ready | Scene::MusicWait | Scene::Racing => {
            ensure!(state.window.is_some(), "missing race window owner")
        }
        Scene::Postrace
        | Scene::LossSetup
        | Scene::LossShowWait
        | Scene::LossPrompt
        | Scene::LossClose => ensure!(state.postrace.is_some(), "missing results owner"),
        _ => {}
    }
    if matches!(state.scene, Scene::LossPrompt | Scene::LossClose)
        && state.session.active_humans != 0
    {
        ensure!(state.prompt.is_some(), "missing human continue owner");
    }
    if state.scene == Scene::LossClose {
        ensure!(
            state.frontend_motion.is_some() && state.window.is_some(),
            "missing loss contraction"
        );
    }
    ensure!(
        state.events.len() <= 1024
            && state.events.windows(2).all(|pair| pair[0].id < pair[1].id)
            && state
                .events
                .last()
                .is_none_or(|event| event.id < state.next_event)
            && state.events.iter().all(|event| event.tick <= state.ticks),
        "invalid event journal"
    );
    ensure!(
        state
            .input
            .players
            .iter()
            .all(|input| input.held.len() <= 256
                && input.pressed.len() <= 256
                && input.released.len() <= 256),
        "invalid input snapshot"
    );
    Ok(())
}
fn window_valid(window: &crate::presentation::transitions::Window) -> Result<()> {
    ensure!(
        window.selector <= 1 && window.buffers.iter().all(|buffer| buffer.len() == 260),
        "invalid window snapshot"
    );
    Ok(())
}
fn fade_valid(fade: &crate::presentation::transitions::Fade) -> Result<()> {
    ensure!(
        fade.enabled_layers <= 31 && (!fade.primed || fade.animation_counter <= 32),
        "invalid fade snapshot"
    );
    Ok(())
}
fn motion_valid(motion: &crate::presentation::transitions::FrontendWindow) -> Result<()> {
    ensure!(motion.setup_waits <= 2, "invalid window setup waits");
    Ok(())
}
