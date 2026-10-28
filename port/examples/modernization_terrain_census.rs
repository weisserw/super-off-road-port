//! Differential terrain evidence, separate from source-derived unit contracts.
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use super_off_road_port::world::{terrain, tracks::TERRAIN_BYTES};

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let rom = std::fs::read(args.next().context("ROM path required")?)?;
    let output = args.next().context("output JSON path required")?;
    let tables = terrain::load_sampling_tables(&rom)?;
    let starts = [
        0x20000, 0x216b0, 0x22d60, 0x24410, 0x28000, 0x296b0, 0x2ad60, 0x2c410, 0x50000, 0x516aa,
        0x52d5a, 0x5440a, 0x58000, 0x596b0, 0x5ad60, 0x5c410,
    ];
    let fractions = [
        [0, 0],
        [255, 0],
        [0, 255],
        [255, 255],
        [127, 128],
        [128, 127],
        [128, 128],
        [127, 127],
        [64, 32],
        [32, 64],
        [129, 127],
        [127, 129],
    ];
    let rows: Vec<u8> = (0..44).chain(128..172).chain([44, 127, 172, 255]).collect();
    let mut views = Vec::new();
    for (track, start) in starts.into_iter().enumerate() {
        let data = terrain::decode_terrain(&rom[start..start + TERRAIN_BYTES])?;
        let mut digest = Sha256::new();
        let mut counts = [0u64; 3];
        for &row in &rows {
            for column in 0..=255u8 {
                for [x, y] in fractions {
                    digest.update([column, row, x, y]);
                    let position = [
                        u16::from(column) * 256 + x as u16,
                        u16::from(row) * 256 + y as u16,
                    ];
                    match terrain::sample_at(&data, &tables, position, track as u8, column ^ row) {
                        Ok(sample) => match sample.surface {
                            terrain::Surface::Height(height) => {
                                counts[0] += 1;
                                digest.update([0, height, sample.control_history]);
                            }
                            terrain::Surface::Blocked(direction) => {
                                counts[1] += 1;
                                digest.update([1, direction.0, sample.control_history]);
                            }
                        },
                        Err(error) => {
                            counts[2] += 1;
                            let text = error.to_string();
                            digest.update([2]);
                            digest.update((text.len() as u16).to_le_bytes());
                            digest.update(text.as_bytes());
                        }
                    }
                }
            }
        }
        views.push(serde_json::json!({"track":track,"sha256":format!("{:x}",digest.finalize()),"height_blocked_error_counts":counts}));
    }
    std::fs::write(
        output,
        serde_json::to_vec_pretty(&serde_json::json!({
            "version":1,"scope":"16 source windows; all256 column aliases; both supported row banks; unsupported row endpoints;12 fraction boundaries; track8 bias; heights/blocks/history/error text",
            "rom_sha256":format!("{:x}",Sha256::digest(&rom)),"views":views,
            "evidence_kind":"Starting-build differential census; independent source expectations remain in terrain.trace.md and unit contracts"
        }))?,
    )?;
    Ok(())
}
