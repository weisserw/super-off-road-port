//! Voice-domain state; even SPC array indices become one semantic record per voice.
use super::definitions::AudioDefinitions;
use anyhow::{Context, Result, ensure};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum EnvelopeStage {
    #[default]
    Rise,
    Fall,
    Sustain,
    Release,
    Done,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Voice {
    pub active: bool,
    pub instrument: u8,
    pub volume: [u8; 2],
    pub duration: u8,
    pub fixed_duration: u8,
    pub force_duration: bool,
    pub transpose: u8,
    pub skip_transpose: u8,
    pub suppress_key_on: bool,
    pub current_pitch: u16,
    pub target_pitch: u16,
    pub dsp_pitch: u16,
    pub pitch_low_offset: u8,
    pub portamento_step: u8,
    pub pitch_modulation: u16,
    pub modulation_direction: i8,
    pub modulation_direction_reload: i8,
    pub modulation_delay: u8,
    pub modulation_delay_reload: u8,
    pub modulation_period: u8,
    pub modulation_period_reload: u8,
    pub modulation_step: u8,
    pub note_origin: u8,
    pub envelope_target: u8,
    pub envelope_current: u8,
    pub envelope_delay: u8,
    pub envelope_interval: u8,
    pub envelope_step: u8,
    pub target_step: u8,
    pub target_delay: u8,
    pub target_interval: u8,
    pub target_countdown: u8,
    pub release_delta: u8,
    pub release_fixed: u8,
    pub release_threshold: u8,
    pub auxiliary_delay: u8,
    pub auxiliary_reload: u8,
    pub auxiliary_cursor: u8,
    pub amplitude: u8,
    pub secondary_mode: bool,
    pub primary_mode: bool,
    pub auxiliary_enable: bool,
    pub envelope_record: u8,
    pub stage: EnvelopeStage,
    pub envelope_tick: u8,
    pub stage_countdown: u8,
    pub release_amplitude: u8,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyMasks {
    pub on: u8,
    pub off: u8,
}
pub fn start_note(
    voice: &mut Voice,
    note: u8,
    duration: Option<u8>,
    mask: u8,
    keys: &mut KeyMasks,
    definitions: &AudioDefinitions,
) -> Result<()> {
    let timing = if voice.force_duration || voice.fixed_duration == 0 {
        duration.context("missing note duration")?
    } else {
        voice.fixed_duration
    };
    if note != 0 {
        let mut origin =
            note.wrapping_add(definitions.instrument_note_base[usize::from(voice.instrument)]);
        let skip = voice.skip_transpose & 1 != 0;
        voice.skip_transpose >>= 1;
        if !skip {
            origin = origin.wrapping_add(voice.transpose);
        }
        voice.note_origin = origin;
        voice.envelope_target = origin;
        voice.envelope_delay = voice.envelope_interval;
        if voice.envelope_interval == 0 {
            voice.envelope_current = origin;
            voice.target_pitch = calculate_pitch(definitions, origin, voice.instrument);
        }
        voice.modulation_delay = voice.modulation_delay_reload;
        voice.modulation_period = voice.modulation_period_reload >> 1;
        voice.modulation_direction = voice.modulation_direction_reload;
        voice.pitch_modulation = 0;
        if voice.secondary_mode && !voice.auxiliary_enable {
            voice.auxiliary_cursor = 0;
            voice.auxiliary_delay = voice.auxiliary_reload;
        }
        voice.target_countdown = voice.target_delay;
        if voice.primary_mode && !voice.auxiliary_enable {
            voice.stage = EnvelopeStage::Rise;
            voice.envelope_tick = 0;
            voice.stage_countdown = 1;
        }
        if !voice.suppress_key_on {
            keys.on |= mask;
            keys.off |= mask;
        }
    }
    if voice.force_duration || voice.fixed_duration == 0 {
        voice.force_duration = false;
    }
    voice.duration = timing;
    voice.release_threshold = if voice.release_delta != 0 {
        timing.wrapping_sub(voice.release_delta)
    } else {
        voice.release_fixed
    };
    Ok(())
}
pub fn calculate_pitch(definitions: &AudioDefinitions, note: u8, instrument: u8) -> u16 {
    let index = usize::from(note);
    let base = u16::from_le_bytes([definitions.pitch_low[index], definitions.pitch_high[index]]);
    base.wrapping_add(
        ((u32::from(base) * u32::from(definitions.instrument_scale[usize::from(instrument)])) >> 8)
            as u16,
    )
}
pub fn step_target(voice: &mut Voice) {
    if voice.target_step == 0 {
        return;
    }
    voice.target_countdown = voice.target_countdown.wrapping_sub(1);
    if voice.target_countdown != 0 {
        return;
    }
    if voice.note_origin != voice.envelope_target {
        voice.envelope_target = voice.note_origin;
        voice.target_countdown = voice.target_delay;
    } else {
        voice.envelope_target = voice.envelope_target.wrapping_add(voice.target_step);
        voice.target_countdown = voice.target_interval;
    }
}
pub fn slew_envelope(voice: &mut Voice, definitions: &AudioDefinitions) {
    if voice.envelope_delay == 0 {
        voice.envelope_current = voice.envelope_target;
    } else {
        voice.envelope_delay = voice.envelope_delay.wrapping_sub(1);
        if voice.envelope_delay != 0 {
            return;
        }
        voice.envelope_delay = voice.envelope_interval;
        if voice.envelope_current == voice.envelope_target {
            return;
        }
        if voice.envelope_current > voice.envelope_target {
            let next = voice.envelope_current.wrapping_sub(voice.envelope_step);
            voice.envelope_current = if next < voice.envelope_target {
                voice.envelope_target
            } else {
                next
            };
        } else {
            let next = voice.envelope_current.wrapping_add(voice.envelope_step);
            voice.envelope_current = if next >= voice.envelope_target {
                voice.envelope_target
            } else {
                next
            };
        }
    }
    voice.target_pitch = calculate_pitch(definitions, voice.envelope_current, voice.instrument);
}
pub fn update_modulation(voice: &mut Voice) {
    if voice.modulation_delay != 0 {
        voice.modulation_delay = voice.modulation_delay.wrapping_sub(1);
        return;
    }
    if voice.modulation_direction == 0 {
        return;
    }
    let delta = if voice.modulation_direction < 0 {
        0xff00 | u16::from(voice.modulation_step.wrapping_neg())
    } else {
        u16::from(voice.modulation_step)
    };
    voice.pitch_modulation = voice.pitch_modulation.wrapping_add(delta);
    voice.modulation_period = voice.modulation_period.wrapping_sub(1);
    if voice.modulation_period == 0 {
        voice.modulation_period = voice.modulation_period_reload;
        if voice.modulation_period_reload != 0 {
            voice.modulation_direction = voice.modulation_direction.wrapping_neg();
        }
    }
}
pub fn update_portamento(voice: &mut Voice) {
    if voice.portamento_step == 0 {
        voice.current_pitch = voice.target_pitch;
        return;
    }
    if voice.current_pitch < voice.target_pitch {
        let next = voice
            .current_pitch
            .wrapping_add(u16::from(voice.portamento_step));
        voice.current_pitch = if next >= voice.target_pitch {
            voice.target_pitch
        } else {
            next
        };
    } else if voice.current_pitch > voice.target_pitch {
        let next = voice
            .current_pitch
            .wrapping_add(0xff00 | u16::from(voice.portamento_step.wrapping_neg()));
        voice.current_pitch = if next < voice.target_pitch {
            voice.target_pitch
        } else {
            next
        };
    }
}
pub fn update_auxiliary(voice: &mut Voice, stream: &[u8]) -> Result<()> {
    if !voice.secondary_mode {
        return Ok(());
    }
    let mut next = voice.clone();
    if next.auxiliary_delay != 0 {
        next.auxiliary_delay = next.auxiliary_delay.wrapping_sub(1);
        if next.auxiliary_delay == 0 {
            let mut visited = [false; 256];
            loop {
                let cursor = usize::from(next.auxiliary_cursor);
                ensure!(!visited[cursor], "zero-time auxiliary restart");
                visited[cursor] = true;
                let value = *stream
                    .get(cursor)
                    .context("auxiliary cursor beyond stream")?;
                next.auxiliary_cursor = next.auxiliary_cursor.wrapping_add(1);
                if value == 0x80 {
                    next.auxiliary_cursor = 0;
                    continue;
                }
                if value & 0x80 != 0 {
                    next.auxiliary_cursor = next.auxiliary_cursor.wrapping_sub(1);
                } else {
                    next.amplitude = value;
                }
                break;
            }
            next.auxiliary_delay = next.auxiliary_reload;
        }
    }
    if next.duration == next.release_threshold {
        let marker = stream
            .iter()
            .take(256)
            .position(|value| value & 0x80 != 0)
            .context("missing auxiliary release marker")?;
        next.auxiliary_cursor = (marker as u8).wrapping_add(1);
    }
    *voice = next;
    Ok(())
}
pub fn interpolate_stage(voice: &mut Voice, row: [u8; 7]) {
    match voice.stage {
        EnvelopeStage::Rise => {
            let duration = row[2];
            if duration == 0 {
                return;
            }
            let product = u16::from(row[3].wrapping_sub(row[1])) * u16::from(voice.envelope_tick);
            let divisor = duration.wrapping_sub(1);
            let quotient = if divisor == 0 {
                0xff
            } else {
                (product / u16::from(divisor)) as u8
            };
            voice.amplitude = row[1].wrapping_add(quotient);
            voice.envelope_tick = voice.envelope_tick.wrapping_add(1);
            if voice.envelope_tick == duration {
                voice.stage = EnvelopeStage::Fall;
                voice.envelope_tick = 0;
            }
        }
        EnvelopeStage::Fall => {
            let duration = row[4];
            if duration == 0 {
                return;
            }
            let remaining = duration.wrapping_sub(voice.envelope_tick).wrapping_sub(1);
            if remaining == 0 {
                voice.amplitude = row[5];
                voice.stage = EnvelopeStage::Sustain;
                voice.envelope_tick = 0;
            } else {
                voice.amplitude = row[5].wrapping_add(
                    (u16::from(row[3].wrapping_sub(row[5])) * u16::from(remaining)
                        / u16::from(duration)) as u8,
                );
                voice.envelope_tick = voice.envelope_tick.wrapping_add(1);
            }
        }
        EnvelopeStage::Release => {
            let duration = row[6];
            if duration == 0 {
                return;
            }
            let remaining = duration.wrapping_sub(voice.envelope_tick).wrapping_sub(1);
            if remaining == 0 {
                voice.amplitude = 0;
                voice.stage = EnvelopeStage::Done;
            } else {
                voice.amplitude = (u16::from(voice.release_amplitude) * u16::from(remaining)
                    / u16::from(duration)) as u8;
                voice.envelope_tick = voice.envelope_tick.wrapping_add(1);
            }
        }
        _ => {}
    }
}
pub fn service_envelope_stage(voice: &mut Voice, row: [u8; 7]) {
    if voice.duration != 0 && voice.duration == voice.release_threshold {
        voice.stage = EnvelopeStage::Release;
        voice.release_amplitude = voice.amplitude;
        voice.envelope_tick = 0;
    }
    voice.stage_countdown = voice.stage_countdown.wrapping_sub(1);
    if voice.stage_countdown == 0 {
        voice.stage_countdown = row[0];
        interpolate_stage(voice, row);
    }
}
pub fn combine_pitch(voice: &mut Voice) {
    let (low, carry) = (voice.current_pitch as u8).overflowing_add(voice.pitch_modulation as u8);
    let (low, offset_carry) = low.overflowing_add(voice.pitch_low_offset);
    let high = ((voice.current_pitch >> 8) as u8)
        .wrapping_add((voice.pitch_modulation >> 8) as u8)
        .wrapping_add(u8::from(offset_carry))
        .wrapping_add(u8::from(carry));
    voice.dsp_pitch = u16::from_le_bytes([low, high]);
}
#[cfg(test)]
mod tests {
    use super::super::definitions::*;
    use super::*;
    fn definitions() -> AudioDefinitions {
        AudioDefinitions {
            score: Default::default(),
            data: SequenceData::default(),
            songs: vec![],
            effects: vec![],
            samples: vec![],
            pitch_low: [0; 256],
            pitch_high: [0; 256],
            instrument_note_base: [0; 256],
            instrument_scale: [0; 256],
            initial_envelopes: vec![],
            dsp_initialization: vec![],
            command_targets: vec![],
        }
        .with_compiled_score()
    }
    #[test]
    fn pitch_uses_unsigned_instrument_scale_multiply_add_without_discarding_carry() {
        let mut d = definitions();
        d.pitch_low[7] = 0x80;
        d.pitch_high[7] = 0x12;
        d.instrument_scale[3] = 0x80;
        assert_eq!(calculate_pitch(&d, 7, 3), 0x1bc0);
        d.pitch_low[254] = 0xff;
        d.pitch_high[254] = 0xff;
        d.instrument_scale[33] = 1;
        assert_eq!(calculate_pitch(&d, 254, 33), 0x00fe);
    }
    #[test]
    fn note_initializes_current_envelope_before_ac_target_helper_and_pitch_tick() {
        let mut d = definitions();
        d.instrument_note_base[2] = 3;
        d.pitch_low[10] = 0x80;
        d.pitch_high[10] = 0x12;
        let mut v = Voice {
            instrument: 2,
            transpose: 2,
            target_step: 4,
            target_delay: 1,
            envelope_current: 99,
            ..Voice::default()
        };
        let mut k = KeyMasks::default();
        start_note(&mut v, 5, Some(8), 4, &mut k, &d).unwrap();
        assert_eq!(
            (v.note_origin, v.envelope_target, v.envelope_current),
            (10, 10, 10)
        );
        assert_eq!(v.target_pitch, 0x1280);
        assert_eq!(v.target_countdown, 1);
        assert_eq!(k, KeyMasks { on: 4, off: 4 });
        step_target(&mut v);
        assert_eq!(v.envelope_target, 14);
    }
    #[test]
    fn rest_retains_instrument_envelope_pitch_and_keys_but_consumes_timing() {
        let mut v = Voice {
            instrument: 3,
            envelope_target: 20,
            envelope_current: 19,
            target_pitch: 0x1234,
            skip_transpose: 1,
            ..Voice::default()
        };
        let mut k = KeyMasks { on: 2, off: 4 };
        start_note(&mut v, 0, Some(9), 1, &mut k, &definitions()).unwrap();
        assert_eq!(
            (v.duration, v.target_pitch, v.instrument, v.skip_transpose),
            (9, 0x1234, 3, 1)
        );
        assert_eq!(k, KeyMasks { on: 2, off: 4 });
    }
    #[test]
    fn one_shot_skip_shifts_once_and_note_arithmetic_wraps_byte() {
        let mut d = definitions();
        d.instrument_note_base[0] = 250;
        let mut v = Voice {
            transpose: 10,
            skip_transpose: 1,
            ..Voice::default()
        };
        let mut k = KeyMasks::default();
        start_note(&mut v, 10, Some(3), 1, &mut k, &d).unwrap();
        assert_eq!(v.note_origin, 4);
        assert_eq!(v.skip_transpose, 0);
        start_note(&mut v, 10, Some(3), 1, &mut k, &d).unwrap();
        assert_eq!(v.note_origin, 14);
    }
    #[test]
    fn fixed_duration_omits_operand_but_force_flag_consumes_once() {
        let mut v = Voice {
            fixed_duration: 7,
            ..Voice::default()
        };
        let mut k = KeyMasks::default();
        start_note(&mut v, 1, None, 1, &mut k, &definitions()).unwrap();
        assert_eq!(v.duration, 7);
        v.force_duration = true;
        start_note(&mut v, 1, Some(11), 1, &mut k, &definitions()).unwrap();
        assert_eq!(v.duration, 11);
        assert!(!v.force_duration);
        start_note(&mut v, 1, None, 1, &mut k, &definitions()).unwrap();
        assert_eq!(v.duration, 7);
    }
    #[test]
    fn suppress_key_on_preserves_both_pending_masks_not_only_on() {
        let mut v = Voice {
            suppress_key_on: true,
            ..Voice::default()
        };
        let mut k = KeyMasks { on: 2, off: 8 };
        start_note(&mut v, 1, Some(4), 1, &mut k, &definitions()).unwrap();
        assert_eq!(k, KeyMasks { on: 2, off: 8 });
    }
    #[test]
    fn release_threshold_subtraction_wraps_and_fixed_used_only_when_delta_zero() {
        let mut v = Voice {
            release_delta: 5,
            release_fixed: 9,
            ..Voice::default()
        };
        start_note(
            &mut v,
            0,
            Some(3),
            1,
            &mut KeyMasks::default(),
            &definitions(),
        )
        .unwrap();
        assert_eq!(v.release_threshold, 254);
        v.release_delta = 0;
        start_note(
            &mut v,
            0,
            Some(3),
            1,
            &mut KeyMasks::default(),
            &definitions(),
        )
        .unwrap();
        assert_eq!(v.release_threshold, 9);
    }
    #[test]
    fn target_step_resets_changed_origin_before_adding_and_wraps() {
        let mut v = Voice {
            target_step: 10,
            note_origin: 250,
            envelope_target: 99,
            target_countdown: 1,
            target_delay: 2,
            target_interval: 3,
            ..Voice::default()
        };
        step_target(&mut v);
        assert_eq!((v.envelope_target, v.target_countdown), (250, 2));
        v.target_countdown = 1;
        step_target(&mut v);
        assert_eq!((v.envelope_target, v.target_countdown), (4, 3));
    }
    #[test]
    fn zero_target_countdown_underflows_without_stepping_and_disabled_is_unchanged() {
        let mut v = Voice {
            target_step: 1,
            target_countdown: 0,
            envelope_target: 4,
            ..Voice::default()
        };
        step_target(&mut v);
        assert_eq!((v.target_countdown, v.envelope_target), (255, 4));
        v.target_step = 0;
        let old = v.clone();
        step_target(&mut v);
        assert_eq!(v, old);
    }
    #[test]
    fn zero_slew_delay_snaps_and_recalculates_even_when_interval_nonzero() {
        let mut d = definitions();
        d.pitch_low[7] = 55;
        let mut v = Voice {
            envelope_target: 7,
            envelope_current: 2,
            envelope_delay: 0,
            envelope_interval: 8,
            ..Voice::default()
        };
        slew_envelope(&mut v, &d);
        assert_eq!(v.envelope_current, 7);
        assert_eq!(v.target_pitch, 55);
    }
    #[test]
    fn slew_moves_toward_target_and_clamps_at_crossing() {
        let d = definitions();
        let mut v = Voice {
            envelope_current: 8,
            envelope_target: 10,
            envelope_step: 3,
            envelope_delay: 1,
            envelope_interval: 4,
            ..Voice::default()
        };
        slew_envelope(&mut v, &d);
        assert_eq!((v.envelope_current, v.envelope_delay), (10, 4));
        v.envelope_current = 12;
        v.envelope_delay = 1;
        slew_envelope(&mut v, &d);
        assert_eq!(v.envelope_current, 10);
    }
    #[test]
    fn slew_byte_underflow_is_not_saturating_integer_subtract() {
        let mut v = Voice {
            envelope_current: 2,
            envelope_target: 1,
            envelope_step: 5,
            envelope_delay: 1,
            envelope_interval: 1,
            ..Voice::default()
        };
        slew_envelope(&mut v, &definitions());
        assert_eq!(v.envelope_current, 253);
    }
    #[test]
    fn modulation_delay_returns_on_zero_crossing_and_period_reverses_after_add() {
        let mut v = Voice {
            modulation_delay: 1,
            modulation_direction: 1,
            modulation_step: 3,
            modulation_period: 1,
            modulation_period_reload: 4,
            ..Voice::default()
        };
        update_modulation(&mut v);
        assert_eq!(v.pitch_modulation, 0);
        assert_eq!(v.modulation_delay, 0);
        update_modulation(&mut v);
        assert_eq!(v.pitch_modulation, 3);
        assert_eq!((v.modulation_direction, v.modulation_period), (-1, 4));
        update_modulation(&mut v);
        assert_eq!(v.pitch_modulation, 0);
    }
    #[test]
    fn negative_modulation_zero_step_preserves_low_but_decrements_high() {
        let mut v = Voice {
            modulation_direction: -1,
            modulation_step: 0,
            pitch_modulation: 0x1234,
            modulation_period: 2,
            ..Voice::default()
        };
        update_modulation(&mut v);
        assert_eq!(v.pitch_modulation, 0x1134);
    }
    #[test]
    fn zero_modulation_period_reload_does_not_reverse_direction() {
        let mut v = Voice {
            modulation_direction: 1,
            modulation_step: 2,
            modulation_period: 1,
            modulation_period_reload: 0,
            ..Voice::default()
        };
        update_modulation(&mut v);
        assert_eq!(
            (
                v.pitch_modulation,
                v.modulation_direction,
                v.modulation_period
            ),
            (2, 1, 0)
        );
    }
    #[test]
    fn portamento_zero_snaps_and_nonzero_steps_in_both_directions() {
        let mut v = Voice {
            current_pitch: 1,
            target_pitch: 100,
            ..Voice::default()
        };
        update_portamento(&mut v);
        assert_eq!(v.current_pitch, 100);
        v.current_pitch = 98;
        v.portamento_step = 3;
        update_portamento(&mut v);
        assert_eq!(v.current_pitch, 100);
        v.current_pitch = 102;
        update_portamento(&mut v);
        assert_eq!(v.current_pitch, 100);
    }
    #[test]
    fn portamento_subtract_byte_zero_quirk_not_a_saturating_math_oracle() {
        let mut v = Voice {
            current_pitch: 0x0101,
            target_pitch: 0x00ff,
            portamento_step: 4,
            ..Voice::default()
        };
        update_portamento(&mut v);
        assert_eq!(v.current_pitch, 0x00ff);
    }
    #[test]
    fn final_pitch_keeps_low_add_carry_separate_from_modulation_carry() {
        let mut v = Voice {
            current_pitch: 0x12ff,
            pitch_modulation: 0x0001,
            pitch_low_offset: 0xff,
            ..Voice::default()
        };
        combine_pitch(&mut v);
        assert_eq!(v.dsp_pitch, 0x13ff);
        v.current_pitch = 0x12ff;
        v.pitch_modulation = 0x0002;
        v.pitch_low_offset = 0xff;
        combine_pitch(&mut v);
        assert_eq!(v.dsp_pitch, 0x1400);
    }
    #[test]
    fn auxiliary_80_restarts_and_other_negative_tokens_hold_cursor() {
        let mut v = Voice {
            duration: 1,
            secondary_mode: true,
            auxiliary_delay: 1,
            auxiliary_reload: 2,
            ..Voice::default()
        };
        update_auxiliary(&mut v, &[7, 0x81, 9]).unwrap();
        assert_eq!(
            (v.amplitude, v.auxiliary_cursor, v.auxiliary_delay),
            (7, 1, 2)
        );
        v.auxiliary_delay = 1;
        update_auxiliary(&mut v, &[7, 0x81, 9]).unwrap();
        assert_eq!((v.amplitude, v.auxiliary_cursor), (7, 1));
        v.auxiliary_cursor = 1;
        v.auxiliary_delay = 1;
        update_auxiliary(&mut v, &[4, 0x80]).unwrap();
        assert_eq!((v.amplitude, v.auxiliary_cursor), (4, 1));
    }
    #[test]
    fn auxiliary_zero_delay_does_not_read_and_release_scans_first_negative() {
        let mut v = Voice {
            secondary_mode: true,
            auxiliary_delay: 0,
            duration: 5,
            release_threshold: 5,
            auxiliary_cursor: 0,
            amplitude: 9,
            ..Voice::default()
        };
        update_auxiliary(&mut v, &[1, 2, 0x81, 3]).unwrap();
        assert_eq!((v.amplitude, v.auxiliary_cursor), (9, 3));
    }
    #[test]
    fn rise_interpolates_duration_minus_one_and_advances_at_exact_duration() {
        let row = [1, 10, 3, 30, 3, 6, 3];
        let mut v = Voice::default();
        interpolate_stage(&mut v, row);
        assert_eq!(
            (v.amplitude, v.envelope_tick, v.stage),
            (10, 1, EnvelopeStage::Rise)
        );
        interpolate_stage(&mut v, row);
        assert_eq!(v.amplitude, 20);
        interpolate_stage(&mut v, row);
        assert_eq!(
            (v.amplitude, v.envelope_tick, v.stage),
            (30, 0, EnvelopeStage::Fall)
        );
    }
    #[test]
    fn zero_duration_rise_returns_without_advancing_or_clearing_tick() {
        let mut v = Voice {
            amplitude: 7,
            envelope_tick: 2,
            ..Voice::default()
        };
        let old = v.clone();
        interpolate_stage(&mut v, [1, 10, 0, 30, 3, 6, 3]);
        assert_eq!(v, old);
    }
    #[test]
    fn fall_uses_duration_minus_tick_minus_one_then_finishes_at_last_tick() {
        let mut v = Voice {
            stage: EnvelopeStage::Fall,
            ..Voice::default()
        };
        let row = [1, 10, 3, 30, 3, 6, 3];
        interpolate_stage(&mut v, row);
        assert_eq!((v.amplitude, v.envelope_tick), (22, 1));
        interpolate_stage(&mut v, row);
        assert_eq!(v.amplitude, 14);
        interpolate_stage(&mut v, row);
        assert_eq!(
            (v.amplitude, v.stage, v.envelope_tick),
            (6, EnvelopeStage::Sustain, 0)
        );
    }
    #[test]
    fn release_scales_saved_amplitude_and_done_does_not_reset_tick() {
        let mut v = Voice {
            stage: EnvelopeStage::Release,
            release_amplitude: 30,
            ..Voice::default()
        };
        let row = [1, 0, 0, 0, 0, 0, 3];
        interpolate_stage(&mut v, row);
        assert_eq!((v.amplitude, v.envelope_tick), (20, 1));
        interpolate_stage(&mut v, row);
        assert_eq!(v.amplitude, 10);
        interpolate_stage(&mut v, row);
        assert_eq!(
            (v.amplitude, v.envelope_tick, v.stage),
            (0, 2, EnvelopeStage::Done)
        );
    }
    #[test]
    fn zero_duration_fall_and_release_leave_state_unchanged() {
        for stage in [EnvelopeStage::Fall, EnvelopeStage::Release] {
            let mut v = Voice {
                stage,
                amplitude: 7,
                envelope_tick: 2,
                ..Voice::default()
            };
            let old = v.clone();
            interpolate_stage(&mut v, [1, 10, 0, 30, 0, 6, 0]);
            assert_eq!(v, old);
        }
    }
    #[test]
    fn release_entry_runs_before_timer_dispatch_and_duration_zero_does_not_reenter() {
        let row = [1, 0, 0, 0, 0, 0, 4];
        let mut v = Voice {
            duration: 3,
            release_threshold: 3,
            amplitude: 20,
            stage: EnvelopeStage::Sustain,
            stage_countdown: 1,
            ..Voice::default()
        };
        service_envelope_stage(&mut v, row);
        assert_eq!(
            (v.release_amplitude, v.amplitude, v.stage, v.envelope_tick),
            (20, 15, EnvelopeStage::Release, 1)
        );
        assert_eq!(v.stage_countdown, 1);
        v.duration = 0;
        v.amplitude = 9;
        service_envelope_stage(&mut v, row);
        assert_eq!(v.release_amplitude, 20);
    }
    #[test]
    fn sustain_and_done_dispatch_are_noops_while_stage_timer_reloads() {
        for stage in [EnvelopeStage::Sustain, EnvelopeStage::Done] {
            let mut v = Voice {
                stage,
                duration: 8,
                release_threshold: 3,
                amplitude: 12,
                stage_countdown: 1,
                ..Voice::default()
            };
            service_envelope_stage(&mut v, [4, 0, 0, 0, 0, 0, 0]);
            assert_eq!((v.amplitude, v.stage_countdown, v.stage), (12, 4, stage));
        }
    }
    #[test]
    fn duration_one_rise_uses_documented_divide_zero_result_not_host_divide_panic() {
        let mut v = Voice::default();
        // MUL at tick0 yields YA=0; hardware DIV YA,0 yields quotientFF.
        // Ten plus FF wraps to9, then tick1 matches duration1 and advances.
        interpolate_stage(&mut v, [1, 10, 1, 30, 0, 0, 0]);
        assert_eq!(
            (v.amplitude, v.envelope_tick, v.stage),
            (9, 0, EnvelopeStage::Fall)
        );
    }
}
