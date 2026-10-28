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
