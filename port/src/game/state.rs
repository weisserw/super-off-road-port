//! Owned, cloneable native continuation state. Frontends are views of these owners.
use crate::{
    controls::ControllerWord,
    frontend::{attract, names, results, shop},
    presentation::transitions as t,
    race,
    runtime::RandomState,
    trucks::{ai, motion::Truck},
};
use romatlas_port::input::MultiInputState;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Scene {
    FrontendEntry,
    Splashes,
    AttractPanel,
    HighscoreFade,
    Highscores,
    HighscoreClose,
    Names,
    Shop,
    RacePrepare,
    RaceFade,
    Ready,
    MusicWait,
    Racing,
    AttractFade,
    Postrace,
    LossSetup,
    LossShowWait,
    LossFade,
    LossPrompt,
    LossClose,
    LossRestart,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplashNext {
    InitializeMoney,
    Race,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShopNext {
    FirstRace,
    Postrace,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Menu {
    Idle(names::Selection),
    Names(names::Selection),
    Shop(shop::Shop),
}
impl Menu {
    pub fn selection(&self) -> &names::Selection {
        match self {
            Self::Idle(value) | Self::Names(value) => value,
            Self::Shop(value) => &value.selection,
        }
    }
    pub fn selection_mut(&mut self) -> &mut names::Selection {
        match self {
            Self::Idle(value) | Self::Names(value) => value,
            Self::Shop(value) => &mut value.selection,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub id: u64,
    pub tick: u64,
    pub kind: String,
    pub details: serde_json::Value,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub ticks: u64,
    pub scene: Scene,
    pub nmi_enabled: bool,
    pub nmi: u8,
    pub heartbeat: u8,
    pub random: RandomState,
    pub session: attract::Session,
    pub race: race::RaceState,
    pub menu: Menu,
    pub shop_redraw_field: u16,
    pub sequence_index: usize,
    pub input: MultiInputState,
    pub automatic: [ControllerWord; 4],
    pub splash: Option<attract::Splash>,
    pub splash_next: SplashNext,
    pub splash_cold: bool,
    pub panel: Option<attract::Panel>,
    pub highscores: Option<attract::Highscores>,
    pub fade: Option<t::Fade>,
    /// Moved into a frontend continuation while its source window call is active.
    pub window: Option<t::Window>,
    pub frontend_motion: Option<t::FrontendWindow>,
    pub transition: Option<t::RaceTransition>,
    pub pause: t::Pause,
    pub postrace: Option<results::Postrace>,
    pub prompt: Option<results::ContinuePrompt>,
    pub game_over_timer: u16,
    pub loss_restart: bool,
    pub shop_next: ShopNext,
    pub added_records: results::AddedRecords,
    pub events: Vec<Event>,
    pub next_event: u64,
    pub fault: Option<String>,
}
pub fn selection() -> names::Selection {
    names::Selection {
        sequence: None,
        resume: names::Resume::Sample,
        page: 0,
        done_mask: 0,
        countdown: 0,
        editors: [names::Editor::default(); 2],
        temporary: [[0; 10]; 2],
        phase: names::Phase::Complete,
        sampled: [ControllerWord(0); 2],
        window_abort_enabled: false,
    }
}
impl State {
    pub fn cold(random: RandomState) -> Self {
        // Deterministic native power-on policy for source RAM not explicitly
        // initialized at boot; warm entry does not repeat this initialization.
        let records = attract::SessionRecords::from_legacy(
            [0; 48],
            [0; 32],
            [0; 60],
            [0; 30],
            std::array::from_fn(|i| i as u8),
            [0; 12],
        );
        let session = attract::Session {
            signature: [0; 5],
            records,
            wallets: [[0; 6]; 4],
            selected_wallet: 0,
            continuation_progress: 0,
            page_limit: 0,
            joined_pages: [0; 2],
            active_humans: 1,
            order_slot: 0,
            attract_rotation: 0,
            start_reentry: 0,
            inhibit_video: 0x80,
            enabled_layers: 0,
            window_shape: 0,
            nmi_and_automatic_inputs: false,
            audio_commands: vec![],
            phase: attract::EntryPhase::InitialLicense,
        };
        let sprite = race::BannerSprite {
            coordinates: 0,
            tile_attributes: 0,
        };
        let race = race::RaceState {
            route_tail_byte: 0,
            world: Default::default(),
            video: Default::default(),
            sounds: vec![],
            entry: race::RaceEntry::Running,
            working_pose: 0,
            trucks: std::array::from_fn(|_| Truck::default()),
            progress: race::Progress {
                drivers: [ai::Driver::default(); 4],
                active_humans: 1,
                animation_frame: 0,
                winner_plus_one: 0,
                banner_state: 0,
            },
            timer: Default::default(),
            banner: race::BannerDisplay {
                primary: sprite,
                secondary: sprite,
                secondary_state: 0,
                fixed_pair: [sprite; 2],
            },
            prepared: 0,
            terrain_history: 0,
            render_mode: 0,
            video_inhibit: 0x80,
            enabled_layers: 0,
        };
        Self {
            ticks: 0,
            scene: Scene::FrontendEntry,
            nmi_enabled: false,
            nmi: 0,
            heartbeat: 0,
            random,
            session,
            race,
            menu: Menu::Idle(selection()),
            shop_redraw_field: 0,
            sequence_index: 0,
            input: MultiInputState::default(),
            automatic: [ControllerWord(0); 4],
            splash: None,
            splash_next: SplashNext::InitializeMoney,
            splash_cold: true,
            panel: None,
            highscores: None,
            fade: None,
            window: Some(names::new_window()),
            frontend_motion: None,
            transition: None,
            pause: t::Pause {
                owner_plus_one: 0,
                phase: t::PausePhase::Running,
            },
            postrace: None,
            prompt: None,
            game_over_timer: 0,
            loss_restart: false,
            shop_next: ShopNext::FirstRace,
            added_records: Default::default(),
            events: vec![],
            next_event: 0,
            fault: None,
        }
    }
    pub fn event(&mut self, kind: &str, details: serde_json::Value) {
        self.events.push(Event {
            id: self.next_event,
            tick: self.ticks,
            kind: kind.into(),
            details,
        });
        self.next_event = self.next_event.wrapping_add(1);
        if self.events.len() > 1024 {
            self.events.remove(0);
        }
    }
    pub fn scene(&mut self, next: Scene) {
        if next != self.scene {
            let previous = self.scene;
            self.scene = next;
            self.event(
                "scene_changed",
                serde_json::json!({"from":format!("{previous:?}"),"to":format!("{next:?}")}),
            );
        }
    }
}
