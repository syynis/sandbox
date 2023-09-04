use std::time::Duration;

use bevy::prelude::*;

use bevy::sprite::MaterialMesh2dBundle;
use bevy::time::Stopwatch;
use bevy::utils::hashbrown::HashMap;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::PanCam;
use bevy_pancam::PanCamPlugin;
use bevy_prototype_debug_lines::DebugLines;
use bevy_prototype_debug_lines::DebugLinesPlugin;
use bevy_prototype_lyon::prelude::GeometryBuilder;
use bevy_prototype_lyon::prelude::ShapeBundle;
use bevy_prototype_lyon::prelude::ShapePlugin;
use bevy_xpbd_2d::math::*;
use bevy_xpbd_2d::prelude::*;
use leafwing_input_manager::prelude::*;
use sandbox::input::InputPlugin;

const PPM: f32 = 32.0;

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
        InputManagerPlugin::<ActionKind>::default(),
    ));
    app.insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Gravity(Vector::NEG_Y * 320.0));

    app.add_systems(Startup, setup);
    app.add_systems(Update, (respawn_player, movement, draw_look_dir));

    app.run();
}

#[derive(Component)]
pub struct Terrain;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub enum LookDir {
    Left,
    Right,
}

impl LookDir {
    pub fn as_vec(&self) -> Vector {
        use LookDir::*;
        match *self {
            Left => Vector::NEG_X,
            Right => Vector::X,
        }
    }

    pub fn opposite(&self) -> Self {
        use LookDir::*;
        match *self {
            Left => Right,
            Right => Left,
        }
    }

    pub fn as_action_kind(&self) -> ActionKind {
        match *self {
            Self::Left => ActionKind::Left,
            Self::Right => ActionKind::Right,
        }
    }
}

#[derive(Component)]
pub struct Controllable;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum ActionKind {
    Left,
    Right,
    Jump,
}

#[derive(Bundle)]
pub struct Control {
    controllable: Controllable,
    input: InputManagerBundle<ActionKind>,
}

impl Default for Control {
    fn default() -> Self {
        use ActionKind::*;

        let mut input_map = InputMap::default();
        input_map.insert(KeyCode::A, Left);
        input_map.insert(KeyCode::D, Right);
        input_map.insert(KeyCode::Space, Jump);
        Self {
            controllable: Controllable,
            input: InputManagerBundle {
                input_map,
                ..default()
            },
        }
    }
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

    let triangle = bevy_prototype_lyon::shapes::Polygon {
        points: right_triangle_points(Vector::ZERO, 1. * 8., Vector::new(-1., 1.)),
        closed: true,
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

    // Ramp
    cmds.spawn((
        ShapeBundle {
            path: GeometryBuilder::build_as(&triangle),
            ..default()
        },
        RigidBody::Static,
        Position(Vector::NEG_X * 8.0 * 24.0 + Vector::NEG_Y * 8. * 2.),
        make_right_triangle(Vector::ZERO, 1. * 8., Vector::new(-1., 1.)),
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

fn respawn_player(
    mut cmds: Commands,
    keys: Res<Input<KeyCode>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    player_query: Query<Entity, With<Player>>,
) {
    if keys.just_pressed(KeyCode::F) {
        if let Some(player_entity) = player_query.get_single().ok() {
            cmds.entity(player_entity).despawn_recursive();
        }
        let cuboid = (
            Collider::cuboid(8.0, 12.0),
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Box::new(8.0, 12.0, 1.0).into()).into(),
                material: materials.add(ColorMaterial::from(Color::rgb(0.47, 0.58, 0.8))),
                ..default()
            },
            Friction::new(0.),
        );

        cmds.spawn((
            Player,
            cuboid.clone(),
            RigidBody::Dynamic,
            Position(Vector::new(-20., 0.)),
            Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
            LockedAxes::new().lock_rotation(),
            Control::default(),
            ShapeCaster::new(
                Collider::cuboid(7.5, 11.5),
                Vector::NEG_Y * 0.05,
                0.,
                Vector::NEG_Y,
            )
            .with_ignore_origin_penetration(true) // Don't count player's collider
            .with_max_time_of_impact(0.2)
            .with_max_hits(1),
            LookDir::Right,
            ExternalForce::default().with_persistence(false),
        ));
    }
}

fn movement(
    action_state_query: Query<&ActionState<ActionKind>>,
    mut query_player: Query<
        (
            Entity,
            &Position,
            &mut LinearVelocity,
            &mut ExternalForce,
            &ShapeHits,
            &mut LookDir,
        ),
        With<Controllable>,
    >,
    q_terrain: Query<(Entity), With<Terrain>>,
    mut collisions: EventReader<Collision>,
    q_colliding: Query<(Entity, &CollidingEntities)>,
    spatial_query: SpatialQuery,
    time: Res<Time>,
    mut jump_extender: Local<Stopwatch>,
    mut coyote: Local<Stopwatch>,
    mut near_wall_coyote: Local<Stopwatch>,
    mut disabled_inputs: Local<HashMap<ActionKind, Timer>>,
) {
    for action_state in action_state_query.iter() {
        for disabled_input in disabled_inputs.iter_mut() {
            disabled_input.1.tick(time.delta());
        }
        jump_extender.tick(time.delta());
        near_wall_coyote.tick(time.delta());
        coyote.tick(time.delta());
        for (player_entity, pos, mut vel, mut force, ground, mut look_dir) in
            query_player.iter_mut()
        {
            let grounded = !ground.is_empty();
            if grounded {
                println!("grounded");
                coyote.reset();
            }

            let air_resistance = if !grounded { 12. } else { 0. };
            let falling = vel.y < 0.;
            let can_coyote = coyote.elapsed_secs() < 0.15 && falling;
            let can_jump = grounded || can_coyote;
            if action_state.just_pressed(ActionKind::Jump) && can_jump {
                if can_coyote {
                    println!("coyote");
                }
                vel.y = 96.;
                jump_extender.reset();
            }

            let near_wall = if let Some(hit) = spatial_query.cast_ray(
                **pos,
                look_dir.as_vec(),
                4.5,
                true,
                SpatialQueryFilter::new().without_entities([player_entity]),
            ) {
                q_terrain.get(hit.entity).ok().is_some()
            } else {
                false
            };

            if near_wall && !grounded {
                near_wall_coyote.reset();
            }

            // TODO cleaner
            // Wall jump
            let was_near_wall = near_wall_coyote.elapsed_secs() < 0.1;

            let press_in_look_dir = match *look_dir {
                LookDir::Left => action_state.pressed(ActionKind::Left),
                LookDir::Right => action_state.pressed(ActionKind::Right),
            };
            if action_state.just_pressed(ActionKind::Jump) && press_in_look_dir {
                if (near_wall || was_near_wall) {
                    println!("wall jump");
                    vel.y += 128.;
                    **vel += look_dir.opposite().as_vec() * 160.;
                    disabled_inputs.insert(
                        look_dir.as_action_kind(),
                        Timer::new(Duration::from_secs_f32(0.5), TimerMode::Once),
                    );
                }
            }

            if action_state.pressed(ActionKind::Jump)
                && !grounded
                && jump_extender.elapsed_secs() < 0.2
            {
                vel.y += 4.;
            }

            if action_state.pressed(ActionKind::Left)
                && disabled_inputs
                    .get(&ActionKind::Left)
                    .map_or_else(|| true, |timer| timer.finished())
            {
                vel.x -= 16. - air_resistance;
                *look_dir = LookDir::Left;
            }
            if action_state.pressed(ActionKind::Right)
                && disabled_inputs
                    .get(&ActionKind::Right)
                    .map_or_else(|| true, |timer| timer.finished())
            {
                vel.x += 16. - air_resistance;
                *look_dir = LookDir::Right;
            }
            vel.x *= 0.95;
        }
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
