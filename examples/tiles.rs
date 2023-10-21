use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::PanCam;
use bevy_pancam::PanCamPlugin;
use bevy_xpbd_2d::math::Vector;
use bevy_xpbd_2d::prelude::*;
use sandbox::editor::render::setup_display;
use sandbox::editor::EditorPlugin;
use sandbox::editor::EditorState;
use sandbox::entity::pebble::SpawnPebble;
use sandbox::entity::player::DespawnPlayerCommand;
use sandbox::entity::player::Player;
use sandbox::entity::player::SpawnPlayerCommand;
use sandbox::input::InputPlugin;
use sandbox::level::layer::Layer;
use sandbox::level::placement::StorageAccess;
use sandbox::level::placement::TileProperties;
use sandbox::level::tile::InsertTileColliderCommand;
use sandbox::level::tile::TileKind;
use sandbox::level::tpos_wpos;
use sandbox::level::LevelPlugin;
use sandbox::level::SpawnMapCommand;
use sandbox::level::TileCursor;
use sandbox::lifetime::LifetimePlugin;
use sandbox::phys::movement::LookDir;
use sandbox::phys::PhysPlugin;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins.set(ImagePlugin::default_nearest()),
        PanCamPlugin::default(),
        InputPlugin::<PanCam>::default(),
        WorldInspectorPlugin::default().run_if(enable_inspector),
        LevelPlugin,
        PhysPlugin,
        LifetimePlugin,
        EditorPlugin,
    ));
    app.insert_resource(ClearColor(Color::DARK_GRAY))
        .insert_resource(Gravity(Vector::NEG_Y * 320.0))
        .insert_resource(SubstepCount(3));

    app.add_systems(Startup, (setup, setup_display));
    app.add_systems(
        Update,
        (
            toggle_inspector,
            spawn_collisions,
            respawn_player,
            draw_look_dir,
            spawn_rock,
        ),
    );

    app.run();
}

fn respawn_player(mut cmds: Commands, keys: Res<Input<KeyCode>>, tile_cursor: Res<TileCursor>) {
    let Some(tile_cursor) = **tile_cursor else {
        return;
    };
    let pos = tpos_wpos(&tile_cursor);
    if keys.just_pressed(KeyCode::F) {
        cmds.add(DespawnPlayerCommand);
        let size = Vector::new(14., 14.);
        cmds.add(SpawnPlayerCommand::new(pos, size, ()));
    }
}

fn spawn_rock(mut cmds: Commands, keys: Res<Input<KeyCode>>, tile_cursor: Res<TileCursor>) {
    let Some(tile_cursor) = **tile_cursor else {
        return;
    };
    let pos = tpos_wpos(&tile_cursor);
    if keys.just_pressed(KeyCode::G) {
        cmds.add(SpawnPebble {
            pos,
            vel: Vec2::ZERO,
            lifetime: Some(3.0),
        })
    }
}

fn draw_look_dir(q_player: Query<(&LookDir, &Transform), With<Player>>, mut gizmos: Gizmos) {
    if let Some((dir, transform)) = q_player.get_single().ok() {
        match dir {
            LookDir::Left => gizmos.line(
                transform.translation,
                transform.translation + dir.as_vec().extend(0.) * 16.,
                Color::RED,
            ),
            LookDir::Right => gizmos.line(
                transform.translation,
                transform.translation + dir.as_vec().extend(0.) * 16.,
                Color::RED,
            ),
        }
    }
}

fn enable_inspector(state: Res<EditorState>) -> bool {
    state.enabled.inspector
}

fn toggle_inspector(keys: Res<Input<KeyCode>>, mut state: ResMut<EditorState>) {
    if keys.just_pressed(KeyCode::F1) {
        state.enabled.inspector = !state.enabled.inspector;
    }
}

fn setup(mut cmds: Commands) {
    cmds.spawn((
        Camera2dBundle::default(),
        PanCam {
            grab_buttons: vec![MouseButton::Middle],
            enabled: true,
            ..default()
        },
    ));

    cmds.add(SpawnMapCommand::new(UVec2::new(64, 32), 16));
}

fn spawn_collisions(
    keys: Res<Input<KeyCode>>,
    mut cmds: Commands,
    tiles: StorageAccess,
    tiles_pos: Query<(&TilePos, &TileTextureIndex, &TileFlip)>,
) {
    if keys.just_pressed(KeyCode::Q) {
        tiles
            .storage(Layer::World)
            .unwrap()
            .iter()
            .for_each(|tile_entity| {
                let Some(tile_entity) = tile_entity else {
                    return;
                };

                let Ok((pos, id, flip)) = tiles_pos.get(*tile_entity) else {
                    return;
                };

                let center = tpos_wpos(pos);

                let properties = TileProperties {
                    id: *id,
                    flip: *flip,
                };
                let kind = TileKind::from(properties.id);

                cmds.add(InsertTileColliderCommand {
                    tile_entity: *tile_entity,
                    pos: center,
                    properties,
                    kind,
                });
            });
    }
}
