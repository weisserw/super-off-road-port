//! Immutable resources after the common file/ROM boundary. No game state lives here.
use crate::{
    assets::{Assets, RECIPES},
    controls::DirectionTables,
    frontend::{attract, names, results, shop},
    presentation::{memory, transitions},
    race::{self, assets::RaceAssets},
    trucks::{ai, motion},
    world::{terrain, tracks},
};
use anyhow::{Context, Result, ensure};
use romatlas_port::assets::{MemoryAssets, Origin, Rom};
use sha2::{Digest, Sha256};
use std::sync::Arc;
#[derive(Clone, Debug)]
pub struct Choice {
    pub id: String,
    pub original: Option<u8>,
    pub reversed: bool,
}
pub struct Resources {
    pub raw: Assets,
    pub rom: Arc<[u8]>,
    pub identity: String,
    pub tracks: tracks::Catalogue,
    pub sequence: Vec<Choice>,
    pub frontend: attract::FrontendAssets,
    pub names: names::NameAssets,
    pub shop: shop::ShopAssets,
    pub results: results::ResultAssets,
    pub transitions: transitions::TransitionAssets,
    pub shared: memory::SharedGraphics,
    pub motion: motion::MotionTables,
    pub poses: crate::presentation::poses::PoseAtlas,
    pub sampling: terrain::SamplingTables,
    pub ai: ai::AiTables,
    pub directions: DirectionTables,
    pub banners: race::BannerTables,
    pub sound: Arc<crate::sound::definitions::AudioDefinitions>,
}
impl Resources {
    pub fn load(rom: &Rom, overrides: &MemoryAssets) -> Result<Self> {
        overrides.validate()?;
        let defaults = MemoryAssets::default();
        let raw = Assets::load(
            rom,
            if overrides.manifests.contains_key("resources.json") {
                overrides
            } else {
                &defaults
            },
        )?;
        let mut bytes = rom.read(0, crate::config::ROM_BYTES)?.to_vec();
        // Only explicit replacement slots overlay storage. Defaults in an
        // overlapping consumer window cannot erase an earlier file replacement.
        for recipe in RECIPES {
            let asset = raw.resources.get(recipe.id).context("missing raw recipe")?;
            if matches!(asset.origin, Origin::File(_)) {
                bytes[recipe.offset..recipe.offset + recipe.length].copy_from_slice(&asset.bytes);
            }
        }
        let descriptors = tracks::decode_descriptors(
            &bytes[tracks::PARAMETERS_ROM..tracks::PARAMETERS_ROM + tracks::PARAMETERS_BYTES],
            &bytes[tracks::GRAPHICS_POINTERS_ROM
                ..tracks::GRAPHICS_POINTERS_ROM + tracks::ORIGINAL_TRACK_COUNT * 3],
        )?;
        let original = descriptors
            .iter()
            .enumerate()
            .map(|(id, descriptor)| {
                Ok(tracks::NamedTrack {
                    id: format!("track-{id:02}"),
                    content: tracks::load_content(&bytes, descriptor, id as u8)?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let catalogue = tracks::load_catalogue(
            original,
            if overrides.manifests.contains_key("tracks.json") {
                overrides
            } else {
                &defaults
            },
        )?;
        let original_id = |id: &str| -> Option<u8> {
            (0..tracks::ORIGINAL_TRACK_COUNT)
                .find(|value| id == format!("track-{value:02}"))
                .map(|value| value as u8)
        };
        let ordered = overrides
            .manifests
            .get("tracks.json")
            .is_some_and(|manifest| !manifest.order.is_empty());
        let sequence = if ordered {
            catalogue
                .order
                .iter()
                .map(|id| Choice {
                    id: id.clone(),
                    original: original_id(id),
                    reversed: false,
                })
                .collect()
        } else {
            let mut choices = tracks::decode_order(
                &bytes[tracks::ORDER_ROM..tracks::ORDER_ROM + tracks::ORDER_SLOTS],
            )?
            .into_iter()
            .map(|selection| Choice {
                id: format!("track-{:02}", selection.original_track),
                original: Some(selection.original_track),
                reversed: selection.direction == tracks::Direction::Reversed,
            })
            .collect::<Vec<_>>();
            choices.extend(
                catalogue
                    .order
                    .iter()
                    .filter(|id| original_id(id).is_none())
                    .map(|id| Choice {
                        id: id.clone(),
                        original: None,
                        reversed: false,
                    }),
            );
            choices
        };
        ensure!(!sequence.is_empty(), "empty native race sequence");
        let mut digest = Sha256::new();
        digest.update(&bytes);
        // Raw added slots are observable through the shared resource collection,
        // so bind them too even when no gameplay consumer selects them yet.
        for asset in &raw.resources.assets {
            digest.update(asset.id.as_bytes());
            digest.update(&asset.bytes);
        }
        for track in &catalogue.tracks {
            digest.update(track.id.as_bytes());
            digest.update(serde_json::to_vec(&track.content)?);
        }
        for choice in &sequence {
            digest.update(choice.id.as_bytes());
            digest.update([u8::from(choice.reversed)]);
        }
        let identity = format!("{:x}", digest.finalize());
        let frontend = attract::load_assets(&bytes)?;
        let names = names::load_assets(&bytes)?;
        let shop = shop::load_assets(&bytes)?;
        let results = results::load_assets(&bytes)?;
        let transitions = transitions::load_assets(&bytes)?;
        let shared = memory::load_shared(&bytes)?;
        let motion = motion::load_motion_tables(&bytes)?;
        let poses = crate::presentation::poses::PoseAtlas::load(&bytes, &motion.body_poses)?;
        let sampling = terrain::load_sampling_tables(&bytes)?;
        let ai = ai::load_tables(&bytes)?;
        let directions = DirectionTables::from_table_bytes(
            &bytes[crate::controls::HORIZONTAL_TABLE_ROM
                ..crate::controls::HORIZONTAL_TABLE_ROM + crate::controls::DIRECTION_TABLE_BYTES],
            &bytes[crate::controls::VERTICAL_TABLE_ROM
                ..crate::controls::VERTICAL_TABLE_ROM + crate::controls::DIRECTION_TABLE_BYTES],
        )?;
        let banners = race::load_banner_tables(&bytes)?;
        let sound = Arc::new(crate::sound::definitions::load_audio(&bytes)?);
        Ok(Self {
            raw,
            rom: Arc::from(bytes),
            identity,
            tracks: catalogue,
            sequence,
            frontend,
            names,
            shop,
            results,
            transitions,
            shared,
            motion,
            poses,
            sampling,
            ai,
            directions,
            banners,
            sound,
        })
    }
    pub fn arcade_race(&self, state: &super::state::State) -> Result<RaceAssets> {
        let mut race = self.race(state.sequence_index, state.session.order_slot)?;
        if state.arcade.accepted && state.arcade.random {
            crate::frontend::arcade::randomize(&mut race, &state.arcade, &self.sequence)?;
        }
        Ok(race)
    }
    pub fn race(&self, index: usize, source_order: u8) -> Result<RaceAssets> {
        let choice = self
            .sequence
            .get(index)
            .context("selected sequence beyond catalogue")?;
        let track = self
            .tracks
            .tracks
            .iter()
            .find(|track| track.id == choice.id)
            .context("selected named track missing")?;
        RaceAssets::from_content(
            self.rom.clone(),
            track.content.clone(),
            choice.original,
            source_order,
            choice.reversed,
        )
    }
}
