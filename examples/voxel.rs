use std::{fs, path::PathBuf};

use bevy::{
    ecs::system::Command,
    prelude::*,
    reflect::{TypePath, TypeUuid},
};
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::{PanCam, PanCamPlugin};
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

    app.add_systems(Startup, setup);
    app.add_systems(Update, (load_palette, apply_deferred, spawn_map).chain());
    app.add_systems(Update, test);
    app.add_systems(Update, test2);
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
            let get_color = |idx: usize| -> Color { palette.sun.colors[1][10 * self.layer + idx] };

            let sprite = |color: Color| -> Sprite {
                Sprite {
                    color,
                    custom_size: Some(Vec2::splat(20.)),
                    ..default()
                }
            };

            world
                .spawn(SpriteBundle {
                    sprite: sprite(get_color(0)),
                    transform: Transform::from_translation(pos),
                    ..default()
                })
                .with_children(|builder| {
                    for i in 1..10 {
                        builder.spawn(SpriteBundle {
                            sprite: sprite(get_color(i)),
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

fn spawn_map(mut cmds: Commands, palette: Res<Palette>, mut once: Local<bool>) {
    if *once {
        return;
    }
    *once = true;
    let map = [
        [
            [0, 0, 0, 0, 0, 0, 0, 0],
            [1, 1, 1, 1, 1, 1, 1, 0],
            [1, 0, 0, 0, 0, 0, 1, 1],
            [1, 0, 0, 0, 0, 0, 1, 1],
            [1, 1, 1, 0, 0, 1, 1, 0],
        ],
        [
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 1, 1, 1, 0, 0, 0],
            [0, 0, 1, 0, 0, 0, 0, 0],
            [1, 0, 1, 0, 0, 0, 1, 1],
            [1, 1, 1, 0, 0, 1, 1, 0],
        ],
        [
            [0, 1, 1, 1, 1, 1, 0, 0],
            [0, 1, 1, 1, 1, 1, 0, 0],
            [0, 0, 0, 0, 1, 1, 0, 0],
            [0, 0, 0, 1, 1, 1, 0, 0],
            [0, 0, 1, 1, 1, 1, 1, 0],
        ],
    ];

    cmds.insert_resource(ClearColor(palette.meta.skycolor));

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

fn test(
    keys: Res<Input<KeyCode>>,
    asset_server: Res<AssetServer>,
    mut manifests: ResMut<Manifests>,
) {
    if keys.just_pressed(KeyCode::S) {
        let stone_manifest: Handle<TileManifest> = asset_server.load("tiles/stones.manifest.ron");
        manifests.0.push(stone_manifest);
    }
}

fn test2(
    keys: Res<Input<KeyCode>>,
    manifests: Res<Assets<TileManifest>>,
    manifest_handles: Res<Manifests>,
) {
    if keys.just_pressed(KeyCode::T) {
        for manifest_handle in manifest_handles.0.iter() {
            let Some(manifest) = manifests.get(manifest_handle) else {
                return;
            };

            println!(
                "{}",
                ron::ser::to_string_pretty(&manifest, ron::ser::PrettyConfig::default()).unwrap()
            );
        }
    }
}
