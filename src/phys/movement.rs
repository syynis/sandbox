use std::time::Duration;

use bevy::{prelude::*, time::Stopwatch, utils::hashbrown::HashMap};
use bevy_xpbd_2d::{
    math::{Scalar, Vector},
    prelude::*,
};
use leafwing_input_manager::prelude::*;

use crate::entity::player::Player;

use super::terrain::{Pole, PoleType, Terrain};

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<ActionKind>::default());
        app.add_systems(
            Update,
            (
                setup_movement_state,
                (horizontal_movement, jump, wall_jump).after(setup_movement_state),
                pole_climb,
                pole_movement,
                pole_gravity,
            ),
        );
        app.insert_resource(DisabledInputs::default())
            .insert_resource(MovementState::default());
        app.register_type::<MovementState>();
    }
}

#[derive(Component, Clone)]
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
    Up,
    Down,
    Left,
    Right,
    Jump,
}

#[derive(Event)]
pub enum MovementAction {
    Move(Scalar),
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
        input_map.insert(KeyCode::W, Up);
        input_map.insert(KeyCode::S, Down);
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

#[derive(Component)]
pub struct MovementProperties {
    pub acceleration: Scalar,
    pub damping: Scalar,
    pub jump_impulse: Scalar,
}

impl MovementProperties {
    pub fn new(acceleration: Scalar, damping: Scalar, jump_impulse: Scalar) -> Self {
        Self {
            acceleration,
            damping,
            jump_impulse,
        }
    }
}

impl Default for MovementProperties {
    fn default() -> Self {
        Self::new(30., 0.9, 7.)
    }
}

fn keyboard_input(
    mut movement_events: EventWriter<MovementAction>,
    action_state_query: Query<&ActionState<ActionKind>>,
) {
    let Ok(action_state) = action_state_query.get_single() else {
        return;
    };
    let left = action_state.pressed(ActionKind::Left);
    let right = action_state.pressed(ActionKind::Right);
    let direction = (right as i8 - left as i8) as Scalar;

    if direction != 0. {
        movement_events.send(MovementAction::Move(direction));
    }

    if action_state.just_pressed(ActionKind::Jump) {
        movement_events.send(MovementAction::Jump);
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct DisabledInputs(HashMap<ActionKind, Timer>);

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct MovementState {
    pub grounded: bool,
    // TODO combine these into one
    pub facing_wall: bool,
    pub wall_left: bool,
    pub wall_right: bool,
    pub falling: bool,
}

// TODO Think about extract movement constants. Jump height, horizontal velocity, wall jump impulse, etc.
fn setup_movement_state(
    player_query: Query<
        (
            Entity,
            &Position,
            &LinearVelocity,
            &ShapeHits,
            &LookDir,
            &ColliderAabb,
        ),
        With<Controllable>,
    >,
    q_terrain: Query<Entity, With<Terrain>>,
    spatial_query: SpatialQuery,
    mut movement_state: ResMut<MovementState>,
) {
    let Ok((player_entity, pos, vel, ground, look_dir, collider_aabb)) = player_query.get_single()
    else {
        return;
    };

    let grounded = !ground.is_empty();
    let falling = vel.y < 0.;

    // Casts a ray just outside the player into the given look direction
    let ray_in_look_dir = |dir: LookDir| -> bool {
        spatial_query
            .cast_ray(
                **pos,
                dir.as_vec(),
                collider_aabb.half_extents().x + 0.5,
                true,
                SpatialQueryFilter::new().without_entities([player_entity]),
            )
            .map_or(false, |hit| q_terrain.get(hit.entity).ok().is_some())
    };

    let facing_wall = ray_in_look_dir(look_dir.clone());
    let wall_left = ray_in_look_dir(LookDir::Left);
    let wall_right = ray_in_look_dir(LookDir::Right);

    *movement_state = MovementState {
        grounded,
        facing_wall,
        wall_left,
        wall_right,
        falling,
    }
}

fn horizontal_movement(
    action_state_query: Query<&ActionState<ActionKind>>,
    mut player_query: Query<
        (&mut LinearVelocity, &mut LookDir),
        (With<Controllable>, Without<PoleClimb>),
    >,
    movement_state: Res<MovementState>,
    disabled_inputs: Res<DisabledInputs>,
) {
    let Ok(action_state) = action_state_query.get_single() else {
        return;
    };

    let Ok((mut vel, mut look_dir)) = player_query.get_single_mut() else {
        return;
    };

    let MovementState {
        falling,
        wall_left: left_wall,
        wall_right: right_wall,
        ..
    } = *movement_state;

    let max_speed = 128.;

    // Disabled movement from wall jump
    let left_enabled = disabled_inputs
        .get(&ActionKind::Left)
        .map_or(true, |timer| timer.finished());
    let right_enabled = disabled_inputs
        .get(&ActionKind::Right)
        .map_or(true, |timer| timer.finished());

    if action_state.pressed(ActionKind::Left) && left_enabled {
        vel.x -= 6.;
        *look_dir = LookDir::Left;

        // Slide down walls
        if left_wall && falling {
            vel.y = -30.;
        }
    }
    if action_state.pressed(ActionKind::Right) && right_enabled {
        vel.x += 6.;
        *look_dir = LookDir::Right;

        // Slide down walls
        if right_wall && falling {
            vel.y = -30.;
        }
    }

    // Never exceed max speed
    vel.x = vel.x.clamp(-max_speed, max_speed);
}

fn jump(
    action_state_query: Query<&ActionState<ActionKind>>,
    mut player_query: Query<&mut LinearVelocity, (With<Controllable>, Without<PoleClimb>)>,
    time: Res<Time>,
    movement_state: Res<MovementState>,
    mut jump_extender: Local<Stopwatch>,
    mut coyote: Local<Stopwatch>,
) {
    let Ok(action_state) = action_state_query.get_single() else {
        return;
    };

    let Ok(mut vel) = player_query.get_single_mut() else {
        return;
    };

    jump_extender.tick(time.delta());
    coyote.tick(time.delta());

    let MovementState {
        grounded, falling, ..
    } = *movement_state;

    if grounded {
        coyote.reset();
    }

    let can_coyote = coyote.elapsed_secs() < 0.15 && falling;
    let can_jump = grounded || can_coyote;

    if action_state.just_pressed(ActionKind::Jump) && can_jump {
        vel.y = 96.;
        jump_extender.reset();
    }

    // Hold jump to extend height
    if action_state.pressed(ActionKind::Jump) && !grounded && jump_extender.elapsed_secs() < 0.2 {
        vel.y += 4.;
    }
}

fn wall_jump(
    action_state_query: Query<&ActionState<ActionKind>>,
    mut player_query: Query<
        (&mut LinearVelocity, &LookDir),
        (With<Controllable>, Without<PoleClimb>),
    >,
    time: Res<Time>,
    movement_state: Res<MovementState>,
    mut disabled_inputs: ResMut<DisabledInputs>,
) {
    let Ok(action_state) = action_state_query.get_single() else {
        return;
    };

    let Ok((mut vel, look_dir)) = player_query.get_single_mut() else {
        return;
    };

    for disabled_input in disabled_inputs.iter_mut() {
        disabled_input.1.tick(time.delta());
    }

    let MovementState {
        facing_wall: near_wall,
        ..
    } = *movement_state;

    if action_state.just_pressed(ActionKind::Jump) {
        let press_in_look_dir = action_state.pressed(look_dir.as_action_kind());
        if press_in_look_dir && near_wall {
            vel.y = 128.;
            **vel += look_dir.opposite().as_vec() * 192.;
            disabled_inputs.insert(
                look_dir.as_action_kind(),
                Timer::new(Duration::from_secs_f32(0.5), TimerMode::Once),
            );
        }
    }
}

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct PoleClimb(pub PoleType);

fn pole_climb(
    mut cmds: Commands,
    poles: Query<&Pole>,
    mut player: Query<
        (Entity, &CollidingEntities, Option<&mut PoleClimb>),
        (With<Controllable>, With<Player>),
    >,
    action_state_query: Query<&ActionState<ActionKind>>,
) {
    let Ok(action_state) = action_state_query.get_single() else {
        return;
    };
    let Ok((player, colliding, climb)) = player.get_single_mut() else {
        return;
    };

    let mut pole = None;
    colliding.0.iter().for_each(|other| {
        if let Ok(p) = poles.get(*other) {
            match pole {
                None => pole = Some(p),
                Some(_) => match p.0 {
                    PoleType::Combined => pole = Some(p),
                    _ => {}
                },
            };
        };
    });

    if let Some(pole) = pole {
        if let Some(mut climb) = climb {
            climb.0 = pole.0;
        } else {
            if action_state.pressed(ActionKind::Up) {
                cmds.entity(player).insert(PoleClimb(pole.0));
            }
        }
    } else {
        cmds.entity(player).remove::<PoleClimb>();
    }
}

fn pole_movement(
    mut cmds: Commands,
    mut player: Query<
        (Entity, &mut LinearVelocity, &mut GravityScale, &PoleClimb),
        (With<Controllable>, With<Player>),
    >,
    action_state_query: Query<&ActionState<ActionKind>>,
) {
    let Ok(action_state) = action_state_query.get_single() else {
        return;
    };

    let Ok((player, mut vel, mut gravity, climb)) = player.get_single_mut() else {
        return;
    };
    gravity.0 = 0.;

    match climb.0 {
        PoleType::Vertical => {
            vel.0 = Vec2::ZERO;
            if action_state.pressed(ActionKind::Up) {
                vel.y = 64.;
            }

            if action_state.pressed(ActionKind::Down) {
                vel.y = -64.;
            }
        }
        PoleType::Horizontal => {
            vel.0 = Vec2::ZERO;
            if action_state.pressed(ActionKind::Left) {
                vel.x = -64.;
            }

            if action_state.pressed(ActionKind::Right) {
                vel.x = 64.;
            }
        }
        PoleType::Combined => {
            vel.0 = Vec2::ZERO;
            if action_state.pressed(ActionKind::Up) {
                vel.y = 64.;
            }

            if action_state.pressed(ActionKind::Down) {
                vel.y = -64.;
            }
            if action_state.pressed(ActionKind::Left) {
                vel.x = -64.;
            }

            if action_state.pressed(ActionKind::Right) {
                vel.x = 64.;
            }
        }
    }

    if action_state.just_pressed(ActionKind::Jump) {
        cmds.entity(player).remove::<PoleClimb>();
    }
}

fn pole_gravity(mut climb: RemovedComponents<PoleClimb>, mut gravity: Query<&mut GravityScale>) {
    for e in climb.iter() {
        if let Some(mut gravity) = gravity.get_mut(e).ok() {
            gravity.0 = 1.0;
        }
    }
}
