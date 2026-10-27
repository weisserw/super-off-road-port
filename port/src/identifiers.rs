//! Closed domain identities from the source. Numeric source encodings stay at
//! loading/serialization boundaries; custom track/resource IDs remain strings.

/// Four parallel truck records, initialized by initialize_race (game.asm:110).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TruckId {
    First,
    Second,
    Third,
    Fourth,
}

/// Broad public scene categories retained from Stage1. The exact resumable
/// scheduler states are game::state::Scene; this enum owns no mutable state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scene {
    Pending,
    License,
    Title,
    DriverSelection,
    NameEntry,
    SpeedShop,
    Ready,
    Race,
    Pause,
    Results,
    Continue,
    GameOver,
}

/// Speed Shop upgrade records initialized at reset_and_boot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Upgrade {
    Steering,
    TopSpeed,
    Acceleration,
    Suspension,
    Nitro,
}

/// Broad public event categories. The bounded native journal records concrete
/// events with IDs, ticks and structured details in game::state::Event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    SceneChanged { from: Scene, to: Scene },
    RaceFinished { winner: TruckId },
    SoundRequested { resource: String },
}
