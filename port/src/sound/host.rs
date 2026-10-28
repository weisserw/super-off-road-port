//! Host command protocol and native scheduling boundary. Port input/output latches are distinct.
use super::sequencer;
use super::{definitions::AudioDefinitions, sequencer::Driver};
use anyhow::{Context, Result};
pub const TIMER_DIVISOR: u16 = 66;
pub const TIMER_CLOCK: u32 = 8000;
pub const DSP_CLOCK: u32 = 32000;
pub const TIMER_SAMPLE_INTERVAL: u32 = 264;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostInput {
    pub heartbeat: u8,
    pub flags: u8,
    pub command: u8,
    pub argument: u8,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostState {
    pub last_heartbeat: u8,
    pub acknowledged: u64,
    pub reload_requested: bool,
    pub second_update_samples: Option<u32>,
    pub status_output: [u8; 4],
}
pub fn race_sound(human_mask: u16, command: u16) -> Option<(u8, u8)> {
    if human_mask as u8 == 0 {
        None
    } else {
        let [command, argument] = command.to_le_bytes();
        Some((command, argument))
    }
}
pub fn select_effect(
    driver: &mut Driver,
    effect: u8,
    definitions: &AudioDefinitions,
) -> Result<Option<usize>> {
    let effect = definitions
        .effects
        .get(usize::from(effect))
        .context("invalid effect selector")?;
    let chosen = if !driver.voices[6].active {
        Some(6)
    } else if !driver.voices[7].active {
        Some(7)
    } else if effect.priority >= driver.effect_priorities[0] {
        Some(6)
    } else if effect.priority >= driver.effect_priorities[1] {
        Some(7)
    } else {
        None
    };
    if let Some(slot) = chosen {
        let v = &mut driver.voices[slot];
        v.target_step = 0;
        v.envelope_interval = 0;
        v.suppress_key_on = false;
        v.modulation_direction_reload = 0;
        v.portamento_step = 0;
        v.secondary_mode = false;
        v.transpose = 0;
        v.pitch_low_offset = 0;
        v.pitch_modulation = 0;
        v.fixed_duration = 0;
        v.release_threshold = 0;
        v.release_delta = 0;
        v.release_fixed = 0;
        v.duration = 1;
        v.active = true;
        driver.noise_mask &= !(1 << slot);
        driver.effect_priorities[slot - 6] = effect.priority;
        driver.sequences[slot] = sequencer::Sequence {
            cursor: definitions.score.resolve(effect.sequence),
            ..Default::default()
        };
    }
    Ok(chosen)
}
pub fn service_host(
    host: &mut HostState,
    driver: &mut Driver,
    input: HostInput,
    definitions: &AudioDefinitions,
) -> Result<()> {
    if input.flags & 8 != 0 {
        run_update(driver, input.flags, definitions)?;
        return Ok(());
    }
    if input.flags == 0x55 {
        host.reload_requested = true;
        driver.dsp_flags = 0xe0;
        driver.pending_dsp.push((0x6c, 0xe0));
        return Ok(());
    }
    match input.command {
        1 => {
            sequencer::load_song(driver, input.argument.wrapping_sub(1), definitions)?;
            driver.master = 0x7f00;
            driver.dsp_flags = 0x33;
            driver.pending_dsp.push((0x6c, 0x33));
            host.acknowledged = host.acknowledged.wrapping_add(1);
            host.last_heartbeat = input.heartbeat;
        }
        2 => {
            select_effect(driver, input.argument, definitions)?;
            host.acknowledged = host.acknowledged.wrapping_add(1);
        }
        3 => {
            driver.fade_step = i16::from(input.argument as i8).wrapping_mul(8);
            host.acknowledged = host.acknowledged.wrapping_add(1);
        }
        _ => {}
    }
    if input.heartbeat != host.last_heartbeat {
        host.last_heartbeat = input.heartbeat;
        run_update(driver, input.flags, definitions)?;
        host.second_update_samples = Some(TIMER_SAMPLE_INTERVAL);
        host.status_output[3] = 0x0f;
    }
    Ok(())
}
pub fn integrate_fade(driver: &mut Driver) {
    if driver.fade_step == 0 {
        return;
    }
    let result = driver.master.wrapping_add(driver.fade_step as u16);
    let clamp = if driver.fade_step > 0 {
        (result as i16) >= 0
    } else {
        (result as i16) < 0
    };
    driver.master = if clamp {
        (result & 0xff) | if driver.fade_step > 0 { 0x7f00 } else { 0 }
    } else {
        result
    };
    if clamp {
        driver.fade_step = 0;
    }
}
pub fn service_sample_clock(
    host: &mut HostState,
    driver: &mut Driver,
    samples: u32,
    flags: u8,
    definitions: &AudioDefinitions,
) -> Result<()> {
    if let Some(remaining) = host.second_update_samples {
        if samples >= remaining {
            host.second_update_samples = None;
            run_update(driver, flags, definitions)?;
            host.status_output[3] = 0x0f;
        } else {
            host.second_update_samples = Some(remaining - samples);
        }
    }
    Ok(())
}

fn run_update(driver: &mut Driver, flags: u8, definitions: &AudioDefinitions) -> Result<()> {
    let writes = sequencer::update_driver(driver, flags, definitions)?;
    driver.pending_dsp.extend(writes);
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::super::{definitions::*, voice::Voice};
    use super::*;
    fn defs() -> AudioDefinitions {
        AudioDefinitions {
            score: Default::default(),
            data: SequenceData {
                segments: vec![DataSegment {
                    origin: DataAddress(0x400),
                    bytes: vec![0x80],
                }],
            },
            songs: vec![],
            effects: vec![
                EffectDefinition {
                    sequence: DataAddress(0x400),
                    priority: 3
                };
                16
            ],
            samples: vec![],
            pitch_low: [0; 256],
            pitch_high: [0; 256],
            instrument_note_base: [0; 256],
            instrument_scale: [0; 256],
            initial_envelopes: vec![[0; 7]; 28],
            dsp_initialization: vec![],
            command_targets: vec![],
        }
        .with_compiled_score()
    }
    #[test]
    fn human_sound_gate_tests_only_low_mask_byte_and_keeps_command_bytes() {
        assert_eq!(race_sound(0, 0x0502), None);
        assert_eq!(race_sound(0x0100, 0x0502), None);
        assert_eq!(race_sound(0x0080, 0x0502), Some((2, 5)));
    }
    #[test]
    fn free_first_reserved_voice_wins_even_if_other_voice_has_lower_priority() {
        let mut d = Driver::default();
        d.voices[7].active = true;
        d.effect_priorities = [9, 0];
        assert_eq!(select_effect(&mut d, 1, &defs()).unwrap(), Some(6));
        assert_eq!(d.effect_priorities, [3, 0]);
        assert!(d.voices[6].active);
    }
    #[test]
    fn busy_first_uses_free_second_before_priority_comparisons() {
        let mut d = Driver::default();
        d.voices[6].active = true;
        d.effect_priorities = [1, 9];
        assert_eq!(select_effect(&mut d, 1, &defs()).unwrap(), Some(7));
        assert_eq!(d.effect_priorities, [1, 3]);
    }
    #[test]
    fn busy_voices_compare_first_priority_then_second_with_equality_accepted() {
        for (priorities, slot) in [([3, 1], Some(6)), ([4, 3], Some(7)), ([4, 4], None)] {
            let mut d = Driver::default();
            d.voices[6].active = true;
            d.voices[7].active = true;
            d.effect_priorities = priorities;
            assert_eq!(select_effect(&mut d, 1, &defs()).unwrap(), slot);
        }
    }
    #[test]
    fn partial_effect_reset_keeps_instrument_pitch_volume_and_clears_only_documented_controls() {
        let mut d = Driver {
            noise_mask: 0xff,
            ..Default::default()
        };
        d.voices[6] = Voice {
            instrument: 9,
            current_pitch: 0x1234,
            target_pitch: 0x5678,
            volume: [3, 4],
            target_step: 7,
            envelope_interval: 5,
            suppress_key_on: true,
            modulation_direction_reload: -1,
            portamento_step: 4,
            secondary_mode: true,
            transpose: 8,
            pitch_low_offset: 9,
            pitch_modulation: 0x9999,
            fixed_duration: 3,
            release_delta: 4,
            release_fixed: 5,
            ..Voice::default()
        };
        select_effect(&mut d, 1, &defs()).unwrap();
        let v = &d.voices[6];
        assert_eq!(
            (v.instrument, v.current_pitch, v.target_pitch, v.volume),
            (9, 0x1234, 0x5678, [3, 4])
        );
        assert_eq!(
            (
                v.target_step,
                v.envelope_interval,
                v.modulation_direction_reload,
                v.portamento_step,
                v.transpose,
                v.pitch_low_offset,
                v.pitch_modulation,
                v.fixed_duration,
                v.release_delta,
                v.release_fixed
            ),
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 0)
        );
        assert!(!v.secondary_mode);
        assert!(!v.suppress_key_on);
        assert_eq!(
            (v.duration, v.release_threshold, d.noise_mask),
            (1, 0, 0xbf)
        );
        assert_eq!(d.sequences[6].cursor.source(), DataAddress(0x400));
    }
    #[test]
    fn second_reserved_effect_clears_only_noise_bit_seven() {
        let mut d = Driver::default();
        d.voices[6].active = true;
        d.noise_mask = 0xff;
        select_effect(&mut d, 1, &defs()).unwrap();
        assert_eq!(d.noise_mask, 0x7f);
    }
    #[test]
    fn rejected_effect_preserves_both_live_voice_records_and_stacks() {
        let mut d = Driver::default();
        d.voices[6].active = true;
        d.voices[7].active = true;
        d.effect_priorities = [4, 4];
        let old = d.clone();
        assert_eq!(select_effect(&mut d, 1, &defs()).unwrap(), None);
        assert_eq!(d, old);
    }
    #[test]
    fn host_effect_acknowledges_even_when_priority_rejects() {
        let mut d = Driver::default();
        d.voices[6].active = true;
        d.voices[7].active = true;
        d.effect_priorities = [4, 4];
        let mut h = HostState::default();
        service_host(
            &mut h,
            &mut d,
            HostInput {
                command: 2,
                argument: 1,
                ..HostInput::default()
            },
            &defs(),
        )
        .unwrap();
        assert_eq!(h.acknowledged, 1);
        assert_eq!(d.effect_priorities, [4, 4]);
    }
    #[test]
    fn fade_command_sign_extends_and_multiplies_argument_by_eight() {
        for (arg, step) in [(3, 24), (0xff, -8), (0x80, -1024)] {
            let mut d = Driver::default();
            let mut h = HostState::default();
            service_host(
                &mut h,
                &mut d,
                HostInput {
                    command: 3,
                    argument: arg,
                    ..HostInput::default()
                },
                &defs(),
            )
            .unwrap();
            assert_eq!(d.fade_step, step);
            assert_eq!(h.acknowledged, 1);
        }
    }
    #[test]
    fn positive_fade_literal_bmi_branch_clamps_nonnegative_sum_immediately() {
        let mut d = Driver {
            master: 0x1000,
            fade_step: 8,
            ..Driver::default()
        };
        integrate_fade(&mut d);
        assert_eq!((d.master, d.fade_step), (0x7f08, 0));
        let mut d = Driver {
            master: 0x7ff8,
            fade_step: 8,
            ..Driver::default()
        };
        integrate_fade(&mut d);
        assert_eq!((d.master, d.fade_step), (0x8000, 8));
    }
    #[test]
    fn negative_fade_clamp_preserves_low_fraction_not_zeroing_word() {
        let mut d = Driver {
            master: 0,
            fade_step: -8,
            ..Driver::default()
        };
        integrate_fade(&mut d);
        assert_eq!((d.master, d.fade_step), (0x00f8, 0));
        let mut d = Driver {
            master: 0x1000,
            fade_step: -8,
            ..Driver::default()
        };
        integrate_fade(&mut d);
        assert_eq!((d.master, d.fade_step), (0x0ff8, -8));
    }
    #[test]
    fn host_fast_flag_runs_update_without_accepting_pending_command_or_heartbeat() {
        let mut d = Driver::default();
        let mut h = HostState::default();
        service_host(
            &mut h,
            &mut d,
            HostInput {
                flags: 8,
                command: 3,
                argument: 1,
                heartbeat: 7,
            },
            &defs(),
        )
        .unwrap();
        assert_eq!(d.updates, 1);
        assert_eq!(d.fade_step, 0);
        assert_eq!((h.acknowledged, h.last_heartbeat), (0, 0));
    }
    #[test]
    fn reload_flag_mutes_and_requests_ipl_without_accepting_command() {
        let mut d = Driver::default();
        let mut h = HostState::default();
        service_host(
            &mut h,
            &mut d,
            HostInput {
                flags: 0x55,
                command: 3,
                argument: 1,
                ..HostInput::default()
            },
            &defs(),
        )
        .unwrap();
        assert!(h.reload_requested);
        assert_eq!(d.dsp_flags, 0xe0);
        assert_eq!(h.acknowledged, 0);
    }
    #[test]
    fn heartbeat_change_does_immediate_update_then_second_at66_timer_ticks() {
        let mut d = Driver::default();
        let mut h = HostState::default();
        service_host(
            &mut h,
            &mut d,
            HostInput {
                heartbeat: 1,
                ..HostInput::default()
            },
            &defs(),
        )
        .unwrap();
        assert_eq!(d.updates, 1);
        assert_eq!(h.second_update_samples, Some(TIMER_SAMPLE_INTERVAL));
        service_sample_clock(&mut h, &mut d, TIMER_SAMPLE_INTERVAL - 1, 0, &defs()).unwrap();
        assert_eq!(d.updates, 1);
        service_sample_clock(&mut h, &mut d, 1, 0, &defs()).unwrap();
        assert_eq!(d.updates, 2);
        assert_eq!(h.second_update_samples, None);
    }
    #[test]
    fn unchanged_heartbeat_and_unknown_command_do_not_ack_or_update() {
        let mut d = Driver::default();
        let mut h = HostState {
            last_heartbeat: 5,
            ..HostState::default()
        };
        service_host(
            &mut h,
            &mut d,
            HostInput {
                heartbeat: 5,
                command: 7,
                argument: 9,
                ..HostInput::default()
            },
            &defs(),
        )
        .unwrap();
        assert_eq!((d.updates, h.acknowledged), (0, 0));
    }
    #[test]
    fn host_music_selector_decrements_before_song_load_then_master_reinitializes() {
        let r = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap();
        let a = load_audio(&r).unwrap();
        let mut d = Driver::default();
        let mut h = HostState::default();
        service_host(
            &mut h,
            &mut d,
            HostInput {
                command: 1,
                argument: 6,
                ..HostInput::default()
            },
            &a,
        )
        .unwrap();
        assert_eq!(d.requested_song, Some(5));
        assert_eq!((d.master, d.dsp_flags), (0x7f00, 0x33));
        assert_eq!(h.acknowledged, 1);
        assert_eq!(d.sequences[0].cursor.source(), a.songs[5].voices[0]);
    }
}
