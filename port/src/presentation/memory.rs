//! ROM graphics/text/rectangle consumers. Physical recipes remain named here;
//! native drawing operates in VRAM words with explicit destination conversion.
use super::{Rectangle, VideoMemory};
use anyhow::{Context, Result, ensure};
pub const FONT_BYTES: usize = 0x800;
pub const INCLUSIVE_FILL_WORDS: usize = 0x401;
pub const SHARED_ROM_PREFIX: std::ops::Range<usize> = 0x15000..0x18000;
pub const SHARED_WRAPPED_TAIL_BYTES: usize = 0xc00;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SharedGraphics {
    pub font_a: Vec<u8>,
    pub font_b: Vec<u8>,
    pub cursor: Vec<u8>,
    pub cursor_palette: Vec<u16>,
    pub text_palette: Vec<u16>,
    pub race_palette: Vec<u16>,
    pub sprite_palette: Vec<u16>,
    pub sprite_prefix: Vec<u8>,
    pub sprite_continuation: Vec<u8>,
    pub hud: Vec<u8>,
}
pub fn load_shared(rom: &[u8]) -> Result<SharedGraphics> {
    const FONT_A: (usize, usize) = (0x45440, FONT_BYTES);
    const FONT_B: (usize, usize) = (0x45a20, FONT_BYTES);
    const CURSOR: (usize, usize) = (0x45de0, 0x400);
    const CURSOR_PALETTE: (usize, usize) = (0x46200, 0x40);
    const TEXT_PALETTE: (usize, usize) = (0x461e0, 0x20);
    const RACE_PALETTE: (usize, usize) = (0x1ec7, 0x200);
    const SPRITE_PALETTE: (usize, usize) = (0x26b00, 0x100);
    const CONTINUATION: (usize, usize) = (0x25ac0, 0x1040);
    const HUD: (usize, usize) = (0x3b600, 0x2000);
    let read = |(at, length): (usize, usize)| {
        rom.get(at..at + length)
            .context("truncated shared graphics")
    };
    Ok(SharedGraphics {
        font_a: read(FONT_A)?.to_vec(),
        font_b: read(FONT_B)?.to_vec(),
        cursor: read(CURSOR)?.to_vec(),
        cursor_palette: decode_palette(read(CURSOR_PALETTE)?)?,
        text_palette: decode_palette(read(TEXT_PALETTE)?)?,
        race_palette: decode_palette(read(RACE_PALETTE)?)?,
        sprite_palette: decode_palette(read(SPRITE_PALETTE)?)?,
        sprite_prefix: rom
            .get(SHARED_ROM_PREFIX)
            .context("truncated shared sprite prefix")?
            .to_vec(),
        sprite_continuation: read(CONTINUATION)?.to_vec(),
        hud: read(HUD)?.to_vec(),
    })
}
pub fn decode_palette(bytes: &[u8]) -> Result<Vec<u16>> {
    const COLOR_MASK: u16 = 0x7fff;
    ensure!(bytes.len().is_multiple_of(2), "incomplete palette word");
    Ok(bytes
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]) & COLOR_MASK)
        .collect())
}
pub fn decode_planar(bytes: &[u8], bpp: u8) -> Result<Vec<[u8; 64]>> {
    const TILE_PIXELS_PER_AXIS: usize = 8;
    ensure!(matches!(bpp, 2 | 4 | 8), "invalid planar depth");
    let tile_bytes = usize::from(bpp) * TILE_PIXELS_PER_AXIS;
    ensure!(
        bytes.len().is_multiple_of(tile_bytes),
        "incomplete planar tile"
    );
    bytes
        .chunks_exact(tile_bytes)
        .map(|tile| romatlas_port::snes::tile(tile, bpp))
        .collect()
}
pub fn decode_rectangle(bytes: &[u8]) -> Result<Rectangle> {
    const DESCRIPTOR_BYTES: usize = 20;
    ensure!(
        bytes.len() == DESCRIPTOR_BYTES,
        "invalid rectangle descriptor length"
    );
    let word = |index| u16::from_le_bytes([bytes[index * 2], bytes[index * 2 + 1]]);
    let descriptor = Rectangle {
        width_flags: word(0),
        rows: word(1),
        palette_remap: std::array::from_fn(|index| word(index + 2)),
    };
    validate_rectangle(&descriptor)?;
    Ok(descriptor)
}
pub fn fill_map(memory: &mut VideoMemory, destination_word: u16, word: u16) {
    for index in 0..INCLUSIVE_FILL_WORDS {
        write_word(memory, destination_word.wrapping_add(index as u16), word);
    }
}
/// The long-indirect read can cross a bank for the second byte of a word, but
/// advancing the source low word does not carry into its bank. Loader owns this.
pub fn draw_rom_rectangle(
    memory: &mut VideoMemory,
    rom: &[u8],
    source_cpu: u32,
    destination_word: u16,
    descriptor: &Rectangle,
) -> Result<()> {
    let words = read_rom_rectangle(rom, source_cpu, descriptor)?;
    draw_rectangle(memory, &words, destination_word, descriptor)
}
pub fn draw_rectangle(
    memory: &mut VideoMemory,
    words: &[u16],
    destination_word: u16,
    descriptor: &Rectangle,
) -> Result<()> {
    const SIGN_FLAG: u16 = 0x8000;
    const PALETTE_SHIFT: u32 = 10;
    const PALETTE_MASK: u16 = 7;
    const RETAINED_ATTRIBUTES: u16 = 0xe3ff;
    let (width, rows) = validate_rectangle(descriptor)?;
    ensure!(words.len() >= width * rows, "truncated rectangle source");
    for row in 0..rows {
        for column in 0..width {
            let mut word = words[row * width + column];
            if descriptor.width_flags & SIGN_FLAG == 0 {
                let palette = usize::from((word >> PALETTE_SHIFT) & PALETTE_MASK);
                word = (word & RETAINED_ATTRIBUTES)
                    | (descriptor.palette_remap[palette] << PALETTE_SHIFT);
            }
            write_word(
                memory,
                destination_word.wrapping_add((row * MAP_ROW_WORDS + column) as u16),
                word,
            );
        }
    }
    Ok(())
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextSize {
    Single,
    Paired,
}
/// Returns separator offset; caller decides whether to consume FF (seven rows).
pub fn draw_text(
    memory: &mut VideoMemory,
    bytes: &[u8],
    destination_word: u16,
    size: TextSize,
) -> Result<usize> {
    const TERMINATOR: u8 = 0xff;
    const TEXT_ATTRIBUTES: u16 = 0x2000;
    let end = bytes
        .iter()
        .position(|&value| value == TERMINATOR)
        .context("unterminated text")?;
    for (index, &glyph) in bytes[..end].iter().enumerate() {
        let at = destination_word.wrapping_add(index as u16);
        match size {
            TextSize::Single => write_word(memory, at, TEXT_ATTRIBUTES | u16::from(glyph)),
            TextSize::Paired => {
                let doubled = glyph.wrapping_mul(2);
                write_word(memory, at, TEXT_ATTRIBUTES | u16::from(doubled));
                write_word(
                    memory,
                    at.wrapping_add(MAP_ROW_WORDS as u16),
                    TEXT_ATTRIBUTES | u16::from(doubled.wrapping_add(1)),
                );
            }
        }
    }
    Ok(end)
}
pub fn draw_seven_rows(
    memory: &mut VideoMemory,
    bytes: &[u8],
    destination_word: u16,
) -> Result<usize> {
    const ROWS: usize = 7;
    const ROW_STRIDE: u16 = 0x40;
    let mut offset = 0;
    let mut ranges = Vec::new();
    for row in 0..ROWS {
        let remaining = bytes.get(offset..).context("missing text row")?;
        let end = remaining
            .iter()
            .position(|&value| value == u8::MAX)
            .context("unterminated large-text row")?;
        ranges.push((row, offset, end));
        offset += end + 1;
    }
    for (row, start, _) in ranges {
        draw_text(
            memory,
            &bytes[start..],
            destination_word.wrapping_add(row as u16 * ROW_STRIDE),
            TextSize::Paired,
        )?;
    }
    Ok(offset)
}
pub fn clear_name_panel(memory: &mut VideoMemory, destination_word: u16) {
    const COLUMNS: usize = 11;
    const ROWS: usize = 14;
    const BLANK_GLYPH: u16 = 0x2000;
    for row in 0..ROWS {
        for column in 0..COLUMNS {
            write_word(
                memory,
                destination_word.wrapping_add((row * MAP_ROW_WORDS + column) as u16),
                BLANK_GLYPH,
            );
        }
    }
}
pub fn upload_fonts(memory: &mut VideoMemory, assets: &SharedGraphics, both: bool) -> Result<()> {
    const FONT_DESTINATION: usize = 0xa000;
    const TEXT_MAP: u16 = 0x4800;
    ensure!(
        assets.font_b.len() == FONT_BYTES && (!both || assets.font_a.len() == FONT_BYTES),
        "invalid font span"
    );
    if both {
        memory.vram[FONT_DESTINATION..FONT_DESTINATION + FONT_BYTES]
            .copy_from_slice(&assets.font_a);
    }
    let destination = FONT_DESTINATION + if both { FONT_BYTES } else { 0 };
    memory.vram[destination..destination + FONT_BYTES].copy_from_slice(&assets.font_b);
    fill_map(memory, TEXT_MAP, 0);
    Ok(())
}
pub fn upload_text_palette(memory: &mut VideoMemory, assets: &SharedGraphics) -> Result<()> {
    const TEXT_COLORS: usize = 16;
    ensure!(
        assets.text_palette.len() == TEXT_COLORS,
        "invalid text palette"
    );
    memory.palette[..TEXT_COLORS].copy_from_slice(&assets.text_palette);
    Ok(())
}
pub fn upload_cursor(memory: &mut VideoMemory, assets: &SharedGraphics) -> Result<()> {
    const CURSOR_DESTINATION: usize = 0xc400;
    const CURSOR_BYTES: usize = 0x400;
    const CURSOR_COLOR_START: usize = 128;
    const CURSOR_COLORS: usize = 32;
    ensure!(
        assets.cursor.len() == CURSOR_BYTES && assets.cursor_palette.len() == CURSOR_COLORS,
        "invalid cursor graphics"
    );
    memory.vram[CURSOR_DESTINATION..CURSOR_DESTINATION + CURSOR_BYTES]
        .copy_from_slice(&assets.cursor);
    memory.palette[CURSOR_COLOR_START..CURSOR_COLOR_START + CURSOR_COLORS]
        .copy_from_slice(&assets.cursor_palette);
    Ok(())
}
/// The first bank-wrapped C00 tail is wholly overwritten by the second
/// transfer before presentation; only surviving ROM-backed bytes are installed.
pub fn upload_shared_sprites(memory: &mut VideoMemory, assets: &SharedGraphics) -> Result<()> {
    const PREFIX_DESTINATION: usize = 0xc400;
    const PREFIX_SURVIVING_BYTES: usize = 0x2bc0;
    const CONTINUATION_DESTINATION: usize = 0xefc0;
    const CONTINUATION_BYTES: usize = 0x1040;
    ensure!(
        assets.sprite_prefix.len() == SHARED_ROM_PREFIX.len()
            && assets.sprite_continuation.len() == CONTINUATION_BYTES,
        "invalid shared sprite spans"
    );
    // Both source transfers finish under blank/inhibit. This installs exactly
    // their final surviving bytes, avoiding fabricated wrapped WRAM contents.
    memory.vram[PREFIX_DESTINATION..CONTINUATION_DESTINATION]
        .copy_from_slice(&assets.sprite_prefix[..PREFIX_SURVIVING_BYTES]);
    memory.vram[CONTINUATION_DESTINATION..].copy_from_slice(&assets.sprite_continuation);
    Ok(())
}
/// Conditional diagnostic entry only: no established natural caller or UI.
pub fn append_hex_byte(output: &mut [u8; 256], cursor: &mut u8, byte: u8) {
    const NIBBLE_MASK: u8 = 0xf;
    for nibble in [byte >> 4, byte & NIBBLE_MASK] {
        output[usize::from(*cursor)] = nibble + 1;
        *cursor = cursor.wrapping_add(1);
    }
}
pub fn stage_diagnostic_fields(output: &mut [u8; 256], cursor: &mut u8, fields: [u16; 2]) {
    const FIRST_GLYPH: u8 = 2;
    *cursor = FIRST_GLYPH;
    for field in fields {
        let [low, high] = field.to_le_bytes();
        append_hex_byte(output, cursor, high);
        append_hex_byte(output, cursor, low);
        *cursor = cursor.wrapping_add(1);
    }
}

pub const MAP_ROW_WORDS: usize = 32;
pub fn write_word(memory: &mut VideoMemory, destination_word: u16, value: u16) {
    let address = destination_word.wrapping_mul(2);
    let [low, high] = value.to_le_bytes();
    memory.vram[usize::from(address)] = low;
    memory.vram[usize::from(address.wrapping_add(1))] = high;
}
pub fn validate_rectangle(descriptor: &Rectangle) -> Result<(usize, usize)> {
    const WIDTH_MASK: u16 = 0x7fff;
    let width = usize::from(descriptor.width_flags & WIDTH_MASK);
    let rows = usize::from(descriptor.rows);
    ensure!(
        width != 0
            && rows != 0
            && width * rows <= super::VRAM_BYTES / 2
            && descriptor.palette_remap.iter().all(|&palette| palette < 8),
        "invalid rectangle dimensions/remap"
    );
    Ok((width, rows))
}
pub fn read_rom_rectangle(rom: &[u8], source_cpu: u32, descriptor: &Rectangle) -> Result<Vec<u16>> {
    let (width, rows) = validate_rectangle(descriptor)?;
    let bank = source_cpu & 0xff0000;
    let mut low = source_cpu as u16;
    let mut words = Vec::with_capacity(width * rows);
    for _ in 0..width * rows {
        let first_cpu = bank | u32::from(low);
        let first = crate::world::tracks::physical(crate::world::tracks::CpuSource(first_cpu))?;
        // Fetch's second byte is a 24-bit long-indirect increment; advancing
        // the pointer for the next word wraps ONLY the low word.
        let second = crate::world::tracks::physical(crate::world::tracks::CpuSource(
            first_cpu
                .checked_add(1)
                .context("rectangle CPU address overflow")?,
        ))?;
        words.push(u16::from_le_bytes([
            *rom.get(first).context("truncated rectangle low byte")?,
            *rom.get(second).context("truncated rectangle high byte")?,
        ]));
        low = low.wrapping_add(2);
    }
    Ok(words)
}
pub fn upload_sprite_palette(memory: &mut VideoMemory, assets: &SharedGraphics) -> Result<()> {
    const UPPER_PALETTE: usize = 128;
    ensure!(
        assets.sprite_palette.len() == UPPER_PALETTE,
        "invalid shared sprite palette"
    );
    memory.palette[UPPER_PALETTE..].copy_from_slice(&assets.sprite_palette);
    Ok(())
}
