//! Compare an observed original effect at its source sequence/duration phase.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use super_off_road_port::sound::{
    definitions::load_audio,
    sequencer::{Driver, update_driver},
};
fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let definitions = load_audio(&std::fs::read(args.next().context("ROM path required")?)?)?;
    let observed: Value = serde_json::from_slice(&std::fs::read(
        args.next().context("original effect JSON required")?,
    )?)?;
    let output = args.next().context("output JSON required")?;
    let selector = observed["effect_id"]
        .as_u64()
        .context("effect ID required")? as usize;
    let mut driver = Driver::default();
    driver.envelopes.rows = definitions.initial_envelopes.clone();
    driver.voices[6].active = true;
    driver.voices[6].duration = 1;
    driver.sequences[6].cursor = definitions
        .score
        .resolve(definitions.effects[selector].sequence);
    let mut matches = Vec::new();
    for tick in 0..128 {
        update_driver(&mut driver, 0, &definitions)?;
        for name in ["before", "after"] {
            let source = &observed[name];
            let voice = &driver.voices[6];
            if source["sequence"][6] == driver.sequences[6].cursor.source().0
                && source["duration"][6] == voice.duration
            {
                let native = json!({"pitch":voice.dsp_pitch,"sample":voice.instrument,"amplitude":voice.amplitude});
                for field in ["pitch", "sample", "amplitude"] {
                    ensure!(
                        native[field] == source[field][6],
                        "effect{selector}/{name} {field}:{} != original{}",
                        native[field],
                        source[field][6]
                    );
                }
                matches.push(json!({"landmark":name,"voice_tick":tick,"native":native}));
            }
        }
        if matches.len() == 2 {
            break;
        }
    }
    ensure!(
        matches.len() == 2,
        "original effect phases were not reached"
    );
    std::fs::write(
        output,
        serde_json::to_vec_pretty(
            &json!({"passed":true,"effect":selector,"matches":matches,"scope":"Exact source-voice pitch/instrument/amplitude at same source sequence/duration phase; no physical frame-clock or PCM alignment claim"}),
        )?,
    )?;
    Ok(())
}
