//! Immutable selected-track resources. Original table limits apply only while
//! decoding ROM; added named content supplies its own world and display layout.
use super::{BannerLayout, load_banner_tables, original_banner_layout};
use crate::{
    presentation::{
        self, HudTables, TrackPresentation, memory::SharedGraphics, sprites::AnimationTables,
    },
    world::{
        objects::{self, ObjectContent, ObjectTables},
        tracks::{self, TrackContent},
    },
};
use anyhow::{Context, Result};
use std::sync::Arc;

pub struct RaceAssets {
    pub rom: Arc<[u8]>,
    pub track: TrackContent,
    pub presentation: TrackPresentation,
    pub objects: ObjectContent,
    pub object_tables: ObjectTables,
    pub graphics: SharedGraphics,
    pub hud: HudTables,
    pub animations: AnimationTables,
    pub banner: BannerLayout,
    pub reversed: bool,
}
impl RaceAssets {
    pub fn from_original(rom: &[u8], track: u8, order_slot: u8, reversed: bool) -> Result<Self> {
        let parameters = rom
            .get(tracks::PARAMETERS_ROM..tracks::PARAMETERS_ROM + tracks::PARAMETERS_BYTES)
            .context("truncated track parameters")?;
        let graphics = rom
            .get(
                tracks::GRAPHICS_POINTERS_ROM
                    ..tracks::GRAPHICS_POINTERS_ROM + tracks::ORIGINAL_TRACK_COUNT * 3,
            )
            .context("truncated graphics pointers")?;
        let descriptors = tracks::decode_descriptors(parameters, graphics)?;
        let descriptor = descriptors
            .get(usize::from(track))
            .context("invalid original track")?;
        let content = tracks::load_content(rom, descriptor, track)?;
        Self::from_content(Arc::from(rom), content, Some(track), order_slot, reversed)
    }
    pub fn from_content(
        rom: Arc<[u8]>,
        track: TrackContent,
        original_track: Option<u8>,
        order_slot: u8,
        reversed: bool,
    ) -> Result<Self> {
        let objects = match &track.objects {
            Some(objects) => {
                objects::validate_content(objects)?;
                objects.clone()
            }
            None => objects::load_original_content(
                &rom,
                original_track.context("added track requires object definitions")?,
                order_slot,
            )?,
        };
        let presentation = match &track.presentation {
            Some(layout) => {
                presentation::validate_track_presentation(layout)?;
                layout.clone()
            }
            None => presentation::load_track_presentation(
                &rom,
                original_track.context("added track requires presentation")?,
            )?,
        };
        // Added content uses its explicit countdown center for the banner sprite;
        // original sixteen layouts retain their independently decoded table.
        let banner = if let Some(original) = original_track {
            original_banner_layout(original, &load_banner_tables(&rom)?)?
        } else {
            BannerLayout {
                coordinates: u16::from_le_bytes(presentation.countdown.center),
                install_fixed_pair: false,
            }
        };
        Ok(Self {
            graphics: presentation::memory::load_shared(&rom)?,
            hud: presentation::load_hud_tables(&rom)?,
            animations: presentation::sprites::load_animations(&rom)?,
            object_tables: objects::load_object_tables(&rom)?,
            rom,
            track,
            presentation,
            objects,
            banner,
            reversed,
        })
    }
}
