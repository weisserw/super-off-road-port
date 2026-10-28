//! Compare native query results to read-only observations of original execution.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use super_off_road_port::world::{terrain, tracks::TERRAIN_BYTES};
#[derive(Deserialize)]
struct Probe {
    position: [u16; 2],
    track: u8,
    history_before: u8,
    history_after: u8,
    blocked: bool,
    value: u8,
}
fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let rom = std::fs::read(args.next().context("ROM path required")?)?;
    let probes: Vec<Probe> = serde_json::from_slice(&std::fs::read(
        args.next().context("observations JSON required")?,
    )?)?;
    let output = args.next().context("output JSON required")?;
    let starts = [
        0x20000, 0x216b0, 0x22d60, 0x24410, 0x28000, 0x296b0, 0x2ad60, 0x2c410, 0x50000, 0x516aa,
        0x52d5a, 0x5440a, 0x58000, 0x596b0, 0x5ad60, 0x5c410,
    ];
    let tables = terrain::load_sampling_tables(&rom)?;
    let views: Vec<_> = starts
        .into_iter()
        .map(|start| terrain::decode_terrain(&rom[start..start + TERRAIN_BYTES]))
        .collect::<Result<_>>()?;
    for (index, probe) in probes.iter().enumerate() {
        let data = views
            .get(usize::from(probe.track))
            .context("invalid observed track")?;
        let actual = terrain::sample_at(
            data,
            &tables,
            probe.position,
            probe.track,
            probe.history_before,
        )?;
        let expected = terrain::Sample {
            surface: if probe.blocked {
                terrain::Surface::Blocked(terrain::BlockDirection(probe.value))
            } else {
                terrain::Surface::Height(probe.value)
            },
            control_history: probe.history_after,
        };
        ensure!(
            actual == expected,
            "probe{index} at{:?}: {actual:?} != original{expected:?}",
            probe.position
        );
    }
    std::fs::write(
        output,
        serde_json::to_vec_pretty(
            &serde_json::json!({"passed":true,"probes":probes.len(),"scope":"Exact height/blocked direction/history comparison at observed natural original wheel sampler call boundaries; no cross-engine frame-clock alignment claim"}),
        )?,
    )?;
    Ok(())
}
