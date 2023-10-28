use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::{PanCam, PanCamPlugin};
use sandbox::{input::InputPlugin, level::LevelPlugin};

pub fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins.set(ImagePlugin::default_nearest()),
        PanCamPlugin::default(),
        InputPlugin::<PanCam>::default(),
        WorldInspectorPlugin::default(),
        LevelPlugin,
    ));
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(mut cmds: Commands, asset_server: Res<AssetServer>) {
    cmds.spawn((
        Camera2dBundle::default(),
        PanCam {
            grab_buttons: vec![MouseButton::Middle],
            enabled: true,
            ..default()
        },
    ));

    let tiles: Handle<Image> = asset_server.load("tiles.png");

    let size = TilemapSize::from(UVec2::new(16, 16));
    let storage = TileStorage::empty(size);
    let tilemap_entity = cmds.spawn_empty().id();

    let tile_size = TilemapTileSize::from(Vec2::splat(16.));
    let grid_size = tile_size.into();
    let map_type = TilemapType::Square;

    cmds.entity(tilemap_entity).insert((
        TilemapBundle {
            grid_size,
            map_type,
            size,
            storage,
            texture: TilemapTexture::Single(tiles),
            tile_size,
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        },
        Name::new("Level"),
    ));
}
