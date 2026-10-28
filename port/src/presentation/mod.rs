//! Source display staging and pure native Mode1 composition. Derivations and
//! cross-owner boundaries are in rendering.trace.md. Native owners implement source rules; source derivations and conditional limits remain in the trace.
pub mod effects;
pub mod memory;
pub mod poses;
pub mod sprites;
pub mod transitions;
pub const HUD_DIGIT_TILE_BASE: u8 = 0x15;
use crate::{race::RaceTimer, world::tracks::TrackContent};
use anyhow::{Context, Result, ensure};
pub const VRAM_BYTES: usize = 65536;
pub const PALETTE_COLORS: usize = 256;
pub const OAM_BYTES: usize = 544;
pub const SORTED_SPRITES: usize = 16;
pub const CONTACT_FRAMES: usize = 31;
pub const HIDDEN_Y: u8 = 0xf0;
pub const RACE_MAP_WORDS: [u16; 3] = [0x4000, 0x4400, 0x4800];
/// Native video storage, not a CPU register or RAM map. CPU/VRAM conversion is
/// confined to loading/drawing; renderer only reads the last visible state.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VideoMemory {
    #[serde(with = "crate::snapshot::boxed_array")]
    pub vram: Box<[u8; VRAM_BYTES]>,
    #[serde(with = "crate::snapshot::array")]
    pub palette: [u16; PALETTE_COLORS],
    #[serde(with = "crate::snapshot::array")]
    pub staged_oam: [u8; OAM_BYTES],
    #[serde(with = "crate::snapshot::array")]
    pub visible_oam: [u8; OAM_BYTES],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layer {
    pub map_byte_base: u16,
    pub tile_byte_base: u16,
    /// BGxSC screen-size bits: an extra 32-tile page in each selected axis.
    pub wide: bool,
    pub tall: bool,
    #[serde(with = "crate::snapshot::array")]
    pub scroll: [u16; 2],
    pub bits_per_pixel: u8,
}
/// Source Mode1 configuration expressed without emulating PPU writes.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Display {
    #[serde(with = "crate::snapshot::array")]
    pub layers: [Layer; 3],
    pub bg3_high: bool,
    pub object_tile_byte_base: u16,
    pub main_layers: u8,
    pub sub_layers: u8,
    pub main_window_layers: u8,
    pub sub_window_layers: u8,
    pub color_math_layers: u8,
    pub use_subscreen: bool,
    pub half_color: bool,
    pub subtract_color: bool,
    pub fixed_color: u16,
    pub brightness: u8,
    pub force_blank: bool,
    /// Inclusive per-line source window; left>right closes it.
    pub window_edges: Vec<[u8; 2]>,
    pub masked_layers: u8,
    /// Active window1 inversion per layer, separate from retained caller config.
    pub window_inverted_layers: u8,
}
pub fn reset_display(display: &mut Display) {
    const FULL_BRIGHTNESS: u8 = 15;
    display.force_blank = true;
    display.brightness = FULL_BRIGHTNESS;
    for layer in &mut display.layers {
        layer.scroll = [0; 2];
        layer.map_byte_base = 0;
        layer.tile_byte_base = 0;
        layer.wide = false;
        layer.tall = false;
    }
    display.main_layers = 0;
    display.sub_layers = 0;
    display.main_window_layers = 0;
    display.sub_window_layers = 0;
    display.color_math_layers = 0;
    display.use_subscreen = false;
    display.half_color = false;
    display.subtract_color = false;
    display.masked_layers = 0;
    display.window_inverted_layers = 0;
    display.window_edges.fill([1, 0]);
    // COLDATA0 selects no components; the established fixed color survives.
}
pub fn configure_race(display: &mut Display) {
    const BACKGROUND_BASES: [u16; 3] = [0x8000, 0x8800, 0x9000];
    const TILE_BASES: [u16; 3] = [0, 0, 0xa000];
    const DEPTHS: [u8; 3] = [4, 4, 2];
    const OBJECT_BASE: u16 = 0xc000;
    for index in 0..3 {
        display.layers[index].map_byte_base = BACKGROUND_BASES[index];
        display.layers[index].tile_byte_base = TILE_BASES[index];
        display.layers[index].bits_per_pixel = DEPTHS[index];
        display.layers[index].wide = false;
        display.layers[index].tall = false;
    }
    display.bg3_high = true;
    display.object_tile_byte_base = OBJECT_BASE;
}
pub fn initialize_race_oam(memory: &mut VideoMemory, display: &mut Display) {
    const LOW_BYTES: usize = 512;
    const DEFAULT_ATTRIBUTES: u8 = 0x21;
    const SECOND_ATTRIBUTES: u8 = 0x29;
    const LARGE_PAIRS: u8 = 0xaa;
    const HIGH_OVERRIDE: usize = 0x208;
    const ZERO_OVERRIDE: usize = 0x218;
    const INITIAL_MATH_LAYERS: u8 = 0x17;
    const BACKGROUND_LAYERS: u8 = 7;
    for record in memory.staged_oam[..LOW_BYTES].chunks_exact_mut(4) {
        record.copy_from_slice(&[0, HIDDEN_Y, 0, DEFAULT_ATTRIBUTES]);
    }
    memory.staged_oam[7] = SECOND_ATTRIBUTES;
    memory.staged_oam[LOW_BYTES..].fill(LARGE_PAIRS);
    memory.staged_oam[HIGH_OVERRIDE] = 0xa2;
    memory.staged_oam[ZERO_OVERRIDE] = 0;
    display.color_math_layers = INITIAL_MATH_LAYERS;
    display.use_subscreen = true;
    display.half_color = true;
    display.subtract_color = false;
    display.sub_layers = BACKGROUND_LAYERS;
}
/// Sample original BG/OBJ priorities and source main/sub/window/math setup.
/// Inspection has no simulation/audio/cursor side effects.
pub fn pixel(memory: &VideoMemory, display: &Display, x: u8, y: u8) -> Result<[u8; 4]> {
    let objects = visible_objects(memory)?;
    composed_pixel(memory, display, &objects, x, y, y)
}
pub fn render(memory: &VideoMemory, display: &Display) -> Result<Vec<u8>> {
    let (width, height) = crate::config::SIZE;
    if display.force_blank || display.brightness == 0 {
        return Ok([0, 0, 0, 255].repeat((width * height) as usize));
    }
    let objects = visible_objects(memory)?;
    let mut output = Vec::with_capacity((width * height * 4) as usize);
    // SNES visible scanlines are 1..=224: framebuffer row zero is console
    // scanline one. BG scroll uses that console scanline; OBJ evaluation's
    // one-line pipeline makes OAM Y=0 appear on the first framebuffer row.
    // Initial original license capture established the missing one-line origin;
    // pixel() intentionally remains a console-coordinate sampling primitive.
    const FIRST_VISIBLE_SCANLINE: u32 = 1;
    for y in 0..height {
        for x in 0..width {
            output.extend(composed_pixel(
                memory,
                display,
                &objects,
                x as u8,
                (y + FIRST_VISIBLE_SCANLINE) as u8,
                y as u8,
            )?);
        }
    }
    Ok(output)
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Rectangle {
    pub width_flags: u16,
    pub rows: u16,
    pub palette_remap: [u16; 8],
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MapImage {
    pub rectangle: Rectangle,
    pub words: Vec<u16>,
    pub destination_word: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TrackPresentation {
    /// Added named tracks explicitly supply ready/pause center and destination.
    pub countdown: transitions::CountdownLayout,
    pub title: MapImage,
    pub banner_destination_word: u16,
    pub hud_anchor: [u8; 2],
    pub overlays: Vec<MapImage>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HudTables {
    pub relative_x: [u8; SORTED_SPRITES],
    pub relative_y: [u8; SORTED_SPRITES],
    pub attributes: [u8; SORTED_SPRITES],
    pub banner: MapImage,
}
pub fn load_hud_tables(rom: &[u8]) -> Result<HudTables> {
    const RELATIVE_X_ROM: usize = 0x4411;
    const RELATIVE_Y_ROM: usize = 0x4421;
    const ATTRIBUTES_ROM: usize = 0x4401;
    const BANNER_DESCRIPTOR: usize = 0x4471;
    const BANNER_SOURCE: u32 = 0x08e220;
    let bytes = |at| {
        rom.get(at..at + SORTED_SPRITES)
            .context("truncated HUD sprite table")
    };
    let rectangle = memory::decode_rectangle(
        rom.get(BANNER_DESCRIPTOR..BANNER_DESCRIPTOR + 20)
            .context("truncated banner descriptor")?,
    )?;
    let words = memory::read_rom_rectangle(rom, BANNER_SOURCE, &rectangle)?;
    Ok(HudTables {
        relative_x: bytes(RELATIVE_X_ROM)?.try_into()?,
        relative_y: bytes(RELATIVE_Y_ROM)?.try_into()?,
        attributes: bytes(ATTRIBUTES_ROM)?.try_into()?,
        banner: MapImage {
            rectangle,
            words,
            destination_word: 0,
        },
    })
}
pub fn load_track_presentation(rom: &[u8], track: u8) -> Result<TrackPresentation> {
    const TITLE_SOURCES: usize = 0x4305;
    const TITLE_RECTANGLES: usize = 0x4335;
    const TITLE_DESTINATIONS: usize = 0x43e1;
    const ANCHOR_X: usize = 0x4431;
    const ANCHOR_Y: usize = 0x4441;
    const BANNER_DESTINATIONS: usize = 0x4451;
    const COUNTDOWN_X: usize = 0x55ee;
    const COUNTDOWN_Y: usize = 0x55fe;
    const COUNTDOWN_DESTINATIONS: usize = 0x560e;
    const EIGHT_DESCRIPTOR: usize = 0x4123;
    const FOURTEEN_DESCRIPTOR: usize = 0x4137;
    ensure!(
        usize::from(track) < crate::world::tracks::ORIGINAL_TRACK_COUNT,
        "invalid original presentation track"
    );
    let index = usize::from(track);
    let read = |at| rom.get(at).copied().context("truncated presentation byte");
    let word = |at| -> Result<u16> { Ok(u16::from_le_bytes([read(at)?, read(at + 1)?])) };
    let map = |source, descriptor, destination| -> Result<MapImage> {
        let rectangle = memory::decode_rectangle(
            rom.get(descriptor..descriptor + 20)
                .context("truncated track rectangle")?,
        )?;
        Ok(MapImage {
            words: memory::read_rom_rectangle(rom, source, &rectangle)?,
            rectangle,
            destination_word: destination,
        })
    };
    let source_at = TITLE_SOURCES + index * 3;
    let title_source = u32::from(word(source_at)?) | (u32::from(read(source_at + 2)?) << 16);
    let descriptor = crate::world::tracks::physical(crate::world::tracks::CpuSource(u32::from(
        word(TITLE_RECTANGLES + index * 2)?,
    )))?;
    let title = map(
        title_source,
        descriptor,
        word(TITLE_DESTINATIONS + index * 2)?,
    )?;
    let overlays = match track {
        8 => vec![map(0x0d8c00, EIGHT_DESCRIPTOR, 0x4281)?],
        14 => vec![
            map(0x0dd412, FOURTEEN_DESCRIPTOR, 0x40d9)?,
            map(0x0dd41a, FOURTEEN_DESCRIPTOR, 0x40ca)?,
        ],
        _ => Vec::new(),
    };
    Ok(TrackPresentation {
        countdown: transitions::CountdownLayout {
            center: [read(COUNTDOWN_X + index)?, read(COUNTDOWN_Y + index)?],
            destination_word: word(COUNTDOWN_DESTINATIONS + index * 2)?,
        },
        title,
        banner_destination_word: word(BANNER_DESTINATIONS + index * 2)?,
        hud_anchor: [read(ANCHOR_X + index)?, read(ANCHOR_Y + index)?],
        overlays,
    })
}
pub fn validate_track_presentation(content: &TrackPresentation) -> Result<()> {
    const WIDTH_MASK: u16 = 0x7fff;
    for map in std::iter::once(&content.title).chain(&content.overlays) {
        let width = usize::from(map.rectangle.width_flags & WIDTH_MASK);
        let rows = usize::from(map.rectangle.rows);
        anyhow::ensure!(
            width != 0 && rows != 0 && map.words.len() == width * rows,
            "invalid track map dimensions"
        );
        anyhow::ensure!(
            map.rectangle
                .palette_remap
                .iter()
                .all(|&palette| palette < 8),
            "invalid palette remap"
        );
    }
    Ok(())
}
pub fn initialize_hud(
    memory: &mut VideoMemory,
    rom: &[u8],
    content: &TrackPresentation,
    tables: &HudTables,
    reversed: bool,
    timer: &mut RaceTimer,
) -> Result<()> {
    const HUD_ROM: usize = 0x3b600;
    const HUD_BYTES: usize = 0x2000;
    const HUD_DESTINATION: usize = 0xa000;
    const HUD_MAP: u16 = 0x4800;
    const BORDER_STARTS: [u16; 2] = [0x4800, 0x4b40];
    const BORDER_WORDS: usize = 64;
    const BORDER_TILE: u16 = 0x2ddb;
    const DIGIT_BASE: u8 = HUD_DIGIT_TILE_BASE;
    validate_track_presentation(content)?;
    let graphics = rom
        .get(HUD_ROM..HUD_ROM + HUD_BYTES)
        .context("truncated HUD graphics")?;
    memory.vram[HUD_DESTINATION..HUD_DESTINATION + HUD_BYTES].copy_from_slice(graphics);
    memory::fill_map(memory, HUD_MAP, 0);
    for start in BORDER_STARTS {
        for index in 0..BORDER_WORDS {
            memory::write_word(memory, start + index as u16, BORDER_TILE);
        }
    }
    if reversed {
        for map in &content.overlays {
            memory::draw_rectangle(memory, &map.words, map.destination_word, &map.rectangle)?;
        }
    }
    memory::draw_rectangle(
        memory,
        &tables.banner.words,
        content.banner_destination_word,
        &tables.banner.rectangle,
    )?;
    memory::draw_rectangle(
        memory,
        &content.title.words,
        content.title.destination_word,
        &content.title.rectangle,
    )?;
    for (output, index) in (0..SORTED_SPRITES).rev().enumerate() {
        memory.staged_oam[output * 4..output * 4 + 4].copy_from_slice(&[
            tables.relative_x[index].wrapping_add(content.hud_anchor[0]),
            tables.relative_y[index].wrapping_add(content.hud_anchor[1]),
            DIGIT_BASE,
            tables.attributes[index],
        ]);
    }
    memory.staged_oam[0x36] = 0x1f;
    memory.staged_oam[0x200..0x204].fill(0);
    *timer = RaceTimer::default();
    Ok(())
}
pub fn update_hud_digits(
    memory: &mut VideoMemory,
    laps: [u8; 4],
    nitro: [crate::trucks::nitro::NitroStock; 4],
    timer: &mut RaceTimer,
) {
    const DIGIT_BASE: u8 = HUD_DIGIT_TILE_BASE;
    const LAP_START: usize = 0x22;
    const TIMER_HIGH: usize = 0x3e;
    const TIMER_MIDDLE_HIGH: usize = 0x3a;
    const TIMER_MIDDLE_LOW: usize = 0x32;
    for (output, slot) in (0..4).rev().enumerate() {
        memory.staged_oam[LAP_START + output * 4] = laps[slot].wrapping_add(DIGIT_BASE);
        let [units, tens] = nitro[slot].display_digits();
        memory.staged_oam[2 + output * 8] = units.wrapping_add(DIGIT_BASE);
        memory.staged_oam[6 + output * 8] = tens.wrapping_add(DIGIT_BASE);
    }
    crate::race::advance_timer(timer);
    let digits = timer
        .visible_digits()
        .map(|digit| digit.wrapping_add(DIGIT_BASE));
    for (at, digit) in [TIMER_HIGH, TIMER_MIDDLE_HIGH, TIMER_MIDDLE_LOW]
        .into_iter()
        .zip(digits)
    {
        memory.staged_oam[at] = digit;
    }
}
pub fn install_track_video(
    memory: &mut VideoMemory,
    track: &TrackContent,
    palette: &[u16; PALETTE_COLORS],
) -> Result<()> {
    const GRAPHICS_END: usize = 0x8000;
    const FIRST_MAP: usize = 0x8080;
    const SECOND_MAP: usize = 0x8880;
    const SECOND_PRIORITY: u8 = 0x20;
    ensure!(
        track.graphics.len() == crate::world::tracks::GRAPHICS_BYTES
            && track
                .backgrounds
                .iter()
                .all(|bytes| bytes.len() == crate::world::tracks::BACKGROUND_BYTES),
        "invalid track video spans"
    );
    memory.vram[..GRAPHICS_END].copy_from_slice(&track.graphics);
    memory.palette = *palette;
    for destination in RACE_MAP_WORDS {
        memory::fill_map(memory, destination, 0);
    }
    memory.vram[FIRST_MAP..FIRST_MAP + track.backgrounds[0].len()]
        .copy_from_slice(&track.backgrounds[0]);
    for (index, &byte) in track.backgrounds[1].iter().enumerate() {
        memory.vram[SECOND_MAP + index] = byte | if index & 1 != 0 { SECOND_PRIORITY } else { 0 };
    }
    Ok(())
}
fn visible_objects(memory: &VideoMemory) -> Result<Vec<romatlas_port::snes::ppu::Object>> {
    const OBJECT_COUNT: u8 = 128;
    (0..OBJECT_COUNT)
        .map(|index| romatlas_port::snes::ppu::Object::from_oam(&memory.visible_oam, index))
        .collect()
}
fn composed_pixel(
    memory: &VideoMemory,
    display: &Display,
    objects: &[romatlas_port::snes::ppu::Object],
    x: u8,
    y: u8,
    object_y: u8,
) -> Result<[u8; 4]> {
    use romatlas_port::snes::ppu::{Background, Composition, Pixel, mode1_bg_rank, screen_pixel};
    const OBJECT_LAYER: u8 = 4;
    const OBJECT_RANKS: [u8; 4] = [2, 5, 8, 11];
    let mut candidates = [Pixel {
        color: 0,
        rank: 0,
        layer: 0,
        math: false,
    }; 4];
    let mut candidate_count = 0;
    for (index, layer) in display.layers.iter().enumerate() {
        if (display.main_layers | display.sub_layers) & (1 << index) == 0 {
            continue;
        }
        let background = Background {
            map_base: layer.map_byte_base,
            tile_base: layer.tile_byte_base,
            wide: layer.wide,
            tall: layer.tall,
            large_tiles: false,
            bpp: layer.bits_per_pixel,
            palette_base: 0,
            scroll_x: layer.scroll[0],
            scroll_y: layer.scroll[1],
        };
        if let Some((color, high)) = background.sample(&memory.vram, u16::from(x), u16::from(y))? {
            candidates[candidate_count] = Pixel {
                color: memory.palette[usize::from(color)],
                rank: mode1_bg_rank(index as u8, high, display.bg3_high)?,
                layer: index as u8,
                math: true,
            };
            candidate_count += 1;
        }
    }
    if (display.main_layers | display.sub_layers) & (1 << OBJECT_LAYER) != 0 {
        for object in objects {
            if let Some(color) = object.sample(
                &memory.vram,
                display.object_tile_byte_base,
                0,
                0,
                x,
                object_y,
            )? {
                candidates[candidate_count] = Pixel {
                    color: memory.palette[usize::from(color)],
                    rank: OBJECT_RANKS[usize::from(object.priority)],
                    layer: OBJECT_LAYER,
                    math: object.palette >= 4,
                };
                candidate_count += 1;
                break; // First visible OBJ wins before comparison with BG priority.
            }
        }
    }
    let inside = display
        .window_edges
        // HDMA's first edge pair applies to the first visible output row;
        // unlike BG tile sampling it does not include the skipped line zero.
        .get(usize::from(object_y))
        .is_some_and(|edges| edges[0] <= x && x <= edges[1]);
    let screen = |enabled: u8, windows: u8| {
        screen_pixel(
            candidates[..candidate_count]
                .iter()
                .copied()
                .filter(|pixel| {
                    let bit = 1 << pixel.layer;
                    enabled & bit != 0
                        && !((inside ^ (display.window_inverted_layers & bit != 0))
                            && display.masked_layers & windows & bit != 0)
                }),
            memory.palette[0],
        )
    };
    Ok(Composition {
        math_layers: display.color_math_layers,
        subtract: display.subtract_color,
        half: display.half_color,
        use_subscreen: display.use_subscreen,
        fixed_color: display.fixed_color,
        clip_main: false,
        prevent_math: false,
        brightness: display.brightness,
        force_blank: display.force_blank,
    }
    .compose(
        screen(display.main_layers, display.main_window_layers),
        screen(display.sub_layers, display.sub_window_layers),
    ))
}
impl Default for VideoMemory {
    fn default() -> Self {
        Self {
            vram: Box::new([0; VRAM_BYTES]),
            palette: [0; PALETTE_COLORS],
            staged_oam: [0; OAM_BYTES],
            visible_oam: [0; OAM_BYTES],
        }
    }
}
impl Default for Display {
    fn default() -> Self {
        let mut display = Self {
            layers: [Layer {
                map_byte_base: 0,
                tile_byte_base: 0,
                wide: false,
                tall: false,
                scroll: [0; 2],
                bits_per_pixel: 2,
            }; 3],
            bg3_high: false,
            object_tile_byte_base: 0,
            main_layers: 0,
            sub_layers: 0,
            main_window_layers: 0,
            sub_window_layers: 0,
            color_math_layers: 0,
            use_subscreen: false,
            half_color: false,
            subtract_color: false,
            fixed_color: 0,
            brightness: 15,
            force_blank: true,
            window_edges: vec![[1, 0]; crate::config::SIZE.1 as usize],
            masked_layers: 0,
            window_inverted_layers: 0,
        };
        configure_race(&mut display);
        display
    }
}

#[cfg(test)]
mod tests;
