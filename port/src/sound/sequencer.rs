//! Game sequence language and per-voice control flow, never SPC instruction execution.
#[cfg(test)]
pub use super::score::{COMMAND_FIRST, COMMAND_LAST, Command, decode_command};
use super::score::{Control, Event, EventRef, Repetitions, ScoreGraph};
use super::{definitions::*, voice::*};
use anyhow::{Context, Result, ensure};
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Frame {
    Call(EventRef),
    Loop {
        start: EventRef,
        remaining: Repetitions,
    },
}
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sequence {
    pub cursor: EventRef,
    pub stack: Vec<Frame>,
    pub auxiliary: DataAddress,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Driver {
    pub voices: [Voice; VOICES],
    pub sequences: [Sequence; VOICES],
    pub keys: KeyMasks,
    pub noise_mask: u8,
    pub dsp_flags: u8,
    pub master: u16,
    pub fade_step: i16,
    pub slow_counter: u8,
    pub requested_song: Option<u8>,
    pub selected_song_shadow: u8,
    pub effect_priorities: [u8; 2],
    pub envelopes: EnvelopeDefinitions,
    pub updates: u64,
    /// Ordered writes awaiting the sole native mixer; includes host-generated updates.
    pub pending_dsp: Vec<(u8, u8)>,
}
impl Default for Driver {
    fn default() -> Self {
        Self {
            voices: std::array::from_fn(|_| Voice::default()),
            sequences: std::array::from_fn(|_| Sequence::default()),
            keys: KeyMasks::default(),
            noise_mask: 0,
            dsp_flags: 0,
            master: 0,
            fade_step: 0,
            slow_counter: 0,
            requested_song: None,
            selected_song_shadow: 0,
            effect_priorities: [0; 2],
            envelopes: EnvelopeDefinitions {
                rows: vec![[0; 7]; 28],
            },
            updates: 0,
            pending_dsp: Vec::new(),
        }
    }
}
pub fn load_song(driver: &mut Driver, song: u8, definitions: &AudioDefinitions) -> Result<()> {
    let song_data = definitions
        .songs
        .get(usize::from(song))
        .context("invalid song selector")?;
    let mut envelopes = driver.envelopes.clone();
    if song != 0 {
        let bytes = (0..CONFIGURATION_BYTES)
            .map(|i| {
                definitions.data.read(DataAddress(
                    song_data.configuration.0.wrapping_add(i as u16),
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        install_configuration(&mut envelopes, &bytes)?;
    }
    let updates = driver.updates;
    let dsp_flags = driver.dsp_flags;
    let pending_dsp = driver.pending_dsp.clone();
    let mut next = Driver {
        envelopes,
        updates,
        dsp_flags,
        pending_dsp,
        requested_song: Some(song),
        ..Driver::default()
    };
    for slot in 0..VOICES {
        next.voices[slot].active = true;
        next.voices[slot].duration = 1;
        if slot < 3 {
            next.voices[slot].envelope_current = 1;
        }
        next.sequences[slot].cursor = definitions.score.resolve(song_data.voices[slot]);
    }
    *driver = next;
    Ok(())
}
fn apply_control(
    driver: &mut Driver,
    slot: usize,
    control: Control,
    definitions: &AudioDefinitions,
) -> Result<()> {
    ensure!(slot < VOICES, "invalid voice slot");
    if control == Control::ReloadSongOne {
        load_song(driver, 1, definitions)?;
        driver.master = 0x7f00;
        return Ok(());
    }
    let voice = &mut driver.voices[slot];
    let sequence = &mut driver.sequences[slot];
    match control {
        Control::Stop => voice.active = false,
        Control::Jump(target) => sequence.cursor = target,
        Control::Call(target) => {
            ensure!(sequence.stack.len() < 256, "sequence call stack limit");
            sequence.stack.push(Frame::Call(sequence.cursor));
            sequence.cursor = target;
        }
        Control::Return => {
            let Some(Frame::Call(target)) = sequence.stack.last() else {
                anyhow::bail!("unbalanced sequence return");
            };
            sequence.cursor = *target;
            sequence.stack.pop();
        }
        Control::BeginLoop(count) => {
            ensure!(sequence.stack.len() < 256, "sequence loop stack limit");
            sequence.stack.push(Frame::Loop {
                start: sequence.cursor,
                remaining: count,
            });
        }
        Control::RepeatLoop => {
            let Some(Frame::Loop { start, remaining }) = sequence.stack.last_mut() else {
                anyhow::bail!("unbalanced loop repeat");
            };
            if remaining.repeat() {
                sequence.cursor = *start;
            } else {
                sequence.stack.pop();
            }
        }
        Control::FixedDuration(value) => voice.fixed_duration = value,
        Control::ForceDuration => voice.force_duration = true,
        Control::Transpose(value) => voice.transpose = value,
        Control::Instrument(value) => voice.instrument = value,
        Control::LeftVolume(value) => voice.volume[0] = value,
        Control::RightVolume(value) => voice.volume[1] = value,
        Control::AuxiliaryStream { reload, stream } => {
            voice.auxiliary_reload = reload;
            sequence.auxiliary = stream;
            voice.amplitude = 0;
            voice.auxiliary_cursor = 0;
            voice.primary_mode = false;
            voice.auxiliary_enable = false;
            voice.secondary_mode = true;
        }
        Control::PitchOffset(value) => voice.pitch_low_offset = value,
        Control::Modulation {
            direction,
            delay,
            step,
            period,
        } => {
            voice.modulation_direction_reload = direction;
            voice.modulation_delay_reload = delay;
            voice.modulation_step = step;
            voice.modulation_period_reload = period;
            voice.modulation_period = period;
        }
        Control::Portamento(value) => voice.portamento_step = value,
        Control::DisableModulation => voice.modulation_direction_reload = 0,
        Control::ReleaseDelta(value) => {
            voice.release_delta = value;
            voice.release_fixed = 0;
        }
        Control::ReleaseFixed(value) => {
            voice.release_fixed = value;
            voice.release_delta = 0;
        }
        Control::Slew { step, interval } => {
            voice.envelope_step = step;
            voice.envelope_interval = interval;
        }
        Control::DisableSlew => voice.envelope_interval = 0,
        Control::TargetEnvelope {
            step,
            interval,
            delay,
        } => {
            voice.target_step = step;
            voice.target_interval = interval;
            voice.target_delay = delay;
            voice.target_countdown = delay;
        }
        Control::EnvelopeRecord(value) => {
            voice.envelope_record = value.wrapping_mul(7);
            voice.primary_mode = true;
            voice.secondary_mode = false;
            voice.auxiliary_enable = false;
        }
        Control::NoiseOn => driver.noise_mask |= 1 << slot,
        Control::NoiseOff => driver.noise_mask &= !(1 << slot),
        Control::NoiseClock(value) => driver.dsp_flags = (value & 31) | 0x20,
        Control::PrimaryMode => {
            voice.primary_mode = true;
            voice.secondary_mode = false;
            voice.auxiliary_enable = false;
        }
        Control::AuxiliaryEnable => voice.auxiliary_enable = true,
        Control::SecondaryMode => {
            voice.secondary_mode = true;
            voice.primary_mode = false;
            voice.auxiliary_enable = false;
        }
        Control::AllowKeyOn => voice.suppress_key_on = false,
        Control::SuppressKeyOn => voice.suppress_key_on = true,
        Control::SkipTranspose => voice.skip_transpose = 1,
        Control::ReloadSongOne => unreachable!(),
    }
    Ok(())
}
#[cfg(test)]
fn apply_command(
    driver: &mut Driver,
    slot: usize,
    command: Command,
    operands: &[u8],
    definitions: &AudioDefinitions,
) -> Result<()> {
    ensure!(slot < VOICES, "invalid voice slot");
    let expected = super::score::COMMANDS
        .iter()
        .find(|(value, _)| *value == command)
        .unwrap()
        .1;
    ensure!(operands.len() == expected, "invalid sequence operands");
    let mut values = [0; 3];
    values[..operands.len()].copy_from_slice(operands);
    apply_control(
        driver,
        slot,
        definitions.score.control(command, &values),
        definitions,
    )
}
impl Sequence {
    pub fn bind(&mut self, graph: &ScoreGraph) {
        self.cursor = graph.resolve(self.cursor.source());
        for frame in &mut self.stack {
            match frame {
                Frame::Call(target) => *target = graph.resolve(target.source()),
                Frame::Loop { start, .. } => *start = graph.resolve(start.source()),
            }
        }
    }
}
pub fn parse_to_note(
    driver: &mut Driver,
    slot: usize,
    definitions: &AudioDefinitions,
) -> Result<()> {
    ensure!(slot < VOICES, "invalid parser voice");
    let mut next = driver.clone();
    for _ in 0..4096 {
        let node = definitions.score.fetch(&mut next.sequences[slot].cursor)?;
        let (pitch, timing) = match node.event {
            Event::Fault(error) => return error.fail(),
            Event::Control(control) => {
                next.sequences[slot].cursor = node.next;
                apply_control(&mut next, slot, control, definitions)?;
                if !next.voices[slot].active {
                    *driver = next;
                    return Ok(());
                }
                continue;
            }
            Event::Note { pitch, timing } => (pitch, timing),
            Event::Rest(timing) => (0, timing),
        };
        let need = next.voices[slot].force_duration || next.voices[slot].fixed_duration == 0;
        let duration = if need {
            let (duration, continuation) = match timing.explicit {
                Ok(value) => value,
                Err(error) => return error.fail(),
            };
            next.sequences[slot].cursor = continuation;
            Some(duration)
        } else {
            next.sequences[slot].cursor = node.next;
            None
        };
        start_note(
            &mut next.voices[slot],
            pitch,
            duration,
            1 << slot,
            &mut next.keys,
            definitions,
        )?;
        *driver = next;
        return Ok(());
    }
    anyhow::bail!("zero-time sequence command cycle")
}
pub fn update_voice(
    driver: &mut Driver,
    slot: usize,
    definitions: &AudioDefinitions,
) -> Result<()> {
    ensure!(slot < VOICES, "invalid voice update");
    if !driver.voices[slot].active {
        return Ok(());
    }
    driver.voices[slot].duration = driver.voices[slot].duration.wrapping_sub(1);
    if driver.voices[slot].duration == 0 {
        parse_to_note(driver, slot, definitions)?;
        if !driver.voices[slot].active {
            return Ok(());
        }
    }
    let voice = &mut driver.voices[slot];
    step_target(voice);
    slew_envelope(voice, definitions);
    update_modulation(voice);
    update_portamento(voice);
    if voice.secondary_mode {
        let origin = driver.sequences[slot].auxiliary.0;
        let mut stream = Vec::new();
        for offset in 0..256 {
            match definitions
                .data
                .read(DataAddress(origin.wrapping_add(offset)))
            {
                Ok(value) => stream.push(value),
                Err(_) => break,
            }
        }
        update_auxiliary(voice, &stream)?;
    }
    let mut row = [0; 7];
    for (field, value) in row.iter_mut().enumerate() {
        let at = usize::from(voice.envelope_record) + field;
        *value = *driver
            .envelopes
            .rows
            .get(at / 7)
            .and_then(|row| row.get(at % 7))
            .context("envelope record outside live data")?;
    }
    service_envelope_stage(voice, row);
    combine_pitch(voice);
    Ok(())
}
pub fn update_driver(
    driver: &mut Driver,
    host_flags: u8,
    definitions: &AudioDefinitions,
) -> Result<Vec<(u8, u8)>> {
    super::host::integrate_fade(driver);
    if host_flags & 4 != 0 {
        driver.slow_counter = driver.slow_counter.wrapping_sub(1);
        if driver.slow_counter != 0 {
            return Ok(Vec::new());
        }
        driver.slow_counter = 32;
    }
    let mut writes = vec![(0x4c, driver.keys.on), (0x3d, driver.noise_mask)];
    driver.keys.on = 0;
    for slot in 0..VOICES {
        let previous = driver.dsp_flags;
        update_voice(driver, slot, definitions)?;
        if driver.dsp_flags != previous {
            writes.push((0x6c, driver.dsp_flags));
        }
    }
    writes.push((0x5c, driver.keys.off));
    driver.keys.off = 0;
    for slot in 0..VOICES {
        let voice = &driver.voices[slot];
        let base = slot as u8 * 16;
        let [low, high] = voice.dsp_pitch.to_le_bytes();
        writes.extend([
            (base, voice.volume[0]),
            (base + 1, voice.volume[1]),
            (base + 2, low),
            (base + 3, high),
            (base + 4, voice.instrument),
            (base + 7, voice.amplitude),
        ]);
    }
    writes.extend([
        (0x0c, (driver.master >> 8) as u8),
        (0x1c, (driver.master >> 8) as u8),
        (0x5c, 0),
    ]);
    driver.updates = driver.updates.wrapping_add(1);
    Ok(writes)
}
#[cfg(test)]
mod tests {
    use super::*;
    const SCRIPT: DataAddress = DataAddress(0x400);
    fn defs(bytes: &[u8]) -> AudioDefinitions {
        AudioDefinitions {
            score: Default::default(),
            data: SequenceData {
                segments: vec![DataSegment {
                    origin: SCRIPT,
                    bytes: bytes.to_vec(),
                }],
            },
            songs: vec![],
            effects: vec![],
            samples: vec![],
            pitch_low: [0; 256],
            pitch_high: [0; 256],
            instrument_note_base: [0; 256],
            instrument_scale: [0; 256],
            initial_envelopes: vec![[1; 7]; 28],
            dsp_initialization: vec![],
            command_targets: vec![],
        }
        .with_compiled_score()
    }
    fn driver() -> Driver {
        let mut d = Driver::default();
        d.voices[0].active = true;
        d.voices[0].duration = 1;
        d.sequences[0].cursor = (SCRIPT).into();
        d
    }
    #[test]
    fn every_even_command_decodes_exact_operand_count_and_b6_b8_actual_vectors() {
        let counts = [
            0, 2, 2, 0, 1, 0, 1, 0, 1, 1, 1, 1, 3, 1, 3, 3, 1, 0, 1, 1, 2, 0, 3, 1, 0, 0, 1, 0, 0,
            0, 0, 0, 0, 0,
        ];
        for (i, count) in counts.into_iter().enumerate() {
            assert_eq!(
                decode_command(COMMAND_FIRST + i as u8 * 2)
                    .unwrap()
                    .operands,
                count
            );
        }
        assert_eq!(decode_command(0xb6).unwrap().command, Command::PrimaryMode);
        assert_eq!(
            decode_command(0xb8).unwrap().command,
            Command::AuxiliaryEnable
        );
    }
    #[test]
    fn invalid_odd_or_outside_dispatch_tokens_are_not_silently_stopped() {
        for b in [0x7f, 0x81, 0xc3, 0xff] {
            assert!(decode_command(b).is_err());
        }
    }
    #[test]
    fn compiled_command_edge_preserves_next_note_at_word_boundary() {
        let data = SequenceData {
            segments: vec![DataSegment {
                origin: DataAddress(0x4ff),
                bytes: vec![0x8e, 0x22],
            }],
        };
        let graph = ScoreGraph::compile(&data);
        let mut head = graph.resolve(DataAddress(0x4ff));
        let first = graph.fetch(&mut head).unwrap();
        assert_eq!(first.event, Event::Control(Control::ForceDuration));
        assert_eq!(first.next.source(), DataAddress(0x500));
        head = first.next;
        assert!(matches!(
            graph.fetch(&mut head).unwrap().event,
            Event::Note { pitch: 0x22, .. }
        ));
    }
    #[test]
    fn stop_only_deactivates_voice_without_invented_key_off_or_pitch_reset() {
        let mut d = driver();
        d.voices[0].current_pitch = 0x1234;
        d.keys = KeyMasks { on: 2, off: 4 };
        apply_command(&mut d, 0, Command::Stop, &[], &defs(&[])).unwrap();
        assert!(!d.voices[0].active);
        assert_eq!(d.voices[0].current_pitch, 0x1234);
        assert_eq!(d.keys, KeyMasks { on: 2, off: 4 });
    }
    #[test]
    fn jump_reads_little_endian_target_and_parser_continues_to_note() {
        let mut d = driver();
        parse_to_note(&mut d, 0, &defs(&[0x82, 0x04, 0x04, 0x80, 7, 9])).unwrap();
        assert_eq!(d.sequences[0].cursor.source(), DataAddress(0x406));
        assert_eq!((d.voices[0].duration, d.voices[0].note_origin), (9, 7));
    }
    #[test]
    fn nested_calls_push_pointer_after_operands_and_return_last_in_first_out() {
        let mut d = driver();
        d.sequences[0].cursor = (DataAddress(0x403)).into();
        apply_command(&mut d, 0, Command::Call, &[0x20, 4], &defs(&[])).unwrap();
        d.sequences[0].cursor = (DataAddress(0x426)).into();
        apply_command(&mut d, 0, Command::Call, &[0x40, 4], &defs(&[])).unwrap();
        assert_eq!(
            d.sequences[0].stack,
            [
                Frame::Call(DataAddress(0x403).into()),
                Frame::Call(DataAddress(0x426).into())
            ]
        );
        apply_command(&mut d, 0, Command::Return, &[], &defs(&[])).unwrap();
        assert_eq!(d.sequences[0].cursor.source(), DataAddress(0x426));
        apply_command(&mut d, 0, Command::Return, &[], &defs(&[])).unwrap();
        assert_eq!(d.sequences[0].cursor.source(), DataAddress(0x403));
    }
    #[test]
    fn counted_loop_rewinds_then_drops_three_byte_frame_at_count_one() {
        let mut d = driver();
        d.sequences[0].cursor = (DataAddress(0x402)).into();
        apply_command(&mut d, 0, Command::BeginLoop, &[2], &defs(&[])).unwrap();
        d.sequences[0].cursor = (DataAddress(0x410)).into();
        apply_command(&mut d, 0, Command::RepeatLoop, &[], &defs(&[])).unwrap();
        assert_eq!(d.sequences[0].cursor.source(), DataAddress(0x402));
        assert_eq!(
            d.sequences[0].stack,
            [Frame::Loop {
                start: DataAddress(0x402).into(),
                remaining: (1u8).into()
            }]
        );
        d.sequences[0].cursor = (DataAddress(0x410)).into();
        apply_command(&mut d, 0, Command::RepeatLoop, &[], &defs(&[])).unwrap();
        assert_eq!(d.sequences[0].cursor.source(), DataAddress(0x410));
        assert!(d.sequences[0].stack.is_empty());
    }
    #[test]
    fn loop_zero_count_wraps_to255_instead_of_early_exit() {
        let mut d = driver();
        apply_command(&mut d, 0, Command::BeginLoop, &[0], &defs(&[])).unwrap();
        apply_command(&mut d, 0, Command::RepeatLoop, &[], &defs(&[])).unwrap();
        assert_eq!(
            d.sequences[0].stack,
            [Frame::Loop {
                start: SCRIPT.into(),
                remaining: (255u8).into()
            }]
        );
    }
    #[test]
    fn unbalanced_stack_or_missing_operand_rejected_without_partial_voice_mutation() {
        let mut d = driver();
        let old = d.clone();
        assert!(apply_command(&mut d, 0, Command::Return, &[], &defs(&[])).is_err());
        assert_eq!(d, old);
        assert!(apply_command(&mut d, 0, Command::TargetEnvelope, &[1, 2], &defs(&[])).is_err());
        assert_eq!(d, old);
    }
    #[test]
    fn duration_force8e_has_no_operand_and_next_token_remains_note() {
        let mut d = driver();
        parse_to_note(&mut d, 0, &defs(&[0x8c, 7, 0x8e, 5, 11])).unwrap();
        assert_eq!((d.voices[0].fixed_duration, d.voices[0].duration), (7, 11));
        assert_eq!(d.sequences[0].cursor.source(), DataAddress(0x405));
        assert!(!d.voices[0].force_duration);
    }
    #[test]
    fn fixed_duration_stream_does_not_consume_following_note_as_duration() {
        let mut d = driver();
        parse_to_note(&mut d, 0, &defs(&[0x8c, 7, 5, 6])).unwrap();
        assert_eq!(
            (d.voices[0].duration, d.sequences[0].cursor.source()),
            (7, DataAddress(0x403))
        );
    }
    #[test]
    fn simple_setters_mutate_actual_selected_voice_and_no_neighbor() {
        let mut d = driver();
        for (c, b) in [
            (Command::Transpose, 3),
            (Command::Instrument, 33),
            (Command::LeftVolume, 0x90),
            (Command::RightVolume, 0x70),
            (Command::PitchOffset, 0xfe),
            (Command::Portamento, 4),
        ] {
            apply_command(&mut d, 0, c, &[b], &defs(&[])).unwrap();
        }
        let v = &d.voices[0];
        assert_eq!(
            (
                v.transpose,
                v.instrument,
                v.volume,
                v.pitch_low_offset,
                v.portamento_step
            ),
            (3, 33, [0x90, 0x70], 0xfe, 4)
        );
        assert_eq!(d.voices[1], Voice::default());
    }
    #[test]
    fn auxiliary98_consumes_three_operands_and_clears_only_documented_modes() {
        let mut d = driver();
        d.voices[0].primary_mode = true;
        d.voices[0].auxiliary_enable = true;
        d.voices[0].amplitude = 9;
        apply_command(
            &mut d,
            0,
            Command::AuxiliaryStream,
            &[4, 0x34, 0x12],
            &defs(&[]),
        )
        .unwrap();
        assert_eq!(d.sequences[0].auxiliary, DataAddress(0x1234));
        let v = &d.voices[0];
        assert_eq!(
            (v.auxiliary_reload, v.auxiliary_cursor, v.amplitude),
            (4, 0, 0)
        );
        assert!(v.secondary_mode);
        assert!(!v.primary_mode);
        assert!(!v.auxiliary_enable);
    }
    #[test]
    fn positive_and_negative_modulation_have_three_operands_and_disable_keeps_live_direction() {
        let mut d = driver();
        for (c, sign) in [
            (Command::PositiveModulation, 1),
            (Command::NegativeModulation, -1),
        ] {
            apply_command(&mut d, 0, c, &[2, 3, 4], &defs(&[])).unwrap();
            let v = &d.voices[0];
            assert_eq!(
                (
                    v.modulation_direction_reload,
                    v.modulation_delay_reload,
                    v.modulation_step,
                    v.modulation_period_reload,
                    v.modulation_period
                ),
                (sign, 2, 3, 4, 4)
            );
        }
        d.voices[0].modulation_direction = -1;
        apply_command(&mut d, 0, Command::DisableModulation, &[], &defs(&[])).unwrap();
        assert_eq!(
            (
                d.voices[0].modulation_direction_reload,
                d.voices[0].modulation_direction
            ),
            (0, -1)
        );
    }
    #[test]
    fn release_delta_setters_clear_opposite_field() {
        let mut d = driver();
        apply_command(&mut d, 0, Command::ReleaseDelta, &[7], &defs(&[])).unwrap();
        assert_eq!(
            (d.voices[0].release_delta, d.voices[0].release_fixed),
            (7, 0)
        );
        apply_command(&mut d, 0, Command::ReleaseFixed, &[4], &defs(&[])).unwrap();
        assert_eq!(
            (d.voices[0].release_delta, d.voices[0].release_fixed),
            (0, 4)
        );
    }
    #[test]
    fn slew_and_ac_target_configuration_preserve_current_until_tick() {
        let mut d = driver();
        d.voices[0].envelope_current = 9;
        apply_command(&mut d, 0, Command::Slew, &[3, 4], &defs(&[])).unwrap();
        apply_command(&mut d, 0, Command::TargetEnvelope, &[2, 5, 7], &defs(&[])).unwrap();
        let v = &d.voices[0];
        assert_eq!(
            (
                v.envelope_step,
                v.envelope_interval,
                v.target_step,
                v.target_interval,
                v.target_delay,
                v.target_countdown,
                v.envelope_current
            ),
            (3, 4, 2, 5, 7, 7, 9)
        );
        apply_command(&mut d, 0, Command::DisableSlew, &[], &defs(&[])).unwrap();
        assert_eq!(d.voices[0].envelope_interval, 0);
    }
    #[test]
    fn seven_byte_record_selector_wraps_product_byte_and_changes_modes() {
        let mut d = driver();
        apply_command(&mut d, 0, Command::EnvelopeRecord, &[40], &defs(&[])).unwrap();
        let v = &d.voices[0];
        assert_eq!(v.envelope_record, 24);
        assert!(v.primary_mode);
        assert!(!v.secondary_mode);
        assert!(!v.auxiliary_enable);
    }
    #[test]
    fn b6_and_b8_follow_vectors_not_misleading_nearby_label_order() {
        let mut d = driver();
        d.voices[0].secondary_mode = true;
        apply_command(&mut d, 0, Command::PrimaryMode, &[], &defs(&[])).unwrap();
        assert!(d.voices[0].primary_mode);
        assert!(!d.voices[0].secondary_mode);
        assert!(!d.voices[0].auxiliary_enable);
        apply_command(&mut d, 0, Command::AuxiliaryEnable, &[], &defs(&[])).unwrap();
        assert!(d.voices[0].auxiliary_enable);
        assert!(d.voices[0].primary_mode);
    }
    #[test]
    fn ba_has_no_operand_and_selects_secondary_without_overwriting_stream() {
        let mut d = driver();
        d.sequences[0].auxiliary = DataAddress(0x1234);
        d.voices[0].primary_mode = true;
        d.voices[0].auxiliary_enable = true;
        apply_command(&mut d, 0, Command::SecondaryMode, &[], &defs(&[])).unwrap();
        assert_eq!(d.sequences[0].auxiliary, DataAddress(0x1234));
        assert!(d.voices[0].secondary_mode);
        assert!(!d.voices[0].primary_mode);
        assert!(!d.voices[0].auxiliary_enable);
    }
    #[test]
    fn noise_masks_affect_selected_voice_and_flg_uses_low_five_bits() {
        let mut d = driver();
        d.noise_mask = 1;
        apply_command(&mut d, 6, Command::NoiseOn, &[], &defs(&[])).unwrap();
        assert_eq!(d.noise_mask, 0x41);
        apply_command(&mut d, 6, Command::NoiseOff, &[], &defs(&[])).unwrap();
        assert_eq!(d.noise_mask, 1);
        apply_command(&mut d, 0, Command::NoiseClock, &[0xff], &defs(&[])).unwrap();
        assert_eq!(d.dsp_flags, 0x3f);
    }
    #[test]
    fn key_suppression_and_skip_commands_have_no_operands() {
        let mut d = driver();
        apply_command(&mut d, 0, Command::SuppressKeyOn, &[], &defs(&[])).unwrap();
        assert!(d.voices[0].suppress_key_on);
        apply_command(&mut d, 0, Command::AllowKeyOn, &[], &defs(&[])).unwrap();
        assert!(!d.voices[0].suppress_key_on);
        apply_command(&mut d, 0, Command::SkipTranspose, &[], &defs(&[])).unwrap();
        assert_eq!(d.voices[0].skip_transpose, 1);
    }
    #[test]
    fn inactive_voice_does_not_decrement_or_touch_pending_masks() {
        let mut d = driver();
        d.voices[0].active = false;
        let old = d.clone();
        update_voice(&mut d, 0, &defs(&[])).unwrap();
        assert_eq!(d, old);
    }
    #[test]
    fn duration_zero_wraps255_while_duration_one_parses_then_runs_pitch_helpers() {
        let mut d = driver();
        d.voices[0].duration = 0;
        d.voices[0].envelope_delay = 2;
        d.voices[0].target_pitch = 0x1234;
        update_voice(&mut d, 0, &defs(&[0x80])).unwrap();
        assert_eq!(d.voices[0].duration, 255);
        assert_eq!(d.voices[0].current_pitch, 0x1234);
        d.voices[0].duration = 1;
        update_voice(&mut d, 0, &defs(&[1, 3])).unwrap();
        assert_eq!(d.voices[0].duration, 3);
        assert_eq!(d.sequences[0].cursor.source(), DataAddress(0x402));
    }
    #[test]
    fn original_song_load_resets_priorities_keys_controls_but_keeps141_live_envelope_overlap() {
        let r = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap();
        let a = load_audio(&r).unwrap();
        let mut d = driver();
        d.envelopes.rows = a.initial_envelopes.clone();
        d.envelopes.rows[0] = [5; 7];
        d.effect_priorities = [5, 9];
        d.master = 0x1234;
        d.fade_step = -8;
        d.keys.on = 7;
        d.voices[0].current_pitch = 0xffff;
        load_song(&mut d, 1, &a).unwrap();
        assert_eq!(d.effect_priorities, [0, 0]);
        assert_eq!(d.keys, KeyMasks::default());
        assert_eq!((d.master, d.fade_step, d.selected_song_shadow), (0, 0, 0));
        assert_eq!(d.sequences[0].cursor.source(), DataAddress(0x1568));
        assert!(
            d.voices
                .iter()
                .all(|v| v.active && v.duration == 1 && v.current_pitch == 0)
        );
        // The eleven duration-byte stores also alias the first three current-envelope cells.
        assert_eq!(
            [
                d.voices[0].envelope_current,
                d.voices[1].envelope_current,
                d.voices[2].envelope_current
            ],
            [1, 1, 1]
        );
        assert_eq!(d.voices[3].envelope_current, 0);
        assert_eq!(d.envelopes.rows[0], [5; 7]);
        assert_eq!(d.envelopes.rows[1], r[0x70d58..0x70d5f]);
    }
    #[test]
    fn reload_c2_forces_song_one_even_when_selected_song_differs() {
        let r = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap();
        let a = load_audio(&r).unwrap();
        let mut d = driver();
        d.selected_song_shadow = 7;
        apply_command(&mut d, 0, Command::ReloadSongOne, &[], &a).unwrap();
        assert_eq!(d.requested_song, Some(1));
        assert_eq!(d.sequences[0].cursor.source(), DataAddress(0x1568));
        assert_eq!(d.master, 0x7f00);
    }
    #[test]
    fn complete_driver_commits_prior_key_on_then_new_key_off_and_six_registers_per_voice() {
        let mut d = driver();
        d.keys = KeyMasks { on: 4, off: 8 };
        d.voices[0].volume = [0x70, 0x60];
        d.voices[0].instrument = 3;
        d.voices[0].amplitude = 0x40;
        let writes = update_driver(&mut d, 0, &defs(&[1, 3])).unwrap();
        assert_eq!(&writes[..2], &[(0x4c, 4), (0x3d, 0)]);
        let off = writes.iter().position(|p| p.0 == 0x5c).unwrap();
        assert_eq!(writes[off], (0x5c, 9));
        assert_eq!(
            &writes[off + 1..off + 7],
            &[(0, 0x70), (1, 0x60), (2, 0), (3, 0), (4, 3), (7, 0x40)]
        );
        assert_eq!(d.keys.on, 1);
        assert_eq!(d.keys.off, 0);
        assert_eq!(writes.last(), Some(&(0x5c, 0)));
    }
    #[test]
    fn malformed_command_chain_or_unavailable_note_data_returns_error_not_infinite_host_loop() {
        let mut d = driver();
        assert!(parse_to_note(&mut d, 0, &defs(&[0x82, 0, 4])).is_err());
        let mut d = driver();
        assert!(parse_to_note(&mut d, 0, &defs(&[0x92])).is_err());
    }
    #[test]
    fn slow_flag_updates_fade_before_divider_and_preserves_pending_keys_until_voice_tick() {
        let mut d = driver();
        d.voices[0].duration = 5;
        d.master = 0x1000;
        d.fade_step = -8;
        d.slow_counter = 2;
        d.keys.on = 4;
        assert!(update_driver(&mut d, 4, &defs(&[])).unwrap().is_empty());
        assert_eq!(
            (d.master, d.slow_counter, d.voices[0].duration, d.keys.on),
            (0x0ff8, 1, 5, 4)
        );
        assert!(!update_driver(&mut d, 4, &defs(&[])).unwrap().is_empty());
        assert_eq!(
            (d.master, d.slow_counter, d.voices[0].duration),
            (0x0ff0, 32, 4)
        );
        d.slow_counter = 0;
        assert!(update_driver(&mut d, 4, &defs(&[])).unwrap().is_empty());
        assert_eq!(d.slow_counter, 255);
    }
    #[test]
    fn original_effect_prefixes_initialize_instrument_before_first_nonzero_note() {
        let r = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap();
        let a = load_audio(&r).unwrap();
        // Literal instrument/duration pairs from each independently read prefix.
        for (effect, instrument, duration) in [(1, 25, 80), (5, 14, 60), (6, 14, 60)] {
            let mut d = driver();
            d.sequences[0].cursor = (a.effects[effect].sequence).into();
            parse_to_note(&mut d, 0, &a).unwrap();
            assert_eq!(
                (d.voices[0].instrument, d.voices[0].duration),
                (instrument, duration)
            );
            assert_ne!(d.voices[0].target_pitch, 0);
            assert_eq!(d.voices[0].envelope_current, d.voices[0].note_origin);
            assert_eq!(d.keys, KeyMasks { on: 1, off: 1 });
        }
    }
}

#[cfg(test)]
mod modernization_contracts {
    use super::*;
    fn definitions(bytes: &[u8]) -> AudioDefinitions {
        let mut definitions = load_audio(&std::fs::read("../input/game.rom").unwrap()).unwrap();
        definitions.data = SequenceData {
            segments: vec![DataSegment {
                origin: DataAddress(0x400),
                bytes: bytes.to_vec(),
            }],
        };
        definitions.compile_score();
        definitions
    }
    fn active() -> Driver {
        let mut driver = Driver::default();
        driver.voices[0].active = true;
        driver.sequences[0].cursor = (DataAddress(0x400)).into();
        driver
    }
    #[test]
    fn modernization_audio_duration_context_defers_dormant_odd_command() {
        let defs = definitions(&[5, 0x91, 7, 9]);
        let mut variable = active();
        parse_to_note(&mut variable, 0, &defs).unwrap();
        assert_eq!(variable.voices[0].duration, 0x91);
        assert_eq!(variable.sequences[0].cursor.source(), DataAddress(0x402));
        let mut fixed = active();
        fixed.voices[0].fixed_duration = 1;
        parse_to_note(&mut fixed, 0, &defs).unwrap();
        assert_eq!(fixed.sequences[0].cursor.source(), DataAddress(0x401));
        let retained = fixed.clone();
        assert!(parse_to_note(&mut fixed, 0, &defs).is_err());
        assert_eq!(fixed, retained);
    }
    #[test]
    fn modernization_audio_forced_rest_consumes_once_and_keeps_next_note_fixed() {
        let defs = definitions(&[0, 13, 5]);
        let mut driver = active();
        driver.voices[0].fixed_duration = 7;
        driver.voices[0].force_duration = true;
        driver.voices[0].skip_transpose = 3;
        parse_to_note(&mut driver, 0, &defs).unwrap();
        assert_eq!(driver.voices[0].duration, 13);
        assert_eq!(driver.voices[0].skip_transpose, 3);
        assert!(!driver.voices[0].force_duration);
        assert_eq!(driver.keys, KeyMasks::default());
        assert_eq!(driver.sequences[0].cursor.source(), DataAddress(0x402));
        parse_to_note(&mut driver, 0, &defs).unwrap();
        assert_eq!(driver.voices[0].duration, 7);
        assert_eq!(driver.sequences[0].cursor.source(), DataAddress(0x403));
    }
    #[test]
    fn modernization_audio_source_wrap_and_missing_successor_are_deferred() {
        let mut defs = definitions(&[]);
        defs.data.segments = vec![
            DataSegment {
                origin: DataAddress(0xffff),
                bytes: vec![5],
            },
            DataSegment {
                origin: DataAddress(0),
                bytes: vec![9],
            },
        ];
        defs.compile_score();
        let mut driver = active();
        driver.sequences[0].cursor = (DataAddress(0xffff)).into();
        parse_to_note(&mut driver, 0, &defs).unwrap();
        assert_eq!(driver.voices[0].duration, 9);
        assert_eq!(driver.sequences[0].cursor.source(), DataAddress(1));
        let retained = driver.clone();
        assert!(parse_to_note(&mut driver, 0, &defs).is_err());
        assert_eq!(driver, retained);
    }
}
