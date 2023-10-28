use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;
use leafwing_input_manager::prelude::*;

use crate::level::tpos_wpos;

use super::{
    history::{HandleHistoryEvents, History, HistoryEvent},
    Pos,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerActions>::default());
        app.add_systems(Startup, setup);
        app.add_systems(
            Update,
            (
                handle_player_actions,
                handle_history
                    .before(HandleHistoryEvents)
                    .after(handle_player_actions),
            ),
        );
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Actionlike, Clone, Copy, Hash, Debug, PartialEq, Eq, Reflect)]
pub enum PlayerActions {
    Up,
    Right,
    Down,
    Left,
}

fn setup(mut cmds: Commands) {
    cmds.spawn((
        (InputManagerBundle::<PlayerActions> {
            input_map: player_actions(),
            ..default()
        },),
        Name::new("PlayerActions"),
    ));
    cmds.spawn((
        Name::new("Player"),
        Player,
        Pos::default(),
        History::<Pos>::default(),
        SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2 { x: 16., y: 16. }),
                ..default()
            },
            transform: Transform::from_translation(tpos_wpos(&TilePos::default()).extend(1.)),
            ..default()
        },
    ));
}

fn player_actions() -> InputMap<PlayerActions> {
    use PlayerActions::*;
    let mut input_map = InputMap::default();

    input_map.insert(KeyCode::W, Up);
    input_map.insert(KeyCode::D, Right);
    input_map.insert(KeyCode::S, Down);
    input_map.insert(KeyCode::A, Left);

    input_map
}

pub fn handle_player_actions(
    mut player_q: Query<&mut Pos, With<Player>>,
    player_actions: Query<&ActionState<PlayerActions>>,
) {
    let Some(mut player_pos) = player_q.get_single_mut().ok() else {
        return;
    };

    let Some(player_actions) = player_actions.get_single().ok() else {
        return;
    };

    player_actions
        .get_just_pressed()
        .iter()
        .for_each(|action| match action {
            PlayerActions::Up => player_pos.y += 1,
            PlayerActions::Right => player_pos.x += 1,
            PlayerActions::Down => player_pos.y = player_pos.y.saturating_sub(1),
            PlayerActions::Left => player_pos.x = player_pos.x.saturating_sub(1),
        });
}

pub fn handle_history(
    mut history_events: EventWriter<HistoryEvent>,
    mut player_q: Query<&Pos, (Changed<Pos>, With<Player>)>,
) {
    if player_q.get_single_mut().is_ok() {
        history_events.send(HistoryEvent::Record);
    };
}
