//! Immutable source resources shared by every frontend. These are raw boundaries,
//! not decoded graphics/music or proof that a consumer has been implemented.
use anyhow::{Result, ensure};
use romatlas_port::assets::{Asset, Collection, MemoryAssets, Origin, Rom};

#[path = "resource_definitions.rs"]
mod definitions;
pub use definitions::RECIPES;

pub const MANIFESTS: &[&str] = &["resources.json", "tracks.json"];
pub struct Recipe {
    pub id: &'static str,
    pub offset: usize,
    pub length: usize,
}

pub struct Assets {
    pub resources: Collection,
}
impl Assets {
    pub fn load(rom: &Rom, overrides: &MemoryAssets) -> Result<Self> {
        ensure!(rom.sha256 == crate::config::ROM_SHA256, "wrong game ROM");
        let defaults = RECIPES
            .iter()
            .map(|recipe| {
                Ok(Asset {
                    id: recipe.id.into(),
                    bytes: rom.read(recipe.offset, recipe.length)?.to_vec(),
                    origin: Origin::Rom {
                        offset: recipe.offset,
                        length: recipe.length,
                    },
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let resources = overrides.collection(MANIFESTS[0], defaults, validate)?;
        Ok(Self { resources })
    }
}

/// Existing raw slots retain their source storage size. Added slots are extensible
/// bounded byte resources; later semantic loaders must validate their formats.
fn validate(asset: &Asset) -> Result<()> {
    ensure!(!asset.bytes.is_empty(), "empty resource: {}", asset.id);
    if let Some(recipe) = RECIPES.iter().find(|recipe| recipe.id == asset.id) {
        ensure!(
            asset.bytes.len() == recipe.length,
            "wrong raw resource size: {}",
            asset.id
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_recipes_are_unique_and_within_matching_rom() {
        let rom = Rom::open(
            std::path::Path::new("../input/game.rom"),
            crate::config::ROM_SHA256,
        )
        .unwrap();
        let assets = Assets::load(&rom, &MemoryAssets::default()).unwrap();
        assert_eq!(assets.resources.assets.len(), RECIPES.len());
        assert!(rom.read(usize::MAX, 2).is_err());
        assert!(rom.read(524288, 1).is_err());
        assert!(Rom::from_bytes(vec![0; 32], crate::config::ROM_SHA256).is_err());
        let mut ids = std::collections::BTreeSet::new();
        for recipe in RECIPES {
            assert!(ids.insert(recipe.id));
            assert_eq!(
                assets.resources.get(recipe.id).unwrap().bytes,
                rom.read(recipe.offset, recipe.length).unwrap()
            );
        }
    }
    #[test]
    fn overrides_reject_bad_original_size_and_accept_added_named_slots() {
        let original = RECIPES
            .iter()
            .find(|recipe| recipe.id == "track_order_and_direction")
            .unwrap();
        let invalid = Asset {
            id: original.id.into(),
            bytes: vec![0],
            origin: Origin::File("wrong.bin".into()),
        };
        assert!(validate(&invalid).is_err());
        let mut memory = MemoryAssets::default();
        memory.manifests.insert(
            "resources.json".into(),
            romatlas_port::assets::Manifest {
                version: 1,
                entries: vec![romatlas_port::assets::FileAsset {
                    id: "custom-track-data".into(),
                    file: "track.bin".into(),
                }],
                order: vec![],
            },
        );
        memory.files.insert("track.bin".into(), vec![1, 2]);
        let collection = memory
            .collection("resources.json", vec![], validate)
            .unwrap();
        assert_eq!(collection.get("custom-track-data").unwrap().bytes, [1, 2]);
        memory.files.insert("track.bin".into(), vec![]);
        assert!(
            memory
                .collection("resources.json", vec![], validate)
                .is_err()
        );
    }
}
