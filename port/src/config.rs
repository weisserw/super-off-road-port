//! Shared game identity and frontend configuration. Both backends use these values.
pub const ROM_SHA256: &str = "24b687f95bb9737d223738f13c00aeaa7d697fa9e2fd50597b81d0cfa2160daa";
pub const ROM_BYTES: usize = 524288;
pub const DISPLAY_ASPECT: (u32, u32) = (4, 3);
pub const SIZE: (u32, u32) = (256, 224);
pub const RATE: romatlas_port::timing::TickRate = romatlas_port::timing::TickRate::SNES_NTSC;
/// Native power-on policy for untouched WRAM controls outside the terrain
/// copy. Source can probe these at the course edges; no source write
/// establishes their physical-console power-on value.
pub const UNCOPIED_TERRAIN_CONTROL: u8 = 0;
pub fn settings() -> romatlas_port::settings::Settings {
    let defaults = romatlas_port::snes::default_settings();
    romatlas_port::settings::Settings {
        width: (defaults.height * DISPLAY_ASPECT.0 + DISPLAY_ASPECT.1 / 2) / DISPLAY_ASPECT.1,
        ..defaults
    }
}

/// Stable original-track championships, independent of gameplay entropy.
pub const CHAMPIONSHIP_RACES: usize = 8;
pub const CHAMPIONSHIP_NAMES: [&str; 4] = ["THUNDER", "HURRICANE", "TORNADO", "CYCLONE"];
pub const CHAMPIONSHIP_SEEDS: [[u8; 5]; 4] = [
    [0x54, 0x4f, 0x52, 0x4e, 0x41], // Thunder: former Tornado schedule.
    [0x10, 0x00, 0x52, 0x52, 0x49], // Hurricane: first Classic course, forward.
    [0x54, 0x48, 0x55, 0x4e, 0x44], // Tornado: former Thunder schedule.
    [0x3a, 0x10, 0x43, 0x4c, 0x4f], // Cyclone: second Classic course, forward.
];

// Initial Championship tuning; keep changes here isolated from Arcade contracts.
pub const CHAMPIONSHIP_BASE_BONUS: [i8; 3] = [-1, 0, 1];
pub const CHAMPIONSHIP_RELIEF_LIMIT: [i8; 3] = [-3, -2, -1];
pub const CHAMPIONSHIP_CATCHUP_LIMIT: [i8; 3] = [0, 1, 2];
pub const CHAMPIONSHIP_CAR_BONUS: [i8; 4] = [0, 1, 0, 2];
pub const CHAMPIONSHIP_NITRO_PERCENT: [u64; 4] = [20, 20, 20, 45];
pub const CHAMPIONSHIP_NITRO_LIMIT: [u8; 4] = [3, 3, 3, 6];
pub const CHAMPIONSHIP_VICTORY: &str = "Conglaturation!!! You have completed a great game";

/// CPU nitro pacing at the shared approximately 60 Hz race cadence.
pub const CHAMPIONSHIP_NITRO_CHANCE: [u8; 4] = [32, 32, 28, 48];
pub const CHAMPIONSHIP_NITRO_COOLDOWN: [u16; 4] = [360, 360, 360, 300];
pub const CHAMPIONSHIP_NITRO_START_DELAY: [u16; 4] = [180, 180, 240, 210];
