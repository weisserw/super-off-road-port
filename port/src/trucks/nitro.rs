//! Nitro inventory in charges. Packed decimal belongs to source assets, HUD and
//! version2 saves; it is not the movement/shop representation.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stock {
    Charges(u8),
    /// Historical saves accepted all byte values. Preserve those exceptional
    /// states without assigning invalid decimal digits a new gameplay meaning.
    Legacy(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "u8", into = "u8")]
pub struct NitroStock(Stock);

impl Default for NitroStock {
    fn default() -> Self {
        Self::new(0)
    }
}

impl NitroStock {
    pub const CAPACITY: u8 = 99;

    pub const fn new(charges: u8) -> Self {
        assert!(charges <= Self::CAPACITY);
        Self(Stock::Charges(charges))
    }

    /// Source asset and legacy save boundary. Do not normalize malformed digits.
    pub fn from_encoded(encoded: u8) -> Self {
        if encoded >> 4 <= 9 && encoded & 15 <= 9 {
            Self::new((encoded >> 4) * 10 + (encoded & 15))
        } else {
            Self(Stock::Legacy(encoded))
        }
    }

    pub fn encoded(self) -> u8 {
        match self.0 {
            Stock::Charges(charges) => ((charges / 10) << 4) | (charges % 10),
            Stock::Legacy(encoded) => encoded,
        }
    }

    /// Digits are returned in HUD order: units, then tens.
    pub fn display_digits(self) -> [u8; 2] {
        match self.0 {
            Stock::Charges(charges) => [charges % 10, charges / 10],
            Stock::Legacy(encoded) => [encoded & 15, encoded >> 4],
        }
    }

    pub fn at_cap(self, cap: Self) -> bool {
        match (self.0, cap.0) {
            (Stock::Charges(charges), Stock::Charges(limit)) => charges >= limit,
            _ => self.encoded() >= cap.encoded(),
        }
    }

    /// Caller owns the press edge, prepared-frame guard and subsequent effects.
    pub fn consume(&mut self) -> bool {
        match self.0 {
            Stock::Charges(0) => return false,
            Stock::Charges(charges) => *self = Self::new(charges - 1),
            Stock::Legacy(encoded) => {
                let borrowed = if encoded & 15 == 0 { 6 } else { 0 };
                *self = Self::from_encoded(encoded.wrapping_sub(1).wrapping_sub(borrowed));
            }
        }
        true
    }

    /// Source shop/inactive conversion checks its own cap before refilling.
    /// A conditional refill at99 wraps to0 rather than saturating.
    pub fn refill_one(&mut self) {
        *self = match self.0 {
            Stock::Charges(charges) => Self::new((charges + 1) % (Self::CAPACITY + 1)),
            Stock::Legacy(encoded) => Self::from_encoded(legacy_increment(encoded)),
        };
    }

    /// Pickups retain the complete previous stock on overflow, unlike a shop
    /// refill. The caller retains pickup count/state, sound and projection order.
    pub fn award_pickup(&mut self, charges: u16) {
        let current = match self.0 {
            Stock::Charges(current) => u16::from(current),
            Stock::Legacy(encoded) => u16::from(encoded >> 4) * 10 + u16::from(encoded & 15),
        };
        let total = current + charges;
        if total <= u16::from(Self::CAPACITY) {
            *self = Self::new(total as u8);
        }
    }
}

// Faithful-port compatibility for nondecimal save bytes only; normal inventory
// operations above never execute packed arithmetic. This matches the historical
// increment_decimal policy, not a new claim about invalid hardware decimal ADC.
fn legacy_increment(encoded: u8) -> u8 {
    let low = (encoded & 15) + 1;
    let adjusted = if low > 9 { low + 6 } else { low };
    let high = (encoded & 0xf0).wrapping_add(adjusted);
    if high > 0x99 {
        high.wrapping_add(0x60)
    } else {
        high
    }
}

impl From<u8> for NitroStock {
    fn from(encoded: u8) -> Self {
        Self::from_encoded(encoded)
    }
}
impl From<NitroStock> for u8 {
    fn from(stock: NitroStock) -> Self {
        stock.encoded()
    }
}
