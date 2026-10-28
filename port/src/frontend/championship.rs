//! Finite championship rules. Economy, schedule and adaptation are independent
//! of the Arcade modes and of the gameplay random stream.
use super::{
    arcade::Arcade,
    attract::Session,
    shop::{self, Item, Purchase, ShopAssets},
};
use crate::{
    game::resources::Choice,
    presentation::{self, Display, VideoMemory, memory},
    runtime::RandomState,
    trucks::motion::{Truck, Upgrades},
};
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    #[default]
    Normal,
    Hard,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Championship {
    pub active: bool,
    /// 0 Championship, 1 Random, 2 Classic.
    pub mode: u8,
    pub cup: usize,
    pub difficulty: Difficulty,
    pub previous: u16,
    pub schedule: Vec<(usize, bool, u16)>,
    pub completed: usize,
    pub points: [u16; 4],
    pub finishes: [[u8; 4]; 4],
    pub last_rank: [u8; 4],
    pub earned: [u64; 4],
    pub before_awards: [u64; 4],
    pub poor_streak: [u8; 4],
    pub adjustment: i8,
    pub purchased: Option<[Upgrades; 4]>,
}
impl Championship {
    fn difficulty_index(&self) -> usize {
        match self.difficulty {
            Difficulty::Easy => 0,
            Difficulty::Normal => 1,
            Difficulty::Hard => 2,
        }
    }
    pub fn pressed(&mut self, held: u16) -> u16 {
        let pressed = held & !self.previous;
        self.previous = held;
        pressed
    }
    pub fn start(&mut self, sequence: &[Choice]) -> Result<()> {
        let mut rng = RandomState(crate::config::CHAMPIONSHIP_SEEDS[self.cup]);
        // Select by original identity, not catalogue order or custom additions.
        let originals: Vec<_> = (0..16)
            .filter_map(|id| sequence.iter().position(|c| c.original == Some(id)))
            .map(|i| sequence[i].clone())
            .collect();
        ensure!(
            originals.len() == 16,
            "championship needs all sixteen original tracks"
        );
        let mut random = Arcade {
            accepted: true,
            ..Default::default()
        };
        random.start(&originals, &mut rng);
        self.schedule.clear();
        for cursor in 0..crate::config::CHAMPIONSHIP_RACES {
            random.cursor = cursor;
            random.roll(&mut rng);
            let id = &originals[random.order[cursor]].id;
            let index = sequence.iter().position(|c| &c.id == id).unwrap();
            self.schedule.push((index, random.reversed, random.seed));
        }
        self.active = true;
        Ok(())
    }
    pub fn race_mode(&self, sequence_index: usize) -> Arcade {
        let (index, reversed, seed) = self
            .schedule
            .iter()
            .copied()
            .find(|&(i, _, _)| i == sequence_index)
            .unwrap_or(self.schedule[0]);
        Arcade {
            accepted: true,
            random: true,
            order: vec![index],
            reversed,
            seed,
            ..Default::default()
        }
    }
    pub fn standings(&self) -> [usize; 4] {
        let mut order = [0, 1, 2, 3];
        order.sort_by_key(|&slot| {
            (
                std::cmp::Reverse(self.points[slot]),
                std::cmp::Reverse(self.finishes[slot][0]),
                std::cmp::Reverse(self.finishes[slot][1]),
                std::cmp::Reverse(self.finishes[slot][2]),
                self.last_rank
                    .iter()
                    .position(|&s| usize::from(s) == slot)
                    .unwrap_or(slot),
            )
        });
        order
    }
    pub fn finish_race(&mut self, rank: [u8; 4], session: &Session, trucks: &mut [Truck; 4]) {
        self.last_rank = rank;
        for (place, &slot) in rank.iter().enumerate() {
            let slot = usize::from(slot);
            self.points[slot] += [9, 6, 3, 0][place];
            self.finishes[slot][place] += 1;
            self.poor_streak[slot] = if place >= 2 {
                self.poor_streak[slot].saturating_add(1)
            } else {
                0
            };
            self.earned[slot] +=
                wallet(&session.wallets[slot]).saturating_sub(self.before_awards[slot]) * 100;
        }
        if let Some(purchased) = self.purchased.take() {
            for (truck, upgrades) in trucks.iter_mut().zip(purchased) {
                truck.upgrades = upgrades;
            }
        }
        self.completed += 1;
        self.adapt(session.active_humans);
    }
    fn adapt(&mut self, humans: u8) {
        let order = self.standings();
        let Some(human) = order.into_iter().find(|&s| humans & (1 << s) != 0) else {
            return;
        };
        let Some(cpu) = order.into_iter().find(|&s| humans & (1 << s) == 0) else {
            return;
        };
        let lead = self.points[human] as i16 - self.points[cpu] as i16;
        // A single upset never grants relief; a human leader never gets it.
        let target = if lead <= -9 && self.poor_streak[human] >= 2 {
            crate::config::CHAMPIONSHIP_RELIEF_LIMIT[self.difficulty_index()]
        } else if lead >= 6 {
            crate::config::CHAMPIONSHIP_CATCHUP_LIMIT[self.difficulty_index()]
        } else {
            0
        };
        self.adjustment += (target - self.adjustment).signum();
        if lead >= 0 {
            self.adjustment = self.adjustment.max(0);
        }
    }
    pub fn buy(
        &self,
        session: &mut Session,
        trucks: &mut [Truck; 4],
        assets: &ShopAssets,
    ) -> Result<()> {
        let selected = session.selected_wallet;
        for slot in 0..4 {
            if session.active_humans & (1 << slot) != 0 {
                continue;
            }
            session.selected_wallet = slot as u8;
            shop::normalize_wallet(session)?;
            let budget = wallet(&session.wallets[slot]);
            let nitro_budget = budget * crate::config::CHAMPIONSHIP_NITRO_PERCENT[slot] / 100;
            let mut spent = 0;
            let limit = crate::config::CHAMPIONSHIP_NITRO_LIMIT[slot];
            for _ in 0..limit {
                let price = u64::from(assets.costs[0]) * 100;
                if spent + price > nitro_budget
                    || trucks[slot]
                        .nitro
                        .at_cap(crate::trucks::nitro::NitroStock::new(limit))
                {
                    break;
                }
                if shop::purchase(
                    session,
                    trucks,
                    (slot / 2) as u8,
                    slot % 2,
                    Item::Nitro,
                    assets,
                )? != Purchase::Bought
                {
                    break;
                }
                shop::normalize_wallet(session)?;
                spent += price;
            }
            // Balance all permanent fields; tie order gives each rival a small personality.
            let priorities = if slot == 2 {
                [3, 1, 0, 2]
            } else {
                [2, 0, 1, 3]
            };
            loop {
                let u = trucks[slot].upgrades;
                let levels = [u.acceleration, u.steering, u.speed, u.suspension];
                let mut fields = priorities;
                fields.sort_by_key(|&i| levels[i]);
                let mut bought = false;
                for field in fields {
                    let item = [
                        Item::Acceleration,
                        Item::Steering,
                        Item::Speed,
                        Item::Suspension,
                    ][field];
                    if shop::purchase(session, trucks, (slot / 2) as u8, slot % 2, item, assets)?
                        == Purchase::Bought
                    {
                        shop::normalize_wallet(session)?;
                        bought = true;
                        break;
                    }
                }
                if !bought {
                    break;
                }
            }
        }
        session.selected_wallet = selected;
        Ok(())
    }
    pub fn prepare(&mut self, trucks: &mut [Truck; 4], humans: u8) {
        self.purchased = Some(trucks.each_ref().map(|t| t.upgrades));
        let base = crate::config::CHAMPIONSHIP_BASE_BONUS[self.difficulty_index()];
        for (slot, truck) in trucks.iter_mut().enumerate() {
            truck.championship = true;
            truck.championship_speed_bonus = 0;
            if humans & (1 << slot) != 0 {
                continue;
            }
            let bonus = base + crate::config::CHAMPIONSHIP_CAR_BONUS[slot] + self.adjustment;
            // Keep rival identities at the normal upgrade ceiling without using
            // the original large jump to speed selector seven.
            truck.championship_speed_bonus = bonus.clamp(0, 6) as u8;
            let level = |v: u8| (i16::from(v) + i16::from(bonus)).clamp(0, 6) as u8;
            truck.upgrades = Upgrades {
                acceleration: level(truck.upgrades.acceleration),
                steering: level(truck.upgrades.steering),
                speed: level(truck.upgrades.speed),
                suspension: level(truck.upgrades.suspension),
            };
        }
    }
    pub fn validate(&self, sequence: &[Choice]) -> Result<()> {
        ensure!(
            self.mode < 3
                && self.cup < 4
                && self.completed <= 8
                && (-3..=2).contains(&self.adjustment),
            "invalid championship state"
        );
        if let Some(purchased) = self.purchased {
            ensure!(
                self.active
                    && self.completed < 8
                    && purchased.iter().all(|u| [
                        u.speed,
                        u.acceleration,
                        u.steering,
                        u.suspension
                    ]
                    .iter()
                    .all(|&v| v <= 6)),
                "invalid championship purchased upgrades"
            );
        }
        if self.active {
            ensure!(self.schedule.len() == 8, "invalid championship schedule");
            let mut seen = Vec::new();
            for &(i, _, _) in &self.schedule {
                let original = sequence.get(i).and_then(|c| c.original);
                ensure!(
                    original.is_some() && !seen.contains(&original),
                    "invalid championship track"
                );
                seen.push(original);
            }
            let mut rank = self.last_rank;
            rank.sort_unstable();
            ensure!(
                self.completed == 0 || rank == [0, 1, 2, 3],
                "invalid championship rank"
            );
            for slot in 0..4 {
                ensure!(
                    self.finishes[slot]
                        .iter()
                        .map(|&v| usize::from(v))
                        .sum::<usize>()
                        == self.completed
                        && self.points[slot]
                            == u16::from(self.finishes[slot][0]) * 9
                                + u16::from(self.finishes[slot][1]) * 6
                                + u16::from(self.finishes[slot][2]) * 3,
                    "invalid championship points"
                );
            }
        }
        Ok(())
    }
}
pub fn wallet(digits: &[u8; 6]) -> u64 {
    digits.iter().fold(0, |n, &d| n * 10 + u64::from(d))
}

fn text_glyph(byte: u8) -> u8 {
    match byte {
        b'A'..=b'Z' => byte - b'A' + 11,
        b'a'..=b'z' => byte - b'a' + 11,
        b'0'..=b'9' => byte - b'0' + 1,
        b'!' => 40,
        b'/' => 39,
        _ => 0,
    }
}

/// Preserve the native font while positioning this line between tile columns.
pub fn draw_victory_line(video: &mut VideoMemory, row: u16, text: &str) -> Result<()> {
    let source: Vec<_> = text.bytes().map(text_glyph).collect();
    ensure!(
        source.len() <= 31,
        "victory line exceeds centered font span"
    );
    let mut shifted = Vec::new();
    for i in 0..=source.len() {
        let glyph = 80 + i;
        for y in 0..16 {
            for plane in 0..2 {
                let source_byte = |g: u8| {
                    video.vram[0xa000 + (usize::from(g) * 2 + y / 8) * 16 + (y % 8) * 2 + plane]
                };
                let current = source.get(i).map_or(0, |&g| source_byte(g));
                let previous = if i == 0 {
                    0
                } else {
                    source_byte(source[i - 1])
                };
                let at = 0xa000 + (glyph * 2 + y / 8) * 16 + (y % 8) * 2 + plane;
                video.vram[at] = (current >> 4) | (previous << 4);
            }
        }
        shifted.push(glyph as u8);
    }
    shifted.push(255);
    memory::draw_text(
        video,
        &shifted,
        0x4800 + row * 32 + (32 - source.len() as u16) / 2,
        memory::TextSize::Paired,
    )?;
    Ok(())
}

pub fn screen(
    video: &mut VideoMemory,
    display: &mut Display,
    shared: &memory::SharedGraphics,
    lines: &[(u16, String)],
    selected: Option<u16>,
) -> Result<()> {
    presentation::reset_display(display);
    presentation::configure_race(display);
    memory::upload_fonts(video, shared, true)?;
    memory::upload_text_palette(video, shared)?;
    memory::fill_map(video, 0x4800, 0);
    video.palette[0] = 0;
    display.main_layers = 4;
    display.force_blank = false;
    display.brightness = 15;
    // The paired font's unused glyph 40 supplies the finale's punctuation.
    for (y, bits) in [24u8, 24, 24, 24, 24, 24, 24, 24, 24, 24, 0, 0, 24, 24, 0, 0]
        .into_iter()
        .enumerate()
    {
        let at = 0xa000 + (40 * 2 + y / 8) * 16 + (y % 8) * 2;
        video.vram[at] = bits;
        video.vram[at + 1] = bits;
    }
    for (row, text) in lines {
        let mut glyphs: Vec<u8> = text.bytes().map(text_glyph).collect();
        glyphs.push(255);
        memory::draw_text(
            video,
            &glyphs,
            0x4800 + row * 32 + (32 - text.len().min(32) as u16) / 2,
            memory::TextSize::Paired,
        )?;
    }
    if let Some(row) = selected {
        let choice_width = lines
            .iter()
            .filter(|(row, _)| *row >= 9)
            .map(|(_, text)| text.len().min(32) as u16)
            .max()
            .unwrap_or(0);
        let cursor_column = ((32 - choice_width) / 2).saturating_sub(2);
        memory::draw_text(
            video,
            &[34, 255],
            0x4800 + row * 32 + cursor_column,
            memory::TextSize::Paired,
        )?;
    }
    Ok(())
}

/// Name entry commits both panel slots, including placeholders for an unjoined
/// player. Championship uses the original CPU identities for those slots.
pub fn restore_cpu_initials(session: &mut Session, defaults: &[u8; 12]) {
    let humans = session.joined_pages[0] | (session.joined_pages[1] << 2);
    for slot in 0..4 {
        if humans & (1 << slot) == 0 {
            session.records.truck_names[slot * 3..slot * 3 + 3]
                .copy_from_slice(&defaults[slot * 3..slot * 3 + 3]);
        }
    }
}

/// Use the original sixteen-color race bodies and palettes, with a fixed pose
/// that does not replace the race's retained pose selections or truck state.
pub fn draw_cars(
    video: &mut VideoMemory,
    display: &mut Display,
    shared: &memory::SharedGraphics,
    atlas: &presentation::poses::PoseAtlas,
    frame: presentation::poses::BodyFrame,
    x: u8,
    order: [usize; 4],
) -> Result<()> {
    use presentation::poses::PoseSelection;
    memory::upload_sprite_palette(video, shared)?;
    atlas.present(video, &std::array::from_fn(|_| PoseSelection::frame(frame)))?;
    for record in video.staged_oam[..512].chunks_exact_mut(4) {
        record.copy_from_slice(&[0, presentation::HIDDEN_Y, 0, 0]);
    }
    video.staged_oam[512..].fill(0);
    video.staged_oam[512] = 0xaa;
    for (place, slot) in order.into_iter().enumerate() {
        video.staged_oam[place * 4..place * 4 + 4].copy_from_slice(&[
            x,
            (7 + place as u8 * 3) * 8,
            (slot * 2) as u8,
            0x20 | (slot as u8 * 2),
        ]);
    }
    // These screens inhibit race NMI uploads and present their own static OAM.
    video.visible_oam = video.staged_oam;
    display.main_layers |= 0x10;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sequence() -> Vec<Choice> {
        (0..64)
            .map(|i| Choice {
                id: format!("track-{}", i % 16),
                original: Some(i % 16),
                reversed: false,
            })
            .collect()
    }
    #[test]
    fn championship_cups_are_stable_unique_and_ignore_custom_catalogue_entries() {
        let original = sequence();
        let mut extended = original.clone();
        extended.insert(
            0,
            Choice {
                id: "custom".into(),
                original: None,
                reversed: false,
            },
        );
        let mut schedules = Vec::new();
        for cup in 0..4 {
            let mut a = Championship {
                cup,
                ..Default::default()
            };
            let mut b = a.clone();
            a.start(&original).unwrap();
            b.start(&extended).unwrap();
            let expected_openers = [(0, false), (4, false), (8, true), (6, false)];
            let (index, reversed, _) = a.schedule[0];
            assert_eq!(
                (original[index].original.unwrap(), reversed),
                expected_openers[cup]
            );
            if cup == 0 || cup == 2 {
                let expected = if cup == 0 {
                    [0, 5, 1, 15, 2, 12, 4, 8]
                } else {
                    [8, 6, 0, 9, 12, 10, 1, 4]
                };
                assert_eq!(
                    a.schedule
                        .iter()
                        .map(|&(i, _, _)| original[i].original.unwrap())
                        .collect::<Vec<_>>(),
                    expected
                );
            }

            let identities = |s: &Championship, seq: &[Choice]| {
                s.schedule
                    .iter()
                    .map(|&(i, r, seed)| (seq[i].original.unwrap(), r, seed))
                    .collect::<Vec<_>>()
            };
            assert_eq!(identities(&a, &original), identities(&b, &extended));
            a.validate(&original).unwrap();
            schedules.push(identities(&a, &original));
        }
        for i in 0..4 {
            for j in 0..i {
                assert_ne!(schedules[i], schedules[j]);
            }
        }
    }
    #[test]
    fn championship_relief_requires_sustained_deficit_and_uses_stronger_human() {
        let mut s = Championship {
            points: [18, 0, 6, 27],
            poor_streak: [1, 3, 0, 0],
            ..Default::default()
        };
        s.adapt(3);
        assert_eq!(s.adjustment, 0);
        s.poor_streak[0] = 2;
        s.adapt(3);
        assert_eq!(s.adjustment, -1);
        s.adapt(3);
        assert_eq!(s.adjustment, -2);
        s.points[0] = 30;
        s.adapt(3);
        assert_eq!(s.adjustment, 0);
        s.points[0] = 45;
        s.adapt(3);
        assert_eq!(s.adjustment, 1);
        // A single loss by a previous leader cannot grant relief.
        s.points = [45, 0, 6, 48];
        s.poor_streak[0] = 1;
        s.adapt(3);
        assert_eq!(s.adjustment, 0);
    }
    #[test]
    fn championship_ties_use_wins_then_seconds_then_thirds_then_final_race() {
        let mut s = Championship {
            points: [18; 4],
            finishes: [[1, 1, 1, 0], [2, 0, 0, 1], [1, 1, 1, 0], [1, 1, 1, 0]],
            last_rank: [3, 2, 0, 1],
            ..Default::default()
        };
        assert_eq!(s.standings(), [1, 3, 2, 0]);
        s.finishes[3] = [1, 2, 0, 0];
        assert_eq!(s.standings(), [1, 3, 2, 0]);
    }
}
