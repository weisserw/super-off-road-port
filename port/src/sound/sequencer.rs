//! Game sequence language and per-voice control flow, never SPC instruction execution.
use super::{definitions::*, voice::*};
use anyhow::{Context, Result, ensure};
pub const COMMAND_FIRST: u8 = 0x80;
pub const COMMAND_LAST: u8 = 0xc2;
const COMMANDS: [(Command, usize); 34] = [
    (Command::Stop, 0),
    (Command::Jump, 2),
    (Command::Call, 2),
    (Command::Return, 0),
    (Command::BeginLoop, 1),
    (Command::RepeatLoop, 0),
    (Command::FixedDuration, 1),
    (Command::ForceDuration, 0),
    (Command::Transpose, 1),
    (Command::Instrument, 1),
    (Command::LeftVolume, 1),
    (Command::RightVolume, 1),
    (Command::AuxiliaryStream, 3),
    (Command::PitchOffset, 1),
    (Command::PositiveModulation, 3),
    (Command::NegativeModulation, 3),
    (Command::Portamento, 1),
    (Command::DisableModulation, 0),
    (Command::ReleaseDelta, 1),
    (Command::ReleaseFixed, 1),
    (Command::Slew, 2),
    (Command::DisableSlew, 0),
    (Command::TargetEnvelope, 3),
    (Command::EnvelopeRecord, 1),
    (Command::NoiseOn, 0),
    (Command::NoiseOff, 0),
    (Command::NoiseClock, 1),
    (Command::PrimaryMode, 0),
    (Command::AuxiliaryEnable, 0),
    (Command::SecondaryMode, 0),
    (Command::AllowKeyOn, 0),
    (Command::SuppressKeyOn, 0),
    (Command::SkipTranspose, 0),
    (Command::ReloadSongOne, 0),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Stop,
    Jump,
    Call,
    Return,
    BeginLoop,
    RepeatLoop,
    FixedDuration,
    ForceDuration,
    Transpose,
    Instrument,
    LeftVolume,
    RightVolume,
    AuxiliaryStream,
    PitchOffset,
    PositiveModulation,
    NegativeModulation,
    Portamento,
    DisableModulation,
    ReleaseDelta,
    ReleaseFixed,
    Slew,
    DisableSlew,
    TargetEnvelope,
    EnvelopeRecord,
    NoiseOn,
    NoiseOff,
    NoiseClock,
    PrimaryMode,
    AuxiliaryEnable,
    SecondaryMode,
    AllowKeyOn,
    SuppressKeyOn,
    SkipTranspose,
    ReloadSongOne,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommandDefinition {
    pub command: Command,
    pub operands: usize,
}
pub fn decode_command(token: u8) -> Result<CommandDefinition> {
    ensure!(
        (COMMAND_FIRST..=COMMAND_LAST).contains(&token) && token & 1 == 0,
        "invalid sequence command"
    );
    let (command, operands) = COMMANDS[usize::from((token - COMMAND_FIRST) / 2)];
    Ok(CommandDefinition { command, operands })
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Frame {
    Call(DataAddress),
    Loop { start: DataAddress, remaining: u8 },
}
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sequence {
    pub cursor: DataAddress,
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
        next.sequences[slot].cursor = song_data.voices[slot];
    }
    *driver = next;
    Ok(())
}
pub fn read_sequence_byte(sequence: &mut Sequence, data: &SequenceData) -> Result<u8> {
    let value = data.read(sequence.cursor)?;
    sequence.cursor.0 = sequence.cursor.0.wrapping_add(1);
    Ok(value)
}
pub fn apply_command(
    driver: &mut Driver,
    slot: usize,
    command: Command,
    operands: &[u8],
    definitions: &AudioDefinitions,
) -> Result<()> {
    ensure!(slot < VOICES, "invalid voice slot");
    let expected = COMMANDS
        .iter()
        .find(|(value, _)| *value == command)
        .unwrap()
        .1;
    ensure!(operands.len() == expected, "invalid sequence operands");
    if command == Command::ReloadSongOne {
        load_song(driver, 1, definitions)?;
        driver.master = 0x7f00;
        return Ok(());
    }
    let voice = &mut driver.voices[slot];
    let sequence = &mut driver.sequences[slot];
    let address = || DataAddress(u16::from_le_bytes([operands[0], operands[1]]));
    match command {
        Command::Stop => voice.active = false,
        Command::Jump => sequence.cursor = address(),
        Command::Call => {
            ensure!(sequence.stack.len() < 256, "sequence call stack limit");
            sequence.stack.push(Frame::Call(sequence.cursor));
            sequence.cursor = address();
        }
        Command::Return => {
            let Some(Frame::Call(target)) = sequence.stack.last() else {
                anyhow::bail!("unbalanced sequence return");
            };
            sequence.cursor = *target;
            sequence.stack.pop();
        }
        Command::BeginLoop => {
            ensure!(sequence.stack.len() < 256, "sequence loop stack limit");
            sequence.stack.push(Frame::Loop {
                start: sequence.cursor,
                remaining: operands[0],
            });
        }
        Command::RepeatLoop => {
            let Some(Frame::Loop { start, remaining }) = sequence.stack.last_mut() else {
                anyhow::bail!("unbalanced loop repeat");
            };
            *remaining = remaining.wrapping_sub(1);
            if *remaining == 0 {
                sequence.stack.pop();
            } else {
                sequence.cursor = *start;
            }
        }
        Command::FixedDuration => voice.fixed_duration = operands[0],
        Command::ForceDuration => voice.force_duration = true,
        Command::Transpose => voice.transpose = operands[0],
        Command::Instrument => voice.instrument = operands[0],
        Command::LeftVolume => voice.volume[0] = operands[0],
        Command::RightVolume => voice.volume[1] = operands[0],
        Command::AuxiliaryStream => {
            voice.auxiliary_reload = operands[0];
            sequence.auxiliary = DataAddress(u16::from_le_bytes([operands[1], operands[2]]));
            voice.amplitude = 0;
            voice.auxiliary_cursor = 0;
            voice.primary_mode = false;
            voice.auxiliary_enable = false;
            voice.secondary_mode = true;
        }
        Command::PitchOffset => voice.pitch_low_offset = operands[0],
        Command::PositiveModulation | Command::NegativeModulation => {
            voice.modulation_direction_reload = if command == Command::PositiveModulation {
                1
            } else {
                -1
            };
            voice.modulation_delay_reload = operands[0];
            voice.modulation_step = operands[1];
            voice.modulation_period_reload = operands[2];
            voice.modulation_period = operands[2];
        }
        Command::Portamento => voice.portamento_step = operands[0],
        Command::DisableModulation => voice.modulation_direction_reload = 0,
        Command::ReleaseDelta => {
            voice.release_delta = operands[0];
            voice.release_fixed = 0;
        }
        Command::ReleaseFixed => {
            voice.release_fixed = operands[0];
            voice.release_delta = 0;
        }
        Command::Slew => {
            voice.envelope_step = operands[0];
            voice.envelope_interval = operands[1];
        }
        Command::DisableSlew => voice.envelope_interval = 0,
        Command::TargetEnvelope => {
            voice.target_step = operands[0];
            voice.target_interval = operands[1];
            voice.target_delay = operands[2];
            voice.target_countdown = operands[2];
        }
        Command::EnvelopeRecord => {
            voice.envelope_record = operands[0].wrapping_mul(7);
            voice.primary_mode = true;
            voice.secondary_mode = false;
            voice.auxiliary_enable = false;
        }
        Command::NoiseOn => driver.noise_mask |= 1 << slot,
        Command::NoiseOff => driver.noise_mask &= !(1 << slot),
        Command::NoiseClock => driver.dsp_flags = (operands[0] & 31) | 0x20,
        Command::PrimaryMode => {
            voice.primary_mode = true;
            voice.secondary_mode = false;
            voice.auxiliary_enable = false;
        }
        Command::AuxiliaryEnable => voice.auxiliary_enable = true,
        Command::SecondaryMode => {
            voice.secondary_mode = true;
            voice.primary_mode = false;
            voice.auxiliary_enable = false;
        }
        Command::AllowKeyOn => voice.suppress_key_on = false,
        Command::SuppressKeyOn => voice.suppress_key_on = true,
        Command::SkipTranspose => voice.skip_transpose = 1,
        Command::ReloadSongOne => unreachable!(),
    }
    Ok(())
}
pub fn parse_to_note(
    driver: &mut Driver,
    slot: usize,
    definitions: &AudioDefinitions,
) -> Result<()> {
    ensure!(slot < VOICES, "invalid parser voice");
    let mut next = driver.clone();
    for _ in 0..4096 {
        let token = read_sequence_byte(&mut next.sequences[slot], &definitions.data)?;
        if token < 0x80 {
            let need = next.voices[slot].force_duration || next.voices[slot].fixed_duration == 0;
            let duration = if need {
                Some(read_sequence_byte(
                    &mut next.sequences[slot],
                    &definitions.data,
                )?)
            } else {
                None
            };
            start_note(
                &mut next.voices[slot],
                token,
                duration,
                1 << slot,
                &mut next.keys,
                definitions,
            )?;
            *driver = next;
            return Ok(());
        }
        let metadata = decode_command(token)?;
        let operands = (0..metadata.operands)
            .map(|_| read_sequence_byte(&mut next.sequences[slot], &definitions.data))
            .collect::<Result<Vec<_>>>()?;
        apply_command(&mut next, slot, metadata.command, &operands, definitions)?;
        if !next.voices[slot].active {
            *driver = next;
            return Ok(());
        }
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
    }
    fn driver() -> Driver {
        let mut d = Driver::default();
        d.voices[0].active = true;
        d.voices[0].duration = 1;
        d.sequences[0].cursor = SCRIPT;
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
    fn sequence_reader_increments_word_pointer_and_preserves_fetched_token() {
        let data = SequenceData {
            segments: vec![DataSegment {
                origin: DataAddress(0x4ff),
                bytes: vec![0x8e, 0x22],
            }],
        };
        let mut s = Sequence {
            cursor: DataAddress(0x4ff),
            ..Sequence::default()
        };
        assert_eq!(read_sequence_byte(&mut s, &data).unwrap(), 0x8e);
        assert_eq!(s.cursor, DataAddress(0x500));
        assert_eq!(read_sequence_byte(&mut s, &data).unwrap(), 0x22);
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
        assert_eq!(d.sequences[0].cursor, DataAddress(0x406));
        assert_eq!((d.voices[0].duration, d.voices[0].note_origin), (9, 7));
    }
    #[test]
    fn nested_calls_push_pointer_after_operands_and_return_last_in_first_out() {
        let mut d = driver();
        d.sequences[0].cursor = DataAddress(0x403);
        apply_command(&mut d, 0, Command::Call, &[0x20, 4], &defs(&[])).unwrap();
        d.sequences[0].cursor = DataAddress(0x426);
        apply_command(&mut d, 0, Command::Call, &[0x40, 4], &defs(&[])).unwrap();
        assert_eq!(
            d.sequences[0].stack,
            [
                Frame::Call(DataAddress(0x403)),
                Frame::Call(DataAddress(0x426))
            ]
        );
        apply_command(&mut d, 0, Command::Return, &[], &defs(&[])).unwrap();
        assert_eq!(d.sequences[0].cursor, DataAddress(0x426));
        apply_command(&mut d, 0, Command::Return, &[], &defs(&[])).unwrap();
        assert_eq!(d.sequences[0].cursor, DataAddress(0x403));
    }
    #[test]
    fn counted_loop_rewinds_then_drops_three_byte_frame_at_count_one() {
        let mut d = driver();
        d.sequences[0].cursor = DataAddress(0x402);
        apply_command(&mut d, 0, Command::BeginLoop, &[2], &defs(&[])).unwrap();
        d.sequences[0].cursor = DataAddress(0x410);
        apply_command(&mut d, 0, Command::RepeatLoop, &[], &defs(&[])).unwrap();
        assert_eq!(d.sequences[0].cursor, DataAddress(0x402));
        assert_eq!(
            d.sequences[0].stack,
            [Frame::Loop {
                start: DataAddress(0x402),
                remaining: 1
            }]
        );
        d.sequences[0].cursor = DataAddress(0x410);
        apply_command(&mut d, 0, Command::RepeatLoop, &[], &defs(&[])).unwrap();
        assert_eq!(d.sequences[0].cursor, DataAddress(0x410));
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
                start: SCRIPT,
                remaining: 255
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
        assert_eq!(d.sequences[0].cursor, DataAddress(0x405));
        assert!(!d.voices[0].force_duration);
    }
    #[test]
    fn fixed_duration_stream_does_not_consume_following_note_as_duration() {
        let mut d = driver();
        parse_to_note(&mut d, 0, &defs(&[0x8c, 7, 5, 6])).unwrap();
        assert_eq!(
            (d.voices[0].duration, d.sequences[0].cursor),
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
        assert_eq!(d.sequences[0].cursor, DataAddress(0x402));
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
        assert_eq!(d.sequences[0].cursor, DataAddress(0x1568));
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
        assert_eq!(d.sequences[0].cursor, DataAddress(0x1568));
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
            d.sequences[0].cursor = a.effects[effect].sequence;
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
