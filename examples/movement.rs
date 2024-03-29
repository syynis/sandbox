use bevy::prelude::*;

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::PanCam;
use bevy_pancam::PanCamPlugin;
use bevy_prototype_lyon::prelude::ShapePlugin;
use bevy_xpbd_2d::math::*;
use bevy_xpbd_2d::prelude::*;
use sandbox::entity::player::DespawnPlayerCommand;
use sandbox::entity::player::Player;
use sandbox::entity::player::SpawnPlayerCommand;
use sandbox::input::CursorPos;
use sandbox::input::InputPlugin;
use sandbox::phys::movement::LookDir;
use sandbox::phys::terrain::Terrain;
use sandbox::phys::PhysPlugin;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins,
        PanCamPlugin::default(),
        WorldInspectorPlugin::new(),
        ShapePlugin,
        InputPlugin::<PanCam>::default(),
        PhysPlugin,
    ));
    app.insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Gravity(Vector::NEG_Y * 320.0));

    app.add_systems(Startup, setup);
    app.add_systems(Update, (respawn_player, draw_look_dir));

    app.run();
}

pub fn make_right_triangle(corner: Vector, size: Scalar, dir: Vector) -> Collider {
    Collider::triangle(
        corner + Vector::X * size * dir.x,
        corner + Vector::Y * size * dir.y,
        corner,
    )
}

pub fn right_triangle_points(corner: Vector, size: Scalar, dir: Vector) -> Vec<Vector> {
    vec![
        corner + Vector::X * size * dir.x,
        corner + Vector::Y * size * dir.y,
        corner,
    ]
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

    let square_sprite = Sprite {
        color: Color::rgb(0.7, 0.7, 0.8),
        custom_size: Some(Vec2::splat(8.0)),
        ..default()
    };

    // Floor
    cmds.spawn((
        SpriteBundle {
            sprite: square_sprite.clone(),
            transform: Transform::from_scale(Vec3::new(100.0, 1.0, 1.0)),
            ..default()
        },
        RigidBody::Static,
        Position(Vector::NEG_Y * 8.0 * 6.0),
        Collider::cuboid(8.0 * 100.0, 8.0),
        LockedAxes::new().lock_rotation(),
        Terrain,
    ));

    // Wall
    cmds.spawn((
        SpriteBundle {
            sprite: square_sprite.clone(),
            transform: Transform::from_scale(Vec3::new(1., 30., 1.)),
            ..default()
        },
        RigidBody::Static,
        Position(Vector::NEG_X * 8. * 24. + Vector::Y * 8. * 9.),
        Collider::cuboid(8., 8. * 30.),
        LockedAxes::new().lock_rotation(),
        Terrain,
    ));

    // Wall
    cmds.spawn((
        SpriteBundle {
            sprite: square_sprite.clone(),
            transform: Transform::from_scale(Vec3::new(1., 30., 1.)),
            ..default()
        },
        RigidBody::Static,
        Position(Vector::NEG_X * 8. * 8. + Vector::Y * 8. * 12.),
        Collider::cuboid(8., 8. * 30.),
        LockedAxes::new().lock_rotation(),
        Terrain,
    ));

    // Wall
    cmds.spawn((
        SpriteBundle {
            sprite: square_sprite.clone(),
            transform: Transform::from_scale(Vec3::new(1., 30., 1.)),
            ..default()
        },
        RigidBody::Static,
        Position(Vector::NEG_X * 8. * 16. + Vector::Y * 8. * 12.),
        Collider::cuboid(8., 8. * 30.),
        LockedAxes::new().lock_rotation(),
        Terrain,
    ));

    // Box
    cmds.spawn((
        SpriteBundle {
            sprite: square_sprite.clone(),
            transform: Transform::from_scale(Vec3::new(10., 5., 1.)),
            ..default()
        },
        RigidBody::Static,
        Position(Vector::X * 8. * 2.),
        Collider::cuboid(8. * 10., 8. * 5.),
        LockedAxes::new().lock_rotation(),
        Terrain,
    ));

    // Box
    cmds.spawn((
        SpriteBundle {
            sprite: square_sprite.clone(),
            transform: Transform::from_scale(Vec3::new(8., 3., 1.)),
            ..default()
        },
        RigidBody::Static,
        Position(Vector::X * 8. * 20. + Vector::NEG_Y * 8. * 1.),
        Collider::cuboid(8. * 8., 8. * 3.),
        LockedAxes::new().lock_rotation(),
        Terrain,
    ));
}

fn respawn_player(mut cmds: Commands, keys: Res<Input<KeyCode>>, cursor_pos: Res<CursorPos>) {
    let pos = **cursor_pos;
    if keys.just_pressed(KeyCode::F) {
        cmds.add(DespawnPlayerCommand);
        let size = Vector::new(14., 14.);
        cmds.add(SpawnPlayerCommand::new(pos, size, ()));
    }
}

fn draw_look_dir(q_player: Query<(&LookDir, &Transform), With<Player>>, mut gizmos: Gizmos) {
    if let Some((dir, transform)) = q_player.get_single().ok() {
        let pos = transform.translation.truncate();
        match dir {
            LookDir::Left => gizmos.line_2d(pos, pos + dir.as_vec() * 16., Color::RED),
            LookDir::Right => gizmos.line_2d(pos, pos + dir.as_vec() * 16., Color::RED),
        }
    }
}
