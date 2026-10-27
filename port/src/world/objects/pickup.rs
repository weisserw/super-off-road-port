//! Pickup lifecycle, reward quantities and decoded presentation/award resources.
//! Original state numbers are asset and version2-save encodings, never the
//! dispatcher for ordinary spawn, collection or expiry. See objects.trace.md.
use super::{PICKUP_TYPE_CAP, PICKUP_TYPES};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickupKind {
    Money,
    Nitro,
}
impl PickupKind {
    pub fn for_frame(frame: u8) -> Self {
        if frame & 1 == 0 {
            Self::Money
        } else {
            Self::Nitro
        }
    }
    pub fn index(self) -> usize {
        match self {
            Self::Money => 0,
            Self::Nitro => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickupReward {
    Money(u8),
    Nitro(u16),
}
impl PickupReward {
    /// Asset-boundary rule: overrides can change reward category independently
    /// of the collected kind. Source pending-money addition consumes a byte.
    fn decode(state: u16) -> Self {
        const MONEY_BASE: u16 = 6;
        const FIRST_MONEY: u16 = 7;
        const NITRO_BASE: u16 = 2;
        if state >= FIRST_MONEY {
            Self::Money(state.wrapping_sub(MONEY_BASE) as u8)
        } else {
            Self::Nitro(state.wrapping_sub(NITRO_BASE))
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "u16", into = "u16")]
pub enum PickupPhase {
    #[default]
    Absent,
    Available(PickupKind),
    AwardDisplay(PickupReward),
    /// Full Game snapshots reject these values, as before. Direct domain calls
    /// can retain them until presentation rejects their unsupported tile lookup.
    Legacy(u16),
}
impl PickupPhase {
    pub fn from_encoded(state: u16) -> Self {
        match state {
            0 => Self::Absent,
            1 => Self::Available(PickupKind::Money),
            2 => Self::Available(PickupKind::Nitro),
            3..=6 => Self::AwardDisplay(PickupReward::Nitro(state - 2)),
            7..=11 => Self::AwardDisplay(PickupReward::Money((state - 6) as u8)),
            value => Self::Legacy(value),
        }
    }
    pub fn encoded(self) -> u16 {
        match self {
            Self::Absent => 0,
            Self::Available(PickupKind::Money) => 1,
            Self::Available(PickupKind::Nitro) => 2,
            Self::AwardDisplay(PickupReward::Nitro(charges)) => charges + 2,
            Self::AwardDisplay(PickupReward::Money(units)) => u16::from(units) + 6,
            Self::Legacy(value) => value,
        }
    }
    pub fn supported(self) -> bool {
        !matches!(self, Self::Legacy(_))
    }
}
impl From<u16> for PickupPhase {
    fn from(value: u16) -> Self {
        Self::from_encoded(value)
    }
}
impl From<PickupPhase> for u16 {
    fn from(value: PickupPhase) -> Self {
        value.encoded()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PickupArt {
    pub primary: u16,
    pub extra: Option<u16>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Collection {
    pub phase: PickupPhase,
    pub reward: PickupReward,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Award {
    Decoded(Collection),
    /// Malformed override arithmetic historically fails only when collected in
    /// checked builds. Keep it deferred rather than failing unrelated loading.
    Overflow {
        base: u16,
        count: u16,
    },
}

/// Immutable decoded outcomes for each kind and its five accepted old counts.
/// Art includes the incidental state11 adjacent-ROM pair used by legacy saves.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PickupAssets {
    awards: [[Award; PICKUP_TYPE_CAP as usize + 1]; PICKUP_TYPES],
    art: [PickupArt; 11],
}
impl PickupAssets {
    pub fn decode(bases: [u16; PICKUP_TYPES], tiles: [u16; 11], extra: [u16; 9]) -> Self {
        Self {
            awards: std::array::from_fn(|kind| {
                std::array::from_fn(|count| {
                    if let Some(state) = bases[kind].checked_add(count as u16) {
                        Award::Decoded(Collection {
                            phase: PickupPhase::from_encoded(state),
                            reward: PickupReward::decode(state),
                        })
                    } else {
                        Award::Overflow {
                            base: bases[kind],
                            count: count as u16,
                        }
                    }
                })
            }),
            art: std::array::from_fn(|index| PickupArt {
                primary: tiles[index],
                extra: if index >= 2 {
                    Some(extra[index - 2])
                } else {
                    None
                },
            }),
        }
    }
    pub fn collection(&self, kind: PickupKind, old_count: u16) -> Result<Collection> {
        let award = self.awards[kind.index()]
            .get(usize::from(old_count))
            .context("unsupported pickup count")?;
        Ok(match *award {
            Award::Decoded(collection) => collection,
            Award::Overflow { base, count } => {
                let state = base + count;
                Collection {
                    phase: PickupPhase::from_encoded(state),
                    reward: PickupReward::decode(state),
                }
            }
        })
    }
    /// Presentation boundary only. Gameplay never derives an award from this
    /// index; phase/reward is already owned by the pickup and decoded outcomes.
    pub fn art(&self, phase: PickupPhase) -> Result<PickupArt> {
        let index = phase
            .encoded()
            .checked_sub(1)
            .context("cannot project absent pickup")?;
        self.art
            .get(usize::from(index))
            .copied()
            .context("invalid pickup state")
    }
}
