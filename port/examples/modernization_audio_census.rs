//! Before/after evidence for all original main music/effect selectors.
//! Original expectations remain separately source-backed; this is differential.
use anyhow::{Context, Result, ensure};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{path::Path, sync::Arc};
use super_off_road_port::sound::{
    definitions::{AudioDefinitions, load_audio},
    host::HostInput,
    sequencer::update_driver,
    synthesis::Sound,
};
fn normalized(value: &impl Serialize) -> Result<Vec<u8>> {
    Ok(serde_json::to_vec(&serde_json::to_value(value)?)?)
}
fn digest(value: &impl Serialize) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(normalized(value)?)))
}
fn heartbeat(sound: &mut Sound, frame: usize) -> Result<()> {
    sound.service_host(HostInput {
        heartbeat: (frame as u8).wrapping_add(3),
        ..HostInput::default()
    })
}
fn run_case(
    definitions: Arc<AudioDefinitions>,
    kind: &str,
    selector: u8,
    root: &Path,
) -> Result<Value> {
    let mut sound = Sound::new(definitions)?;
    sound.service_host(HostInput {
        command: 1,
        argument: if kind == "song" { selector + 1 } else { 2 },
        heartbeat: 1,
        flags: 0,
    })?;
    if kind == "effect" {
        sound.service_host(HostInput {
            command: 2,
            argument: selector,
            heartbeat: 2,
            flags: 0,
        })?;
    }
    let mut audit = sound.driver.clone();
    let mut dsp = Sha256::new();
    let mut milestones = Vec::new();
    for tick in 0..512 {
        let writes = update_driver(&mut audit, 0, &sound.definitions)
            .with_context(|| format!("DSP tick{tick}"))?;
        dsp.update(normalized(&writes)?);
        dsp.update(normalized(&audit)?);
        if [0, 15, 127, 511].contains(&tick) {
            let value = serde_json::to_value(&audit)?;
            std::fs::write(
                root.join(format!("{kind}-{selector:02}-driver-{tick:03}.json")),
                serde_json::to_vec_pretty(&value)?,
            )?;
            milestones.push(json!({"tick":tick,"driver_sha256":digest(&value)?}));
        }
    }
    let mut pcm = Sha256::new();
    let mut resume = String::new();
    for frame in 0..120 {
        heartbeat(&mut sound, frame)?;
        let samples = sound
            .render_output(800, 48000)
            .with_context(|| format!("PCM frame{frame}"))?;
        for sample in &samples {
            pcm.update(sample.to_le_bytes());
        }
        let snapshot = sound.snapshot_json()?;
        let value: Value = serde_json::from_str(&snapshot)?;
        if [0, 1, 15, 59, 119].contains(&frame) {
            std::fs::write(
                root.join(format!("{kind}-{selector:02}-sound-{frame:03}.json")),
                serde_json::to_vec_pretty(&value)?,
            )?;
            milestones.push(json!({"frame":frame,"snapshot_sha256":digest(&value)?,"observe_sha256":digest(&sound.observe())?}));
        }
        if frame == 59 {
            resume = snapshot.clone();
        }
        if frame == 60 {
            sound.restore_snapshot_json(&resume)?;
            heartbeat(&mut sound, frame)?;
            let restored = sound.render_output(800, 48000)?;
            ensure!(restored == samples, "snapshot PCM continuation changed");
            let after: Value = serde_json::from_str(&sound.snapshot_json()?)?;
            ensure!(after == value, "snapshot full audio state changed");
        }
    }
    Ok(
        json!({"kind":kind,"selector":selector,"status":"passed","dsp_driver_sha256":format!("{:x}",dsp.finalize()),"pcm_sha256":format!("{:x}",pcm.finalize()),"milestones":milestones,"snapshot_continuation":true}),
    )
}
fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let rom = std::fs::read(args.next().context("ROM path required")?)?;
    let output = args.next().context("new output directory required")?;
    let root = Path::new(&output);
    std::fs::create_dir(root)?;
    let definitions = Arc::new(load_audio(&rom)?);
    let mut cases = Vec::new();
    for kind in ["song", "effect"] {
        for selector in 0..16 {
            let result = match run_case(definitions.clone(), kind, selector, root) {
                Ok(value) => value,
                Err(error) => {
                    json!({"kind":kind,"selector":selector,"status":"error","error":format!("{error:#}")})
                }
            };
            println!("{kind}{selector}: {}", result["status"]);
            cases.push(result);
        }
    }
    std::fs::write(
        root.join("census.json"),
        serde_json::to_vec_pretty(
            &json!({"version":1,"evidence_kind":"Differential starting-build behavior, not independent original fidelity certification","rom_sha256":format!("{:x}",Sha256::digest(&rom)),"scope":"All16 songs and16 effects;512 ordered DSP/Driver updates and120 frontend-cadence audio frames per selector; canonical full snapshots/observations,48kHz PCM and snapshot continuation","cases":cases}),
        )?,
    )?;
    Ok(())
}
