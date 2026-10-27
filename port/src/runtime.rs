//! Console-cadence state and reset boundary. Contracts are derived in runtime.trace.md.
//! Native rules preserve source widths without a CPU/RAM emulator.

pub const TRUCK_COUNT: usize = 4;
pub const RNG_STATE_BYTES: usize = 5;
pub const TERRAIN_CORNERS: usize = 6;
pub const FRONTEND_SIGNATURE_BYTES: usize = 5;

/// Source's five retained feedback bytes. Boot does not write these bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RandomState(pub [u8; RNG_STATE_BYTES]);

/// Width describes the caller's result consumption, not an emulated register file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RandomResultWidth {
    Byte,
    Word,
}

impl RandomState {
    /// Replace the source's uninitialized power-on bytes with host entropy.
    /// The generator has an absorbing zero state; the high seven bits of byte4
    /// disappear on the first update, so those bits alone are not a usable seed.
    pub fn from_entropy() -> anyhow::Result<Self> {
        use anyhow::Context;
        loop {
            let mut history = [0; RNG_STATE_BYTES];
            getrandom::getrandom(&mut history).context("cannot seed game random history")?;
            if history[..RNG_STATE_BYTES - 1].iter().any(|&byte| byte != 0)
                || history[RNG_STATE_BYTES - 1] & 1 != 0
            {
                return Ok(Self(history));
            }
        }
    }

    /// Update all feedback bytes; word callers receive the adjacent new history
    /// byte in the high result byte. Caller carry does not enter this transform.
    pub fn next(&mut self, width: RandomResultWidth) -> u16 {
        const NIBBLE_BITS: u32 = 4;
        const FEEDBACK_HIGH_BIT: u32 = 7;
        let [first, second, third, fourth, fifth] = self.0;
        // The first ROR supplies old byte4 bit0, not the caller's carry. The
        // temporary overlapping word stores converge to this same history.
        let feedback = (fourth >> 1) | ((fifth & 1) << FEEDBACK_HIGH_BIT);
        let mixed = ((second >> NIBBLE_BITS) | third.wrapping_shl(NIBBLE_BITS)) ^ feedback;
        self.0 = [mixed, first, second, third, fourth];
        match width {
            RandomResultWidth::Byte => u16::from(mixed),
            RandomResultWidth::Word => u16::from_le_bytes([mixed, first]),
        }
    }
}

/// Explicit source boot writes before frontend defaults/attract processing.
/// Nonzero retained fields distinguish a write from accidentally zeroing all RAM.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BootState {
    pub steering_upgrades: [u8; TRUCK_COUNT],
    pub speed_upgrades: [u8; TRUCK_COUNT],
    pub acceleration_upgrades: [u8; TRUCK_COUNT],
    pub suspension_upgrades: [u8; TRUCK_COUNT],
    pub acceleration_fractions: [u16; TRUCK_COUNT],
    pub terrain_corners: [u16; TERRAIN_CORNERS],
    pub frontend_start_reentry: bool,
    pub active_human_mask: u8,
    pub track_order_slot: u8,
    pub random: RandomState,
    pub frontend_signature: [u8; FRONTEND_SIGNATURE_BYTES],
}

/// Cross-owner effects, in source order. Domain decoding/upload algorithms are
/// separately owned by presentation/audio/attract/tracks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BootEffect {
    ResetVideo,
    DisableInterrupts,
    UploadAudioDefinitions,
    BlankAndInhibitVideo,
    ConfigureRaceLayers,
    LoadFontAndClearBackground,
    InitializeSpriteTransfer,
    FrontendAudioCommand,
    EnterFrontend,
    SelectFirstTrack,
}

pub fn prepare_boot(state: &mut BootState) -> Vec<BootEffect> {
    use BootEffect::*;
    const FIRST_HUMAN: u8 = 1;
    state.steering_upgrades[0] = 0;
    state.speed_upgrades[0] = 0;
    state.acceleration_upgrades[0] = 0;
    state.suspension_upgrades.fill(0);
    state.acceleration_fractions.fill(0);
    state.terrain_corners.fill(0);
    state.frontend_start_reentry = false;
    state.active_human_mask = FIRST_HUMAN;
    state.track_order_slot = 0;
    // Frontend initialization owns its later stock reset. Neither this boot
    // block nor warm frontend entry is a license to overwrite retained history.
    vec![
        ResetVideo,
        DisableInterrupts,
        UploadAudioDefinitions,
        BlankAndInhibitVideo,
        ConfigureRaceLayers,
        LoadFontAndClearBackground,
        InitializeSpriteTransfer,
        FrontendAudioCommand,
        BlankAndInhibitVideo,
        EnterFrontend,
        ResetVideo,
        InitializeSpriteTransfer,
        SelectFirstTrack,
    ]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameCounters {
    pub nmi: u8,
    pub animation: u8,
    pub audio_heartbeat: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameEffect {
    UploadStagedObjects,
    UploadStagedTruckPoses,
    SelectHudTransferDestination,
    CountersAdvanced(FrameCounters),
    SendAudioHeartbeat(u8),
    ControllerPollCompleted,
}

/// One completed NMI boundary; video inhibit suppresses only presentation work.
/// The caller schedules this boundary only when NMI is enabled.
pub fn service_frame_boundary(counters: &mut FrameCounters, video_inhibit: u8) -> Vec<FrameEffect> {
    use FrameEffect::*;
    let mut effects = Vec::new();
    if video_inhibit == 0 {
        effects.extend([
            UploadStagedObjects,
            UploadStagedTruckPoses,
            SelectHudTransferDestination,
        ]);
    }
    counters.nmi = counters.nmi.wrapping_add(1);
    counters.animation = counters.animation.wrapping_add(1);
    counters.audio_heartbeat = counters.audio_heartbeat.wrapping_add(1);
    effects.extend([
        CountersAdvanced(*counters),
        SendAudioHeartbeat(counters.audio_heartbeat),
        ControllerPollCompleted,
    ]);
    effects
}

/// Immutable cartridge facts used to bind startup/persistence configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CartridgeMetadata {
    pub has_battery: bool,
    pub save_ram_bytes: usize,
    pub reset_entry: u16,
    pub native_nmi_entry: u16,
    pub ntsc: bool,
}

/// Decode the 64-byte LoROM metadata block, not CPU address aliases.
pub fn decode_cartridge_metadata(header: &[u8]) -> anyhow::Result<CartridgeMetadata> {
    const HEADER_BYTES: usize = 64;
    const CARTRIDGE_TYPE: usize = 0x16;
    const SRAM_EXPONENT: usize = 0x18;
    const REGION: usize = 0x19;
    const NATIVE_NMI_VECTOR: usize = 0x2a;
    const RESET_VECTOR: usize = 0x3c;
    anyhow::ensure!(header.len() == HEADER_BYTES, "invalid LoROM header length");
    let exponent = header[SRAM_EXPONENT];
    let save_ram_bytes = if exponent == 0 {
        0
    } else {
        1024usize
            .checked_shl(u32::from(exponent))
            .ok_or_else(|| anyhow::anyhow!("invalid SRAM exponent"))?
    };
    let word = |offset| u16::from_le_bytes([header[offset], header[offset + 1]]);
    Ok(CartridgeMetadata {
        has_battery: matches!(
            header[CARTRIDGE_TYPE],
            2 | 5 | 6 | 9 | 10 | 13 | 15 | 0x32 | 0x35
        ),
        save_ram_bytes,
        reset_entry: word(RESET_VECTOR),
        native_nmi_entry: word(NATIVE_NMI_VECTOR),
        ntsc: matches!(header[REGION], 0 | 1 | 13),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config, game::Port};
    use romatlas_port::{
        Game,
        assets::{MemoryAssets, Rom},
        input::InputState,
    };
    use std::path::Path;

    // Literal vectors are independently derived from the bit trace, not from an
    // implementation oracle. The fifth byte contributes only bit zero.
    #[test]
    fn rng_shifts_history_and_combines_nibbles_with_feedback_bit() {
        let mut random = RandomState([0x12, 0x34, 0x56, 0x78, 0x9b]);
        assert_eq!(random.next(RandomResultWidth::Byte), 0xdf);
        assert_eq!(random.0, [0xdf, 0x12, 0x34, 0x56, 0x78]);
        assert_eq!(random.next(RandomResultWidth::Byte), 0x6a);
        assert_eq!(random.0, [0x6a, 0xdf, 0x12, 0x34, 0x56]);
    }

    #[test]
    fn rng_word_result_contains_previous_first_byte_without_changing_update() {
        let mut word = RandomState([0xa5, 0xf0, 0x0f, 0x81, 0x00]);
        let mut byte = word;
        assert_eq!(word.next(RandomResultWidth::Word), 0xa5bf);
        assert_eq!(byte.next(RandomResultWidth::Byte), 0xbf);
        assert_eq!(word.0, [0xbf, 0xa5, 0xf0, 0x0f, 0x81]);
        assert_eq!(word, byte);
    }

    #[test]
    fn rng_feedback_uses_old_fifth_low_bit_and_drops_high_nibble_overflow() {
        let mut even = RandomState([0x01, 0xff, 0xff, 0xff, 0xfe]);
        let mut odd = RandomState([0x01, 0xff, 0xff, 0xff, 0xff]);
        assert_eq!(even.next(RandomResultWidth::Byte), 0x80);
        assert_eq!(odd.next(RandomResultWidth::Byte), 0x00);
        assert_eq!(even.0, [0x80, 0x01, 0xff, 0xff, 0xff]);
        assert_eq!(odd.0, [0x00, 0x01, 0xff, 0xff, 0xff]);
    }

    #[test]
    fn rng_zero_state_is_not_silently_reseeded() {
        let mut zero = RandomState([0; RNG_STATE_BYTES]);
        assert_eq!(zero.next(RandomResultWidth::Word), 0);
        assert_eq!(zero.0, [0; RNG_STATE_BYTES]);
    }

    fn retained_boot_state() -> BootState {
        BootState {
            steering_upgrades: [1, 2, 3, 4],
            speed_upgrades: [2, 3, 4, 5],
            acceleration_upgrades: [3, 4, 5, 6],
            suspension_upgrades: [4, 5, 6, 7],
            acceleration_fractions: [0xffff; TRUCK_COUNT],
            terrain_corners: [0xffff; TERRAIN_CORNERS],
            frontend_start_reentry: true,
            active_human_mask: 0xff,
            track_order_slot: 0xff,
            random: RandomState([1, 2, 3, 4, 5]),
            frontend_signature: [0x12, 0x0f, 0x16, 0x16, 0x19],
        }
    }

    #[test]
    fn explicit_boot_writes_preserve_unwritten_upgrade_slots_and_retained_state() {
        let mut state = retained_boot_state();
        let retained_random = state.random;
        let retained_signature = state.frontend_signature;
        prepare_boot(&mut state);
        // SEP #$30 means the three first STZ writes clear only slot zero.
        // Suspension has four separate stores; the later frontend owns clearing
        // all stock/upgrades. Do not move that side effect into this boundary.
        assert_eq!(state.steering_upgrades, [0, 2, 3, 4]);
        assert_eq!(state.speed_upgrades, [0, 3, 4, 5]);
        assert_eq!(state.acceleration_upgrades, [0, 4, 5, 6]);
        assert_eq!(state.suspension_upgrades, [0; TRUCK_COUNT]);
        assert_eq!(state.acceleration_fractions, [0; TRUCK_COUNT]);
        assert_eq!(state.terrain_corners, [0; TERRAIN_CORNERS]);
        assert!(!state.frontend_start_reentry);
        assert_eq!(state.active_human_mask, 1);
        assert_eq!(state.track_order_slot, 0);
        assert_eq!(state.random, retained_random);
        assert_eq!(state.frontend_signature, retained_signature);
    }

    #[test]
    fn boot_cross_owner_effects_retain_source_call_order() {
        use BootEffect::*;
        assert_eq!(
            prepare_boot(&mut retained_boot_state()),
            vec![
                ResetVideo,
                DisableInterrupts,
                UploadAudioDefinitions,
                BlankAndInhibitVideo,
                ConfigureRaceLayers,
                LoadFontAndClearBackground,
                InitializeSpriteTransfer,
                FrontendAudioCommand,
                BlankAndInhibitVideo,
                EnterFrontend,
                ResetVideo,
                InitializeSpriteTransfer,
                SelectFirstTrack,
            ]
        );
    }

    #[test]
    fn visible_nmi_uploads_staged_content_before_counters_and_audio() {
        use FrameEffect::*;
        let mut counters = FrameCounters {
            nmi: 9,
            animation: 19,
            audio_heartbeat: 29,
        };
        let next = FrameCounters {
            nmi: 10,
            animation: 20,
            audio_heartbeat: 30,
        };
        assert_eq!(
            service_frame_boundary(&mut counters, 0),
            vec![
                UploadStagedObjects,
                UploadStagedTruckPoses,
                SelectHudTransferDestination,
                CountersAdvanced(next),
                SendAudioHeartbeat(30),
                ControllerPollCompleted,
            ]
        );
        assert_eq!(counters, next);
    }

    #[test]
    fn every_nonzero_video_inhibit_skips_uploads_but_not_heartbeat_or_poll_completion() {
        use FrameEffect::*;
        for inhibit in [1, 0x7f, 0x80, 0xff] {
            let mut counters = FrameCounters {
                nmi: 0xff,
                animation: 0xfe,
                audio_heartbeat: 0xff,
            };
            let next = FrameCounters {
                nmi: 0,
                animation: 0xff,
                audio_heartbeat: 0,
            };
            assert_eq!(
                service_frame_boundary(&mut counters, inhibit),
                vec![
                    CountersAdvanced(next),
                    SendAudioHeartbeat(0),
                    ControllerPollCompleted,
                ]
            );
            assert_eq!(counters, next);
        }
    }

    fn port() -> Port {
        let rom = Rom::open(Path::new("../input/game.rom"), config::ROM_SHA256).unwrap();
        Port::load_assets(&rom, &MemoryAssets::default()).unwrap()
    }

    #[test]
    fn shared_game_exposes_runtime_counters_and_reset_reestablishes_determinism() {
        let mut game = port();
        let initial = game.observe();
        assert!(
            initial["runtime"]["nmi_frames"].is_u64(),
            "runtime counter observation is required"
        );
        game.tick(&InputState::default());
        let first = game.observe();
        assert_ne!(initial["runtime"], first["runtime"]);
        game.reset().unwrap();
        assert_eq!(game.observe(), initial);
        game.tick(&InputState::default());
        assert_eq!(game.observe(), first);
    }

    #[test]
    fn runtime_snapshot_replays_rng_counters_and_pending_transition_atomically() {
        let mut game = port();
        game.tick(&InputState::default());
        let state = game.observe();
        assert!(
            state["runtime"]["random_history"].is_array(),
            "semantic RNG history required for deterministic branch verification"
        );
        let saved = game.snapshot_json().unwrap();
        let mut branch = game.clone();
        game.tick(&InputState::default());
        branch.tick(&InputState::default());
        assert_eq!(game.observe(), branch.observe());
        game.restore_snapshot_json(&saved).unwrap();
        assert_eq!(game.observe(), state);
        let mut invalid: serde_json::Value = serde_json::from_str(&saved).unwrap();
        invalid["rom_sha256"] = serde_json::json!("wrong ROM");
        assert!(game.restore_snapshot_json(&invalid.to_string()).is_err());
        assert_eq!(game.observe(), state);
        game.tick(&InputState::default());
        assert_eq!(game.observe(), branch.observe());
    }

    #[test]
    fn matching_rom_header_establishes_reset_nmi_ntsc_and_session_only_records() {
        const LOROM_HEADER_START: usize = 0x7fc0;
        const LOROM_HEADER_END: usize = 0x8000;
        let rom = std::fs::read("../input/game.rom").unwrap();
        let facts = decode_cartridge_metadata(&rom[LOROM_HEADER_START..LOROM_HEADER_END]).unwrap();
        assert_eq!(
            facts,
            CartridgeMetadata {
                has_battery: false,
                save_ram_bytes: 0,
                reset_entry: 0x8000,
                native_nmi_entry: 0x9bca,
                ntsc: true,
            }
        );
    }

    #[test]
    fn metadata_decoder_rejects_truncation_before_reading_vectors() {
        const HEADER_BYTES: usize = 64;
        for bytes in [0, 1, HEADER_BYTES - 1] {
            assert!(decode_cartridge_metadata(&vec![0; bytes]).is_err());
        }
    }

    #[test]
    fn no_battery_hooks_are_added_for_session_only_records() {
        assert_eq!(port().save_version(), None);
    }
}
