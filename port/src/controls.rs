//! Controller sampling and ROM direction lookup contracts; see controls.trace.md.
//! Action translation is shared by desktop/browser. Source repeat/edge state is
//! owned by names, shop, pause and truck motion, not by this sampling boundary.
use anyhow::Result;
use romatlas_port::{assets::Rom, input::MultiInputState};

pub const CONTROLLER_COUNT: usize = crate::runtime::TRUCK_COUNT;
pub const FRONTEND_PLAYERS: usize = 2;
pub const DIRECTION_COUNT: usize = 16;
pub const DIRECTION_TABLE_BYTES: usize = DIRECTION_COUNT * size_of::<i16>();
pub const HORIZONTAL_TABLE_ROM: usize = 0x1438;
pub const VERTICAL_TABLE_ROM: usize = 0x1458;
pub const BUTTON_B: u16 = 0x8000;
pub const BUTTON_Y: u16 = 0x4000;
pub const BUTTON_SELECT: u16 = 0x2000;
pub const BUTTON_START: u16 = 0x1000;
pub const BUTTON_UP: u16 = 0x0800;
pub const BUTTON_DOWN: u16 = 0x0400;
pub const BUTTON_LEFT: u16 = 0x0200;
pub const BUTTON_RIGHT: u16 = 0x0100;
pub const BUTTON_A: u16 = 0x0080;
pub const BUTTON_X: u16 = 0x0040;
pub const BUTTON_L: u16 = 0x0020;
pub const BUTTON_R: u16 = 0x0010;

/// Encoded SNES automatic-controller word at the input/ROM boundary.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControllerWord(pub u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuDelta {
    pub columns: i8,
    pub rows: i8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WordDelta {
    pub first: i16,
    pub second: i16,
}

/// Both consumers use the same sixteen signed words per axis. Natural frontend
/// calls read only the low byte; conditional word-helper callers read full words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectionTables {
    pub first: [i16; DIRECTION_COUNT],
    pub second: [i16; DIRECTION_COUNT],
}
impl DirectionTables {
    pub fn load(rom: &Rom) -> Result<Self> {
        Self::from_table_bytes(
            rom.read(HORIZONTAL_TABLE_ROM, DIRECTION_TABLE_BYTES)?,
            rom.read(VERTICAL_TABLE_ROM, DIRECTION_TABLE_BYTES)?,
        )
    }
    /// Separate slices make both table bounds explicit; no silent partial words.
    pub fn from_table_bytes(first: &[u8], second: &[u8]) -> Result<Self> {
        anyhow::ensure!(
            first.len() == DIRECTION_TABLE_BYTES && second.len() == DIRECTION_TABLE_BYTES,
            "direction table must contain sixteen signed words"
        );
        let decode = |bytes: &[u8]| {
            std::array::from_fn(|i| i16::from_le_bytes([bytes[i * 2], bytes[i * 2 + 1]]))
        };
        Ok(Self {
            first: decode(first),
            second: decode(second),
        })
    }
    /// Natural M8 helper: ignore non-direction high-byte buttons, then consume
    /// the selected word's low byte as a signed cursor step. No axis cancellation.
    pub fn frontend_delta(&self, input_high: u8) -> MenuDelta {
        const DIRECTION_MASK: u8 = 0x0f;
        let index = usize::from(input_high & DIRECTION_MASK);
        MenuDelta {
            columns: self.first[index] as i8,
            rows: self.second[index] as i8,
        }
    }
    /// Conditional native DB00/DP0000 helper: selector is the controller word's
    /// upper-byte low nibble. Reachability is not inferred from this interface.
    pub fn word_delta(&self, controller: ControllerWord) -> WordDelta {
        const DIRECTION_MASK: u16 = 0x0f;
        let index = usize::from((controller.0 >> u8::BITS) & DIRECTION_MASK);
        WordDelta {
            first: self.first[index],
            second: self.second[index],
        }
    }
}

/// Frontend local player pair. Source stores high and low bytes in split arrays;
/// native code keeps each sampled word whole rather than exposing a RAM layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrontendInputs {
    pub players: [ControllerWord; FRONTEND_PLAYERS],
}

/// Every page value selects the same source JOY1/JOY2 pair. Page changes the
/// target participant records elsewhere, not the devices sampled here.
pub fn sample_frontend(_page: u8, automatic: [ControllerWord; CONTROLLER_COUNT]) -> FrontendInputs {
    FrontendInputs {
        players: [automatic[0], automatic[1]],
    }
}

/// Low four enabled bits replace only their own slots. All disabled words retain
/// previous AI/inactive input; automatic words are not masked down to directions.
pub fn copy_human_words(
    retained: &mut [ControllerWord; CONTROLLER_COUNT],
    active_humans: u8,
    automatic: [ControllerWord; CONTROLLER_COUNT],
) {
    for (slot, word) in retained.iter_mut().enumerate() {
        if active_humans & (1 << slot) != 0 {
            *word = automatic[slot];
        }
    }
}

/// Explicit scheduler boundary consumed by race: movement uses retained inputs;
/// checkpoint processing sees the new human copies. AI writes happen later and
/// remain owned by AI/race; this interface does not run motion or checkpoints.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RaceInputPhases {
    pub movement_words: [ControllerWord; CONTROLLER_COUNT],
    pub checkpoint_words: [ControllerWord; CONTROLLER_COUNT],
}
pub fn sample_race_frame(
    retained: &mut [ControllerWord; CONTROLLER_COUNT],
    active_humans: u8,
    automatic: [ControllerWord; CONTROLLER_COUNT],
) -> RaceInputPhases {
    let movement_words = *retained;
    copy_human_words(retained, active_humans, automatic);
    RaceInputPhases {
        movement_words,
        checkpoint_words: *retained,
    }
}

/// Translate held actions, including opposite directions, without synthesizing
/// press/release history. Source owners calculate their own edge transitions.
pub fn encode_players(input: &MultiInputState) -> [ControllerWord; CONTROLLER_COUNT] {
    const ACTIONS: [(&str, u16); 12] = [
        ("b", BUTTON_B),
        ("y", BUTTON_Y),
        ("select", BUTTON_SELECT),
        ("start", BUTTON_START),
        ("up", BUTTON_UP),
        ("down", BUTTON_DOWN),
        ("left", BUTTON_LEFT),
        ("right", BUTTON_RIGHT),
        ("a", BUTTON_A),
        ("x", BUTTON_X),
        ("l", BUTTON_L),
        ("r", BUTTON_R),
    ];
    std::array::from_fn(|slot| {
        ControllerWord(
            ACTIONS
                .iter()
                .filter(|(name, _)| input.players[slot].held(name))
                .fold(0, |word, (_, bit)| word | bit),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use romatlas_port::input::PlayerInput;
    use std::path::Path;

    fn rom() -> Rom {
        Rom::open(Path::new("../input/game.rom"), crate::config::ROM_SHA256).unwrap()
    }
    fn source_tables() -> DirectionTables {
        DirectionTables::load(&rom()).unwrap()
    }
    // Independently inspected signed outcomes at physical1438/1458. These are
    // domain expectations, not copied ROM assets or a second decoder algorithm.
    const FIRST_RESULTS: [i16; DIRECTION_COUNT] =
        [0, 1, -1, 0, 0, 1, -1, 0, 0, 1, -1, 0, 0, 0, 0, 0];
    const SECOND_RESULTS: [i16; DIRECTION_COUNT] =
        [0, 0, 0, 0, 1, 1, 1, 0, -1, -1, -1, 0, 0, 0, 0, 0];

    #[test]
    fn original_rom_loads_all_low_and_high_direction_bytes_as_signed_words() {
        let tables = source_tables();
        assert_eq!(tables.first, FIRST_RESULTS);
        assert_eq!(tables.second, SECOND_RESULTS);
    }

    #[test]
    fn table_parser_uses_little_endian_signed_words_and_independent_axes() {
        // Synthetic non-unit and non-sign-extended low bytes distinguish signed
        // word parsing from merely interpreting the byte helper's outcomes.
        let mut first = [0; DIRECTION_TABLE_BYTES];
        let mut second = [0; DIRECTION_TABLE_BYTES];
        first[..4].copy_from_slice(&[0x34, 0x12, 0xfe, 0xff]);
        second[..4].copy_from_slice(&[0x00, 0x80, 0xff, 0x00]);
        let tables = DirectionTables::from_table_bytes(&first, &second).unwrap();
        assert_eq!(&tables.first[..2], &[0x1234, -2]);
        assert_eq!(&tables.second[..2], &[i16::MIN, 255]);
        assert_eq!(tables.first[DIRECTION_COUNT - 1], 0);
    }

    #[test]
    fn table_validation_rejects_each_truncation_and_trailing_bytes() {
        let valid = [0; DIRECTION_TABLE_BYTES];
        let short = &valid[..DIRECTION_TABLE_BYTES - 1];
        let long = [0; DIRECTION_TABLE_BYTES + 1];
        for (first, second) in [
            (short, valid.as_slice()),
            (valid.as_slice(), short),
            (&long, valid.as_slice()),
            (valid.as_slice(), &long),
            (&[][..], &[][..]),
        ] {
            assert!(DirectionTables::from_table_bytes(first, second).is_err());
        }
    }

    #[test]
    fn all_sixteen_frontend_nibbles_match_source_including_conflicting_axes() {
        let tables = source_tables();
        for nibble in 0..DIRECTION_COUNT {
            assert_eq!(
                tables.frontend_delta(nibble as u8),
                MenuDelta {
                    columns: FIRST_RESULTS[nibble] as i8,
                    rows: SECOND_RESULTS[nibble] as i8,
                },
                "direction nibble {nibble:x}"
            );
        }
    }

    #[test]
    fn frontend_buttons_outside_nibble_do_not_change_direction() {
        let tables = source_tables();
        // B/Y/Select/Start in this byte must not become part of table index.
        for high_buttons in [0x10, 0x20, 0x40, 0x80, 0xf0] {
            assert_eq!(
                tables.frontend_delta(high_buttons | 0x09),
                MenuDelta {
                    columns: 1,
                    rows: -1
                }
            );
        }
    }

    #[test]
    fn frontend_reads_only_low_byte_while_word_helper_keeps_signed_word() {
        let mut tables = DirectionTables {
            first: [0; DIRECTION_COUNT],
            second: [0; DIRECTION_COUNT],
        };
        const SELECTOR: usize = 5;
        tables.first[SELECTOR] = 0x1281;
        tables.second[SELECTOR] = -0x017f; // FE81, low byte81 on both axes.
        assert_eq!(
            tables.frontend_delta(SELECTOR as u8),
            MenuDelta {
                columns: -127,
                rows: -127
            }
        );
        assert_eq!(
            tables.word_delta(ControllerWord(0x05ff)),
            WordDelta {
                first: 0x1281,
                second: -0x017f
            }
        );
    }

    #[test]
    fn conditional_word_helper_uses_upper_nibble_and_ignores_lower_controller_byte() {
        let tables = source_tables();
        for nibble in 0..DIRECTION_COUNT {
            for irrelevant_bits in [0, 0x00ff, 0xf0a5] {
                assert_eq!(
                    tables.word_delta(ControllerWord(((nibble as u16) << 8) | irrelevant_bits)),
                    WordDelta {
                        first: FIRST_RESULTS[nibble],
                        second: SECOND_RESULTS[nibble]
                    }
                );
            }
        }
    }

    fn distinguishable_words() -> [ControllerWord; CONTROLLER_COUNT] {
        [
            ControllerWord(0x91a5),
            ControllerWord(0x6243),
            ControllerWord(0x34c2),
            ControllerWord(0xf87e),
        ]
    }

    #[test]
    fn every_frontend_page_samples_first_two_devices_with_both_bytes_intact() {
        let automatic = distinguishable_words();
        for page in [0, 1, 2, u8::MAX] {
            assert_eq!(
                sample_frontend(page, automatic),
                FrontendInputs {
                    players: [automatic[0], automatic[1]]
                }
            );
        }
    }

    #[test]
    fn all_joined_masks_copy_exact_hardware_words_and_preserve_disabled_ai_slots() {
        let hardware = distinguishable_words();
        let old = [
            ControllerWord(0x0001),
            ControllerWord(0x0002),
            ControllerWord(0x0003),
            ControllerWord(0x0004),
        ];
        for mask in 0..DIRECTION_COUNT {
            let mut retained = old;
            copy_human_words(&mut retained, mask as u8, hardware);
            // Enumerated source bit/slot association, not an implementation oracle.
            for (slot, enabled_bit) in [(0, 0x01), (1, 0x02), (2, 0x04), (3, 0x08)] {
                assert_eq!(
                    retained[slot],
                    if mask & enabled_bit == 0 {
                        old[slot]
                    } else {
                        hardware[slot]
                    }
                );
            }
        }
    }

    #[test]
    fn high_mask_bits_do_not_enable_slots_or_modify_disabled_words() {
        let old = distinguishable_words();
        let mut retained = old;
        copy_human_words(&mut retained, 0xf0, [ControllerWord(0); CONTROLLER_COUNT]);
        assert_eq!(retained, old);
        copy_human_words(&mut retained, 0xf8, [ControllerWord(0); CONTROLLER_COUNT]);
        assert_eq!(retained, [old[0], old[1], old[2], ControllerWord(0)]);
    }

    #[test]
    fn released_human_buttons_clear_their_word_without_clearing_ai() {
        let old = distinguishable_words();
        let mut retained = old;
        copy_human_words(&mut retained, 0x02, [ControllerWord(0); CONTROLLER_COUNT]);
        assert_eq!(retained, [old[0], ControllerWord(0), old[2], old[3]]);
    }

    #[test]
    fn race_motion_uses_previous_sample_then_checkpoint_reads_new_human_words() {
        let old = distinguishable_words();
        let hardware = [
            ControllerWord(BUTTON_B),
            ControllerWord(BUTTON_UP),
            ControllerWord(BUTTON_LEFT),
            ControllerWord(BUTTON_A),
        ];
        let mut retained = old;
        let first = sample_race_frame(&mut retained, 0x05, hardware);
        assert_eq!(first.movement_words, old);
        let after_poll = [hardware[0], old[1], hardware[2], old[3]];
        assert_eq!(first.checkpoint_words, after_poll);
        assert_eq!(retained, after_poll);
        // AI writes after the copy. On the next source frame that generated word
        // is consumed for motion and survives the disabled-human slot copy.
        retained[1] = ControllerWord(BUTTON_B | BUTTON_RIGHT);
        let second = sample_race_frame(&mut retained, 0x05, [ControllerWord(0); CONTROLLER_COUNT]);
        assert_eq!(
            second.movement_words,
            [hardware[0], retained[1], hardware[2], old[3]]
        );
        assert_eq!(
            second.checkpoint_words,
            [ControllerWord(0), retained[1], ControllerWord(0), old[3]]
        );
    }

    #[test]
    fn all_twelve_held_actions_translate_to_the_source_controller_bit_positions() {
        for (name, expected) in [
            ("b", BUTTON_B),
            ("y", BUTTON_Y),
            ("select", BUTTON_SELECT),
            ("start", BUTTON_START),
            ("up", BUTTON_UP),
            ("down", BUTTON_DOWN),
            ("left", BUTTON_LEFT),
            ("right", BUTTON_RIGHT),
            ("a", BUTTON_A),
            ("x", BUTTON_X),
            ("l", BUTTON_L),
            ("r", BUTTON_R),
        ] {
            let mut input = MultiInputState::default();
            input.players[0].held.insert(name.into());
            assert_eq!(
                encode_players(&input),
                [
                    ControllerWord(expected),
                    ControllerWord(0),
                    ControllerWord(0),
                    ControllerWord(0)
                ],
                "{name}"
            );
        }
    }

    #[test]
    fn multiplayer_actions_stay_isolated_and_conflicting_directions_are_not_filtered() {
        let mut input = MultiInputState::default();
        for action in [
            "up",
            "down",
            "left",
            "right",
            "start",
            "a",
            "not_a_game_action",
        ] {
            input.players[0].held.insert(action.into());
        }
        input.players[1].held.insert("b".into());
        input.players[2].held.insert("select".into());
        input.players[3].held.insert("r".into());
        assert_eq!(
            encode_players(&input),
            [
                ControllerWord(0x1f80),
                ControllerWord(BUTTON_B),
                ControllerWord(BUTTON_SELECT),
                ControllerWord(BUTTON_R)
            ]
        );
    }

    #[test]
    fn action_translation_uses_current_held_state_not_host_edge_sets() {
        let mut input = MultiInputState::default();
        input.players[0] = PlayerInput {
            held: ["a".to_owned()].into(),
            pressed: ["b".to_owned()].into(),
            released: ["start".to_owned()].into(),
        };
        assert_eq!(encode_players(&input)[0], ControllerWord(BUTTON_A));
        input.players[0].held.clear();
        assert_eq!(
            encode_players(&input),
            [ControllerWord(0); CONTROLLER_COUNT]
        );
    }
}
