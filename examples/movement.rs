use bevy::prelude::*;

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::PanCam;
use bevy_pancam::PanCamPlugin;
use bevy_prototype_debug_lines::DebugLines;
use bevy_prototype_debug_lines::DebugLinesPlugin;
use bevy_prototype_lyon::prelude::ShapePlugin;
use bevy_xpbd_2d::math::*;
use bevy_xpbd_2d::prelude::*;
use sandbox::entity::player::DespawnPlayerCommand;
use sandbox::entity::player::Player;
use sandbox::entity::player::SpawnPlayerCommand;
use sandbox::input::InputPlugin;
use sandbox::phys::movement::LookDir;
use sandbox::phys::movement::MovementPlugin;
use sandbox::phys::terrain::Terrain;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins,
        PanCamPlugin::default(),
        PhysicsPlugins::default(),
        DebugLinesPlugin::default(),
        WorldInspectorPlugin::new(),
        ShapePlugin,
        InputPlugin,
        MovementPlugin,
    ));
    app.insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Gravity(Vector::NEG_Y * 160.0));

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
        Position(Vector::NEG_X * 8. * 14. + Vector::Y * 8. * 12.),
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

fn respawn_player(mut cmds: Commands, keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::F) {
        cmds.add(DespawnPlayerCommand);
        let pos = Vector::new(100., 100.);
        let size = Vector::new(14., 14.);
        cmds.add(SpawnPlayerCommand { pos, size });
    }
}

fn draw_look_dir(
    q_player: Query<(&LookDir, &Transform), With<Player>>,
    mut lines: ResMut<DebugLines>,
) {
    if let Some((dir, transform)) = q_player.get_single().ok() {
        match dir {
            LookDir::Left => lines.line_colored(
                transform.translation,
                transform.translation + dir.as_vec().extend(0.) * 16.,
                0.,
                Color::RED,
            ),
            LookDir::Right => lines.line_colored(
                transform.translation,
                transform.translation + dir.as_vec().extend(0.) * 16.,
                0.,
                Color::RED,
            ),
        }
    }
}
