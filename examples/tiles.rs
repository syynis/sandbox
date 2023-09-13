use bevy::log::*;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::PanCam;
use bevy_pancam::PanCamPlugin;
use sandbox::input::update_cursor_pos;
use sandbox::input::CursorPos;
use sandbox::input::InputPlugin;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins.set(ImagePlugin::default_nearest()),
        TilemapPlugin,
        PanCamPlugin::default(),
        InputPlugin,
        WorldInspectorPlugin::default(),
    ));
    app.insert_resource(ClearColor(Color::WHITE));
    app.add_systems(Startup, setup);
    app.add_systems(Update, remove_tiles);

    app.run();
}

fn setup(mut cmds: Commands, assets_server: Res<AssetServer>) {
    let tiles: Handle<Image> = assets_server.load("tiles.png");
    cmds.spawn((
        Camera2dBundle::default(),
        PanCam {
            grab_buttons: vec![MouseButton::Middle],
            enabled: true,
            ..default()
        },
    ));

    let map_size = TilemapSize { x: 32, y: 32 };
    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = cmds.spawn_empty().id();

    for x in 0..32u32 {
        for y in 0..32u32 {
            let tile_pos = TilePos { x, y };
            let tile_entity = cmds
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    ..default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    cmds.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(tiles),
        tile_size,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
        ..default()
    });
}

fn remove_tiles(
    mut cmds: Commands,
    mouse: Res<Input<MouseButton>>,
    cursor_pos: Res<CursorPos>,
    mut tile_storage_query: Query<(
        &Transform,
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &mut TileStorage,
    )>,
) {
    let (map_transform, map_size, grid_size, map_type, mut tile_storage) =
        tile_storage_query.get_single_mut().unwrap();
    if mouse.pressed(MouseButton::Left) {
        let cursor_pos = **cursor_pos;
        let cursor_in_map_pos: Vec2 = {
            // Extend the cursor_pos vec3 by 1.0
            let cursor_pos = Vec4::from((cursor_pos.extend(0.0), 1.0));
            let cursor_in_map_pos = map_transform.compute_matrix().inverse() * cursor_pos;
            cursor_in_map_pos.truncate().truncate()
        };
        if let Some(tile_pos) =
            TilePos::from_world_pos(&cursor_in_map_pos, &map_size, &grid_size, &map_type)
        {
            if let Some(tile) = tile_storage.get(&tile_pos) {
                cmds.entity(tile).despawn_recursive();
                tile_storage.remove(&tile_pos);
            }
        }
    }
}
