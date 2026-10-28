//! ROM recipes and decoded data contracts. Uploaded code is not a native algorithm.
use anyhow::{Context, Result, ensure};
pub const ORIGINAL_SONGS: usize = 16;
pub const VOICES: usize = 8;
pub const CONFIGURATION_BYTES: usize = 141;
pub const SAMPLE_DIRECTORY_ENTRIES: usize = 33;
pub const SEQUENCE_DATA_ORIGIN: u16 = 0x0400;
pub const INITIAL_ENVELOPE_ORIGIN: u16 = 0x1090;
pub const LOADED_CONFIGURATION_ORIGIN: u16 = 0x1097;
pub const FIRST_UPLOAD_PHYSICAL: usize = 0x070000;
pub const SECOND_UPLOAD_PHYSICAL: usize = 0x078000;
/// Resource address within an uploaded immutable data view, not a live RAM map.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DataAddress(pub u16);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataSegment {
    pub origin: DataAddress,
    pub bytes: Vec<u8>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SequenceData {
    pub segments: Vec<DataSegment>,
}
impl SequenceData {
    pub fn read(&self, at: DataAddress) -> Result<u8> {
        for segment in self.segments.iter().rev() {
            if let Some(offset) = at.0.checked_sub(segment.origin.0)
                && let Some(&value) = segment.bytes.get(usize::from(offset))
            {
                return Ok(value);
            }
        }
        anyhow::bail!("missing uploaded data at {:04x}", at.0)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UploadRecord {
    pub destination: DataAddress,
    pub bytes: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UploadStream {
    pub records: Vec<UploadRecord>,
    pub entry: DataAddress,
    pub consumed: usize,
}
pub fn decode_upload_stream(bytes: &[u8]) -> Result<UploadStream> {
    let mut records = Vec::new();
    let mut cursor = 0;
    loop {
        let header = bytes
            .get(cursor..cursor + 4)
            .context("truncated upload header")?;
        let length = usize::from(u16::from_le_bytes([header[0], header[1]]));
        let destination = DataAddress(u16::from_le_bytes([header[2], header[3]]));
        cursor += 4;
        if length == 0 {
            return Ok(UploadStream {
                records,
                entry: destination,
                consumed: cursor,
            });
        }
        ensure!(
            usize::from(destination.0) + length <= 65536,
            "upload destination overflow"
        );
        let payload = bytes
            .get(cursor..cursor + length)
            .context("truncated upload payload")?;
        records.push(UploadRecord {
            destination,
            bytes: payload.to_vec(),
        });
        cursor += length;
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SongDefinition {
    pub voices: [DataAddress; VOICES],
    pub configuration: DataAddress,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectDefinition {
    pub sequence: DataAddress,
    pub priority: u8,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SampleDefinition {
    pub start: DataAddress,
    pub loop_address: DataAddress,
    pub blocks: Vec<u8>,
    pub loop_block: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioDefinitions {
    pub score: super::score::ScoreGraph,
    pub data: SequenceData,
    pub songs: Vec<SongDefinition>,
    pub effects: Vec<EffectDefinition>,
    pub samples: Vec<SampleDefinition>,
    pub pitch_low: [u8; 256],
    pub pitch_high: [u8; 256],
    /// Effective source reads cover modular selectors, including out-of-allocation instrument33.
    pub instrument_note_base: [u8; 256],
    pub instrument_scale: [u8; 256],
    pub initial_envelopes: Vec<[u8; 7]>,
    pub dsp_initialization: Vec<(u8, u8)>,
    pub command_targets: Vec<DataAddress>,
}
impl AudioDefinitions {
    pub fn compile_score(&mut self) {
        self.score = super::score::ScoreGraph::compile(&self.data);
    }
    pub fn with_compiled_score(mut self) -> Self {
        self.compile_score();
        self
    }
}
pub fn load_audio(rom: &[u8]) -> Result<AudioDefinitions> {
    let mut data = SequenceData::default();
    for offset in [FIRST_UPLOAD_PHYSICAL, SECOND_UPLOAD_PHYSICAL] {
        let upload = decode_upload_stream(rom.get(offset..).context("missing audio upload")?)?;
        for record in upload.records {
            data.segments.push(DataSegment {
                origin: record.destination,
                bytes: record.bytes,
            });
        }
    }
    let read = |address: u16| data.read(DataAddress(address));
    let word = |address: u16| -> Result<u16> {
        Ok(u16::from_le_bytes([
            read(address)?,
            read(address.wrapping_add(1))?,
        ]))
    };
    let mut songs = Vec::new();
    for song in 0..ORIGINAL_SONGS {
        let mut voices = [DataAddress(0); VOICES];
        for (slot, voice) in voices.iter_mut().enumerate() {
            let low = 0x0e20 + slot as u16 * 0x40 + song as u16;
            *voice = DataAddress(u16::from_le_bytes([read(low)?, read(low + 0x20)?]));
        }
        let column = (song as u8).wrapping_sub(1);
        songs.push(SongDefinition {
            voices,
            configuration: DataAddress(u16::from_le_bytes([
                read(0x1020 + u16::from(column))?,
                read(0x1040 + u16::from(column))?,
            ])),
        });
    }
    let mut effects = Vec::new();
    for id in 0..16 {
        effects.push(EffectDefinition {
            sequence: DataAddress(u16::from_le_bytes([read(0x1060 + id)?, read(0x1070 + id)?])),
            priority: read(0x1080 + id)?,
        });
    }
    let mut samples = Vec::new();
    for id in 0..SAMPLE_DIRECTORY_ENTRIES {
        let at = 0x5000 + id as u16 * 4;
        samples.push(decode_sample(
            &data,
            DataAddress(word(at)?),
            DataAddress(word(at + 2)?),
        )?);
    }
    let mut pitch_low = [0; 256];
    let mut pitch_high = [0; 256];
    let mut instrument_note_base = [0; 256];
    let mut instrument_scale = [0; 256];
    for i in 0..256 {
        pitch_low[i] = read(0x0ced + i as u16)?;
        pitch_high[i] = read(0x0d42 + i as u16)?;
        instrument_note_base[i] = read(0x5084 + i as u16)?;
        instrument_scale[i] = read(0x50a4 + i as u16)?;
    }
    let mut initial_envelopes = Vec::new();
    for row in 0..28 {
        let mut values = [0; 7];
        for (field, value) in values.iter_mut().enumerate() {
            *value = read(INITIAL_ENVELOPE_ORIGIN + row * 7 + field as u16)?;
        }
        initial_envelopes.push(values);
    }
    let mut dsp_initialization = Vec::new();
    for index in 0..256 {
        let register = read(0x0cac + index)?;
        if register & 0x80 != 0 {
            break;
        }
        dsp_initialization.push((register, read(0x0cc6 + index)?));
    }
    let mut command_targets = Vec::new();
    for index in 0..34 {
        command_targets.push(DataAddress(word(0x0a10 + index * 2)?));
    }
    Ok(AudioDefinitions {
        score: super::score::ScoreGraph::compile(&data),
        data,
        songs,
        effects,
        samples,
        pitch_low,
        pitch_high,
        instrument_note_base,
        instrument_scale,
        initial_envelopes,
        dsp_initialization,
        command_targets,
    })
}
pub fn decode_sample(
    data: &SequenceData,
    start: DataAddress,
    loop_address: DataAddress,
) -> Result<SampleDefinition> {
    let mut blocks = Vec::new();
    let mut address = start.0;
    loop {
        ensure!(blocks.len() < 65536, "sample missing END");
        let header = data.read(DataAddress(address))?;
        for offset in 0..9 {
            blocks.push(
                data.read(DataAddress(
                    address
                        .checked_add(offset)
                        .context("sample address overflow")?,
                ))?,
            );
        }
        if header & 1 != 0 {
            break;
        }
        address = address.checked_add(9).context("sample address overflow")?;
    }
    let relative = loop_address
        .0
        .checked_sub(start.0)
        .context("loop before sample")?;
    ensure!(
        relative % 9 == 0 && usize::from(relative) < blocks.len(),
        "invalid sample loop"
    );
    Ok(SampleDefinition {
        start,
        loop_address,
        blocks,
        loop_block: usize::from(relative) / 9,
    })
}
/// Full live seven-byte envelope rows after exact configuration overlap.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvelopeDefinitions {
    pub rows: Vec<[u8; 7]>,
}
pub fn install_configuration(envelopes: &mut EnvelopeDefinitions, bytes: &[u8]) -> Result<()> {
    ensure!(
        bytes.len() == CONFIGURATION_BYTES && envelopes.rows.len() >= 22,
        "invalid live envelope configuration"
    );
    for (offset, &value) in bytes.iter().enumerate() {
        let at = 7 + offset;
        envelopes.rows[at / 7][at % 7] = value;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn rom() -> Vec<u8> {
        std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../input/game.rom")).unwrap()
    }
    #[test]
    fn upload_record_length_destination_and_zero_entry_are_little_endian() {
        let s = decode_upload_stream(&[3, 0, 0, 4, 11, 22, 33, 0, 0, 0, 4, 99]).unwrap();
        assert_eq!(
            s.records,
            [UploadRecord {
                destination: DataAddress(0x0400),
                bytes: vec![11, 22, 33]
            }]
        );
        assert_eq!(s.entry, DataAddress(0x0400));
        assert_eq!(s.consumed, 11);
    }
    #[test]
    fn upload_rejects_truncated_headers_payloads_and_destination_overflow() {
        for bytes in [
            vec![0],
            vec![3, 0, 0, 4, 1],
            vec![2, 0, 255, 255, 1, 2, 0, 0, 0, 4],
        ] {
            assert!(decode_upload_stream(&bytes).is_err());
        }
    }
    #[test]
    fn both_original_uploads_have_exact_7e00_payload_and_entry0400() {
        let r = rom();
        for (origin, dest, entry) in [
            (FIRST_UPLOAD_PHYSICAL, 0x0400, 0xffc0),
            (SECOND_UPLOAD_PHYSICAL, 0x8200, 0x0400),
        ] {
            let s = decode_upload_stream(&r[origin..]).unwrap();
            assert_eq!(s.records.len(), 1);
            assert_eq!(s.records[0].destination, DataAddress(dest));
            assert_eq!(s.records[0].bytes.len(), 0x7e00);
            assert_eq!(s.entry, DataAddress(entry));
            assert_eq!(s.consumed, 0x7e08);
        }
    }
    #[test]
    fn original_parallel_song_rows_include_silent_zero_and_all_sixteen() {
        let a = load_audio(&rom()).unwrap();
        assert_eq!(a.songs.len(), ORIGINAL_SONGS);
        assert_eq!(a.songs[0].voices, [DataAddress(0x1567); VOICES]);
        assert_eq!(
            a.songs[1].voices,
            [
                0x1568, 0x15bb, 0x164a, 0x1670, 0x16a8, 0x1755, 0x17a5, 0x1807
            ]
            .map(DataAddress)
        );
        assert_eq!(a.songs[15].voices[0], DataAddress(0x40b1));
        assert_eq!(a.songs[1].configuration, DataAddress(0x1154));
        assert_eq!(a.songs[15].configuration, DataAddress(0x1536));
    }
    #[test]
    fn split_sample_twenty_excludes_both_upload_headers_and_bank_gap() {
        let a = load_audio(&rom()).unwrap();
        let s = &a.samples[20];
        assert_eq!(s.start, DataAddress(0x7944));
        assert_eq!(s.blocks.len(), 3663);
        let split = 0x8200 - 0x7944;
        assert_eq!(
            &s.blocks[split - 7..split + 11],
            &[
                0x2e, 0x23, 0xff, 0xb4, 0x1f, 0xe1, 0x2f, 0x00, 0xdf, 0x44, 0xed, 0x01, 0xa0, 0xff,
                0xf0, 0x33, 0xca, 0x13
            ]
        );
    }
    #[test]
    fn sample_directory_aliases_silence_and_twentyseven_nonloop_end() {
        let a = load_audio(&rom()).unwrap();
        assert_eq!(a.samples.len(), SAMPLE_DIRECTORY_ENTRIES);
        assert_eq!(a.samples[32].start, DataAddress(0x50c4));
        assert_eq!(a.samples[32].blocks, [3, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(a.samples[27].blocks.len(), 2268);
        assert_eq!(a.samples[27].blocks[2259], 0xb1);
        assert_eq!(a.samples[1].start, a.samples[2].start);
    }
    #[test]
    fn first_end_stops_chain_and_loop_is_block_aligned() {
        let d = SequenceData {
            segments: vec![DataSegment {
                origin: DataAddress(0x400),
                bytes: vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 99],
            }],
        };
        let s = decode_sample(&d, DataAddress(0x400), DataAddress(0x409)).unwrap();
        assert_eq!(s.blocks.len(), 18);
        assert_eq!(s.loop_block, 1);
        assert!(decode_sample(&d, DataAddress(0x400), DataAddress(0x401)).is_err());
    }
    #[test]
    fn sample_without_terminal_end_or_available_nine_bytes_is_rejected() {
        for bytes in [vec![0; 9], vec![3; 8]] {
            let d = SequenceData {
                segments: vec![DataSegment {
                    origin: DataAddress(0x400),
                    bytes,
                }],
            };
            assert!(decode_sample(&d, DataAddress(0x400), DataAddress(0x400)).is_err());
        }
    }
    #[test]
    fn effect_descriptor_six_aliases_five_but_priorities_still_indexed() {
        let a = load_audio(&rom()).unwrap();
        assert_eq!(a.effects.len(), 16);
        assert_eq!(a.effects[5].sequence, DataAddress(0x438c));
        assert_eq!(a.effects[5], a.effects[6]);
        assert_eq!(a.effects[7].priority, 3);
        assert_eq!(a.effects[8].priority, 4);
    }
    #[test]
    fn effective_pitch_tables_retain_zero_and_wrapped_instrument_reads() {
        let a = load_audio(&rom()).unwrap();
        assert_eq!((a.pitch_low[0], a.pitch_high[0]), (0x80, 0x68));
        assert_eq!(a.instrument_note_base[33], a.instrument_scale[1]);
        assert_eq!(a.instrument_scale[33], 0);
    }
    #[test]
    fn configuration_overwrites_twenty_complete_envelopes_and_next_timer() {
        let mut e = EnvelopeDefinitions {
            rows: vec![[0xaa; 7]; 28],
        };
        let c: Vec<u8> = (0..CONFIGURATION_BYTES).map(|x| x as u8).collect();
        install_configuration(&mut e, &c).unwrap();
        assert_eq!(e.rows[0], [0xaa; 7]);
        assert_eq!(e.rows[1], [0, 1, 2, 3, 4, 5, 6]);
        assert_eq!(e.rows[20], [133, 134, 135, 136, 137, 138, 139]);
        assert_eq!(e.rows[21], [140, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa]);
        assert_eq!(e.rows[22], [0xaa; 7]);
    }
    #[test]
    fn invalid_configuration_length_rejected_without_partial_envelope_mutation() {
        let mut e = EnvelopeDefinitions {
            rows: vec![[5; 7]; 28],
        };
        let old = e.clone();
        assert!(install_configuration(&mut e, &[0; 140]).is_err());
        assert_eq!(e, old);
    }
    #[test]
    fn exact_dsp_initialization_preserves_gain_and_adsr_zero_contract() {
        let a = load_audio(&rom()).unwrap();
        assert_eq!(a.dsp_initialization.len(), 25);
        assert_eq!(
            &a.dsp_initialization[..9],
            &[
                (0x6c, 0xe0),
                (0x5c, 0xff),
                (0x2d, 0),
                (0x3d, 0),
                (0x4d, 0),
                (0x6d, 3),
                (0x7d, 0),
                (0x0d, 0),
                (0x5d, 0x50)
            ]
        );
        assert_eq!(a.dsp_initialization[9], (0x0f, 0x7f));
        assert_eq!(a.dsp_initialization[24], (0x75, 0));
    }
    #[test]
    fn all_thirtyfour_command_vectors_preserve_b6_b8_target_order() {
        let a = load_audio(&rom()).unwrap();
        assert_eq!(a.command_targets.len(), 34);
        assert_eq!(a.command_targets[27], DataAddress(0x0bce));
        assert_eq!(a.command_targets[28], DataAddress(0x0bc6));
        assert_eq!(a.command_targets[33], DataAddress(0x0c03));
    }
    #[test]
    fn uploaded_data_has_no_outside_tail_or_implicit_zero_reads() {
        let a = load_audio(&rom()).unwrap();
        assert!(a.data.read(DataAddress(0x03ff)).is_err());
        assert_eq!(a.data.read(DataAddress(0xffff)).unwrap(), rom()[0x7fe03]);
        assert!(load_audio(&rom()[..0x7fe03]).is_err());
    }
}
