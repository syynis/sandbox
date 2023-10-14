use bevy::{
    asset::LoadState,
    prelude::*,
    reflect::{TypePath, TypeUuid},
    utils::hashbrown::HashMap,
};
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub enum TileColorKind {
    #[default]
    Up = 0,
    Neutral = 1,
    Down = 2,
    None = 3,
}

#[derive(Default)]
pub struct TileLayer {
    // 20x20 image
    colors: Vec<TileColorKind>,
}

pub struct Tile {
    layers: Vec<TileLayer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileMeta {
    pub name: String,
    pub size: UVec2,
    pub layer_repeats: Vec<usize>,
}

impl TileMeta {
    // Returns top left corner of sub image corresponding to the tile layer
    // Result has to be multiplied by voxel dimension and texture size
    pub fn get_image_offset(&self, tile_layer: usize) -> usize {
        // TODO cache this
        let mut unrolled = Vec::new();
        for (idx, e) in self.layer_repeats.iter().enumerate() {
            (0..*e).for_each(|_| {
                unrolled.push(idx);
            })
        }
        let image_row = unrolled[tile_layer];
        image_row
    }
}

#[derive(Debug, Serialize, Deserialize, TypeUuid, TypePath)]
#[uuid = "a4da6acf-87fc-465c-bbf3-7af7f49ef0fd"]
pub struct TileManifest {
    pub name: String,
    pub tiles: Vec<TileMeta>,
}

#[derive(Default, Resource)]
pub struct Manifests(pub Vec<Handle<TileManifest>>);

#[derive(Default, Resource)]
pub struct Tiles(pub HashMap<String, (Handle<Image>, TileMeta)>);

pub fn load_manifests(asset_server: Res<AssetServer>, mut manifests: ResMut<Manifests>) {
    let stone_manifest: Handle<TileManifest> = asset_server.load("tiles/stones.manifest.ron");
    manifests.0.push(stone_manifest);
}

pub fn load_tiles(
    mut cmds: Commands,
    asset_server: Res<AssetServer>,
    manifests: Res<Assets<TileManifest>>,
    manifest_handles: Res<Manifests>,
    mut loaded: Local<bool>,
) {
    let manifests_loaded = match asset_server
        .get_group_load_state(manifest_handles.0.iter().map(|handle| handle.id()))
    {
        LoadState::Loaded => true,
        LoadState::Failed => {
            bevy::log::error!("Failed to load tile asset");
            false
        }
        _ => false,
    };
    if *loaded || !manifests_loaded {
        return;
    }
    *loaded = true;

    let mut tiles: HashMap<String, (Handle<Image>, TileMeta)> = HashMap::new();
    for manifest_handle in manifest_handles.0.iter() {
        let Some(manifest) = manifests.get(manifest_handle) else {
            continue;
        };

        for meta in manifest.tiles.iter() {
            let tile_image: Handle<Image> =
                asset_server.load("tiles/".to_owned() + &manifest.name + "/" + &meta.name + ".png");
            tiles.insert(meta.name.clone(), (tile_image, meta.clone()));
        }
    }
    cmds.insert_resource(Tiles(tiles));
}
