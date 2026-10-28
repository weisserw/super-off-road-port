//! Optional arcade rules, retained in snapshots independently of Classic's RNG use.
use crate::{
    controls::*,
    game::resources::Choice,
    presentation::{self, Display, VideoMemory, memory},
    race::assets::RaceAssets,
    runtime::{RandomResultWidth, RandomState},
};
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Arcade {
    pub accepted: bool,
    pub random: bool,
    pub previous: u16,
    pub order: Vec<usize>,
    pub cursor: usize,
    pub reversed: bool,
    pub seed: u16,
}
impl Default for Arcade {
    fn default() -> Self {
        Self {
            accepted: false,
            random: true,
            previous: 0,
            order: vec![],
            cursor: 0,
            reversed: false,
            seed: 0,
        }
    }
}
impl Arcade {
    pub fn input(&mut self, held: u16) -> bool {
        let pressed = held & !self.previous;
        self.previous = held;
        if pressed & (BUTTON_UP | BUTTON_DOWN | BUTTON_SELECT) != 0 {
            self.random = !self.random;
        }
        if pressed & (BUTTON_START | BUTTON_A | BUTTON_B) != 0 {
            self.accepted = true;
        }
        self.accepted
    }
    pub fn start(&mut self, sequence: &[Choice], rng: &mut RandomState) {
        if !self.random {
            return;
        }
        // Each named track appears once per cycle, regardless of repetitions in
        // the original 64-race progression.
        self.order = (0..sequence.len())
            .filter(|&i| !sequence[..i].iter().any(|c| c.id == sequence[i].id))
            .collect();
        for i in (1..self.order.len()).rev() {
            let j = usize::from(rng.next(RandomResultWidth::Word)) % (i + 1);
            self.order.swap(i, j);
        }
        self.cursor = 0;
    }
    pub fn roll(&mut self, rng: &mut RandomState) {
        if self.accepted && self.random {
            self.reversed = rng.next(RandomResultWidth::Byte) & 1 != 0;
            self.seed = rng.next(RandomResultWidth::Word);
        }
    }
    pub fn validate(&self, count: usize) -> Result<()> {
        ensure!(
            self.order.iter().all(|&i| i < count)
                && (self.order.is_empty() || self.cursor < self.order.len()),
            "invalid arcade order"
        );
        ensure!(
            !(self.accepted && self.random && self.order.is_empty()),
            "missing random arcade order"
        );
        Ok(())
    }
}
pub fn randomize(race: &mut RaceAssets, mode: &Arcade, sequence: &[Choice]) -> Result<()> {
    race.reversed = mode.reversed;
    let choice = &sequence[mode.order[mode.cursor]];
    if let Some(track) = choice.original.filter(|_| race.track.objects.is_none()) {
        let slots: Vec<_> = crate::world::tracks::decode_order(
            &race.rom[crate::world::tracks::ORDER_ROM
                ..crate::world::tracks::ORDER_ROM + crate::world::tracks::ORDER_SLOTS],
        )?
        .iter()
        .enumerate()
        .filter_map(|(i, c)| (c.original_track == track).then_some(i as u8))
        .collect();
        if !slots.is_empty() {
            let slot = slots[usize::from(mode.seed) % slots.len()];
            race.objects.dynamic =
                crate::world::objects::load_original_content(&race.rom, track, slot)?.dynamic;
        }
    }
    // Keep source positions and heights so bales stay on the course. Randomly
    // omit bales rather than moving obstacles into walls or the starting grid.
    let mut bits = u32::from(mode.seed) | 0x10000;
    for bale in &mut race.objects.dynamic {
        bits = bits.wrapping_mul(1664525).wrapping_add(1013904223);
        if bits >> 30 == 0 {
            bale.world = [0; 2];
        }
    }
    Ok(())
}

pub fn draw(
    video: &mut VideoMemory,
    display: &mut Display,
    shared: &memory::SharedGraphics,
    random: bool,
) -> Result<()> {
    presentation::reset_display(display);
    presentation::configure_race(display);
    memory::upload_fonts(video, shared, true)?;
    memory::upload_text_palette(video, shared)?;
    video.palette[0] = 0;
    // The original alphabet has no parentheses. Extend its paired-tile font
    // with two small punctuation glyphs, retaining its palette and tile scale.
    for (glyph, mirrored) in [(40usize, false), (41usize, true)] {
        let shape: [u8; 16] = [0, 6, 12, 24, 24, 48, 48, 48, 48, 48, 48, 24, 24, 12, 6, 0];
        for (y, bits) in shape.into_iter().enumerate() {
            let bits = if mirrored { bits.reverse_bits() } else { bits };
            let at = 0xa000 + (glyph * 2 + y / 8) * 16 + (y % 8) * 2;
            video.vram[at] = bits;
            video.vram[at + 1] = bits;
        }
    }
    display.main_layers = 4;
    display.force_blank = false;
    display.brightness = 15;
    for (row, text) in [
        (7, "SELECT MODE"),
        (11, "ARCADE (RANDOM)"),
        (15, "ARCADE (CLASSIC)"),
    ] {
        let mut glyphs: Vec<u8> = text
            .bytes()
            .map(|b| match b {
                b'A'..=b'Z' => b - b'A' + 11,
                b'(' => 40,
                b')' => 41,
                b'/' => 39,
                _ => 0,
            })
            .collect();
        glyphs.push(255);
        memory::draw_text(
            video,
            &glyphs,
            0x4800 + row * 32 + (32 - text.len() as u16) / 2,
            memory::TextSize::Paired,
        )?;
    }
    memory::draw_text(
        video,
        &[34, 255],
        0x4800 + if random { 11 } else { 15 } * 32 + 5,
        memory::TextSize::Paired,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn arcade_menu_requires_a_fresh_press_and_classic_preserves_rng() {
        let mut menu = Arcade {
            previous: BUTTON_START,
            ..Default::default()
        };
        assert!(!menu.input(BUTTON_START));
        assert!(!menu.input(0));
        assert!(!menu.input(BUTTON_DOWN));
        assert!(!menu.random);
        menu.input(0);
        assert!(menu.input(BUTTON_A));
        let mut rng = RandomState([1, 2, 3, 4, 5]);
        let before = rng;
        menu.start(&[], &mut rng);
        menu.roll(&mut rng);
        assert_eq!(rng, before);
    }
    #[test]
    fn arcade_random_cycle_has_each_track_once_and_changes_with_seed() {
        let sequence: Vec<_> = (0..64)
            .map(|i| Choice {
                id: format!("track-{}", i % 16),
                original: Some(i % 16),
                reversed: false,
            })
            .collect();
        let mut orders = Vec::new();
        for seed in [[1, 2, 3, 4, 5], [18, 52, 86, 120, 155]] {
            let mut mode = Arcade {
                accepted: true,
                ..Default::default()
            };
            let mut rng = RandomState(seed);
            mode.start(&sequence, &mut rng);
            mode.roll(&mut rng);
            mode.validate(sequence.len()).unwrap();
            let mut sorted = mode.order.clone();
            sorted.sort_unstable();
            assert_eq!(sorted, (0..16).collect::<Vec<_>>());
            orders.push(mode.order);
        }
        assert_ne!(orders[0], orders[1]);
    }
}
