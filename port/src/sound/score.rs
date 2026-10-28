//! Immutable native music/effect events; source bytes are a compilation input.
use super::definitions::{DataAddress, SequenceData};
use anyhow::{Result, ensure};
use std::collections::BTreeMap;

pub const COMMAND_FIRST: u8 = 0x80;
pub const COMMAND_LAST: u8 = 0xc2;
pub(crate) const COMMANDS: [(Command, usize); 34] = [
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

/// Opaque event handle. The immutable label serves only provenance and the old
/// debug/save schema; ordinary playback follows compiled edges, not addresses.
#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(from = "DataAddress", into = "DataAddress")]
pub struct EventRef {
    id: Option<usize>,
    label: DataAddress,
}
impl EventRef {
    pub fn legacy(label: DataAddress) -> Self {
        Self { id: None, label }
    }
    pub fn source(self) -> DataAddress {
        self.label
    }
}
impl PartialEq for EventRef {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
    }
}
impl Eq for EventRef {}
impl From<DataAddress> for EventRef {
    fn from(value: DataAddress) -> Self {
        Self::legacy(value)
    }
}
impl From<EventRef> for DataAddress {
    fn from(value: EventRef) -> Self {
        value.label
    }
}
/// Musical executions remaining, including the source count0 meaning256.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(from = "u8", into = "u8")]
pub struct Repetitions(u16);
impl From<u8> for Repetitions {
    fn from(value: u8) -> Self {
        Self(if value == 0 { 256 } else { u16::from(value) })
    }
}
impl From<Repetitions> for u8 {
    fn from(value: Repetitions) -> Self {
        value.0 as u8
    }
}
impl Repetitions {
    pub fn repeat(&mut self) -> bool {
        if self.0 == 1 {
            false
        } else {
            self.0 -= 1;
            true
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Control {
    Stop,
    Jump(EventRef),
    Call(EventRef),
    Return,
    BeginLoop(Repetitions),
    RepeatLoop,
    FixedDuration(u8),
    ForceDuration,
    Transpose(u8),
    Instrument(u8),
    LeftVolume(u8),
    RightVolume(u8),
    AuxiliaryStream {
        reload: u8,
        stream: DataAddress,
    },
    PitchOffset(u8),
    Modulation {
        direction: i8,
        delay: u8,
        step: u8,
        period: u8,
    },
    Portamento(u8),
    DisableModulation,
    ReleaseDelta(u8),
    ReleaseFixed(u8),
    Slew {
        step: u8,
        interval: u8,
    },
    DisableSlew,
    TargetEnvelope {
        step: u8,
        interval: u8,
        delay: u8,
    },
    EnvelopeRecord(u8),
    NoiseOn,
    NoiseOff,
    NoiseClock(u8),
    PrimaryMode,
    AuxiliaryEnable,
    SecondaryMode,
    AllowKeyOn,
    SuppressKeyOn,
    SkipTranspose,
    ReloadSongOne,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScoreFault {
    Missing(DataAddress),
    InvalidCommand,
}
impl ScoreFault {
    pub fn fail<T>(self) -> Result<T> {
        match self {
            Self::Missing(at) => anyhow::bail!("missing uploaded data at {:04x}", at.0),
            Self::InvalidCommand => anyhow::bail!("invalid sequence command"),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Timing {
    /// Used only when the live voice requests an explicit duration. Missing data
    /// in this alternative must not invalidate a fixed-duration note/rest.
    pub explicit: std::result::Result<(u8, EventRef), ScoreFault>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Note { pitch: u8, timing: Timing },
    Rest(Timing),
    Control(Control),
    Fault(ScoreFault),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Node {
    pub next: EventRef,
    pub event: Event,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScoreGraph {
    labels: BTreeMap<u16, EventRef>,
    nodes: Vec<(DataAddress, Node)>,
}
impl ScoreGraph {
    pub fn compile(data: &SequenceData) -> Self {
        let mut bytes = BTreeMap::new();
        // Match SequenceData::read: later segments shadow earlier ones; bytes
        // past the16-bit address domain do not wrap into another segment.
        for segment in &data.segments {
            for (offset, &byte) in segment
                .bytes
                .iter()
                .take(65536 - usize::from(segment.origin.0))
                .enumerate()
            {
                bytes.insert(segment.origin.0 + offset as u16, byte);
            }
        }
        let labels = bytes
            .keys()
            .enumerate()
            .map(|(id, &label)| {
                (
                    label,
                    EventRef {
                        id: Some(id),
                        label: DataAddress(label),
                    },
                )
            })
            .collect();
        let mut graph = Self {
            labels,
            nodes: Vec::with_capacity(bytes.len()),
        };
        let read = |label: u16| {
            bytes
                .get(&label)
                .copied()
                .ok_or(ScoreFault::Missing(DataAddress(label)))
        };
        for (&label, &token) in &bytes {
            let next = graph.resolve(DataAddress(label.wrapping_add(1)));
            let event = if token < 0x80 {
                let timing = Timing {
                    explicit: read(label.wrapping_add(1)).map(|duration| {
                        (duration, graph.resolve(DataAddress(label.wrapping_add(2))))
                    }),
                };
                if token == 0 {
                    Event::Rest(timing)
                } else {
                    Event::Note {
                        pitch: token,
                        timing,
                    }
                }
            } else {
                match decode_command(token) {
                    Err(_) => Event::Fault(ScoreFault::InvalidCommand),
                    Ok(metadata) => {
                        let mut operands = [0; 3];
                        let mut fault = None;
                        for (offset, value) in
                            operands.iter_mut().take(metadata.operands).enumerate()
                        {
                            match read(label.wrapping_add(1 + offset as u16)) {
                                Ok(byte) => *value = byte,
                                Err(error) => {
                                    fault = Some(error);
                                    break;
                                }
                            }
                        }
                        if let Some(error) = fault {
                            Event::Fault(error)
                        } else {
                            Event::Control(graph.control(metadata.command, &operands))
                        }
                    }
                }
            };
            let next = if token >= 0x80 {
                if let Ok(metadata) = decode_command(token) {
                    graph.resolve(DataAddress(
                        label.wrapping_add(1 + metadata.operands as u16),
                    ))
                } else {
                    next
                }
            } else {
                next
            };
            graph.nodes.push((DataAddress(label), Node { next, event }));
        }
        graph
    }
    pub fn resolve(&self, label: DataAddress) -> EventRef {
        self.labels
            .get(&label.0)
            .copied()
            .unwrap_or_else(|| EventRef::legacy(label))
    }
    pub fn fetch(&self, playhead: &mut EventRef) -> Result<Node> {
        if let Some(id) = playhead.id
            && let Some((label, node)) = self.nodes.get(id)
            && *label == playhead.label
        {
            return Ok(*node);
        }
        *playhead = self.resolve(playhead.label);
        match playhead.id {
            Some(id) => Ok(self.nodes[id].1),
            None => ScoreFault::Missing(playhead.label).fail(),
        }
    }
    /// Asset codec and test-fixture boundary, not a runtime operand interface.
    pub(crate) fn control(&self, command: Command, p: &[u8; 3]) -> Control {
        let target = || self.resolve(DataAddress(u16::from_le_bytes([p[0], p[1]])));
        match command {
            Command::Stop => Control::Stop,
            Command::Jump => Control::Jump(target()),
            Command::Call => Control::Call(target()),
            Command::Return => Control::Return,
            Command::BeginLoop => Control::BeginLoop(p[0].into()),
            Command::RepeatLoop => Control::RepeatLoop,
            Command::FixedDuration => Control::FixedDuration(p[0]),
            Command::ForceDuration => Control::ForceDuration,
            Command::Transpose => Control::Transpose(p[0]),
            Command::Instrument => Control::Instrument(p[0]),
            Command::LeftVolume => Control::LeftVolume(p[0]),
            Command::RightVolume => Control::RightVolume(p[0]),
            Command::AuxiliaryStream => Control::AuxiliaryStream {
                reload: p[0],
                stream: DataAddress(u16::from_le_bytes([p[1], p[2]])),
            },
            Command::PitchOffset => Control::PitchOffset(p[0]),
            Command::PositiveModulation | Command::NegativeModulation => Control::Modulation {
                direction: if command == Command::PositiveModulation {
                    1
                } else {
                    -1
                },
                delay: p[0],
                step: p[1],
                period: p[2],
            },
            Command::Portamento => Control::Portamento(p[0]),
            Command::DisableModulation => Control::DisableModulation,
            Command::ReleaseDelta => Control::ReleaseDelta(p[0]),
            Command::ReleaseFixed => Control::ReleaseFixed(p[0]),
            Command::Slew => Control::Slew {
                step: p[0],
                interval: p[1],
            },
            Command::DisableSlew => Control::DisableSlew,
            Command::TargetEnvelope => Control::TargetEnvelope {
                step: p[0],
                interval: p[1],
                delay: p[2],
            },
            Command::EnvelopeRecord => Control::EnvelopeRecord(p[0]),
            Command::NoiseOn => Control::NoiseOn,
            Command::NoiseOff => Control::NoiseOff,
            Command::NoiseClock => Control::NoiseClock(p[0]),
            Command::PrimaryMode => Control::PrimaryMode,
            Command::AuxiliaryEnable => Control::AuxiliaryEnable,
            Command::SecondaryMode => Control::SecondaryMode,
            Command::AllowKeyOn => Control::AllowKeyOn,
            Command::SuppressKeyOn => Control::SuppressKeyOn,
            Command::SkipTranspose => Control::SkipTranspose,
            Command::ReloadSongOne => Control::ReloadSongOne,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sound::definitions::DataSegment;
    #[test]
    fn overlapping_asset_segments_preserve_duration_and_alternate_entry() {
        let graph = ScoreGraph::compile(&SequenceData {
            segments: vec![
                DataSegment {
                    origin: DataAddress(0x400),
                    bytes: vec![5, 7],
                },
                DataSegment {
                    origin: DataAddress(0x401),
                    bytes: vec![9],
                },
            ],
        });
        let mut first = graph.resolve(DataAddress(0x400));
        let node = graph.fetch(&mut first).unwrap();
        let Event::Note { pitch, timing } = node.event else {
            panic!("expected note")
        };
        assert_eq!(pitch, 5);
        assert_eq!(timing.explicit.unwrap().0, 9);
        let mut alternate = graph.resolve(DataAddress(0x401));
        assert!(matches!(
            graph.fetch(&mut alternate).unwrap().event,
            Event::Note { pitch: 9, .. }
        ));
    }
    #[test]
    fn segment_tail_past_address_space_is_not_wrapped_into_a_missing_byte() {
        let graph = ScoreGraph::compile(&SequenceData {
            segments: vec![DataSegment {
                origin: DataAddress(0xffff),
                bytes: vec![5, 9],
            }],
        });
        let mut first = graph.resolve(DataAddress(0xffff));
        let node = graph.fetch(&mut first).unwrap();
        let Event::Note { timing, .. } = node.event else {
            panic!("expected note")
        };
        assert_eq!(node.next.source(), DataAddress(0));
        assert_eq!(timing.explicit, Err(ScoreFault::Missing(DataAddress(0))));
    }
    #[test]
    fn serialized_handles_bind_by_labels_rather_than_dense_indices() {
        let segment = DataSegment {
            origin: DataAddress(0x600),
            bytes: vec![9],
        };
        let first = ScoreGraph::compile(&SequenceData {
            segments: vec![segment.clone()],
        });
        let handle = first.resolve(DataAddress(0x600));
        assert_eq!(handle.id, Some(0));
        let wire = serde_json::to_value(handle).unwrap();
        assert_eq!(wire, serde_json::json!(0x600));
        let second = ScoreGraph::compile(&SequenceData {
            segments: vec![
                DataSegment {
                    origin: DataAddress(0x100),
                    bytes: vec![0x80],
                },
                segment,
            ],
        });
        let mut restored: EventRef = serde_json::from_value(wire).unwrap();
        assert!(matches!(
            second.fetch(&mut restored).unwrap().event,
            Event::Note { pitch: 9, .. }
        ));
        assert_eq!(restored.id, Some(1));
        let mut prior_handle = handle;
        assert!(matches!(
            second.fetch(&mut prior_handle).unwrap().event,
            Event::Note { pitch: 9, .. }
        ));
    }
}
