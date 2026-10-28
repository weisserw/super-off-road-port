//! Session leaderboard entries. Digit rows are asset, wallet and save adapters.
use anyhow::{Result, ensure};
use std::cmp::Ordering;

pub const SCORE_ROWS: usize = 10;
const SCORE_DIGITS: usize = 6;
const SCORE_UNIT: u32 = 100;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScoreValue {
    Amount(u32),
    // Nondecimal legacy rows are observable in restored and direct-call state.
    LegacyDigits([u8; SCORE_DIGITS]),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Score(ScoreValue);
impl Score {
    pub fn from_digits(digits: [u8; SCORE_DIGITS]) -> Self {
        if digits.iter().all(|&digit| digit < 10) {
            let amount = digits
                .into_iter()
                .fold(0u32, |value, digit| value * 10 + u32::from(digit))
                * SCORE_UNIT;
            Self(ScoreValue::Amount(amount))
        } else {
            Self(ScoreValue::LegacyDigits(digits))
        }
    }
    pub fn digits(self) -> [u8; SCORE_DIGITS] {
        match self.0 {
            ScoreValue::Amount(amount) => {
                let mut units = amount / SCORE_UNIT;
                let mut digits = [0; SCORE_DIGITS];
                for digit in digits.iter_mut().rev() {
                    *digit = (units % 10) as u8;
                    units /= 10;
                }
                digits
            }
            ScoreValue::LegacyDigits(digits) => digits,
        }
    }
}
impl Ord for Score {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.0, other.0) {
            (ScoreValue::Amount(a), ScoreValue::Amount(b)) => a.cmp(&b),
            _ => self.digits().cmp(&other.digits()),
        }
    }
}
impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
/// Three characters in the game's shared name alphabet, not rendered tilewords.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Initials(pub [u8; 3]);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScoreEntry {
    pub score: Score,
    pub name: Initials,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Leaderboard {
    pub entries: [ScoreEntry; SCORE_ROWS],
    /// Stable storage identities, ascending by displayed rank. Direct callers
    /// may retain repeated IDs; the full Game validates a permutation separately.
    pub order: [u8; SCORE_ROWS],
}
impl Leaderboard {
    pub fn from_legacy(digits: [u8; 60], names: [u8; 30], order: [u8; 10]) -> Self {
        Self {
            entries: std::array::from_fn(|id| ScoreEntry {
                score: Score::from_digits(digits[id * 6..id * 6 + 6].try_into().unwrap()),
                name: Initials(names[id * 3..id * 3 + 3].try_into().unwrap()),
            }),
            order,
        }
    }
    pub fn legacy_digits(&self) -> [u8; 60] {
        std::array::from_fn(|index| self.entries[index / 6].score.digits()[index % 6])
    }
    pub fn legacy_names(&self) -> [u8; 30] {
        std::array::from_fn(|index| self.entries[index / 3].name.0[index % 3])
    }
    pub fn valid_ids(&self) -> bool {
        self.order.iter().all(|&id| usize::from(id) < SCORE_ROWS)
    }
    pub fn insert(&mut self, score: Score, name: Initials) -> Result<bool> {
        ensure!(self.valid_ids(), "invalid score indirection");
        for rank in (0..SCORE_ROWS).rev() {
            let id = usize::from(self.order[rank]);
            if score > self.entries[id].score {
                let storage = self.order[0];
                self.entries[usize::from(storage)] = ScoreEntry { score, name };
                self.order.copy_within(1..rank + 1, 0);
                self.order[rank] = storage;
                return Ok(true);
            }
        }
        Ok(false)
    }

    // Historical tests retain raw fixture construction at an explicit boundary.
    #[cfg(test)]
    pub fn set_legacy_digits(&mut self, digits: [u8; 60]) {
        *self = Self::from_legacy(digits, self.legacy_names(), self.order);
    }
    #[cfg(test)]
    pub fn set_legacy_names(&mut self, names: [u8; 30]) {
        *self = Self::from_legacy(self.legacy_digits(), names, self.order);
    }
    #[cfg(test)]
    pub fn set_legacy_digit(&mut self, index: usize, value: u8) {
        let mut digits = self.legacy_digits();
        digits[index] = value;
        self.set_legacy_digits(digits);
    }
    #[cfg(test)]
    pub fn set_legacy_digit_slice(&mut self, range: std::ops::Range<usize>, values: &[u8]) {
        let mut digits = self.legacy_digits();
        digits[range].copy_from_slice(values);
        self.set_legacy_digits(digits);
    }
    #[cfg(test)]
    pub fn set_legacy_name_slice(&mut self, range: std::ops::Range<usize>, values: &[u8]) {
        let mut names = self.legacy_names();
        names[range].copy_from_slice(values);
        self.set_legacy_names(names);
    }
}
