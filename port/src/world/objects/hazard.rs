//! Native lifecycle and response quantities for the six dynamic hazards.
//! The source's three active types are behavioral stages, not established names
//! for physical objects. Legacy snapshot encodings remain at the wire boundary.
use super::{ObjectSprite, Placement};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HazardStage {
    #[default]
    Inactive,
    Reactive,
    Depleted,
    Struck,
    /// Direct conditional domain calls historically retain unknown types until
    /// a contact attempt rejects them. Full Game saves reject these as before.
    Legacy(u16),
}
impl HazardStage {
    pub fn from_encoded(kind: u16) -> Self {
        match kind {
            0 => Self::Inactive,
            1 => Self::Reactive,
            2 => Self::Depleted,
            3 => Self::Struck,
            value => Self::Legacy(value),
        }
    }
    pub fn encoded(self) -> u16 {
        match self {
            Self::Inactive => 0,
            Self::Reactive => 1,
            Self::Depleted => 2,
            Self::Struck => 3,
            Self::Legacy(value) => value,
        }
    }
    pub fn collision_index(self) -> Option<usize> {
        match self {
            Self::Reactive => Some(0),
            Self::Depleted => Some(1),
            Self::Struck => Some(2),
            _ => None,
        }
    }
    pub fn supported(self) -> bool {
        !matches!(self, Self::Legacy(_))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Clock {
    Complete,
    Remaining(u16),
    LegacyExpired(u16),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResponseClock(Clock);
impl Default for ResponseClock {
    fn default() -> Self {
        Self(Clock::Complete)
    }
}
pub enum ResponseTick {
    Remaining(u16),
    Complete(u16),
}
impl ResponseClock {
    pub fn start(ticks: u16) -> Self {
        Self::from_encoded(ticks)
    }
    pub fn from_encoded(value: u16) -> Self {
        Self(match value {
            0 => Clock::Complete,
            1..=0x7fff => Clock::Remaining(value),
            _ => Clock::LegacyExpired(value),
        })
    }
    pub fn encoded(self) -> u16 {
        match self.0 {
            Clock::Complete => 0,
            Clock::Remaining(ticks) | Clock::LegacyExpired(ticks) => ticks,
        }
    }
    pub fn highlighted(self) -> bool {
        const HIGHLIGHT_REMAINING_TICKS: u16 = 4;
        match self.0 {
            Clock::Complete => false,
            Clock::Remaining(ticks) => ticks >= HIGHLIGHT_REMAINING_TICKS,
            Clock::LegacyExpired(_) => true,
        }
    }
    pub fn valid_snapshot(self) -> bool {
        match self.0 {
            Clock::Remaining(ticks) => ticks <= super::DYNAMIC_RESPONSE_TICKS,
            _ => true,
        }
    }
    pub fn tick(&mut self) -> ResponseTick {
        match self.0 {
            Clock::Complete => ResponseTick::Complete(u16::MAX),
            Clock::Remaining(ticks) => {
                let remaining = ticks - 1;
                *self = Self::from_encoded(remaining);
                ResponseTick::Remaining(remaining)
            }
            Clock::LegacyExpired(value) => {
                let remaining = value.wrapping_sub(1);
                *self = Self::from_encoded(remaining);
                if remaining <= i16::MAX as u16 {
                    ResponseTick::Remaining(remaining)
                } else {
                    ResponseTick::Complete(remaining)
                }
            }
        }
    }
    /// Only the source animation29 completion exception retains a negative
    /// timer. Ordinary response completion stores no underflow sentinel.
    pub fn defer_completion(&mut self, encoded: u16) {
        *self = Self::from_encoded(encoded);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Budget {
    Remaining(u16),
    Spent,
    Legacy(u16),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReactionBudget(Budget);
impl Default for ReactionBudget {
    fn default() -> Self {
        Self::remaining(1)
    }
}
impl ReactionBudget {
    /// Source uses0 means one more complete response, not no reactions.
    pub fn remaining(count: u16) -> Self {
        assert!(count > 0 && count <= 32768);
        Self(Budget::Remaining(count))
    }
    pub fn from_encoded(value: u16) -> Self {
        match value {
            0..=0x7fff => Self::remaining(value + 1),
            0xffff => Self(Budget::Spent),
            _ => Self(Budget::Legacy(value)),
        }
    }
    pub fn encoded(self) -> u16 {
        match self.0 {
            Budget::Remaining(count) => count - 1,
            Budget::Spent => u16::MAX,
            Budget::Legacy(value) => value,
        }
    }
    /// Returns whether another response remains. Signed word behavior is
    /// confined to conditional historical counters, not the normal budget.
    pub fn finish_response(&mut self) -> bool {
        match self.0 {
            Budget::Remaining(1) => {
                self.0 = Budget::Spent;
                false
            }
            Budget::Remaining(count) => {
                self.0 = Budget::Remaining(count - 1);
                true
            }
            Budget::Spent | Budget::Legacy(_) => {
                let value = self.encoded().wrapping_sub(1);
                *self = Self::from_encoded(value);
                value <= i16::MAX as u16
            }
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "DynamicSnapshot", into = "DynamicSnapshot")]
pub struct DynamicObject {
    pub stage: HazardStage,
    pub placement: Placement,
    /// Source presentation cursor also defines the two contact windows.
    pub animation: u16,
    pub response: ResponseClock,
    pub reactions: ReactionBudget,
    pub sprite: ObjectSprite,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DynamicSnapshot {
    kind: u16,
    placement: Placement,
    animation: u16,
    timer: u16,
    uses: u16,
    sprite: ObjectSprite,
}
impl From<DynamicSnapshot> for DynamicObject {
    fn from(saved: DynamicSnapshot) -> Self {
        Self {
            stage: HazardStage::from_encoded(saved.kind),
            placement: saved.placement,
            animation: saved.animation,
            response: ResponseClock::from_encoded(saved.timer),
            reactions: ReactionBudget::from_encoded(saved.uses),
            sprite: saved.sprite,
        }
    }
}
impl From<DynamicObject> for DynamicSnapshot {
    fn from(object: DynamicObject) -> Self {
        Self {
            kind: object.stage.encoded(),
            placement: object.placement,
            animation: object.animation,
            timer: object.response.encoded(),
            uses: object.reactions.encoded(),
            sprite: object.sprite,
        }
    }
}
