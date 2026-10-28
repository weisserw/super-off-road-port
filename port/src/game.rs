//! Shared native game. Desktop, browser and headless drivers consume these APIs.
mod resources;
mod scheduler;
mod snapshots;
mod state;
use anyhow::{Context, Result};
use resources::Resources;
use romatlas_port::{
    Game,
    assets::{MemoryAssets, Rom},
    graphics::Frame,
    input::{InputState, MultiInputState, PlayerInput},
};
use state::{Scene, State};
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
use std::sync::Arc;
#[derive(Clone)]
pub struct Port {
    resources: Arc<Resources>,
    state: State,
    selected: Arc<crate::race::assets::RaceAssets>,
    terrain: Arc<crate::world::terrain::Terrain>,
    sound: crate::sound::synthesis::Sound,
    /// Sample once per loaded game. Reset/replay reuses this power-on history;
    /// snapshots retain it, while independent game launches receive fresh bytes.
    initial_random_history: crate::runtime::RandomState,
}
impl Port {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(rom: &Rom, overrides: Option<&Path>) -> Result<Self> {
        Self::load_seeded(
            rom,
            overrides,
            crate::runtime::RandomState::from_entropy()?.0,
        )
    }
    /// Deterministic headless setup; player-facing loading uses host entropy.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_seeded(
        rom: &Rom,
        overrides: Option<&Path>,
        history: [u8; crate::runtime::RNG_STATE_BYTES],
    ) -> Result<Self> {
        let memory = match overrides {
            Some(root) => {
                let manifests = crate::assets::MANIFESTS
                    .iter()
                    .copied()
                    .filter(|name| root.join(name).is_file())
                    .collect::<Vec<_>>();
                MemoryAssets::from_directory(root, &manifests)?
            }
            None => MemoryAssets::default(),
        };
        Self::load_assets_seeded(rom, &memory, history)
    }
    pub fn load_assets(rom: &Rom, overrides: &MemoryAssets) -> Result<Self> {
        Self::load_assets_seeded(
            rom,
            overrides,
            crate::runtime::RandomState::from_entropy()?.0,
        )
    }
    /// Explicit initial history for reproducible tests and reference comparisons.
    /// Zero is intentionally permitted here, never chosen by normal loading.
    pub fn load_assets_seeded(
        rom: &Rom,
        overrides: &MemoryAssets,
        history: [u8; crate::runtime::RNG_STATE_BYTES],
    ) -> Result<Self> {
        let initial_random_history = crate::runtime::RandomState(history);
        let resources = Arc::new(Resources::load(rom, overrides)?);
        let selected = Arc::new(resources.race(0, 0)?);
        let terrain = Arc::new(crate::world::terrain::decode_terrain(
            &selected.track.terrain,
        )?);
        let sound = crate::sound::synthesis::Sound::new(resources.sound.clone())?;
        let mut game = Self {
            resources,
            state: State::cold(initial_random_history),
            selected,
            terrain,
            sound,
            initial_random_history,
        };
        game.boot()?;
        Ok(game)
    }
    pub fn observe(&self) -> serde_json::Value {
        let state = &self.state;
        let race = &state.race;
        let choice = &self.resources.sequence[state.sequence_index];
        serde_json::json!({"version":1,"implementation":"native","scene":format!("{:?}",state.scene),"simulation_ticks":state.ticks,
            "runtime":{"nmi_frames":state.nmi,"nmi_enabled":state.nmi_enabled,"animation_frame":race.progress.animation_frame,"audio_heartbeat":state.heartbeat,"random_history":state.random.0,"initial_random_history":self.initial_random_history.0},
            "progression":{"sequence_index":state.sequence_index,"sequence_count":self.resources.sequence.len(),"track":choice.id,"original_track":choice.original,"reversed":choice.reversed,"source_order":state.session.order_slot,"active_humans":state.session.active_humans,"winner_plus_one":race.progress.winner_plus_one,"records":state.session.records,"added_records":state.added_records,"wallets":state.session.wallets},
            "actors":race.trucks.iter().enumerate().map(|(slot,truck)|serde_json::json!({"slot":slot,"world":truck.world,"speed":truck.speed,"heading":truck.heading,"controller":truck.controller.0,"nitro":truck.nitro.encoded(),"upgrades":truck.upgrades,"wheels":truck.wheels,"progress":race.progress.drivers[slot],"contact_cooldown":truck.contact_cooldown,"effect_cursor":truck.contact_effect_player.encoded(),"nitro_cursor":truck.nitro_effect.encoded(),"surface_cursor":race.world.contacts[slot].player.encoded()})).collect::<Vec<_>>(),
            "timers":{"race":race.timer,"awards":state.postrace.as_ref().map(|value|&value.awards),"game_over":state.game_over_timer},
            "flags":{"inhibit_video":state.session.inhibit_video,"prepared":race.prepared,"fault":state.fault},
            "display":{"brightness":race.video.display.brightness,"force_blank":race.video.display.force_blank,"scroll":race.video.display.layers.each_ref().map(|layer|layer.scroll),"main_layers":race.video.display.main_layers,"sub_layers":race.video.display.sub_layers,"window_layers":race.video.display.masked_layers,"window_inverted_layers":race.video.display.window_inverted_layers},
            "frontend":{"splash":state.splash,"highscores":state.highscores},
            "transition":{"fade":state.fade,"race":state.transition,"pause":state.pause,"window":state.window,"menu":state.menu},
            "world":race.world,"retained_route_tail":race.route_tail_byte,"audio":self.sound.observe(),"events":state.events,"resources":self.resources.raw.resources.assets.len()})
    }
    fn fail(&mut self, error: anyhow::Error) {
        let message = format!("{error:#}");
        self.state
            .event("fault", serde_json::json!({"message":message}));
        self.state.fault = Some(message);
    }
}
impl Game for Port {
    fn display_aspect_ratio(&self) -> Option<(u32, u32)> {
        Some(crate::config::DISPLAY_ASPECT)
    }
    fn snapshot_json(&self) -> Result<String> {
        self.export_snapshot()
    }
    fn restore_snapshot_json(&mut self, json: &str) -> Result<()> {
        self.import_snapshot(json)
    }
    fn reset(&mut self) -> Result<()> {
        let mut next = self.clone();
        next.state = State::cold(self.initial_random_history);
        next.sound = crate::sound::synthesis::Sound::new(self.resources.sound.clone())?;
        next.selected = Arc::new(next.resources.race(0, 0)?);
        next.terrain = Arc::new(crate::world::terrain::decode_terrain(
            &next.selected.track.terrain,
        )?);
        next.boot()?;
        *self = next;
        Ok(())
    }
    fn tick(&mut self, input: &InputState) {
        let players = std::array::from_fn(|slot| {
            if slot == 0 {
                PlayerInput {
                    held: input.held.clone(),
                    pressed: input.pressed.clone(),
                    released: input.released.clone(),
                }
            } else {
                PlayerInput::default()
            }
        });
        self.tick_players(&MultiInputState {
            primary: input.clone(),
            players,
        });
    }
    fn tick_players(&mut self, input: &MultiInputState) {
        if self.state.fault.is_some() {
            return;
        }
        if let Err(error) = self.advance(input) {
            self.fail(error);
        }
    }
    fn render(&self, frame: &mut Frame) {
        match crate::presentation::render(
            &self.state.race.video.memory,
            &self.state.race.video.display,
        ) {
            Ok(pixels) if pixels.len() == frame.pixels.len() => {
                frame.pixels.copy_from_slice(&pixels)
            }
            _ => frame.clear([0, 0, 0, 255]),
        }
    }
    fn audio(&mut self, frames: usize, sample_rate: u32, samples: &mut Vec<f32>) {
        match self.sound.render_output(frames, sample_rate) {
            Ok(output) => samples.extend(output),
            Err(error) => {
                self.fail(error);
                samples.resize(samples.len() + frames * 2, 0.0);
            }
        }
    }
}
#[cfg(test)]
mod tests;

#[cfg(test)]
mod modernization_tests;

#[cfg(test)]
mod iteration_two_tests;
