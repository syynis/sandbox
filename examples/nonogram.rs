use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::{PanCam, PanCamPlugin};
use sandbox::{
    input::{update_cursor_pos, CursorPos},
    level::LevelPlugin,
    nono::{Cell, Nonogram},
};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));
    app.add_plugin(LevelPlugin)
        .add_plugin(PanCamPlugin::default())
        .add_plugin(WorldInspectorPlugin);
    app.insert_resource(CursorPos::default())
        .add_system(update_cursor_pos);

    app.insert_resource(ClearColor(Color::WHITE));
    app.add_startup_system(setup);
    app.add_startup_system(spawn_nonogram);

    app.add_system(test_nonogram);

    app.run();
}

fn setup(mut cmds: Commands, assets_server: Res<AssetServer>) {
    cmds.spawn((
        Camera2dBundle::default(),
        PanCam {
            grab_buttons: vec![MouseButton::Middle],
            enabled: true,
            ..default()
        },
    ));
}

#[derive(Component, Deref, DerefMut)]
struct EditableNonogram(pub Nonogram);

fn spawn_nonogram(mut cmds: Commands) {
    let nonogram = example_nonogram();
    let (width, height) = nonogram.size;
    cmds.spawn((EditableNonogram(nonogram), TransformBundle::default()));
}

fn example_nonogram() -> Nonogram {
    Nonogram::new(
        (12 * 16, 12 * 16),
        vec![(0, vec![1, 1])],
        vec![(1, vec![2, 1]), (3, vec![3])],
    )
}

fn test_nonogram(
    mut cmds: Commands,
    nonogram_q: Query<(&EditableNonogram, &Transform)>,
    cursor: Res<CursorPos>,
    mouse: Res<Input<MouseButton>>,
    mut tile_storage_q: Query<(
        Entity,
        &Transform,
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &mut TileStorage,
    )>,
) {
    let (tilemap_entity, map_transform, map_size, grid_size, map_type, mut tile_storage) =
        tile_storage_q.get_single_mut().unwrap();
    for (nonogram, transform) in nonogram_q.iter() {
        let (width, height) = nonogram.size;
        let (width, height) = (width as f32, height as f32);
        let (x, y) = (transform.translation.x, transform.translation.y);

        if mouse.pressed(MouseButton::Left)
            && ((x..x + width).contains(&cursor.x) && (y..y + height).contains(&cursor.y))
        {
            let cursor_pos = **cursor;
            let cursor_in_map_pos: Vec2 = {
                // Extend the cursor_pos vec3 by 1.0
                let cursor_pos = Vec4::from((cursor_pos.extend(0.0), 1.0));
                let cursor_in_map_pos = map_transform.compute_matrix().inverse() * cursor_pos;
                cursor_in_map_pos.truncate().truncate()
            };

            if let Some(tile_pos) =
                TilePos::from_world_pos(&cursor_in_map_pos, &map_size, &grid_size, &map_type)
            {
                tile_storage.set(
                    &tile_pos,
                    cmds.spawn(TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(tilemap_entity),
                        ..default()
                    })
                    .id(),
                );
            }
        }
    }
}
