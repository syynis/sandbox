use bevy::{
    asset::LoadState,
    prelude::*,
    reflect::{TypePath, TypeUuid},
    utils::hashbrown::HashMap,
};
use serde::{Deserialize, Serialize};

pub const BASE_TILE_SIZE: usize = 20;

#[derive(Default, Reflect, Copy, Clone)]
pub enum TilePixel {
    #[default]
    Up = 0,
    Neutral = 1,
    Down = 2,
    None = 3,
}

pub struct TileLayer {
    pub colors: Vec<TilePixel>,
}

pub struct Tile {
    pub layers: Vec<TileLayer>,
    computed_tile_layers: Vec<usize>,
    meta: TileMeta,
}

impl Tile {
    pub fn get_kind(&self, sub_layer: usize, pos: usize) -> TilePixel {
        let tile_layer = self.computed_tile_layers[sub_layer];
        self.layers[tile_layer].colors[pos].clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileMeta {
    pub name: String,
    pub size: UVec2,
    pub layer_repeats: Vec<usize>,
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
pub struct TileImages(pub HashMap<String, (Handle<Image>, TileMeta)>);

#[derive(Default, Resource)]
pub struct Tiles(pub HashMap<String, Tile>);

pub fn load_manifests(asset_server: Res<AssetServer>, mut manifests: ResMut<Manifests>) {
    let stone_manifest: Handle<TileManifest> = asset_server.load("tiles/stones.manifest.ron");
    manifests.0.push(stone_manifest);
}

pub fn load_tile_images(
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

    cmds.insert_resource(TileImages(tiles));
}

pub fn load_tiles(
    mut cmds: Commands,
    asset_server: Res<AssetServer>,
    tile_images: Option<Res<TileImages>>,
    images: Res<Assets<Image>>,
    mut loaded: Local<bool>,
) {
    let Some(tile_images) = tile_images else {
        return;
    };

    let tiles_loaded =
        match asset_server.get_group_load_state(tile_images.0.values().map(|v| v.0.id())) {
            LoadState::Loaded => true,
            LoadState::Failed => {
                bevy::log::error!("Failed to load tile asset");
                false
            }
            _ => false,
        };

    if *loaded || !tiles_loaded {
        return;
    }
    *loaded = true;

    let mut tiles: HashMap<String, Tile> = HashMap::new();
    for (name, (handle, meta)) in tile_images.0.iter() {
        let tile_image = images.get(handle).unwrap();

        let get_tile_layer = |sub_layer: usize| -> usize {
            let mut unrolled = Vec::new();
            for (idx, e) in meta.layer_repeats.iter().enumerate() {
                (0..*e).for_each(|_| {
                    unrolled.push(idx);
                })
            }
            let tile_layer = unrolled[sub_layer];
            tile_layer
        };

        let tile = Tile {
            layers: tile_image
                .data
                .chunks(BASE_TILE_SIZE.pow(2) * 4 * (meta.size.x * meta.size.y) as usize)
                .map(|chunk| TileLayer {
                    colors: chunk
                        .chunks(4)
                        .map(|color_data| {
                            let (r, g, b, a) =
                                (color_data[0], color_data[1], color_data[2], color_data[3]);
                            match (r, g, b, a) {
                                (0, 0, 0, 0) => TilePixel::None,
                                (255, 0, 0, 255) => TilePixel::Up,
                                (0, 255, 0, 255) => TilePixel::Neutral,
                                (0, 0, 255, 255) => TilePixel::Down,
                                _ => unreachable!(),
                            }
                        })
                        .collect(),
                })
                .collect(),
            computed_tile_layers: (0..10).map(|idx| get_tile_layer(idx)).collect(),
            meta: meta.clone(),
        };
        tiles.insert(name.clone(), tile);
    }

    cmds.insert_resource(Tiles(tiles));
}
