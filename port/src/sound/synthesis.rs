//! Native DSP synthesis/snapshot adapter contracts; no SPC registers or CPU are executed.
use super::host;
use super::{definitions::*, host::HostState, sequencer::Driver};
use anyhow::{Context, Result, ensure};
use romatlas_port::snes::audio::Mixer;
use romatlas_port::snes::audio::{BrrDecoder, Sample};
use sha2::{Digest, Sha256};
use std::sync::Arc;
pub const NATIVE_SAMPLE_RATE: u32 = 32000;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VoiceOutput {
    pub volume: [u8; 2],
    pub pitch: u16,
    pub sample: u8,
    pub gain: u8,
}
#[derive(Clone, Debug)]
pub struct Sound {
    pub definitions: Arc<AudioDefinitions>,
    pub driver: Driver,
    pub host: HostState,
    pub mixer: Mixer,
    pub native_samples: u64,
    pub output_phase: u64,
    pub held_sample: [i16; 2],
    pub host_flags: u8,
    pub output_rate: u32,
    pub outputs: [VoiceOutput; VOICES],
    pub definition_id: String,
}
impl Sound {
    pub fn new(definitions: Arc<AudioDefinitions>) -> Result<Self> {
        for sample in &definitions.samples {
            Sample::new(&sample.blocks, sample.loop_block)?;
        }
        let definition_id = definition_identity(&definitions);
        let mut driver = Driver::default();
        driver.envelopes.rows = definitions.initial_envelopes.clone();
        if driver.envelopes.rows.is_empty() {
            driver.envelopes.rows = vec![[0; 7]; 28];
        }
        if !definitions.songs.is_empty() {
            super::sequencer::load_song(&mut driver, 0, &definitions)?;
            let writes = super::sequencer::update_driver(&mut driver, 0, &definitions)?;
            driver.pending_dsp.extend(writes);
        }
        let mut sound = Self {
            definitions,
            driver,
            host: HostState::default(),
            mixer: Mixer::with_buffered_noise(),
            native_samples: 0,
            output_phase: 0,
            held_sample: [0; 2],
            host_flags: 0,
            output_rate: 0,
            outputs: [VoiceOutput::default(); VOICES],
            definition_id,
        };
        let initialization = sound.definitions.dsp_initialization.clone();
        for (register, value) in initialization {
            sound.write_dsp(register, value)?;
        }
        sound.flush_driver()?;
        Ok(sound)
    }
    pub fn apply_outputs(
        &mut self,
        voices: &[VoiceOutput; VOICES],
        key_on: u8,
        key_off: u8,
        noise: u8,
        flags: u8,
        master: u8,
    ) -> Result<()> {
        let mut sources: Vec<Option<Sample>> = vec![None; VOICES];
        for slot in 0..VOICES {
            if key_on & (1 << slot) != 0 {
                let source = self
                    .definitions
                    .samples
                    .get(usize::from(voices[slot].sample))
                    .context("invalid sample selector")?;
                sources[slot] = Some(Sample::new(&source.blocks, source.loop_block)?);
            }
        }
        self.outputs = *voices;
        self.mixer.master = [master as i8; 2];
        self.mixer.mute = flags & 0x40 != 0;
        self.mixer.noise_rate = flags & 31;
        if let Some(echo) = &mut self.mixer.echo {
            echo.write_enabled = flags & 0x20 == 0;
        }
        for slot in 0..VOICES {
            let output = voices[slot];
            let voice = &mut self.mixer.voices[slot];
            voice.volume = output.volume.map(|value| value as i8);
            voice.pitch = output.pitch;
            voice.gain = output.gain;
            voice.adsr1 = 0;
            voice.noise = noise & (1 << slot) != 0;
            if key_off & (1 << slot) != 0 || flags & 0x80 != 0 {
                voice.key_off();
            }
            if let Some(source) = sources[slot].take() {
                voice.key_on(source);
            }
        }
        Ok(())
    }
    pub fn render_native(&mut self, frames: usize) -> Result<Vec<[i16; 2]>> {
        ensure!(frames <= 32_000_000, "audio request too large");
        let mut output = Vec::with_capacity(frames);
        for _ in 0..frames {
            self.flush_driver()?;
            let sample = self.mixer.sample();
            self.held_sample = sample;
            output.push(sample);
            self.native_samples = self.native_samples.wrapping_add(1);
            host::service_sample_clock(
                &mut self.host,
                &mut self.driver,
                1,
                self.host_flags,
                &self.definitions,
            )?;
        }
        Ok(output)
    }
    pub fn render_output(&mut self, frames: usize, rate: u32) -> Result<Vec<f32>> {
        ensure!(
            (8000..=192000).contains(&rate) && frames <= 32_000_000,
            "invalid audio output request"
        );
        if self.output_rate != 0 && self.output_rate != rate {
            self.output_phase = self.output_phase * u64::from(rate) / u64::from(self.output_rate);
        }
        self.output_rate = rate;
        let mut output = Vec::with_capacity(frames * 2);
        for _ in 0..frames {
            self.output_phase += u64::from(NATIVE_SAMPLE_RATE);
            while self.output_phase >= u64::from(rate) {
                self.output_phase -= u64::from(rate);
                self.render_native(1)?;
            }
            output.extend(self.held_sample.map(|value| f32::from(value) / 32768.0));
        }
        Ok(output)
    }
    pub fn snapshot_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&SoundSnapshot {
            version: 1,
            definition_id: self.definition_id.clone(),
            driver: self.driver.clone(),
            host: self.host.clone(),
            mixer: self.mixer.snapshot_json()?,
            native_samples: self.native_samples,
            output_phase: self.output_phase,
            held_sample: self.held_sample,
            host_flags: self.host_flags,
            output_rate: self.output_rate,
            outputs: self.outputs,
        })?)
    }
    pub fn restore_snapshot_json(&mut self, json: &str) -> Result<()> {
        ensure!(json.len() <= 16 * 1024 * 1024, "audio snapshot too large");
        let candidate: SoundSnapshot = serde_json::from_str(json)?;
        ensure!(
            candidate.version == 1 && candidate.definition_id == self.definition_id,
            "audio snapshot identity/version mismatch"
        );
        ensure!(
            (candidate.output_rate == 0 && candidate.output_phase == 0)
                || ((8000..=192000).contains(&candidate.output_rate)
                    && candidate.output_phase < u64::from(candidate.output_rate)),
            "invalid output resampler phase"
        );
        ensure!(
            candidate
                .host
                .second_update_samples
                .is_none_or(|samples| samples > 0 && samples <= host::TIMER_SAMPLE_INTERVAL)
                && candidate.driver.envelopes.rows.len() == 28
                && candidate
                    .driver
                    .sequences
                    .iter()
                    .all(|s| s.stack.len() <= 256)
                && candidate.driver.pending_dsp.len() <= 65536,
            "invalid audio scheduler/driver snapshot"
        );
        for (slot, voice) in candidate.driver.voices.iter().enumerate() {
            if voice.active {
                self.definitions
                    .data
                    .read(candidate.driver.sequences[slot].cursor)?;
            }
            ensure!(
                voice.modulation_direction >= -1
                    && voice.modulation_direction <= 1
                    && voice.modulation_direction_reload >= -1
                    && voice.modulation_direction_reload <= 1,
                "invalid modulation direction"
            );
        }
        let mut next = self.clone();
        next.mixer.restore_snapshot_json(&candidate.mixer)?;
        next.driver = candidate.driver;
        next.host = candidate.host;
        next.native_samples = candidate.native_samples;
        next.output_phase = candidate.output_phase;
        next.held_sample = candidate.held_sample;
        next.host_flags = candidate.host_flags;
        next.output_rate = candidate.output_rate;
        next.outputs = candidate.outputs;
        *self = next;
        Ok(())
    }
    pub fn observe(&self) -> serde_json::Value {
        serde_json::json!({"native_samples":self.native_samples,"output_phase":self.output_phase,"output_rate":self.output_rate,"updates":self.driver.updates,"song":self.driver.requested_song,"master":self.driver.master,"fade_step":self.driver.fade_step,"heartbeat":self.host.last_heartbeat,"acknowledged":self.host.acknowledged,"second_update_samples":self.host.second_update_samples,"pending_dsp_writes":self.driver.pending_dsp.len(),"voices":self.driver.voices.iter().enumerate().map(|(slot,voice)|serde_json::json!({"active":voice.active,"sample":self.outputs[slot].sample,"duration":voice.duration,"pitch":voice.dsp_pitch,"amplitude":voice.amplitude,"sequence":self.driver.sequences[slot].cursor.0,"stack_depth":self.driver.sequences[slot].stack.len(),"synthesizing":self.mixer.voices[slot].active()})).collect::<Vec<_>>()})
    }
    /// Host input, sequencer and synthesis share this single mutable owner.
    pub fn service_host(&mut self, input: host::HostInput) -> Result<()> {
        self.host_flags = input.flags;
        host::service_host(&mut self.host, &mut self.driver, input, &self.definitions)?;
        self.flush_driver()
    }
    fn flush_driver(&mut self) -> Result<()> {
        let writes = std::mem::take(&mut self.driver.pending_dsp);
        for (register, value) in writes {
            self.write_dsp(register, value)?;
        }
        Ok(())
    }
    fn write_dsp(&mut self, register: u8, value: u8) -> Result<()> {
        match register {
            0x4c => {
                let voices = self.outputs;
                self.apply_outputs(
                    &voices,
                    value,
                    0,
                    self.driver.noise_mask,
                    self.driver.dsp_flags,
                    (self.driver.master >> 8) as u8,
                )?;
            }
            0x5c => {
                for slot in 0..VOICES {
                    if value & (1 << slot) != 0 {
                        self.mixer.voices[slot].key_off();
                    }
                }
            }
            0x3d => {
                for slot in 0..VOICES {
                    self.mixer.voices[slot].noise = value & (1 << slot) != 0;
                }
            }
            0x6c => {
                self.mixer.mute = value & 0x40 != 0;
                self.mixer.noise_rate = value & 31;
                if value & 0x80 != 0 {
                    for voice in &mut self.mixer.voices {
                        voice.key_off();
                    }
                }
                if let Some(echo) = &mut self.mixer.echo {
                    echo.write_enabled = value & 0x20 == 0;
                }
            }
            0x0c => self.mixer.master[0] = value as i8,
            0x1c => self.mixer.master[1] = value as i8,
            _ if register < 0x80 => {
                let slot = usize::from(register >> 4);
                let field = register & 15;
                let voice = &mut self.mixer.voices[slot];
                let output = &mut self.outputs[slot];
                match field {
                    0 => {
                        output.volume[0] = value;
                        voice.volume[0] = value as i8;
                    }
                    1 => {
                        output.volume[1] = value;
                        voice.volume[1] = value as i8;
                    }
                    2 => {
                        output.pitch = (output.pitch & 0xff00) | u16::from(value);
                        voice.pitch = output.pitch;
                    }
                    3 => {
                        output.pitch = (output.pitch & 0xff) | (u16::from(value) << 8);
                        voice.pitch = output.pitch;
                    }
                    4 => output.sample = value,
                    5 => voice.adsr1 = value,
                    6 => voice.adsr2 = value,
                    7 => {
                        output.gain = value;
                        voice.gain = value;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }
}
/// Game-owned BRR chain wrapper uses shared decoder; retaining original bytes alone is insufficient.
pub fn decode_brr_pcm(sample: &SampleDefinition, blocks: usize) -> Result<Vec<i16>> {
    Sample::new(&sample.blocks, sample.loop_block)?;
    ensure!(blocks <= 1_000_000, "BRR decode request too large");
    let mut decoder = BrrDecoder::default();
    let mut output = Vec::new();
    let mut index = 0;
    for _ in 0..blocks {
        let start = index * 9;
        let block = decoder.decode(&sample.blocks[start..start + 9])?;
        output.extend(block.samples);
        if block.end {
            if !block.looping {
                break;
            }
            index = sample.loop_block;
        } else {
            index += 1;
        }
    }
    Ok(output)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn definitions(blocks: Vec<u8>) -> Arc<AudioDefinitions> {
        Arc::new(AudioDefinitions {
            data: SequenceData::default(),
            songs: vec![],
            effects: vec![],
            samples: vec![SampleDefinition {
                start: DataAddress(0x400),
                loop_address: DataAddress(0x400),
                blocks,
                loop_block: 0,
            }],
            pitch_low: [0; 256],
            pitch_high: [0; 256],
            instrument_note_base: [0; 256],
            instrument_scale: [0; 256],
            initial_envelopes: vec![[0; 7]; 28],
            dsp_initialization: vec![],
            command_targets: vec![],
        })
    }
    fn sound() -> Sound {
        Sound::new(definitions(vec![
            0xc3, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
        ]))
        .unwrap()
    }
    fn outputs() -> [VoiceOutput; VOICES] {
        let mut o = [VoiceOutput::default(); VOICES];
        o[0] = VoiceOutput {
            volume: [127, 127],
            pitch: 4096,
            sample: 0,
            gain: 127,
        };
        o
    }
    #[test]
    fn brr_signed_nibble_order_range_and_terminal_flags_feed_actual_decoder() {
        let s = SampleDefinition {
            start: DataAddress(0x400),
            loop_address: DataAddress(0x400),
            blocks: vec![0x13, 0x17, 0x8f, 0, 0, 0, 0, 0, 0],
            loop_block: 0,
        };
        let p = decode_brr_pcm(&s, 1).unwrap();
        assert_eq!(&p[..4], &[2, 14, -16, -2]);
        assert_eq!(p.len(), 16);
    }
    #[test]
    fn brr_loop_keeps_predictor_history_instead_of_restarting_on_each_iteration() {
        let s = SampleDefinition {
            start: DataAddress(0x400),
            loop_address: DataAddress(0x400),
            blocks: vec![0x27, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11],
            loop_block: 0,
        };
        let p = decode_brr_pcm(&s, 2).unwrap();
        assert_eq!(&p[..4], &[4, 6, 8, 10]);
        assert_eq!(p[16], 34);
        assert_eq!(p.len(), 32);
    }
    #[test]
    fn actual_sample_mixer_applies_direct_gain_signed_stereo_and_master() {
        let mut s = sound();
        let mut o = outputs();
        o[0].volume = [127, 0x80];
        s.apply_outputs(&o, 1, 0, 0, 0x20, 127).unwrap();
        let p = s.render_native(2).unwrap();
        assert_eq!(p, [[4000, -4033], [4000, -4033]]);
        assert_eq!(s.native_samples, 2);
    }
    #[test]
    fn terminal_nonloop_sample_stops_while_looping_sample_continues() {
        let mut s = Sound::new(definitions(vec![
            0xc1, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
        ]))
        .unwrap();
        s.apply_outputs(&outputs(), 1, 0, 0, 0x20, 127).unwrap();
        let p = s.render_native(20).unwrap();
        assert_eq!(p[0], [4000, 4000]);
        assert_eq!(p[19], [0, 0]);
        let mut s = sound();
        s.apply_outputs(&outputs(), 1, 0, 0, 0x20, 127).unwrap();
        assert_eq!(s.render_native(20).unwrap()[19], [4000, 4000]);
    }
    #[test]
    fn key_off_releases_direct_gain_envelope_and_retrigger_resets_source() {
        let mut s = sound();
        s.apply_outputs(&outputs(), 1, 0, 0, 0x20, 127).unwrap();
        assert_eq!(s.render_native(1).unwrap(), [[4000, 4000]]);
        s.apply_outputs(&outputs(), 0, 1, 0, 0x20, 127).unwrap();
        let p = s.render_native(255).unwrap();
        assert!(p[0][0] < 4000);
        assert_eq!(p[254], [0, 0]);
        s.apply_outputs(&outputs(), 1, 0, 0, 0x20, 127).unwrap();
        assert_eq!(s.render_native(1).unwrap(), [[4000, 4000]]);
    }
    #[test]
    fn mute_advances_live_audio_state_and_noise_bit_selects_noise_source() {
        let mut s = sound();
        s.apply_outputs(&outputs(), 1, 0, 1, 0x60, 127).unwrap();
        assert_eq!(s.render_native(3).unwrap(), [[0, 0]; 3]);
        assert_eq!(s.native_samples, 3);
        s.apply_outputs(&outputs(), 0, 0, 1, 0x20, 127).unwrap();
        assert_eq!(s.render_native(1).unwrap(), [[-32006, -32006]]);
    }
    #[test]
    fn invalid_sample_source_rejected_before_any_live_state_changes() {
        let mut s = sound();
        let before = s.snapshot_json().unwrap();
        let mut o = outputs();
        o[0].sample = 1;
        assert!(s.apply_outputs(&o, 1, 0, 0, 0x20, 127).is_err());
        assert_eq!(s.snapshot_json().unwrap(), before);
    }
    #[test]
    fn sample_clock_executes_scheduled_second_driver_update_at264_samples() {
        let mut s = sound();
        s.host.second_update_samples = Some(264);
        s.render_native(263).unwrap();
        assert_eq!(s.driver.updates, 0);
        s.render_native(1).unwrap();
        assert_eq!(s.driver.updates, 1);
        assert_eq!(s.host.second_update_samples, None);
    }
    #[test]
    fn cloned_active_audio_forks_predictor_envelope_noise_resampler_and_pending_timer() {
        let mut s = sound();
        s.apply_outputs(&outputs(), 1, 0, 0, 0x20, 127).unwrap();
        s.host.second_update_samples = Some(264);
        s.render_output(17, 44100).unwrap();
        let mut branch = s.clone();
        let original = s.snapshot_json().unwrap();
        let expected = branch.render_output(100, 44100).unwrap();
        assert_eq!(s.snapshot_json().unwrap(), original);
        assert_eq!(s.render_output(100, 44100).unwrap(), expected);
        assert_eq!(s.snapshot_json().unwrap(), branch.snapshot_json().unwrap());
    }
    #[test]
    fn full_snapshot_restores_live_synthesis_and_rejects_bad_state_atomically() {
        let mut s = sound();
        s.apply_outputs(&outputs(), 1, 0, 0, 0x20, 127).unwrap();
        s.render_output(7, 48000).unwrap();
        let checkpoint = s.snapshot_json().unwrap();
        let expected = s.render_output(70, 48000).unwrap();
        s.restore_snapshot_json(&checkpoint).unwrap();
        assert_eq!(s.render_output(70, 48000).unwrap(), expected);
        let before = s.snapshot_json().unwrap();
        assert!(s.restore_snapshot_json("{\"version\":999}").is_err());
        assert_eq!(s.snapshot_json().unwrap(), before);
    }
    #[test]
    fn output_chunking_preserves_fractional_sample_clock_and_stereo_count() {
        let mut whole = sound();
        whole.apply_outputs(&outputs(), 1, 0, 0, 0x20, 127).unwrap();
        let mut split = whole.clone();
        let expected = whole.render_output(441, 44100).unwrap();
        let mut got = split.render_output(13, 44100).unwrap();
        got.extend(split.render_output(428, 44100).unwrap());
        assert_eq!(got, expected);
        assert_eq!(got.len(), 882);
        assert_eq!(whole.native_samples, 320);
        assert_eq!(
            whole.snapshot_json().unwrap(),
            split.snapshot_json().unwrap()
        );
    }
    #[test]
    fn semantic_inspection_does_not_drain_audio_or_change_sequence_state() {
        let mut s = sound();
        s.apply_outputs(&outputs(), 1, 0, 0, 0x20, 127).unwrap();
        s.render_native(7).unwrap();
        let before = s.snapshot_json().unwrap();
        let a = s.observe();
        assert_eq!(a, s.observe());
        assert_eq!(a["native_samples"], 7);
        assert_eq!(s.snapshot_json().unwrap(), before);
    }
    #[test]
    fn dsp_soft_reset_releases_all_voices_without_unmute_restarting_samples() {
        let mut s = sound();
        s.apply_outputs(&outputs(), 1, 0, 0, 0x20, 127).unwrap();
        s.render_native(1).unwrap();
        s.apply_outputs(&outputs(), 0, 0, 0, 0xe0, 127).unwrap();
        s.render_native(300).unwrap();
        s.apply_outputs(&outputs(), 0, 0, 0, 0x20, 127).unwrap();
        assert_eq!(s.render_native(1).unwrap(), [[0, 0]]);
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SoundSnapshot {
    version: u32,
    definition_id: String,
    driver: Driver,
    host: HostState,
    mixer: String,
    native_samples: u64,
    output_phase: u64,
    held_sample: [i16; 2],
    host_flags: u8,
    output_rate: u32,
    outputs: [VoiceOutput; VOICES],
}
fn definition_identity(definitions: &AudioDefinitions) -> String {
    // Stable, typed resource binding: only immutable decoded definitions may share.
    let mut digest = Sha256::new();
    for segment in &definitions.data.segments {
        digest.update(segment.origin.0.to_le_bytes());
        digest.update((segment.bytes.len() as u64).to_le_bytes());
        digest.update(&segment.bytes);
    }
    for song in &definitions.songs {
        for voice in song.voices {
            digest.update(voice.0.to_le_bytes());
        }
        digest.update(song.configuration.0.to_le_bytes());
    }
    for effect in &definitions.effects {
        digest.update(effect.sequence.0.to_le_bytes());
        digest.update([effect.priority]);
    }
    for sample in &definitions.samples {
        digest.update(sample.start.0.to_le_bytes());
        digest.update(sample.loop_address.0.to_le_bytes());
        digest.update((sample.loop_block as u64).to_le_bytes());
        digest.update((sample.blocks.len() as u64).to_le_bytes());
        digest.update(&sample.blocks);
    }
    for table in [
        &definitions.pitch_low,
        &definitions.pitch_high,
        &definitions.instrument_note_base,
        &definitions.instrument_scale,
    ] {
        digest.update(table);
    }
    for row in &definitions.initial_envelopes {
        digest.update(row);
    }
    for &(register, value) in &definitions.dsp_initialization {
        digest.update([register, value]);
    }
    for target in &definitions.command_targets {
        digest.update(target.0.to_le_bytes());
    }
    format!("{:x}", digest.finalize())
}
