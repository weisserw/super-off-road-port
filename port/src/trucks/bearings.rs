//! Immutable source bearing lookup shared by steering and collision normals.
//! Both consumers use the same signed-vector quantizer, but retain their own
//! axis reduction and heading correction. Every source cell is preserved.
use anyhow::{Context, Result};

pub const BEARING_CELLS: usize = 256;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BearingTable {
    pub cells: [u8; BEARING_CELLS],
}
impl BearingTable {
    pub fn load(rom: &[u8]) -> Result<Self> {
        const BEARINGS_ROM: usize = 0x1cc7;
        Ok(Self {
            cells: rom
                .get(BEARINGS_ROM..BEARINGS_ROM + BEARING_CELLS)
                .context("truncated shared bearing table")?
                .try_into()?,
        })
    }
}
