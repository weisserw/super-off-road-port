//! Native source scheduling and actual owner calls at NMI boundaries.
use super::state::{Menu, ShopNext, SplashNext};
use super::*;
use crate::{
    controls,
    frontend::{attract, names, results, shop},
    presentation::{self, memory, sprites, transitions as t},
    race, runtime,
    sound::host,
    trucks::motion,
    world::{terrain, tracks},
};
use anyhow::ensure;
// These are source-visible frontend register encodings and VRAM word destinations.
const VIDEO_UPLOAD_INHIBITED: u8 = 0x80;
const LOSS_VISIBLE_LAYERS: u8 = 0x15;
const LOSS_TEXT_MAP: u16 = 0x4800;
const GAME_OVER_TEXT: u16 = 0x4b0b;
const EMPTY_RECORD_TIME: [u8; 2] = [9, 0x99];
const EMPTY_RECORD_GLYPH: u8 = 0x2a;

/// Whether a frontend return consumes the current NMI boundary. Keeping this
/// explicit prevents an extracted handler from adding/removing a visible wait.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SceneStep {
    Yield,
    Continue,
}

impl Port {
    pub(super) fn boot(&mut self) -> Result<()> {
        self.resources
            .poses
            .resolve(&mut self.state.race.video.poses)?;
        let mut boot = runtime::BootState {
            steering_upgrades: self
                .state
                .race
                .trucks
                .each_ref()
                .map(|truck| truck.upgrades.steering),
            speed_upgrades: self
                .state
                .race
                .trucks
                .each_ref()
                .map(|truck| truck.upgrades.speed),
            acceleration_upgrades: self
                .state
                .race
                .trucks
                .each_ref()
                .map(|truck| truck.upgrades.acceleration),
            suspension_upgrades: self
                .state
                .race
                .trucks
                .each_ref()
                .map(|truck| truck.upgrades.suspension),
            acceleration_fractions: self
                .state
                .race
                .trucks
                .each_ref()
                .map(|truck| truck.acceleration_fraction),
            terrain_corners: [0; 6],
            frontend_start_reentry: false,
            active_human_mask: 1,
            track_order_slot: 0,
            random: self.state.random,
            frontend_signature: self.state.session.signature,
        };
        for effect in runtime::prepare_boot(&mut boot) {
            use runtime::BootEffect::*;
            if effect == EnterFrontend {
                break;
            }
            match effect {
                ResetVideo => presentation::reset_display(&mut self.state.race.video.display),
                DisableInterrupts => self.state.nmi_enabled = false,
                UploadAudioDefinitions => {} // Sound::new already consumed both uploads and initialized the native driver.
                BlankAndInhibitVideo => {
                    self.state.session.inhibit_video = VIDEO_UPLOAD_INHIBITED;
                    self.state.race.video.display.force_blank = true;
                }
                ConfigureRaceLayers => {
                    presentation::configure_race(&mut self.state.race.video.display)
                }
                LoadFontAndClearBackground => memory::upload_fonts(
                    &mut self.state.race.video.memory,
                    &self.resources.shared,
                    false,
                )?,
                InitializeSpriteTransfer => {} // Explicit staged/visible native arrays replace the DMA channel configuration.
                FrontendAudioCommand => self.send_sound(attract::FRONTEND_AUDIO)?,
                EnterFrontend | SelectFirstTrack => {} // EnterFrontend below suspends; track selection occurs only after it returns.
            }
        }
        for slot in 0..4 {
            let truck = &mut self.state.race.trucks[slot];
            truck.upgrades.steering = boot.steering_upgrades[slot];
            truck.upgrades.speed = boot.speed_upgrades[slot];
            truck.upgrades.acceleration = boot.acceleration_upgrades[slot];
            truck.upgrades.suspension = boot.suspension_upgrades[slot];
            truck.acceleration_fraction = boot.acceleration_fractions[slot];
        }
        self.state.random = boot.random;
        self.enter_frontend()?;
        Ok(())
    }
    pub(super) fn send_sound(&mut self, command: u16) -> Result<()> {
        let [command_byte, argument] = command.to_le_bytes();
        self.sound.service_host(host::HostInput {
            heartbeat: self.state.heartbeat,
            flags: self.sound.host_flags,
            command: command_byte,
            argument,
        })?;
        self.state.event(
            "sound_requested",
            serde_json::json!({"command":command_byte,"argument":argument}),
        );
        Ok(())
    }
    fn flush_sounds(&mut self) -> Result<()> {
        for command in std::mem::take(&mut self.state.session.audio_commands) {
            self.send_sound(command)?;
        }
        for command in std::mem::take(&mut self.state.race.sounds) {
            self.send_sound(command)?;
        }
        if let Some(transition) = &mut self.state.transition {
            let commands = std::mem::take(&mut transition.audio_commands);
            for command in commands {
                self.send_sound(command)?;
            }
        }
        Ok(())
    }
    pub(super) fn sync_race_views(&mut self) {
        self.state.race.progress.active_humans = self.state.session.active_humans;
        self.state.race.video_inhibit = self.state.session.inhibit_video;
        self.state.race.enabled_layers = self.state.session.enabled_layers;
    }
    fn enter_frontend(&mut self) -> Result<()> {
        self.state.arcade = Default::default();
        self.state.session.active_humans = 1;
        self.state.session.order_slot = 0;
        self.state.session.inhibit_video = VIDEO_UPLOAD_INHIBITED;
        self.reclaim_window();
        if let Some(window) = &mut self.state.window {
            window.shape = t::ShapeId::Frontend;
        }
        attract::begin_entry(
            &mut self.state.session,
            &mut self.state.race.video.display,
            &self.resources.frontend,
        );
        self.state.nmi_enabled = true;
        self.state.panel = None;
        self.state.splash = None;
        self.state.highscores = None;
        self.state.transition = None;
        self.state.postrace = None;
        self.state.prompt = None;
        self.state.fade = None;
        self.state.frontend_motion = None;
        if self.state.session.phase == attract::EntryPhase::InitialLicense {
            self.state.splash_next = SplashNext::InitializeMoney;
            self.state.splash_cold = true;
            self.state.scene(Scene::Splashes);
        } else {
            self.initialize_money()?;
        }
        self.sync_race_views();
        self.flush_sounds()
    }
    fn initialize_money(&mut self) -> Result<()> {
        attract::initialize_session(
            &mut self.state.session,
            &mut self.state.race.trucks,
            &mut self.state.race.video.display,
        );
        if self.state.session.phase == attract::EntryPhase::DriverSelection {
            self.begin_names();
        } else {
            let mut panel = attract::begin_panel(
                &mut self.state.session,
                &mut self.state.race.trucks,
                &self.resources.frontend,
                &mut self.state.race.video.memory,
                &mut self.state.race.video.display,
            )?;
            if let Some(mut window) = self.state.window.take() {
                panel.motion = t::begin_frontend(
                    t::FrontendKind::Scrolling,
                    &mut window,
                    &mut self.state.race.video.display,
                );
                panel.motion.abort_enabled = 1;
                panel.window = window;
            }
            self.state.panel = Some(panel);
            self.state.scene(Scene::AttractPanel);
        }
        Ok(())
    }
    pub(super) fn begin_names(&mut self) {
        if !self.state.arcade.accepted {
            self.reclaim_window();
            self.state.arcade.previous = self.state.automatic[0].0;
            self.state.scene(Scene::ArcadeMenu);
            return;
        }
        self.reclaim_window();
        let selection = self.take_selection();
        self.state.menu = Menu::Names(selection);
        names::begin_selection(&mut self.state.session, self.state.menu.selection_mut());
        self.state.scene(Scene::Names);
    }
    fn take_selection(&mut self) -> names::Selection {
        match std::mem::replace(&mut self.state.menu, Menu::Idle(state::selection())) {
            Menu::Idle(selection) | Menu::Names(selection) => selection,
            Menu::Shop(shop) => shop.selection,
        }
    }
    fn begin_shop(&mut self, next: ShopNext) -> Result<()> {
        self.reclaim_window();
        let mut selection = self.take_selection();
        selection.page = 0;
        selection.editors[1].button_history = self.state.pause.owner_plus_one;
        self.state.pause.owner_plus_one = 0;
        self.state.menu = Menu::Shop(shop::Shop {
            selection,
            redraw_field: self.state.shop_redraw_field,
            phase: shop::Phase::Setup,
            fade: None,
            window: None,
            window_motion: None,
        });
        self.state.shop_next = next;
        self.state.scene(Scene::Shop);
        let Menu::Shop(shop) = &mut self.state.menu else {
            unreachable!()
        };
        shop::install_page(
            shop,
            &mut self.state.session,
            &self.state.race.trucks,
            &self.resources.shop,
            &self.resources.names,
            &self.resources.shared,
            &mut self.state.race.video.memory,
            &mut self.state.race.video.display,
        )?;
        Ok(())
    }
    fn begin_highscores(&mut self) -> Result<()> {
        self.reclaim_window();
        if let Some(window) = &mut self.state.window {
            window.hdma_enabled = false;
        }
        self.state.race.video.display.force_blank = true;
        names::install_driver_panels(
            &self.resources.names,
            0,
            0,
            true,
            &mut self.state.race.video.memory,
            &mut self.state.race.video.display,
        )?;
        let map = &self.resources.names.heading;
        memory::draw_rectangle(
            &mut self.state.race.video.memory,
            &map.words,
            map.destination_word,
            &map.rectangle,
        )?;
        memory::upload_fonts(
            &mut self.state.race.video.memory,
            &self.resources.shared,
            true,
        )?;
        results::draw_highscores(
            &self.resources.results,
            &self.state.session.records,
            &mut self.state.race.video.memory,
        )?;
        self.state.race.video.display.layers[2].scroll[1] = attract::HIGHSCORE_INITIAL_SCROLL;
        self.start_fade(t::FadeDirection::In, self.state.session.enabled_layers);
        self.state.scene(Scene::HighscoreFade);
        Ok(())
    }
    fn start_fade(&mut self, direction: t::FadeDirection, layers: u8) {
        self.state.fade = Some(t::Fade {
            direction,
            primed: false,
            complete: false,
            animation_counter: self.state.race.progress.animation_frame,
            enabled_layers: layers,
        });
    }
    fn fade_frame(&mut self) -> Result<bool> {
        let fade = self.state.fade.as_mut().context("missing active fade")?;
        t::advance_fade(fade, &mut self.state.race.video.display);
        self.state.race.progress.animation_frame = fade.animation_counter;
        let done = fade.complete;
        if done {
            self.state.fade = None;
        }
        Ok(done)
    }
    fn reclaim_window(&mut self) {
        if let Some(panel) = self.state.panel.take() {
            self.state.window = Some(panel.window);
        }
        if let Some(names::Sequence::Window { window, .. }) =
            self.state.menu.selection_mut().sequence.take()
        {
            self.state.window = Some(window);
        }
        if let Menu::Shop(shop) = &mut self.state.menu {
            if let Some(window) = shop.window.take() {
                self.state.window = Some(window);
            }
            shop.window_motion = None;
        }
        if self.state.window.is_none() {
            self.state.window = Some(names::new_window());
        }
    }
    fn start_frontend_window(&mut self, kind: t::FrontendKind) -> Result<()> {
        self.reclaim_window();
        let window = self
            .state
            .window
            .as_mut()
            .context("missing source window")?;
        self.state.frontend_motion = Some(t::begin_frontend(
            kind,
            window,
            &mut self.state.race.video.display,
        ));
        Ok(())
    }
    fn frontend_window_frame(&mut self) -> Result<bool> {
        let motion = self
            .state
            .frontend_motion
            .as_mut()
            .context("missing frontend geometry")?;
        t::advance_frontend(
            motion,
            self.state
                .window
                .as_mut()
                .context("missing frontend window")?,
            &self.resources.transitions.shapes[0],
            &mut self.state.race.video.display,
            self.state.automatic,
        )?;
        Ok(motion.complete)
    }
    fn record_key(&self) -> results::RecordKey {
        let choice = &self.resources.sequence[self.state.sequence_index];
        match choice.original {
            Some(id) => results::RecordKey::Original(id),
            None => results::RecordKey::Added(choice.id.clone()),
        }
    }
    fn prepare_race(&mut self) -> Result<()> {
        self.reclaim_window();
        self.selected = Arc::new(self.resources.arcade_race(&self.state)?);
        self.terrain = Arc::new(terrain::decode_terrain(&self.selected.track.terrain)?);
        if let results::RecordKey::Added(id) = self.record_key() {
            self.state
                .added_records
                .0
                .entry(id)
                .or_insert(results::TrackRecord {
                    time: EMPTY_RECORD_TIME,
                    name: [EMPTY_RECORD_GLYPH; 3],
                });
        }
        self.sync_race_views();
        self.state.pause.owner_plus_one =
            std::mem::take(&mut self.state.menu.selection_mut().editors[1].button_history);
        self.state.pause.phase = t::PausePhase::Running;
        let selected = self.selected.clone();
        let resources = self.resources.clone();
        let terrain = self.terrain.clone();
        let direction = usize::from(selected.reversed);
        let route = &selected.track.routes[direction];
        let start = route.grid_origin()?;
        let grid = tracks::prepare_grid(
            start,
            selected.track.headings[direction],
            self.state.session.order_slot,
        );
        let environment = self.frame_environment(&resources, &selected, &terrain, true);
        race::prepare_selected_race(
            &mut self.state.race,
            &environment,
            grid,
            &mut self.state.random,
        )?;
        self.state.session.inhibit_video = self.state.race.video_inhibit;
        self.state.session.enabled_layers = self.state.race.enabled_layers;
        self.state.fade = match &self.state.race.entry {
            race::RaceEntry::FadeIn(fade) => Some(fade.clone()),
            _ => None,
        };
        self.state.race.entry = race::RaceEntry::Running;
        if let Some(window) = &mut self.state.window {
            window.hdma_enabled = false;
        }
        self.state.nmi_enabled = true;
        self.state.scene(Scene::RaceFade);
        self.state.event("race_prepared",serde_json::json!({"track":self.resources.sequence[self.state.sequence_index].id,"reversed":selected.reversed}));
        Ok(())
    }
    fn frame_environment<'a>(
        &self,
        resources: &'a Resources,
        selected: &'a race::assets::RaceAssets,
        terrain: &'a terrain::Terrain,
        prepared: bool,
    ) -> race::FrameEnvironment<'a> {
        race::FrameEnvironment {
            assets: selected,
            motion: motion::MotionEnvironment {
                tables: &resources.motion,
                terrain,
                sampling: &resources.sampling,
                projection: terrain::Projection {
                    offset: selected.track.projection_offset,
                    scale: selected.track.projection_scale,
                },
                original_track: self.resources.sequence[self.state.sequence_index]
                    .original
                    .unwrap_or(0xff),
                active_humans: self.state.session.active_humans,
                prepared,
                animation: self.state.race.progress.animation_frame,
            },
            route: &selected.track.routes[usize::from(selected.reversed)],
            ai: &resources.ai,
            banners: &resources.banners,
            automatic: self.state.automatic,
        }
    }
    fn begin_race_window(&mut self, kind: t::RaceTransitionKind) -> Result<()> {
        self.reclaim_window();
        let mut transition = t::RaceTransition {
            kind,
            phase: t::RacePhase::BeginWait,
            remaining_waits: 0,
            hold_timer: 0,
            countdown: Default::default(),
            audio_commands: vec![],
            banner_calls: 0,
            animation_calls: 0,
        };
        let gate = u16::from(self.state.session.active_humans);
        t::begin_race_transition(
            kind,
            gate,
            &mut transition,
            self.state.window.as_mut().unwrap(),
            self.selected.presentation.countdown,
            &mut self.state.race.video.display,
            &mut self.state.session.inhibit_video,
        );
        // Source $077D is one word shared by countdown drawing and banner service.
        self.state.race.progress.banner_state = transition.countdown.state;
        self.state.transition = Some(transition);
        Ok(())
    }
    fn race_window_frame(&mut self) -> Result<bool> {
        let transition = self
            .state
            .transition
            .as_mut()
            .context("missing race window continuation")?;
        // Countdown is a continuation view of the canonical source $077D word.
        transition.countdown.state = self.state.race.progress.banner_state;
        let before = (transition.banner_calls, transition.animation_calls);
        t::advance_race_transition(
            transition,
            self.state.window.as_mut().context("missing race window")?,
            self.selected.presentation.countdown,
            &self.resources.transitions,
            &mut self.state.race.video.memory,
            &mut self.state.race.video.display,
            &mut self.state.session.inhibit_video,
        )?;
        self.state.race.progress.banner_state = transition.countdown.state;
        let calls = (
            transition.banner_calls.wrapping_sub(before.0),
            transition.animation_calls.wrapping_sub(before.1),
        );
        let done = transition.phase == t::RacePhase::Complete;
        if !self.state.window.as_ref().unwrap().hdma_enabled {
            self.state
                .race
                .video
                .display
                .window_edges
                .fill(t::CLOSED_EDGE);
        }
        race::service_transition_visuals(
            &mut self.state.race,
            &self.selected,
            &self.resources.banners,
            calls.0,
            calls.1,
        )?;
        self.state.transition.as_mut().unwrap().countdown.state =
            self.state.race.progress.banner_state;
        Ok(done)
    }
    fn postrace_owner(&mut self, complete: bool) -> Result<()> {
        let key = self.record_key();
        let state = &mut self.state;
        results::advance_postrace(
            state.race.timer,
            &key,
            &mut state.added_records,
            state.postrace.as_mut().context("missing postrace owner")?,
            &mut state.session,
            &mut state.race.progress,
            &mut state.race.trucks,
            &mut state.race.world.pending_awards,
            &self.resources.results,
            &mut state.race.video.memory,
            &mut state.race.video.display,
            complete,
        )
    }
    fn begin_loss_display(&mut self) -> Result<()> {
        let loser = self
            .state
            .postrace
            .as_ref()
            .context("missing loser rank")?
            .rank[3];
        if let Some(window) = &mut self.state.window {
            window.hdma_enabled = false;
        }
        let video = &mut self.state.race.video;
        sprites::hide_frontend(&mut video.sprites, &mut video.memory)?;
        memory::upload_fonts(&mut video.memory, &self.resources.shared, true)?;
        memory::upload_text_palette(&mut video.memory, &self.resources.shared)?;
        memory::upload_cursor(&mut video.memory, &self.resources.shared)?;
        memory::fill_map(&mut video.memory, LOSS_TEXT_MAP, 0);
        if self.state.session.active_humans != 0 {
            let prompt = results::ContinuePrompt {
                choice: 0,
                loser,
                dismissed: false,
            };
            results::stage_continue(
                &prompt,
                &self.resources.results,
                &self.resources.shop.cursor_tiles,
                &mut video.memory,
            )?;
            self.state.prompt = Some(prompt);
        } else {
            memory::draw_text(
                &mut video.memory,
                &self.resources.results.game_over_text,
                GAME_OVER_TEXT,
                memory::TextSize::Paired,
            )?;
        }
        self.state.scene(Scene::LossShowWait);
        Ok(())
    }
    // These scene steps preserve the same window/history owners and service
    // order as their domain calls. Yield ends this NMI's scene pump; Continue
    // dispatches the next scene before the common view/sound flush.
    fn advance_names_scene(&mut self, resources: &Resources) -> Result<SceneStep> {
        let Menu::Names(selection) = &mut self.state.menu else {
            anyhow::bail!("name scene missing owner");
        };
        let previous = selection.phase;
        names::advance(
            selection,
            &mut self.state.session,
            &resources.names,
            &resources.shared,
            &resources.directions,
            &mut self.state.race.video.memory,
            &mut self.state.race.video.display,
            &mut self.state.race.video.sprites,
            self.state.automatic,
            self.state.race.progress.animation_frame,
        )?;
        if previous == names::Phase::FinalWait {
            let mut window = self
                .state
                .window
                .take()
                .context("name contraction lost source window")?;
            let motion = t::begin_frontend(
                t::FrontendKind::Contracting,
                &mut window,
                &mut self.state.race.video.display,
            );
            selection.sequence = Some(names::Sequence::Window { window, motion });
        }
        if matches!(previous, names::Phase::EntryFade | names::Phase::FadeIn)
            && let Some(names::Sequence::Fade(fade)) = &selection.sequence
        {
            self.state.race.progress.animation_frame = fade.animation_counter;
        }
        if selection.phase == names::Phase::Ack {
            if let Some(names::Sequence::Window { window, .. }) = selection.sequence.take() {
                self.state.window = Some(window);
            }
            names::finish_page(selection, &mut self.state.session);
        }
        if selection.phase == names::Phase::Complete {
            let history = selection.editors[1].button_history;
            self.state.pause.owner_plus_one = history;
            self.begin_shop(ShopNext::FirstRace)?;
        }
        Ok(SceneStep::Yield)
    }
    fn advance_shop_scene(&mut self, resources: &Resources) -> Result<SceneStep> {
        let Menu::Shop(shop) = &mut self.state.menu else {
            anyhow::bail!("shop scene missing owner");
        };
        let previous = shop.phase;
        shop::advance(
            shop,
            &mut self.state.session,
            &mut self.state.race.trucks,
            &resources.shop,
            &resources.names,
            &resources.shared,
            &resources.directions,
            &mut self.state.race.video.memory,
            &mut self.state.race.video.display,
            self.state.automatic,
        )?;
        if previous == shop::Phase::FinalWait {
            let mut window = self
                .state
                .window
                .take()
                .context("shop expansion lost source window")?;
            let motion = t::begin_frontend(
                t::FrontendKind::Expanding,
                &mut window,
                &mut self.state.race.video.display,
            );
            shop.window = Some(window);
            shop.window_motion = Some(motion);
        }
        if previous == shop::Phase::FadeIn
            && let Some(fade) = &shop.fade
        {
            self.state.race.progress.animation_frame = fade.animation_counter;
        }
        if shop.phase == shop::Phase::Ack {
            self.state.window = shop.window.take();
            shop::finish_page(shop, &mut self.state.session);
        }
        if shop.phase == shop::Phase::Complete {
            self.state.shop_redraw_field = shop.redraw_field;
            let selection = self.take_selection();
            self.state.menu = Menu::Idle(selection);
            if self.state.shop_next == ShopNext::FirstRace {
                attract::finish_driver_selection(
                    &mut self.state.session,
                    &mut self.state.race.trucks,
                );
                if self.state.session.phase == attract::EntryPhase::RestartEntry {
                    self.enter_frontend()?;
                    return Ok(SceneStep::Yield);
                }
                self.state.race.video.display.force_blank = true;
                self.state.race.video.display.masked_layers = 0;
                presentation::reset_display(&mut self.state.race.video.display);
                self.state.sequence_index = self.state.arcade.order.first().copied().unwrap_or(0);
                self.state.arcade.roll(&mut self.state.random);
                self.state.scene(Scene::RacePrepare);
                return Ok(SceneStep::Continue);
            }
            self.postrace_owner(true)?;
            self.postrace_owner(false)?;
            self.finish_postrace()?;
            return Ok(SceneStep::Continue);
        }
        Ok(SceneStep::Yield)
    }
    pub(super) fn advance(&mut self, input: &MultiInputState) -> Result<()> {
        self.state.ticks = self.state.ticks.wrapping_add(1);
        self.state.input = input.clone();
        self.state.automatic = controls::encode_players(input);
        if !self.state.nmi_enabled {
            return Ok(());
        }
        let mut counters = runtime::FrameCounters {
            nmi: self.state.nmi,
            animation: self.state.race.progress.animation_frame,
            audio_heartbeat: self.state.heartbeat,
        };
        for effect in
            runtime::service_frame_boundary(&mut counters, self.state.session.inhibit_video)
        {
            match effect {
                runtime::FrameEffect::UploadStagedObjects => {
                    self.state.race.video.memory.visible_oam =
                        self.state.race.video.memory.staged_oam
                }
                runtime::FrameEffect::UploadStagedTruckPoses => self.resources.poses.present(
                    &mut self.state.race.video.memory,
                    &self.state.race.video.poses,
                )?,
                runtime::FrameEffect::CountersAdvanced(counters) => {
                    self.state.nmi = counters.nmi;
                    self.state.heartbeat = counters.audio_heartbeat;
                    self.state.race.progress.animation_frame = counters.animation;
                }
                runtime::FrameEffect::SendAudioHeartbeat(heartbeat) => {
                    self.sound.service_host(host::HostInput {
                        heartbeat,
                        flags: self.sound.host_flags,
                        ..Default::default()
                    })?
                }
                runtime::FrameEffect::SelectHudTransferDestination
                | runtime::FrameEffect::ControllerPollCompleted => {} // Native indexed drawing and the captured automatic words replace hardware latches.
            }
        }
        let resources = self.resources.clone();
        for _ in 0..32 {
            match self.state.scene {
                Scene::FrontendEntry => {
                    self.enter_frontend()?;
                    break;
                }
                Scene::Splashes => {
                    if self.state.splash.is_none() {
                        let splash = attract::begin_splashes(
                            &resources.frontend,
                            &mut self.state.race.video.memory,
                            &mut self.state.race.video.display,
                            &mut self.state.session.inhibit_video,
                        )?;
                        if let Some(window) = &mut self.state.window {
                            window.hdma_enabled = false;
                        }
                        self.state.session.enabled_layers = 1;
                        self.state.splash = Some(splash);
                        break;
                    }
                    let splash = self.state.splash.as_mut().unwrap();
                    let previous = splash.phase;
                    attract::advance_splashes(
                        splash,
                        &mut self.state.session.signature,
                        &resources.frontend,
                        &mut self.state.race.video.memory,
                        &mut self.state.race.video.display,
                        self.state.automatic,
                    )?;
                    if matches!(
                        previous,
                        attract::SplashPhase::FadeInLicense
                            | attract::SplashPhase::FadeOutLicense
                            | attract::SplashPhase::FadeInSoftware
                            | attract::SplashPhase::FadeOutSoftware
                    ) {
                        self.state.race.progress.animation_frame = splash.fade.animation_counter;
                    }
                    match splash.phase {
                        attract::SplashPhase::Reentry => {
                            self.state.splash = None;
                            self.begin_names();
                        }
                        attract::SplashPhase::Done => {
                            self.state.splash = None;
                            if self.state.splash_next == SplashNext::InitializeMoney {
                                attract::finish_initial_splashes(
                                    &mut self.state.session,
                                    &resources.results.defaults,
                                );
                                self.initialize_money()?;
                            } else {
                                attract::finish_attract_presentation(
                                    &mut self.state.session,
                                    &mut self.state.race.trucks,
                                    &mut self.state.race.video.display,
                                    &resources.frontend,
                                );
                                self.state.sequence_index =
                                    usize::from(self.state.session.order_slot)
                                        % resources.sequence.len();
                                presentation::reset_display(&mut self.state.race.video.display);
                                self.state.scene(Scene::RacePrepare);
                                continue;
                            }
                        }
                        _ => {}
                    }
                    break;
                }
                Scene::AttractPanel => {
                    let panel = self.state.panel.as_mut().context("missing attract panel")?;
                    attract::advance_panel(
                        panel,
                        &mut self.state.session,
                        &resources.transitions.shapes[0],
                        &mut self.state.race.video.display,
                        self.state.automatic,
                    )?;
                    match panel.phase {
                        attract::PanelPhase::Reentry => self.begin_names(),
                        attract::PanelPhase::Complete => self.begin_highscores()?,
                        _ => {}
                    }
                    break;
                }
                Scene::HighscoreFade => {
                    if self.fade_frame()? {
                        let mut highscores = attract::Highscores {
                            scroll: 0,
                            timer: 0,
                            finished: false,
                            reentry: false,
                        };
                        attract::begin_highscores(
                            &mut highscores,
                            &mut self.state.race.video.display,
                        );
                        self.state.highscores = Some(highscores);
                        self.state.scene(Scene::Highscores);
                    }
                    break;
                }
                Scene::Highscores => {
                    let highscores = self
                        .state
                        .highscores
                        .as_mut()
                        .context("missing highscore scroll")?;
                    attract::advance_highscores(
                        highscores,
                        self.state.session.signature[0],
                        &mut self.state.race.video.display,
                        self.state.automatic,
                    );
                    if highscores.reentry {
                        self.begin_names();
                    } else if highscores.finished {
                        self.start_frontend_window(t::FrontendKind::Contracting)?;
                        if let Some(motion) = &mut self.state.frontend_motion {
                            motion.abort_enabled = 1;
                        }
                        self.state.scene(Scene::HighscoreClose);
                    }
                    break;
                }
                Scene::HighscoreClose => {
                    if self.frontend_window_frame()? {
                        let timer = &mut self.state.highscores.as_mut().unwrap().timer;
                        if attract::poll_start(
                            timer,
                            self.state.session.signature[0],
                            self.state.automatic,
                        ) == attract::Poll::StartReentry
                        {
                            self.begin_names();
                        } else {
                            self.state.splash = None;
                            self.state.splash_next = SplashNext::Race;
                            self.state.splash_cold = false;
                            self.state.scene(Scene::Splashes);
                        }
                    }
                    break;
                }
                Scene::ArcadeMenu => {
                    let accepted = self.state.arcade.input(self.state.automatic[0].0);
                    crate::frontend::arcade::draw(
                        &mut self.state.race.video.memory,
                        &mut self.state.race.video.display,
                        &resources.shared,
                        self.state.arcade.random,
                    )?;
                    self.state.session.inhibit_video = VIDEO_UPLOAD_INHIBITED;
                    if accepted {
                        self.state
                            .arcade
                            .start(&resources.sequence, &mut self.state.random);
                        self.begin_names();
                    }
                    break;
                }
                Scene::Names => match self.advance_names_scene(&resources)? {
                    SceneStep::Yield => break,
                    SceneStep::Continue => continue,
                },
                Scene::Shop => match self.advance_shop_scene(&resources)? {
                    SceneStep::Yield => break,
                    SceneStep::Continue => continue,
                },
                Scene::RacePrepare => {
                    self.prepare_race()?;
                    break;
                }
                Scene::RaceFade => {
                    if self.fade_frame()? {
                        self.state.transition = None;
                        self.state.scene(Scene::Ready);
                    }
                    break;
                }
                Scene::Ready => {
                    if self.state.transition.is_none() {
                        self.begin_race_window(t::RaceTransitionKind::Ready)?;
                    }
                    if self.race_window_frame()? {
                        self.state.transition = None;
                        self.state.scene(Scene::MusicWait);
                    }
                    break;
                }
                Scene::MusicWait => {
                    if let Some(command) = race::race_music_command(
                        self.state.session.active_humans,
                        self.state.session.order_slot,
                    ) {
                        self.send_sound(command)?;
                    }
                    self.state.scene(Scene::Racing);
                    break;
                }
                Scene::Racing => {
                    match race::poll_race_start(
                        self.state.session.active_humans,
                        self.state.automatic,
                        &mut self.state.session.start_reentry,
                    ) {
                        race::InputDispatch::FrontendRestart => {
                            self.state.menu.selection_mut().editors[1].button_history =
                                std::mem::take(&mut self.state.pause.owner_plus_one);
                            self.enter_frontend()?;
                            break;
                        }
                        race::InputDispatch::HumanPauseService => {
                            // The source aliases menu history and pause owner. A retained
                            // history outside1..4 addresses unestablished I/O; native
                            // power-on policy treats that unsupported owner read released.
                            if self.state.pause.phase == t::PausePhase::Running
                                && self.state.pause.owner_plus_one > 4
                            {
                                self.state.event(
                                    "retained_history_release",
                                    serde_json::json!({"history":self.state.pause.owner_plus_one}),
                                );
                                self.state.pause.owner_plus_one = 0;
                            }
                            let phase = self.state.pause.phase;
                            t::service_pause_frame(
                                &mut self.state.pause,
                                self.state.session.joined_pages,
                                self.state
                                    .window
                                    .as_mut()
                                    .context("pause lost source window")?,
                                self.selected.presentation.countdown,
                                &resources.transitions,
                                &mut self.state.race.video.memory,
                                &mut self.state.race.video.display,
                                &mut self.state.session.inhibit_video,
                                self.state.automatic,
                            )?;
                            if self.state.pause.phase != t::PausePhase::Running {
                                if phase == t::PausePhase::Running {
                                    self.state.event("paused",serde_json::json!({"owner":self.state.pause.owner_plus_one}));
                                }
                                break;
                            }
                            if phase != t::PausePhase::Running {
                                self.state
                                    .race
                                    .video
                                    .display
                                    .window_edges
                                    .fill(t::CLOSED_EDGE);
                                self.state.event(
                                    "resumed",
                                    serde_json::json!({"owner":self.state.pause.owner_plus_one}),
                                );
                            }
                        }
                        _ => {}
                    }
                    self.sync_race_views();
                    let selected = self.selected.clone();
                    let terrain = self.terrain.clone();
                    let environment =
                        self.frame_environment(&resources, &selected, &terrain, false);
                    let (exit, _) = race::update_frame_services(
                        &mut self.state.race,
                        &environment,
                        &mut self.state.random,
                    )?;
                    match exit {
                        race::FrameExit::HumanPostrace => {
                            let route = self.selected.track.routes
                                [usize::from(self.selected.reversed)]
                            .clone();
                            let mut postrace = results::Postrace {
                                route_count: route.checkpoint_count,
                                route: Some(route),
                                loss_route: None,
                                phase: results::PostracePhase::Complete,
                                rank: [0, 1, 2, 3],
                                awards: results::Awards {
                                    timer: 0,
                                    new_record_pending: 0,
                                    phase: results::AwardsFrame::Waiting,
                                },
                            };
                            results::begin_postrace(
                                &self.state.race.trucks,
                                &mut postrace,
                                &mut self.state.session,
                                &mut self.state.race.progress,
                                &mut self.state.race.timer,
                                &mut self.state.race.video.memory,
                                &mut self.state.race.video.display,
                            );
                            self.state.event("race_finished",serde_json::json!({"winner":self.state.race.progress.winner_plus_one}));
                            self.state.postrace = Some(postrace);
                            self.state.scene(Scene::Postrace);
                        }
                        race::FrameExit::AttractRestart => {
                            self.start_fade(
                                t::FadeDirection::Out,
                                self.state.session.enabled_layers,
                            );
                            self.state.scene(Scene::AttractFade);
                        }
                        _ => {}
                    }
                    break;
                }
                Scene::AttractFade => {
                    if self.fade_frame()? {
                        self.enter_frontend()?;
                    }
                    break;
                }
                Scene::Postrace => match self.advance_postrace_scene(&resources)? {
                    SceneStep::Yield => break,
                    SceneStep::Continue => continue,
                },
                Scene::LossSetup
                | Scene::LossShowWait
                | Scene::LossFade
                | Scene::LossPrompt
                | Scene::LossClose
                | Scene::LossRestart => match self.advance_loss_scene()? {
                    SceneStep::Yield => break,
                    SceneStep::Continue => continue,
                },
            }
        }
        self.sync_race_views();
        self.flush_sounds()?;
        Ok(())
    }
    fn advance_postrace_scene(&mut self, resources: &Resources) -> Result<SceneStep> {
        let phase = self
            .state
            .postrace
            .as_ref()
            .context("missing postrace")?
            .phase;
        match phase {
            results::PostracePhase::HudWait => {
                if let Some(window) = &mut self.state.window {
                    window.hdma_enabled = false;
                }
                self.postrace_owner(false)?;
                self.state.transition = None;
            }
            results::PostracePhase::Window => {
                if self.state.transition.is_none() {
                    self.begin_race_window(t::RaceTransitionKind::Postrace)?;
                }
                if self.race_window_frame()? {
                    self.state.transition = None;
                    self.postrace_owner(true)?;
                    self.start_fade(t::FadeDirection::Out, self.state.session.enabled_layers);
                }
            }
            results::PostracePhase::FadeOut => {
                if self.fade_frame()? {
                    self.postrace_owner(true)?;
                    return Ok(SceneStep::Continue);
                }
            }
            results::PostracePhase::PodiumSetup => {
                if let Some(window) = &mut self.state.window {
                    window.hdma_enabled = false;
                }
                memory::upload_fonts(&mut self.state.race.video.memory, &resources.shared, false)?;
                self.postrace_owner(false)?;
                self.start_fade(t::FadeDirection::In, 5);
            }
            results::PostracePhase::FadeIn => {
                if self.fade_frame()? {
                    self.postrace_owner(true)?;
                }
            }
            results::PostracePhase::Awards | results::PostracePhase::RecordWait => {
                let key = self.record_key();
                let postrace = self.state.postrace.as_mut().unwrap();
                if phase == results::PostracePhase::Awards {
                    results::advance_awards(
                        &mut postrace.awards,
                        &mut self.state.session,
                        &mut self.state.added_records,
                        &key,
                        self.state.race.timer,
                        postrace.rank,
                        self.state.race.progress.animation_frame,
                        self.state.automatic,
                        &mut self.state.race.video.memory,
                    )?;
                } else {
                    results::finish_record_wait(
                        &mut postrace.awards,
                        &self.state.session,
                        &self.state.added_records,
                        &key,
                        self.state.race.timer,
                        postrace.rank,
                        self.state.automatic,
                        &mut self.state.race.video.memory,
                    )?;
                }
                self.postrace_owner(false)?;
                if self.state.postrace.as_ref().unwrap().phase == results::PostracePhase::Contract {
                    self.start_frontend_window(t::FrontendKind::Contracting)?;
                }
            }
            results::PostracePhase::Contract => {
                if self.frontend_window_frame()? {
                    self.state.frontend_motion = None;
                    self.postrace_owner(true)?;
                    return Ok(SceneStep::Continue);
                }
            }
            results::PostracePhase::LossPrelude => {
                self.postrace_owner(false)?;
                if let Some(window) = &mut self.state.window {
                    window.hdma_enabled = false;
                }
                self.state.scene(Scene::LossSetup);
            }
            results::PostracePhase::Shop => {
                self.begin_shop(ShopNext::Postrace)?;
            }
            results::PostracePhase::ConvertInactive => {
                self.postrace_owner(false)?;
                self.finish_postrace()?;
                return Ok(SceneStep::Continue);
            }
            results::PostracePhase::Restart => {
                self.enter_frontend()?;
            }
            results::PostracePhase::Complete => {
                self.finish_postrace()?;
                return Ok(SceneStep::Continue);
            }
            results::PostracePhase::LossPrompt | results::PostracePhase::LossUploadWait => {
                self.state.scene(Scene::LossSetup)
            }
        }
        Ok(SceneStep::Yield)
    }

    fn advance_loss_scene(&mut self) -> Result<SceneStep> {
        match self.state.scene {
            Scene::LossSetup => {
                self.postrace_owner(false)?;
                self.state.loss_restart =
                    self.state.postrace.as_ref().unwrap().phase == results::PostracePhase::Restart;
                self.begin_loss_display()?;
            }
            Scene::LossShowWait => {
                if let Some(prompt) = &mut self.state.prompt {
                    results::release_continue(
                        prompt,
                        &mut self.state.session,
                        &mut self.state.race.video.memory,
                        &mut self.state.race.video.display,
                    );
                } else {
                    self.state.session.inhibit_video = 0;
                    self.state.session.enabled_layers = LOSS_VISIBLE_LAYERS;
                }
                self.start_fade(t::FadeDirection::In, LOSS_VISIBLE_LAYERS);
                self.state.scene(Scene::LossFade);
            }
            Scene::LossFade => {
                if self.fade_frame()? {
                    self.state.game_over_timer = results::GAME_OVER_TIMEOUT;
                    self.state.scene(Scene::LossPrompt);
                }
            }
            Scene::LossPrompt => {
                let done = if self.state.session.active_humans != 0 {
                    let prompt = self
                        .state
                        .prompt
                        .as_mut()
                        .context("missing continuation prompt")?;
                    results::poll_continue(
                        prompt,
                        self.state.automatic,
                        &mut self.state.race.video.memory,
                    )?;
                    prompt.dismissed
                } else {
                    results::poll_game_over(&mut self.state.game_over_timer, self.state.automatic)
                };
                if done {
                    sprites::hide_frontend(
                        &mut self.state.race.video.sprites,
                        &mut self.state.race.video.memory,
                    )?;
                    self.start_frontend_window(t::FrontendKind::Contracting)?;
                    self.state.scene(Scene::LossClose);
                }
            }
            Scene::LossClose => {
                if self.frontend_window_frame()? {
                    if self.state.loss_restart {
                        self.enter_frontend()?;
                    } else {
                        let prompt = self
                            .state
                            .prompt
                            .as_ref()
                            .context("missing accepted continuation")?;
                        match results::resolve_choice(
                            &mut self.state.session,
                            prompt.loser,
                            prompt.choice,
                        )? {
                            results::LossRoute::RestartFrontend => self.enter_frontend()?,
                            _ => {
                                self.state.postrace.as_mut().unwrap().phase =
                                    results::PostracePhase::Shop;
                                self.begin_shop(ShopNext::Postrace)?;
                            }
                        }
                    }
                }
            }
            Scene::LossRestart => {
                self.enter_frontend()?;
            }
            _ => unreachable!("loss handler dispatched outside loss scenes"),
        }
        Ok(SceneStep::Yield)
    }

    fn finish_postrace(&mut self) -> Result<()> {
        ensure!(
            self.state
                .postrace
                .as_ref()
                .is_some_and(|postrace| postrace.phase == results::PostracePhase::Complete),
            "postrace returned before owners completed"
        );
        if self.state.arcade.random && self.state.arcade.accepted {
            self.state.arcade.cursor =
                (self.state.arcade.cursor + 1) % self.state.arcade.order.len();
            self.state.sequence_index = self.state.arcade.order[self.state.arcade.cursor];
            self.state.arcade.roll(&mut self.state.random);
        } else {
            self.state.sequence_index =
                (self.state.sequence_index + 1) % self.resources.sequence.len();
        }
        self.state.session.order_slot =
            self.state.session.order_slot.wrapping_add(1) & race::ORIGINAL_ORDER_MASK;
        self.state.postrace = None;
        self.state.prompt = None;
        self.state.scene(Scene::RacePrepare);
        Ok(())
    }
}

#[cfg(test)]
mod alias_tests {
    use super::*;

    #[test]
    fn ready_countdown_and_banner_share_source_word_in_call_order() {
        let rom = Rom::open(Path::new("../input/game.rom"), crate::config::ROM_SHA256).unwrap();
        let mut game = Port::load_assets(&rom, &MemoryAssets::default()).unwrap();
        game.begin_race_window(t::RaceTransitionKind::Ready)
            .unwrap();
        game.state.race.progress.banner_state = 13;
        let transition = game.state.transition.as_mut().unwrap();
        transition.countdown.state = 13;
        transition.countdown.delay = 0;
        transition.phase = t::RacePhase::CountToReady;
        game.race_window_frame().unwrap();
        // Source draw increments word $077D before the banner reads it; $077F
        // remains the independent delay word. No byte narrowing or tile override.
        assert_eq!(game.state.race.progress.banner_state, 14);
        assert_eq!(game.state.transition.as_ref().unwrap().countdown.state, 14);
        assert_eq!(
            game.state.race.banner.primary.tile_attributes,
            game.resources.banners.primary_tiles[14]
        );
    }
}
