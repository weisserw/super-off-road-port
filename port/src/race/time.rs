//! Elapsed source timing units. A service adds sixteen units; these units are
//! not independently established wall-clock milliseconds. Records discard the
//! fractional hundred units, while the HUD shows three whole-time digits.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Time {
    Elapsed(u32),
    /// Version2 accepted nondecimal timer bytes. Keep this exceptional save
    /// state losslessly and use the faithful conditional policy only for it.
    Legacy(EncodedTime),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "EncodedTime", into = "EncodedTime")]
pub struct RaceTimer(Time);

impl Default for RaceTimer {
    fn default() -> Self {
        Self(Time::Elapsed(0))
    }
}

impl RaceTimer {
    pub const UNITS_PER_SERVICE: u32 = 16;
    const FRACTION_UNITS: u32 = 100;
    const HIGH_UNITS: u32 = Self::FRACTION_UNITS * 100;
    const LAST_HIGH: u32 = 9;
    const LAST_MIDDLE: u32 = 99;

    /// Source/save/test-fixture boundary; array order is fraction, middle, high.
    pub fn from_encoded(bytes: [u8; 3]) -> Self {
        EncodedTime {
            fraction_bcd: bytes[0],
            middle_bcd: bytes[1],
            high_bcd: bytes[2],
        }
        .into()
    }

    pub fn encoded(self) -> [u8; 3] {
        let encoded = EncodedTime::from(self);
        [encoded.fraction_bcd, encoded.middle_bcd, encoded.high_bcd]
    }

    /// Original record storage/comparison retains the complete high byte, not
    /// merely the low digit visible on screen. This matters for legacy saves.
    pub fn record_encoded(self) -> [u8; 2] {
        let [_, middle, high] = self.encoded();
        [high, middle]
    }

    pub fn visible_digits(self) -> [u8; 3] {
        match self.0 {
            Time::Elapsed(units) => [
                ((units / Self::HIGH_UNITS) % 10) as u8,
                ((units / (Self::FRACTION_UNITS * 10)) % 10) as u8,
                ((units / Self::FRACTION_UNITS) % 10) as u8,
            ],
            Time::Legacy(encoded) => [
                encoded.high_bcd & 15,
                encoded.middle_bcd >> 4,
                encoded.middle_bcd & 15,
            ],
        }
    }

    pub fn advance(&mut self) {
        match self.0 {
            Time::Elapsed(ref mut units) => {
                // Test before adding. Crossing the limit retains its fraction
                // (9:98.90 -> 9:99.06); the NEXT service freezes it. High and
                // middle are separate tests even for unusual canonical saves.
                if *units / Self::HIGH_UNITS < Self::LAST_HIGH
                    || (*units / Self::FRACTION_UNITS) % 100 < Self::LAST_MIDDLE
                {
                    *units += Self::UNITS_PER_SERVICE;
                }
            }
            Time::Legacy(mut encoded) => {
                encoded.advance_legacy();
                *self = encoded.into();
            }
        }
    }
}

/// Wire representation only. Keeping the version2 field names also preserves
/// public semantic observations and identity-guarded historical save loading.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EncodedTime {
    fraction_bcd: u8,
    middle_bcd: u8,
    high_bcd: u8,
}

impl From<EncodedTime> for RaceTimer {
    fn from(encoded: EncodedTime) -> Self {
        let bytes = [encoded.fraction_bcd, encoded.middle_bcd, encoded.high_bcd];
        if bytes.iter().all(|byte| byte >> 4 <= 9 && byte & 15 <= 9) {
            let decimal = bytes.map(|byte| u32::from(byte >> 4) * 10 + u32::from(byte & 15));
            Self(Time::Elapsed(
                decimal[0] + decimal[1] * Self::FRACTION_UNITS + decimal[2] * Self::HIGH_UNITS,
            ))
        } else {
            Self(Time::Legacy(encoded))
        }
    }
}
impl From<RaceTimer> for EncodedTime {
    fn from(timer: RaceTimer) -> Self {
        match timer.0 {
            Time::Legacy(encoded) => encoded,
            Time::Elapsed(units) => {
                let packed = |value: u32| (((value / 10) << 4) | (value % 10)) as u8;
                Self {
                    fraction_bcd: packed(units % RaceTimer::FRACTION_UNITS),
                    middle_bcd: packed((units / RaceTimer::FRACTION_UNITS) % 100),
                    high_bcd: packed(units / RaceTimer::HIGH_UNITS),
                }
            }
        }
    }
}

impl EncodedTime {
    // Compatibility with the faithful port's accepted nondecimal states only;
    // do not claim an original invalid-BCD hardware experiment for this policy.
    fn advance_legacy(&mut self) {
        fn decimal_add(value: u8, increment: u8, carry: bool) -> (u8, bool) {
            let mut low = u16::from(value & 15) + u16::from(increment & 15) + u16::from(carry);
            let low_carry = low > 9;
            if low_carry {
                low -= 10;
            }
            let mut high = u16::from(value >> 4) + u16::from(increment >> 4) + u16::from(low_carry);
            let carry = high > 9;
            if carry {
                high -= 10;
            }
            (((high << 4) | low) as u8, carry)
        }
        if self.high_bcd < 0x09 || self.middle_bcd < 0x99 {
            let (fraction, carry) = decimal_add(self.fraction_bcd, 0x16, false);
            let (middle, carry) = decimal_add(self.middle_bcd, 0, carry);
            let (high, _) = decimal_add(self.high_bcd, 0, carry);
            *self = Self {
                fraction_bcd: fraction,
                middle_bcd: middle,
                high_bcd: high,
            };
        }
    }
}
