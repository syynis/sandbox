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

struct SubTile(pub Vec<TileLayer>);

impl SubTile {
    pub fn get(&self, adj: &[bool]) -> &TileLayer {
        let horizontal = adj[0];
        let vertical = adj[1];
        let diagonal = adj[2];

        let idx = match (horizontal, vertical, diagonal) {
            (true, true, _) => 4,
            (false, false, true) => 3,
            (true, false, _) => 2,
            (false, true, _) => 1,
            (false, false, false) => 0,
        };

        &self.0[idx]
    }
}

pub struct Material {
    // 0: NW, 1: NE, 2: SE, 3: SW
    sub_tiles: Vec<SubTile>,
}

pub struct Neighbors(pub [bool; 8]);

impl Material {
    pub fn get_from_neighbors(&self, neighbors: Neighbors) -> Tile {
        let tl = self.sub_tiles[0].get(&neighbors.0[0..=2]);
        let tr = self.sub_tiles[1].get(&neighbors.0[2..=4]);
        let br = self.sub_tiles[2].get(&neighbors.0[4..=6]);
        let br = self.sub_tiles[2].get(&neighbors.0[6..=8]);
        Tile {
            layers: Vec::new(),
            computed_tile_layers: vec![0; 10],
            size: UVec2::new(1, 1),
        }
    }
}

pub struct Tile {
    pub layers: Vec<TileLayer>,
    computed_tile_layers: Vec<usize>,
    size: UVec2,
}

impl Tile {
    pub fn get_pixel(&self, sub_layer: usize, pos: usize) -> TilePixel {
        let tile_layer = self.computed_tile_layers[sub_layer];
        self.layers[tile_layer].colors[pos].clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileMeta {
    pub name: String,
    pub size: UVec2,
    pub layer_repeats: Vec<usize>,
    pub is_material: bool,
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

#[derive(Default, Resource)]
pub struct Materials(pub HashMap<String, Material>);

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
    let mut materials: HashMap<String, Material> = HashMap::new();
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

        if meta.is_material {
            let material_rows = 5; // 5 cases A = AIR, S = SOLID : (A,A), (A,S), (S,A), (A,A,D), (S,S)
            let half_tile_size = BASE_TILE_SIZE / 2;
            let res: Vec<SubTile> = tile_image
                .data
                .chunks(4)
                .map(|pixel| {
                    let (r, g, b, a) = (pixel[0], pixel[1], pixel[2], pixel[3]);
                    match (r, g, b, a) {
                        (0, 0, 0, 0) => TilePixel::None,
                        (255, 0, 0, 255) => TilePixel::Up,
                        (0, 255, 0, 255) => TilePixel::Neutral,
                        (0, 0, 255, 255) => TilePixel::Down,
                        _ => unreachable!(),
                    }
                })
                .collect::<Vec<TilePixel>>()
                .chunks(half_tile_size.pow(2) * material_rows)
                .map(|row| {
                    SubTile(
                        (0..material_rows)
                            .map(|r| {
                                let start = r * half_tile_size;
                                TileLayer {
                                    colors: (0..half_tile_size)
                                        .flat_map(move |vy| {
                                            (0..half_tile_size).map(move |vx| (vx, vy))
                                        })
                                        .map(|(vx, vy)| {
                                            let wpos =
                                                start + vx + vy * half_tile_size * material_rows;
                                            row[wpos]
                                        })
                                        .collect(),
                                }
                            })
                            .collect::<Vec<TileLayer>>(),
                    )
                })
                .collect();
            let material = Material { sub_tiles: res };
            materials.insert(name.clone(), material);
        } else {
            let tile = Tile {
                layers: tile_image
                    .data
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
                    .collect::<Vec<TilePixel>>()
                    .chunks(BASE_TILE_SIZE.pow(2) * (meta.size.x * meta.size.y) as usize)
                    .map(|chunk| TileLayer {
                        colors: chunk.to_vec(),
                    })
                    .collect(),
                computed_tile_layers: (0..10).map(|idx| get_tile_layer(idx)).collect(),
                size: meta.size,
            };
            tiles.insert(name.clone(), tile);
        }
    }

    cmds.insert_resource(Tiles(tiles));
    cmds.insert_resource(Materials(materials));
}
