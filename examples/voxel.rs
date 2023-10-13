use std::{fs, path::PathBuf};

use bevy::{
    ecs::system::Command,
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    utils::hashbrown::HashMap,
};
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::{PanCam, PanCamPlugin};
use rand::{
    rngs::ThreadRng,
    seq::{IteratorRandom, SliceRandom},
    thread_rng,
};
use serde::{Deserialize, Serialize};

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins.set(ImagePlugin::default_nearest()),
        PanCamPlugin::default(),
        WorldInspectorPlugin::default(),
        RonAssetPlugin::<TileManifest>::new(&["manifest.ron"]),
    ));

    app.insert_resource(ClearColor(Color::DARK_GRAY));
    app.register_type::<Palette>();
    app.register_type::<PaletteRows>();

    app.insert_resource(Manifests::default());
    app.insert_resource(MapImages::default());

    app.add_systems(Startup, (setup, load_manifests));
    app.add_systems(Update, (load_palette, apply_deferred, spawn_map).chain());
    app.add_systems(Update, load_tiles);
    app.run()
}

pub struct SpawnVoxelCommand {
    pos: Vec3,
    layer: usize,
    flip: bool,
}

impl Command for SpawnVoxelCommand {
    fn apply(self, world: &mut World) {
        let offset = Vec2::splat(1.);
        let l = match self.layer {
            2 => 6,
            x => x,
        } as f32;
        let pos = self.pos + ((offset * 9.).extend(-10.) * l);

        world.resource_scope(|world, palette: Mut<Palette>| {
            let sprite = |color: Color| -> Sprite {
                Sprite {
                    color,
                    custom_size: Some(Vec2::splat(20.)),
                    ..default()
                }
            };

            world
                .spawn(SpriteBundle {
                    sprite: sprite(palette.get_sun_color(1, 0, self.layer)),
                    transform: Transform::from_translation(pos),
                    ..default()
                })
                .with_children(|builder| {
                    for i in 1..10 {
                        builder.spawn(SpriteBundle {
                            sprite: sprite(palette.get_sun_color(1, i, self.layer)),
                            transform: Transform::from_translation(
                                (offset * ((1 - 2 * self.flip as i32) as f32)).extend(-1.)
                                    * i as f32,
                            ),
                            ..default()
                        });
                    }
                });
        });
    }
}

fn setup(mut cmds: Commands, asset_server: Res<AssetServer>) {
    cmds.spawn((
        Camera2dBundle::default(),
        PanCam {
            grab_buttons: vec![MouseButton::Middle],
            ..default()
        },
    ));

    let palette_asset: Handle<Image> = asset_server.load("palette.png");
    cmds.insert_resource(PaletteHandle(palette_asset));
}

#[derive(Default, Resource)]
pub struct MapImages(pub Vec<Handle<Image>>);

fn spawn_map(
    mut cmds: Commands,
    keys: Res<Input<KeyCode>>,
    palette: Res<Palette>,
    mut images: ResMut<Assets<Image>>,
    mut map_images: ResMut<MapImages>,
) {
    if keys.just_pressed(KeyCode::W) {
        cmds.insert_resource(ClearColor(palette.meta.skycolor));

        let mut map = vec![vec![0; 64]; 36];
        let map_width = map[0].len();
        let map_height = map.len();
        let mut rng = thread_rng();

        /*
        for (l, layer) in map.iter().enumerate() {
            for (y, row) in layer.iter().rev().enumerate() {
                for (x, tile) in row.iter().enumerate() {
                    if *tile == 1 {
                        let x = x as f32;
                        let y = y as f32;
                        let pos = Vec3::new(20. * x, 20. * y, 0.);
                        cmds.add(SpawnVoxelCommand {
                            pos,
                            layer: l,
                            flip: false,
                        });
                    }
                }
            }
        }
        */

        let voxel_size = 20;
        let width = map_width * voxel_size;
        let height = map_height * voxel_size;
        let texture_format_size = 4; // 4 channels each a u8
        let voxel_data_size = voxel_size * texture_format_size;
        let size = Extent3d {
            width: width as u32,
            height: height as u32,
            ..default()
        };
        let dimension = TextureDimension::D2;

        for l in 0..3 {
            (0..520).for_each(|_| {
                let row = (0..map_height).choose(&mut rng).unwrap();
                let elem = (0..map_width).choose(&mut rng).unwrap();
                map[row][elem] = 1;
            });
            for idx in 0..10 {
                let mut data: Vec<u8> = vec![255; (texture_format_size * width * height) as usize];
                for (y, row) in map.iter().enumerate() {
                    for (x, tile) in row.iter().enumerate() {
                        let x_part = x * voxel_data_size;
                        let y_part = y * width * voxel_data_size;
                        let start = x_part + y_part;
                        for vy in 0usize..voxel_size {
                            for vx in 0..voxel_size {
                                let pos = start
                                    + vx * texture_format_size as usize
                                    + vy * voxel_data_size * map_width as usize;

                                if *tile == 1 {
                                    let color = palette.get_sun_color(1, idx, l).as_rgba_u8();
                                    let (r, g, b) = (color[0], color[1], color[2]);
                                    data[pos] = r;
                                    data[pos + 1] = g;
                                    data[pos + 2] = b;
                                } else if *tile == 0 {
                                    data[pos + 3] = 0;
                                }
                            }
                        }
                    }
                }

                let image = Image::new(size, dimension, data, TextureFormat::Rgba8Unorm);
                let handle = images.add(image);

                let offset = Vec2::splat(1.);
                let layer = match l {
                    2 => 6,
                    x => x,
                } as f32;
                let pos = (offset * 10.).extend(-10.) * layer;
                cmds.spawn(SpriteBundle {
                    texture: handle.clone(),
                    transform: Transform::from_translation(pos + offset.extend(-1.) * idx as f32),
                    ..default()
                });

                map_images.0.push(handle);
            }
        }
    }
}

fn load_palette(
    mut cmds: Commands,
    palette_handle: Res<PaletteHandle>,
    images: Res<Assets<Image>>,
    mut once: Local<bool>,
) {
    let Some(palette_image) = images.get(&palette_handle.0) else {
        return;
    };

    if *once {
        return;
    }
    *once = true;

    let mut meta = MetaData::default();
    let mut sun = PaletteRows::default();
    let mut shade = PaletteRows::default();
    for (row, pixel_row) in palette_image.data.chunks_exact(32 * 4).enumerate() {
        for (col, pixel) in pixel_row.chunks_exact(4).enumerate() {
            if col == 30 {
                break;
            }
            let (r, g, b, a) = (pixel[0], pixel[1], pixel[2], pixel[3]);
            let color = Color::rgba_u8(r, g, b, a);
            match row {
                0 => {
                    meta.skycolor = color;
                    break;
                }
                1 => {
                    break;
                }
                2..=4 => {
                    sun.colors[row - 2][col] = color;
                }
                5..=7 => {
                    shade.colors[row - 5][col] = color;
                }
                _ => break,
            };
        }
    }

    cmds.insert_resource(Palette { meta, sun, shade });
}

#[derive(Default, Reflect)]
pub struct MetaData {
    pub skycolor: Color,
}

#[derive(Default, Reflect)]
pub struct PaletteRows {
    colors: [[Color; 30]; 3],
}

#[derive(Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct Palette {
    meta: MetaData,
    sun: PaletteRows,
    shade: PaletteRows,
}

impl Palette {
    pub fn get_color(&self, shade: bool, dir: usize, idx: usize, layer: usize) -> Color {
        if shade {
            self.shade.colors[dir][10 * layer + idx]
        } else {
            self.sun.colors[dir][10 * layer + idx]
        }
    }

    pub fn get_sun_color(&self, dir: usize, idx: usize, layer: usize) -> Color {
        self.get_color(false, dir, idx, layer)
    }

    pub fn get_shade_color(&self, dir: usize, idx: usize, layer: usize) -> Color {
        self.get_color(true, dir, idx, layer)
    }
}

#[derive(Resource)]
pub struct PaletteHandle(Handle<Image>);

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

#[derive(Debug, Serialize, Deserialize)]
pub struct TileMeta {
    pub name: String,
    pub size: UVec2,
    pub layer_repeats: Vec<usize>,
}

#[derive(Debug, Serialize, Deserialize, TypeUuid, TypePath)]
#[uuid = "39cadc56-aa9c-4543-8640-a018b74b5052"]
pub struct TileManifest {
    pub name: String,
    pub tiles: Vec<TileMeta>,
}

#[derive(Default, Resource)]
pub struct Manifests(pub Vec<Handle<TileManifest>>);

#[derive(Default, Resource)]
pub struct Tiles(pub HashMap<String, Handle<Image>>);

fn load_manifests(asset_server: Res<AssetServer>, mut manifests: ResMut<Manifests>) {
    let stone_manifest: Handle<TileManifest> = asset_server.load("tiles/stones.manifest.ron");
    manifests.0.push(stone_manifest);
}

fn load_tiles(
    mut cmds: Commands,
    keys: Res<Input<KeyCode>>,
    asset_server: Res<AssetServer>,
    manifests: Res<Assets<TileManifest>>,
    manifest_handles: Res<Manifests>,
) {
    if keys.just_pressed(KeyCode::Q) {
        let mut tiles: HashMap<String, Handle<Image>> = HashMap::new();
        for manifest_handle in manifest_handles.0.iter() {
            let Some(manifest) = manifests.get(manifest_handle) else {
                continue;
            };

            for meta in manifest.tiles.iter() {
                let tile_image: Handle<Image> = asset_server
                    .load("tiles/".to_owned() + &manifest.name + "/" + &meta.name + ".png");
                tiles.insert(meta.name.clone(), tile_image);
            }
        }
        cmds.insert_resource(Tiles(tiles));
    }
}
