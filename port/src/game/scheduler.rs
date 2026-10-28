//! Native source scheduling and actual owner calls at NMI boundaries.
use super::state::{Menu, ShopNext, SplashNext};
use super::*;
use crate::{
    controls,
    frontend::{attract, championship, names, results, shop},
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
        self.state.championship = Default::default();
        self.state.race.progress.championship = false;
        for truck in &mut self.state.race.trucks {
            truck.championship = false;
            truck.championship_speed_bonus = 0;
        }
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
            self.state.championship.previous = self.state.automatic[0].0;
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
        self.state.race.progress.championship = self.state.championship.active;
        if self.state.championship.active {
            self.state.championship.before_awards = self
                .state
                .session
                .wallets
                .each_ref()
                .map(championship::wallet);
            self.state.championship.prepare(
                &mut self.state.race.trucks,
                self.state.session.active_humans,
            );
        }
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
    fn advance_championship_menu(&mut self) -> Result<()> {
        use championship::Difficulty;
        let pressed = self.state.championship.pressed(self.state.automatic[0].0);
        let direction = if pressed & (controls::BUTTON_UP | controls::BUTTON_LEFT) != 0 {
            -1
        } else if pressed
            & (controls::BUTTON_DOWN | controls::BUTTON_RIGHT | controls::BUTTON_SELECT)
            != 0
        {
            1
        } else {
            0
        };
        let confirm =
            pressed & (controls::BUTTON_START | controls::BUTTON_A | controls::BUTTON_B) != 0;
        let season = &mut self.state.championship;
        let (title, choices, selected): (&str, Vec<String>, usize) = match self.state.scene {
            Scene::ArcadeMenu => {
                season.mode = (i16::from(season.mode) + direction).rem_euclid(3) as u8;
                (
                    "SELECT MODE",
                    vec![
                        "CHAMPIONSHIP".into(),
                        "ARCADE RANDOM".into(),
                        "ARCADE CLASSIC".into(),
                    ],
                    usize::from(season.mode),
                )
            }
            Scene::ChampionshipCup => {
                season.cup = (season.cup as i16 + direction).rem_euclid(4) as usize;
                (
                    "SELECT CHAMPIONSHIP",
                    crate::config::CHAMPIONSHIP_NAMES
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    season.cup,
                )
            }
            Scene::ChampionshipDifficulty => {
                let index = match season.difficulty {
                    Difficulty::Easy => 0,
                    Difficulty::Normal => 1,
                    Difficulty::Hard => 2,
                };
                let index = (index + direction).rem_euclid(3) as usize;
                season.difficulty = [Difficulty::Easy, Difficulty::Normal, Difficulty::Hard][index];
                (
                    "SELECT DIFFICULTY",
                    vec!["EASY".into(), "NORMAL".into(), "HARD".into()],
                    index,
                )
            }
            _ => unreachable!(),
        };
        let choice_width = if self.state.scene == Scene::ArcadeMenu {
            choices.iter().map(String::len).max().unwrap_or(0)
        } else {
            0
        };
        let mut lines = vec![(4, title.into())];
        lines.extend(
            choices
                .into_iter()
                .enumerate()
                .map(|(i, s)| (9 + i as u16 * 4, format!("{s:<choice_width$}"))),
        );
        championship::screen(
            &mut self.state.race.video.memory,
            &mut self.state.race.video.display,
            &self.resources.shared,
            &lines,
            Some(9 + selected as u16 * 4),
        )?;
        self.state.session.inhibit_video = VIDEO_UPLOAD_INHIBITED;
        if confirm {
            match self.state.scene {
                Scene::ArcadeMenu if season.mode == 0 => self.state.scene(Scene::ChampionshipCup),
                Scene::ArcadeMenu => {
                    self.state.arcade.accepted = true;
                    self.state.arcade.random = season.mode == 1;
                    self.state
                        .arcade
                        .start(&self.resources.sequence, &mut self.state.random);
                    self.begin_names();
                }
                Scene::ChampionshipCup => self.state.scene(Scene::ChampionshipDifficulty),
                Scene::ChampionshipDifficulty => {
                    season.start(&self.resources.sequence)?;
                    self.state.arcade.accepted = true;
                    self.state.arcade.random = false;
                    // Starting resources are independent of any attract demo.
                    shop::initialize_wallets(&mut self.state.session);
                    for truck in &mut self.state.race.trucks {
                        truck.upgrades = Default::default();
                        truck.nitro = Default::default();
                    }
                    self.begin_names();
                }
                _ => unreachable!(),
            }
        }
        Ok(())
    }
    fn begin_championship_standings(&mut self) {
        self.state.championship.previous = self.state.automatic[0].0 | self.state.automatic[1].0;
        self.state.scene(Scene::ChampionshipStandings);
    }
    fn draw_championship_standings(&mut self) -> Result<()> {
        let season = &self.state.championship;
        let final_screen = season.completed == 8;
        // Center the full visible table, including the truck and earnings header.
        let (truck_column, rank_column, initials_column, points_column, earnings_column) =
            if final_screen {
                (3, 6, 8, 13, 17)
            } else {
                (8, 11, 13, 18, 0)
            };
        let mut lines = vec![
            (
                1,
                if final_screen {
                    "FINAL STANDINGS".into()
                } else {
                    "CHAMPIONSHIP STANDINGS".into()
                },
            ),
            (3, format!("{} RACES REMAINING", 8 - season.completed)),
        ];
        if final_screen {
            let mut header = [b' '; 32];
            for (column, label) in [
                (initials_column, "NAME"),
                (points_column, "PTS"),
                (earnings_column, "TOTAL EARNED"),
            ] {
                header[column..column + label.len()].copy_from_slice(label.as_bytes());
            }
            lines.push((5, String::from_utf8(header.to_vec()).unwrap()));
        }
        for (place, slot) in season.standings().into_iter().enumerate() {
            let name: String = self.state.session.records.truck_names[slot * 3..slot * 3 + 3]
                .iter()
                .map(|&g| {
                    if (11..=36).contains(&g) {
                        (b'A' + g - 11) as char
                    } else if (1..=10).contains(&g) {
                        (b'0' + g - 1) as char
                    } else if g == 39 {
                        '/'
                    } else {
                        ' '
                    }
                })
                .collect();
            let mut line = [b' '; 32];
            line[rank_column] = b'1' + place as u8;
            line[initials_column..initials_column + 3].copy_from_slice(name.as_bytes());
            line[points_column..points_column + 2]
                .copy_from_slice(format!("{:2}", season.points[slot]).as_bytes());
            if final_screen {
                line[earnings_column..earnings_column + 12]
                    .copy_from_slice(format!("{:12}", season.earned[slot]).as_bytes());
            } else {
                line[points_column + 3..points_column + 6].copy_from_slice(b"PTS");
            }
            lines.push((
                7 + place as u16 * 3,
                String::from_utf8(line.to_vec()).unwrap(),
            ));
        }
        if final_screen && self.state.session.active_humans & (1 << season.standings()[0]) != 0 {
            let (first, rest) = crate::config::CHAMPIONSHIP_VICTORY.split_once(' ').unwrap();
            lines.extend([(20, first.into()), (22, rest.into())]);
        }
        lines.push((25, "PRESS START TO CONTINUE".into()));
        championship::screen(
            &mut self.state.race.video.memory,
            &mut self.state.race.video.display,
            &self.resources.shared,
            &lines,
            None,
        )?;
        if final_screen && self.state.session.active_humans & (1 << season.standings()[0]) != 0 {
            let (_, line) = crate::config::CHAMPIONSHIP_VICTORY.split_once(' ').unwrap();
            championship::draw_victory_line(&mut self.state.race.video.memory, 22, line)?;
        }
        championship::draw_cars(
            &mut self.state.race.video.memory,
            &mut self.state.race.video.display,
            &self.resources.shared,
            &self.resources.poses,
            self.resources.motion.body_poses.frame(2, 2, 3),
            truck_column * 8,
            season.standings(),
        )?;
        self.state.session.inhibit_video = VIDEO_UPLOAD_INHIBITED;
        Ok(())
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
            if self.state.championship.active {
                if self.state.session.joined_pages == [0; 2] {
                    self.enter_frontend()?;
                } else {
                    championship::restore_cpu_initials(
                        &mut self.state.session,
                        &resources.results.defaults.truck_names,
                    );
                    self.begin_championship_standings();
                }
            } else {
                self.begin_shop(ShopNext::FirstRace)?;
            }
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
                if self.state.championship.active {
                    // Championship has no free initial CPU nitros.
                    for slot in 0..4 {
                        if self.state.session.active_humans & (1 << slot) == 0 {
                            self.state.race.trucks[slot].nitro = Default::default();
                        }
                    }
                    self.state.championship.buy(
                        &mut self.state.session,
                        &mut self.state.race.trucks,
                        &resources.shop,
                    )?;
                    self.state.sequence_index = self.state.championship.schedule[0].0;
                } else {
                    self.state.arcade.roll(&mut self.state.random);
                }
                self.state.scene(Scene::RacePrepare);
                return Ok(SceneStep::Continue);
            }
            if self.state.championship.active {
                self.state.championship.buy(
                    &mut self.state.session,
                    &mut self.state.race.trucks,
                    &resources.shop,
                )?;
                self.state.postrace.as_mut().unwrap().phase = results::PostracePhase::Complete;
            } else {
                self.postrace_owner(true)?;
                self.postrace_owner(false)?;
            }
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
                Scene::ArcadeMenu | Scene::ChampionshipCup | Scene::ChampionshipDifficulty => {
                    self.advance_championship_menu()?;
                    break;
                }
                Scene::ChampionshipStandings => {
                    self.draw_championship_standings()?;
                    let pressed = self
                        .state
                        .championship
                        .pressed(self.state.automatic[0].0 | self.state.automatic[1].0);
                    if pressed & (controls::BUTTON_START | controls::BUTTON_A | controls::BUTTON_B)
                        != 0
                    {
                        if self.state.championship.completed == 8 {
                            self.state.session.start_reentry = 0;
                            self.enter_frontend()?;
                        } else {
                            self.begin_shop(if self.state.championship.completed == 0 {
                                ShopNext::FirstRace
                            } else {
                                ShopNext::Postrace
                            })?;
                        }
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
                    if self.state.championship.active {
                        let rank = self.state.postrace.as_ref().unwrap().rank;
                        self.state.championship.finish_race(
                            rank,
                            &self.state.session,
                            &mut self.state.race.trucks,
                        );
                        self.state.postrace.as_mut().unwrap().phase = results::PostracePhase::Shop;
                        self.begin_championship_standings();
                        return Ok(SceneStep::Yield);
                    }
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
        if self.state.championship.active {
            self.state.sequence_index =
                self.state.championship.schedule[self.state.championship.completed].0;
        } else if self.state.arcade.random && self.state.arcade.accepted {
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

#[cfg(test)]
mod championship_tests {
    use super::*;
    use crate::{
        controls::ControllerWord,
        frontend::championship::{Championship, Difficulty},
        trucks::ai,
    };
    fn game(humans: u8) -> Port {
        let rom = Rom::open(Path::new("../input/game.rom"), crate::config::ROM_SHA256).unwrap();
        let mut game =
            Port::load_assets_seeded(&rom, &MemoryAssets::default(), [1, 2, 3, 4, 5]).unwrap();
        game.state.session.records = game.resources.results.defaults.clone();
        game.state.session.signature = game.resources.frontend.signature;
        shop::initialize_wallets(&mut game.state.session);
        game.state.session.active_humans = humans;
        game.state.session.joined_pages = [humans, 0];
        game.state.arcade.accepted = true;
        game.state.arcade.random = false;
        game.state
            .championship
            .start(&game.resources.sequence)
            .unwrap();
        game.state.sequence_index = game.state.championship.schedule[0].0;
        game
    }
    #[test]
    fn championship_cpu_shop_spends_real_money_caps_nitros_and_restores_purchases() {
        for difficulty in [Difficulty::Easy, Difficulty::Normal, Difficulty::Hard] {
            let mut g = game(1);
            g.state.championship.difficulty = difficulty;
            g.state.session.wallets = [[0, 0, 8, 0, 0, 0]; 4];
            let human = g.state.race.trucks[0].clone();
            g.state
                .championship
                .buy(
                    &mut g.state.session,
                    &mut g.state.race.trucks,
                    &g.resources.shop,
                )
                .unwrap();
            assert_eq!(g.state.race.trucks[0], human);
            assert_eq!(g.state.session.wallets[0], [0, 0, 8, 0, 0, 0]);
            for slot in 1..4 {
                let truck = &g.state.race.trucks[slot];
                let u = truck.upgrades;
                let levels = [u.acceleration, u.steering, u.speed, u.suspension];
                assert!(levels.iter().all(|&n| n > 0));
                assert!(levels.iter().max().unwrap() - levels.iter().min().unwrap() <= 1);
                let charges = truck.nitro.display_digits();
                let charges = charges[0] + charges[1] * 10;
                assert!(charges <= if slot == 3 { 6 } else { 3 });
                let spent = levels
                    .iter()
                    .zip(g.resources.shop.costs[1..].iter())
                    .map(|(&n, &p)| u64::from(n) * u64::from(p) * 100)
                    .sum::<u64>()
                    + u64::from(charges) * u64::from(g.resources.shop.costs[0]) * 100;
                assert_eq!(
                    championship::wallet(&g.state.session.wallets[slot]) + spent,
                    8000
                );
            }
            let purchased = g.state.race.trucks.each_ref().map(|t| t.upgrades);
            g.prepare_race().unwrap();
            let frozen = g.state.race.trucks.each_ref().map(|t| t.upgrades);
            for _ in 0..200 {
                ai::service_computers(
                    &mut g.state.race.trucks,
                    &mut g.state.race.progress,
                    &g.selected.track.routes[usize::from(g.selected.reversed)],
                    &g.resources.ai,
                    &mut g.state.random,
                )
                .unwrap();
                assert_eq!(g.state.race.trucks.each_ref().map(|t| t.upgrades), frozen);
            }
            g.state.championship.finish_race(
                [3, 1, 2, 0],
                &g.state.session,
                &mut g.state.race.trucks,
            );
            assert_eq!(
                g.state.race.trucks.each_ref().map(|t| t.upgrades),
                purchased
            );
        }
    }
    #[test]
    fn championship_eight_races_keep_fourth_place_human_and_finish_without_ninth_race() {
        for humans in [1, 3] {
            let mut g = game(humans);
            for round in 0..8 {
                g.prepare_race().unwrap();
                let route = g.selected.track.routes[usize::from(g.selected.reversed)].clone();
                g.state.postrace = Some(results::Postrace {
                    loss_route: None,
                    route_count: route.checkpoint_count,
                    route: Some(route),
                    phase: results::PostracePhase::Contract,
                    rank: if humans == 1 {
                        [3, 2, 1, 0]
                    } else {
                        [1, 3, 2, 0]
                    },
                    awards: results::Awards {
                        timer: 0,
                        new_record_pending: 0,
                        phase: results::AwardsFrame::Dismissed,
                    },
                });
                g.state.scene = Scene::Postrace;
                g.state.session.selected_wallet = 3;
                shop::add_award(&mut g.state.session, 3, shop::AwardDigit::TenThousands).unwrap();
                g.state.race.world.pending_awards = [1, 2, 3, 4];
                // Complete the real transition and dispatch the Championship branch.
                g.start_frontend_window(t::FrontendKind::Contracting)
                    .unwrap();
                g.state.frontend_motion.as_mut().unwrap().complete = true;
                let resources = g.resources.clone();
                g.advance_postrace_scene(&resources).unwrap();
                assert_eq!(g.state.session.active_humans, humans);
                assert_eq!(g.state.scene, Scene::ChampionshipStandings);
                g.draw_championship_standings().unwrap();
                if humans == 3 && round == 7 && std::env::var_os("CHAMPIONSHIP_PREVIEW").is_some() {
                    let mut frame = romatlas_port::graphics::Frame::new(256, 224).unwrap();
                    g.render(&mut frame);
                    let mut ppm = b"P6\n256 224\n255\n".to_vec();
                    for pixel in frame.pixels.chunks_exact(4) {
                        ppm.extend_from_slice(&pixel[..3]);
                    }
                    std::fs::write("/tmp/championship-final.ppm", ppm).unwrap();
                }
                assert_eq!(g.state.championship.completed, round + 1);
                assert_eq!(g.state.session.active_humans, humans);
                assert_eq!(g.state.race.world.pending_awards, [0; 4]);
                assert_eq!(g.state.championship.points[0], 0);
                assert_eq!(g.state.championship.earned[0], (round as u64 + 1) * 10000);
                g.sync_race_views();
                let saved = g.snapshot_json().unwrap();
                g.restore_snapshot_json(&saved).unwrap();
                if round < 7 {
                    g.state
                        .championship
                        .buy(
                            &mut g.state.session,
                            &mut g.state.race.trucks,
                            &g.resources.shop,
                        )
                        .unwrap();
                    g.state.postrace.as_mut().unwrap().phase = results::PostracePhase::Complete;
                    g.finish_postrace().unwrap();
                    assert_eq!(
                        g.state.sequence_index,
                        g.state.championship.schedule[round + 1].0
                    );
                }
            }
            assert_eq!(
                g.state.championship.points,
                if humans == 1 {
                    [0, 24, 48, 72]
                } else {
                    [0, 72, 24, 48]
                }
            );
            g.state.championship.previous = 0;
            let mut input = MultiInputState::default();
            input.players[0].held.insert("start".into());
            g.advance(&input).unwrap();
            assert!(!g.state.championship.active);
            assert_eq!(
                g.state.race.trucks.each_ref().map(|t| t.upgrades),
                [Default::default(); 4]
            );
            assert_eq!(g.state.session.wallets, [[0, 0, 1, 0, 0, 0]; 4]);
        }
    }
    #[test]
    fn championship_natural_cpu_races_finish_with_fixed_stats_on_each_difficulty() {
        for humans in [1, 2, 3] {
            for difficulty in [Difficulty::Easy, Difficulty::Normal, Difficulty::Hard] {
                let mut g = game(humans);
                g.state.championship.difficulty = difficulty;
                g.state
                    .championship
                    .buy(
                        &mut g.state.session,
                        &mut g.state.race.trucks,
                        &g.resources.shop,
                    )
                    .unwrap();
                g.prepare_race().unwrap();
                let upgrades = g.state.race.trucks.each_ref().map(|t| t.upgrades);
                let mut finished = false;
                for frame in 0..20000 {
                    g.advance(&MultiInputState::default()).unwrap();
                    assert_eq!(g.state.race.trucks.each_ref().map(|t| t.upgrades), upgrades);
                    if frame == 1000 {
                        let snapshot = g.snapshot_json().unwrap();
                        g.restore_snapshot_json(&snapshot).unwrap();
                    }
                    if g.state.scene == Scene::Postrace {
                        finished = true;
                        break;
                    }
                }
                assert!(
                    finished,
                    "CPU failed to finish {difficulty:?}, humans {humans}"
                );
                assert_eq!(
                    humans & (1 << (g.state.race.progress.winner_plus_one - 1)),
                    0
                );
            }
        }
    }
    #[test]
    fn championship_initial_standings_precede_shop_and_first_race() {
        for humans in [1, 3] {
            let mut g = game(humans);
            // Reproduce name entry's placeholders in the unjoined blue panel.
            g.state.session.records.truck_names[..3].copy_from_slice(&[11, 12, 13]);
            g.state.session.records.truck_names[3..6].copy_from_slice(if humans == 1 {
                &[42, 42, 42]
            } else {
                &[34, 35, 36]
            });
            g.state.menu = Menu::Names(state::selection());
            g.state.scene = Scene::Names;
            let resources = g.resources.clone();
            g.advance_names_scene(&resources).unwrap();
            assert_eq!(g.state.scene, Scene::ChampionshipStandings);
            assert_eq!(g.state.championship.completed, 0);
            assert_eq!(&g.state.session.records.truck_names[..3], &[11, 12, 13]);
            assert_eq!(
                &g.state.session.records.truck_names[3..6],
                if humans == 1 {
                    &resources.results.defaults.truck_names[3..6]
                } else {
                    &[34, 35, 36]
                }
            );
            assert_eq!(
                &g.state.session.records.truck_names[6..],
                &resources.results.defaults.truck_names[6..]
            );
            if humans == 1 && std::env::var_os("CHAMPIONSHIP_PREVIEW").is_some() {
                g.draw_championship_standings().unwrap();
                let mut frame = romatlas_port::graphics::Frame::new(256, 224).unwrap();
                g.render(&mut frame);
                let mut ppm = b"P6\n256 224\n255\n".to_vec();
                for pixel in frame.pixels.chunks_exact(4) {
                    ppm.extend_from_slice(&pixel[..3]);
                }
                std::fs::write("/tmp/championship-initial.ppm", ppm).unwrap();
            }
            g.state.automatic[0] = ControllerWord(controls::BUTTON_START);
            g.state.championship.previous = controls::BUTTON_START;
            let mut input = MultiInputState::default();
            input.players[0].held.insert("start".into());
            g.advance(&input).unwrap();
            assert_eq!(g.state.scene, Scene::ChampionshipStandings);
            g.advance(&MultiInputState::default()).unwrap();
            g.advance(&input).unwrap();
            assert_eq!(g.state.scene, Scene::Shop);
            let Menu::Shop(shop) = &mut g.state.menu else {
                panic!("missing first shop")
            };
            shop.phase = shop::Phase::Complete;
            g.advance_shop_scene(&resources).unwrap();
            assert_eq!(g.state.scene, Scene::RacePrepare);
            assert_eq!(g.state.session.active_humans, humans);
            g.prepare_race().unwrap();
            assert!(g.state.race.progress.championship);
            assert_eq!(g.state.sequence_index, g.state.championship.schedule[0].0);
        }
    }
    #[test]
    fn championship_setup_defaults_and_fresh_press_gate() {
        let mut g = game(1);
        g.state.championship = Championship::default();
        g.state.arcade = Default::default();
        g.state.automatic = [ControllerWord(0); 4];
        g.begin_names();
        for expected in [
            Scene::ChampionshipCup,
            Scene::ChampionshipDifficulty,
            Scene::Names,
        ] {
            g.state.automatic[0] = ControllerWord(controls::BUTTON_START);
            g.advance_championship_menu().unwrap();
            assert_eq!(g.state.scene, expected);
            if expected != Scene::Names {
                g.advance_championship_menu().unwrap();
                assert_eq!(g.state.scene, expected);
                g.state.automatic[0] = ControllerWord(0);
                g.advance_championship_menu().unwrap();
            }
        }
        assert!(g.state.championship.active);
        assert_eq!(g.state.championship.difficulty, Difficulty::Normal);
        assert_eq!(g.state.championship.cup, 0);
    }
}
