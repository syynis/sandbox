use std::time::Duration;

use bevy::{prelude::*, time::Stopwatch, utils::hashbrown::HashMap};
use bevy_xpbd_2d::{math::Vector, prelude::*};
use leafwing_input_manager::prelude::*;

use super::terrain::{Pole, Terrain};

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<ActionKind>::default());
        app.add_systems(Update, movement);
    }
}

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

fn movement(
    action_state_query: Query<&ActionState<ActionKind>>,
    mut query_player: Query<
        (
            Entity,
            &Position,
            &mut LinearVelocity,
            &ShapeHits,
            &mut LookDir,
        ),
        With<Controllable>,
    >,
    q_terrain: Query<Entity, With<Terrain>>,
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
        for (player_entity, pos, mut vel, ground, mut look_dir) in query_player.iter_mut() {
            let grounded = !ground.is_empty();
            if grounded {
                coyote.reset();
            }

            let air_resistance = if !grounded { 8. } else { 0. };
            let falling = vel.y < 0.;
            let can_coyote = coyote.elapsed_secs() < 0.15 && falling;
            let can_jump = grounded || can_coyote;
            if action_state.just_pressed(ActionKind::Jump) && can_jump {
                vel.y = 96.;
                jump_extender.reset();
            }

            let near_wall = if let Some(hit) = spatial_query.cast_ray(
                **pos,
                look_dir.as_vec(),
                7.5,
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
                if near_wall || was_near_wall {
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

#[derive(Component)]
pub struct PoleClimb;

fn handle_pole_climb(poles: Query<&Pole>, player: Query<(Entity), With<Controllable>>) {}
