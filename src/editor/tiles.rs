use bevy::{
    asset::LoadState,
    prelude::*,
    reflect::{TypePath, TypeUuid},
    utils::hashbrown::HashMap,
};
use bevy_ecs_tilemap::tiles::TileFlip;
use serde::{Deserialize, Serialize};

pub const TILE_SIZE: usize = 20;
pub const HALF_TILE_SIZE: usize = TILE_SIZE / 2;
pub const PIXEL_SIZE: usize = 4;

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

struct SubTile {
    quadrant: usize,
    data: Vec<TileLayer>,
}

impl SubTile {
    pub fn get(&self, adj: &[bool]) -> &TileLayer {
        let horizontal = adj[2 * (self.quadrant % 2)];
        let vertical = adj[2 * (1 - (self.quadrant % 2))];
        let diagonal = adj[1];

        let idx = match (horizontal, vertical, diagonal) {
            (true, true, true) => 4,    // All neighbors
            (true, true, false) => 3,   // Only cardinal neighbors
            (true, false, _) => 2,      // Horizontal neighbor
            (false, true, _) => 1,      // Vertical neighbor
            (false, false, true) => 0,  // Only diagonal neighbor
            (false, false, false) => 0, // No neighbors
        };

        &self.data[idx]
    }
}

pub struct Material {
    pub block: BlockMaterial,
    pub slope: SlopeMaterial,
}

pub struct BlockMaterial {
    // 0: NW, 1: NE, 2: SE, 3: SW
    sub_tiles: Vec<SubTile>,
    computed_tile_layers: Vec<usize>,
}

impl BlockMaterial {
    pub fn get_pixel(&self, sub_layer: usize, rpos: usize, neighbors: &Vec<bool>) -> TilePixel {
        let Some(tile_layer) = self.computed_tile_layers.get(sub_layer) else {
            return TilePixel::Neutral;
        };
        let (x, y) = (rpos % TILE_SIZE, rpos / TILE_SIZE);

        let quadrant = match (x, y) {
            ((0..=9), (0..=9)) => 0,     // Top left
            ((10..=19), (0..=9)) => 1,   // Top right
            ((10..=19), (10..=19)) => 2, // Bottom right
            ((0..=9), (10..=19)) => 3,   // Bottom left
            _ => unreachable!(),
        };

        let x = if x > 9 { x - 10 } else { x };
        let y = if y > 9 { y - 10 } else { y };
        let idx = x + y * TILE_SIZE / 2;

        self.sub_tiles[quadrant]
            .get(&neighbors[(quadrant * 2)..=(quadrant * 2 + 2)])
            .colors[idx]
    }
}

pub struct SlopeMaterial {
    sub_tiles: Vec<Vec<Vec<TileLayer>>>,
    computed_tile_layers: Vec<usize>,
}

impl SlopeMaterial {
    pub fn get_pixel(
        &self,
        sub_layer: usize,
        rpos: usize,
        flip: TileFlip,
        neighbors: &Vec<bool>, // Always 2
    ) -> TilePixel {
        let Some(tile_layer) = self.computed_tile_layers.get(sub_layer) else {
            return TilePixel::Neutral;
        };
        let (x, y) = (rpos % TILE_SIZE, rpos / TILE_SIZE);
        let idx = x + y * TILE_SIZE;

        let row = match (flip.x, flip.y) {
            (false, false) => 0,
            (true, false) => 1,
            (false, true) => 2,
            (true, true) => 3,
        };

        let quadrant = match (neighbors[0], neighbors[1]) {
            (false, false) => (0, 2),
            (true, true) => (1, 3),
            (true, false) => {
                if row % 2 == 0 {
                    (1, 2)
                } else {
                    (0, 3)
                }
            }
            (false, true) => {
                if row % 2 == 0 {
                    (0, 3)
                } else {
                    (1, 2)
                }
            }
        };

        let res1 = self.sub_tiles[*tile_layer][row][quadrant.0].colors[idx];
        let res2 = self.sub_tiles[*tile_layer][row][quadrant.1].colors[idx];

        // Only one of the quadrants is colored or both are transparent
        match (res1, res2) {
            (p, TilePixel::None) => p,
            (TilePixel::None, p) => p,
            (_, _) => unreachable!(),
        }
    }
}

pub struct Tile {
    pub layers: Vec<TileLayer>,
    computed_tile_layers: Vec<usize>,
    size: UVec2,
}

impl Tile {
    pub fn get_pixel(&self, sub_layer: usize, rpos: usize) -> TilePixel {
        self.computed_tile_layers
            .get(sub_layer)
            .map_or(TilePixel::Neutral, |tile_layer| {
                self.layers[*tile_layer].colors[rpos].clone()
            })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Meta {
    TileMeta(TileMeta),
    MaterialMeta(MaterialMeta),
}

impl Meta {
    pub fn name(&self) -> String {
        match self {
            Meta::TileMeta(meta) => meta.name.clone(),
            Meta::MaterialMeta(meta) => meta.name.clone(),
        }
    }

    pub fn layer_repeats(&self) -> &Vec<usize> {
        match self {
            Meta::TileMeta(meta) => &meta.layer_repeats,
            Meta::MaterialMeta(meta) => &meta.layer_repeats,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileMeta {
    pub name: String,
    pub size: UVec2,
    pub layer_repeats: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialMeta {
    pub name: String,
    pub name_slope: String,
    pub layer_repeats: Vec<usize>,
    pub layer_repeats_slope: Vec<usize>,
}

#[derive(Debug, Serialize, Deserialize, TypeUuid, TypePath)]
#[uuid = "a4da6acf-87fc-465c-bbf3-7af7f49ef0fd"]
pub struct Manifest {
    pub name: String,
    pub tiles: Vec<Meta>,
}

#[derive(Default, Resource)]
pub struct Manifests(pub Vec<Handle<Manifest>>);

#[derive(Default, Resource)]
pub struct TileImages(pub HashMap<String, (Handle<Image>, Meta)>);

#[derive(Default, Resource)]
pub struct MaterialImages(pub HashMap<String, (Vec<Handle<Image>>, Meta)>);

#[derive(Default, Resource)]
pub struct Tiles(pub HashMap<String, Tile>);

#[derive(Default, Resource)]
pub struct Materials(pub HashMap<String, Material>);

pub fn load_manifests(asset_server: Res<AssetServer>, mut manifests: ResMut<Manifests>) {
    let stone_manifest: Handle<Manifest> = asset_server.load("tiles/stones.manifest.ron");
    manifests.0.push(stone_manifest);
}

pub fn load_tile_images(
    mut cmds: Commands,
    asset_server: Res<AssetServer>,
    manifests: Res<Assets<Manifest>>,
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

    let mut tiles: HashMap<String, (Handle<Image>, Meta)> = HashMap::new();
    let mut materials: HashMap<String, (Vec<Handle<Image>>, Meta)> = HashMap::new();
    for manifest_handle in manifest_handles.0.iter() {
        let Some(manifest) = manifests.get(manifest_handle) else {
            continue;
        };

        for meta in manifest.tiles.iter() {
            let load = |name: &String| -> Handle<Image> {
                asset_server.load("tiles/".to_owned() + &manifest.name + "/" + &name + ".png")
            };
            match meta {
                Meta::TileMeta(tile) => {
                    let handle = load(&tile.name);
                    tiles.insert(tile.name.clone(), (handle, meta.clone()));
                }
                Meta::MaterialMeta(material) => {
                    let handle = load(&material.name);
                    let handle_slope = load(&material.name_slope);
                    materials.insert(
                        material.name.clone(),
                        (vec![handle, handle_slope], meta.clone()),
                    );
                }
            };
        }
    }

    cmds.insert_resource(TileImages(tiles));
    cmds.insert_resource(MaterialImages(materials));
}

pub fn load_tiles(
    mut cmds: Commands,
    asset_server: Res<AssetServer>,
    tile_images: Option<Res<TileImages>>,
    material_images: Option<Res<MaterialImages>>,
    images: Res<Assets<Image>>,
    mut loaded: Local<bool>,
) {
    let Some(tile_images) = tile_images else {
        return;
    };
    let Some(material_images) = material_images else {
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

    let materials_loaded = match asset_server.get_group_load_state(
        material_images
            .0
            .values()
            .flat_map(|v| v.0.iter().map(|handle| handle.id())),
    ) {
        LoadState::Loaded => true,
        LoadState::Failed => {
            bevy::log::error!("Failed to load tile asset");
            false
        }
        _ => false,
    };

    if *loaded || !tiles_loaded || !materials_loaded {
        return;
    }
    *loaded = true;

    let mut tiles: HashMap<String, Tile> = HashMap::new();

    let get_tile_layer = |sub_layer: usize, layer_repeats: &Vec<usize>| -> usize {
        let mut unrolled = Vec::new();
        for (idx, e) in layer_repeats.iter().enumerate() {
            (0..*e).for_each(|_| {
                unrolled.push(idx);
            })
        }
        let tile_layer = unrolled[sub_layer];
        tile_layer
    };

    let compute_tile_layers = |layer_repeats: &Vec<usize>| -> Vec<usize> {
        (0..layer_repeats.iter().sum())
            .map(|idx| get_tile_layer(idx, layer_repeats))
            .collect()
    };

    for (name, (handle, meta)) in tile_images.0.iter() {
        match meta {
            Meta::TileMeta(meta) => {
                let tile_image = images.get(handle).unwrap();
                let tile = Tile {
                    layers: TileLayer::from(&tile_image.data)
                        .colors
                        .chunks(TILE_SIZE.pow(2) * (meta.size.x * meta.size.y) as usize)
                        .map(|chunk| TileLayer {
                            colors: chunk.to_vec(),
                        })
                        .collect(),
                    computed_tile_layers: compute_tile_layers(&meta.layer_repeats),
                    size: meta.size,
                };
                tiles.insert(name.clone(), tile);
            }
            _ => {}
        }
    }

    let mut materials: HashMap<String, Material> = HashMap::new();
    for (name, (handles, meta)) in material_images.0.iter() {
        match meta {
            Meta::MaterialMeta(meta) => {
                let computed_tile_layers = compute_tile_layers(&meta.layer_repeats);
                let computed_tile_layers_slope = compute_tile_layers(&meta.layer_repeats_slope);
                let material_images: Vec<&Image> = handles
                    .iter()
                    .map(|handle| images.get(handle).unwrap())
                    .collect();
                let block_rows = 5; // 5 cases A = AIR, S = SOLID : (A,A), (A,S), (S,A), (A,A,D), (S,S)
                let sub_tiles: Vec<SubTile> = TileLayer::from(&material_images[0].data)
                    .colors
                    .chunks(HALF_TILE_SIZE.pow(2) * block_rows)
                    .enumerate()
                    .map(|(quadrant, row)| SubTile {
                        quadrant,
                        data: (0..block_rows)
                            .map(|r| {
                                let start = r * HALF_TILE_SIZE;
                                TileLayer {
                                    colors: (0..HALF_TILE_SIZE)
                                        .flat_map(move |vy| {
                                            (0..HALF_TILE_SIZE).map(move |vx| (vx, vy))
                                        })
                                        .map(|(vx, vy)| {
                                            let wpos =
                                                start + vx + vy * HALF_TILE_SIZE * block_rows;
                                            row[wpos]
                                        })
                                        .collect(),
                                }
                            })
                            .collect::<Vec<TileLayer>>(),
                    })
                    .collect();
                let block = BlockMaterial {
                    sub_tiles,
                    computed_tile_layers,
                };
                let slope_rows = 4;
                let slope_cols = 4;
                let sub_tiles: Vec<Vec<Vec<TileLayer>>> = TileLayer::from(&material_images[1].data)
                    .colors
                    .chunks(TILE_SIZE.pow(2) * slope_rows * slope_cols)
                    .map(|layer| {
                        layer
                            .chunks(TILE_SIZE.pow(2) * slope_rows)
                            .enumerate()
                            .map(|(_, row)| {
                                (0..slope_rows)
                                    .map(|r| {
                                        let start = r * TILE_SIZE;
                                        TileLayer {
                                            colors: (0..TILE_SIZE)
                                                .flat_map(move |vy| {
                                                    (0..TILE_SIZE).map(move |vx| (vx, vy))
                                                })
                                                .map(|(vx, vy)| {
                                                    let wpos =
                                                        start + vx + vy * TILE_SIZE * slope_rows;
                                                    row[wpos]
                                                })
                                                .collect(),
                                        }
                                    })
                                    .collect::<Vec<TileLayer>>()
                            })
                            .collect()
                    })
                    .collect();
                let slope = SlopeMaterial {
                    sub_tiles,
                    computed_tile_layers: computed_tile_layers_slope,
                };
                let material = Material { block, slope };
                materials.insert(name.clone(), material);
            }
            _ => {}
        }
    }

    cmds.insert_resource(Tiles(tiles));
    cmds.insert_resource(Materials(materials));
}

impl From<&Vec<u8>> for TileLayer {
    fn from(value: &Vec<u8>) -> Self {
        TileLayer {
            colors: value
                .chunks(PIXEL_SIZE)
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
                .collect::<Vec<TilePixel>>(),
        }
    }
}
