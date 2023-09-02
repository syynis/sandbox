use std::time::Duration;

use bevy::prelude::*;

use bevy::sprite::MaterialMesh2dBundle;
use bevy::time::Stopwatch;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::PanCam;
use bevy_pancam::PanCamPlugin;
use bevy_prototype_debug_lines::DebugLinesPlugin;
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
        InputPlugin,
        InputManagerPlugin::<ActionKind>::default(),
    ));
    app.insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Gravity(Vector::NEG_Y * 256.0));

    app.add_systems(Startup, setup);
    app.add_systems(Update, (spawn_player, movement));

    app.run();
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum ActionKind {
    Left,
    Right,
    Jump,
}

#[derive(Component)]
pub struct Controllable;

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
            transform: Transform::from_scale(Vec3::new(50.0, 1.0, 1.0)),
            ..default()
        },
        RigidBody::Static,
        Position(Vector::NEG_Y * 8.0 * 6.0),
        Collider::cuboid(8.0 * 50.0, 8.0),
        LockedAxes::new().lock_rotation(),
    ));

    // Floor
    cmds.spawn((
        SpriteBundle {
            sprite: square_sprite.clone(),
            transform: Transform::from_scale(Vec3::new(80.0, 1.0, 1.0)),
            ..default()
        },
        RigidBody::Static,
        Position(Vector::X * 8. * 80. + Vector::NEG_Y * 8.0 * 6.0),
        Collider::cuboid(8.0 * 80.0, 8.0),
        LockedAxes::new().lock_rotation(),
    ));
}

fn spawn_player(
    mut cmds: Commands,
    keys: Res<Input<KeyCode>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if keys.just_pressed(KeyCode::F) {
        let cuboid = (
            Collider::cuboid(8.0, 8.0),
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Box::new(8.0, 8.0, 8.0).into()).into(),
                material: materials.add(ColorMaterial::from(Color::rgb(0.47, 0.58, 0.8))),
                ..default()
            },
            Friction::new(0.),
        );

        cmds.spawn((
            cuboid.clone(),
            RigidBody::Dynamic,
            Position(Vector::new(0., 0.)),
            Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
            LockedAxes::new().lock_rotation(),
            Control::default(),
            ShapeCaster::new(
                Collider::cuboid(8., 8.),
                Vector::NEG_Y * 0.05,
                0.,
                Vector::NEG_Y,
            )
            .with_ignore_origin_penetration(true) // Don't count player's collider
            .with_max_time_of_impact(0.2)
            .with_max_hits(1),
        ));
    }
}

fn movement(
    action_state_query: Query<&ActionState<ActionKind>>,
    mut query_player: Query<(&mut LinearVelocity, &ShapeHits), With<Controllable>>,
    time: Res<Time>,
    mut jump_extender: Local<Stopwatch>,
    mut coyote: Local<Timer>,
) {
    for action_state in action_state_query.iter() {
        jump_extender.tick(time.delta());
        coyote.tick(time.delta());
        for (mut vel, ground) in query_player.iter_mut() {
            let grounded = !ground.is_empty();

            if grounded {
                coyote.set_duration(Duration::from_secs_f32(0.075));
                coyote.reset();
                coyote.pause();
            } else {
                coyote.unpause();
            }

            let vel_penalty = if !grounded { 8. } else { 0. };
            let falling = vel.y < 0.;
            let can_coyote = !coyote.finished() && falling;
            let can_jump = grounded || can_coyote;
            if action_state.just_pressed(ActionKind::Jump) && can_jump {
                vel.y += 128.;
                jump_extender.reset();
                let remaining = coyote.remaining();
                coyote.tick(remaining);
            }

            if action_state.pressed(ActionKind::Jump)
                && !grounded
                && jump_extender.elapsed_secs() < 0.125
            {
                vel.y += 4.;
            }

            if action_state.pressed(ActionKind::Left) {
                vel.x -= 32. - vel_penalty;
            }
            if action_state.pressed(ActionKind::Right) {
                vel.x += 32. - vel_penalty;
            }
            vel.x *= 0.8;
        }
    }
}
