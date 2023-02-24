use std::char::from_digit;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use bevy_rapier2d::prelude::*;
use sandbox::{
    input::{update_cursor_pos, CursorPos, InputPlugin},
    level::{
        from_world_pos,
        placement::{StorageAccess, TilePlacer},
        world_to_tile_pos, LevelPlugin,
    },
    nono::{Cell, Nonogram},
};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));
    app.add_plugin(LevelPlugin)
        .add_plugin(PanCamPlugin::default())
        .add_plugin(WorldInspectorPlugin)
        .add_plugin(DebugLinesPlugin::default())
        .add_plugin(InputPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(16.0))
        .add_plugin(RapierDebugRenderPlugin::default());

    app.register_type::<CursorPos>();
    app.insert_resource(ClearColor(Color::WHITE));
    app.insert_resource(TileCursor::default())
        .add_system(update_tile_cursor);
    app.add_startup_system(setup);
    app.add_startup_system(spawn_nonogram);
    app.add_system(setup_collider.after("tiles"))
        .add_system(setup_tiles.label("tiles"))
        .add_system(debug_render_nonogram)
        .add_system(toggle_edit)
        .add_system(edit_nonogram)
        .add_system(center_camera_editing);

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

fn setup_tiles(mut tile_placer: TilePlacer, mut once: Local<bool>) {
    if !(*once) {
        for x in 10..16u32 {
            let tile_pos = TilePos { x, y: 0 };
            tile_placer.replace(&tile_pos, TileTextureIndex(0));
        }
        *once = true
    }
}

fn setup_collider(mut cmds: Commands, tiles_q: Query<(Entity, &TilePos), Added<TilePos>>) {
    tiles_q.for_each(|(entity, tile_pos)| {
        let tile_center =
            tile_pos.center_in_world(&TilemapGridSize { x: 16., y: 16. }, &TilemapType::Square);

        cmds.entity(entity).insert((
            Collider::cuboid(8., 8.),
            TransformBundle {
                local: Transform::from_xyz(tile_center.x, tile_center.y, 0.),
                ..default()
            },
        ));
    });
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct TileCursor(pub Option<TilePos>);

pub fn update_tile_cursor(
    world_cursor: Res<CursorPos>,
    mut tile_cursor: ResMut<TileCursor>,
    tile_storage_q: Query<(&Transform, &TilemapSize)>,
) {
    let (map_transform, map_size) = tile_storage_q.get_single().unwrap();
    if world_cursor.is_changed() {
        let cursor_pos = **world_cursor;
        let cursor_in_map_pos: Vec2 = {
            let cursor_pos = Vec4::from((cursor_pos.extend(0.0), 1.0));
            let cursor_in_map_pos = map_transform.compute_matrix().inverse() * cursor_pos;
            cursor_in_map_pos.truncate().truncate()
        };

        **tile_cursor = from_world_pos(&cursor_in_map_pos, &map_size);
    }
}

#[derive(Component, Deref, DerefMut)]
struct EditableNonogram(pub Nonogram);
#[derive(Component)]
struct Editing;

fn spawn_nonogram(mut cmds: Commands) {
    let nonogram = example_nonogram();
    let (width, height) = nonogram.size;
    cmds.spawn((EditableNonogram(nonogram), TransformBundle::default()));
}

fn example_nonogram() -> Nonogram {
    Nonogram::new(
        (10, 10),
        vec![(0, vec![3]), (3, vec![3]), (6, vec![3]), (9, vec![3])],
        vec![(0, vec![3]), (3, vec![3]), (6, vec![3]), (9, vec![3])],
    )
}

fn center_camera_editing(
    nonogram_editing_q: Query<&Transform, (With<EditableNonogram>, With<Editing>)>,
    mut camera_q: Query<(&Camera, &mut OrthographicProjection, &mut Transform), Without<Editing>>,
) {
    if let Some(nonogram_transform) = nonogram_editing_q.get_single().ok() {
        if let Some((camera, mut proj, mut camera_transform)) = camera_q.get_single_mut().ok() {
            camera_transform.translation = nonogram_transform
                .translation
                .truncate()
                .extend(camera_transform.translation.z);
        }
    }
}

fn toggle_edit(
    mut cmds: Commands,
    mut nonogram_q: Query<(Entity, &mut EditableNonogram, &Transform)>,
    nonogram_editing_q: Query<Entity, (With<EditableNonogram>, With<Editing>)>,
    cursor: Res<CursorPos>,
    keys: Res<Input<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::E) {
        if nonogram_editing_q.is_empty() {
            for (entity, nonogram, transform) in nonogram_q.iter_mut() {
                let (width_orig, height_orig) = nonogram.size;
                let (width, height) = (16. * width_orig as f32, 16. * height_orig as f32);
                let (x, y) = (transform.translation.x, transform.translation.y);

                if (x..x + width).contains(&cursor.x) && (y..y + height).contains(&cursor.y) {
                    cmds.entity(entity).insert(Editing);
                    // Nonograms shouldnt overlap but this is currently not enforced so this is here
                    break;
                }
            }
        } else {
            if let Some(entity) = nonogram_editing_q.get_single().ok() {
                let (entity, nonogram, transform) = nonogram_q.get(entity).unwrap();
                if nonogram.is_valid() {
                    cmds.entity(entity).remove::<Editing>();
                } else {
                    info!("Not valid");
                }
            } else {
                error!("Multiple nonograms being edited at the same time");
            }
        }
    }
}

fn edit_nonogram(
    mut tile_placer: TilePlacer,
    mut nonogram_q: Query<(Entity, &mut EditableNonogram, &Transform), With<Editing>>,
    cursor: Res<CursorPos>,
    mouse: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    tile_cursor: Res<TileCursor>,
) {
    for (nonogram_entity, mut nonogram, transform) in nonogram_q.iter_mut() {
        let (width_orig, height_orig) = nonogram.size;
        let (width, height) = (16. * width_orig as f32, 16. * height_orig as f32);
        let (x, y) = (transform.translation.x - 8., transform.translation.y - 8.);
        if (x..x + width).contains(&cursor.x) && (y..y + height).contains(&cursor.y) {
            if mouse.just_pressed(MouseButton::Left) {
                if let Some(tile_pos) = **tile_cursor {
                    tile_placer.try_place(&tile_pos, TileTextureIndex(0));
                    let (map_transform, map_size) = tile_placer.storage.transform_size();
                    // TODO precompute this
                    let nonogram_origin = world_to_tile_pos(
                        transform.translation.truncate(),
                        &map_transform,
                        &map_size,
                    )
                    .unwrap();
                    let new = (
                        (tile_pos.x - nonogram_origin.x) as usize,
                        (tile_pos.y - nonogram_origin.y) as usize,
                    );

                    nonogram.set(new, Cell::Filled);
                }
            }

            if mouse.just_pressed(MouseButton::Right) {
                if let Some(tile_pos) = **tile_cursor {
                    tile_placer.remove(&tile_pos);

                    let (map_transform, map_size) = tile_placer.storage.transform_size();
                    let nonogram_origin = world_to_tile_pos(
                        transform.translation.truncate(),
                        &map_transform,
                        &map_size,
                    )
                    .unwrap();
                    let new = (
                        (tile_pos.x - nonogram_origin.x) as usize,
                        (tile_pos.y - nonogram_origin.y) as usize,
                    );

                    nonogram.set(new, Cell::Empty);
                }
            }
        }
    }
}

fn debug_render_nonogram(
    mut cmds: Commands,
    mut lines: ResMut<DebugLines>,
    nonogram_q: Query<(&EditableNonogram, &Transform)>,
    asset_server: Res<AssetServer>,
    mut once: Local<bool>,
) {
    for (nonogram, transform) in nonogram_q.iter() {
        let (width, height) = nonogram.size;
        let (width_scaled, height_scaled) = (width as f32 * 16., height as f32 * 16.0);
        let extend = Vec3::new(width_scaled, height_scaled, 0.);
        let min = transform.translation - Vec3::new(8., 8., 0.);
        let max = transform.translation + extend + Vec3::new(8., 8., 0.);

        for (start, end) in box_lines(transform.translation, (width_scaled, height_scaled)) {
            lines.line_colored(start, end, 0., Color::RED);
        }

        let font = asset_server.load("fonts/roboto.ttf");
        let text_style = TextStyle {
            font,
            font_size: 16.,
            color: Color::BLACK,
        };

        if !(*once) {
            // Draw horizontal_clues
            nonogram.horizontal_clues.iter().for_each(|(idx, clues)| {
                let height = *idx as f32 * 16. + 8.;
                for (clue_idx, clue) in clues.iter().enumerate().rev() {
                    cmds.spawn(Text2dBundle {
                        text: Text::from_section(
                            from_digit(*clue as u32, 10).unwrap(),
                            text_style.clone(),
                        )
                        .with_alignment(TextAlignment::CENTER),
                        transform: Transform::from_xyz(
                            min.x - (clue_idx + 1) as f32 * 16.,
                            min.y + height,
                            min.z,
                        ),
                        ..default()
                    });
                }
            });

            nonogram.vertical_clues.iter().for_each(|(idx, clues)| {
                let width = *idx as f32 * 16. + 8.;
                for (clue_idx, clue) in clues.iter().enumerate().rev() {
                    cmds.spawn(Text2dBundle {
                        text: Text::from_section(
                            from_digit(*clue as u32, 10).unwrap(),
                            text_style.clone(),
                        )
                        .with_alignment(TextAlignment::CENTER),
                        transform: Transform::from_xyz(
                            min.x + width,
                            min.y - (clue_idx + 1) as f32 * 16.,
                            min.z,
                        ),
                        ..default()
                    });
                }
            });
            *once = true;
        }
    }
}

fn box_lines(origin: Vec3, size: (f32, f32)) -> [(Vec3, Vec3); 4] {
    let (width, height) = size;
    let extend = Vec3::new(width, height, 0.);
    let min = origin - Vec3::new(8., 8., 0.);
    let max = origin + extend - Vec3::new(8., 8., 0.);

    let bottom_right = (min, min + Vec3::new(width, 0., 0.));
    let bottom_up = (min, min + Vec3::new(0., height, 0.));
    let top_left = (max, max - Vec3::new(width, 0., 0.));
    let top_down = (max, max - Vec3::new(0., height, 0.));

    [bottom_right, bottom_up, top_left, top_down]
}
