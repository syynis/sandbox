use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;
use leafwing_input_manager::prelude::*;

use crate::level::tpos_wpos;

use super::{
    history::{HandleHistoryEvents, History, HistoryEvent},
    Dir, Pos, SokobanBlock, SokobanEvents,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerActions>::default())
            .add_systems(Startup, setup)
            .add_systems(Update, handle_player_actions.before(HandleHistoryEvents));
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
        SokobanBlock::Dynamic,
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
    mut player_q: Query<Entity, With<Player>>,
    player_actions: Query<&ActionState<PlayerActions>>,
    mut history_events: EventWriter<HistoryEvent>,
    mut sokoban: SokobanEvents,
) {
    let Some(player) = player_q.get_single_mut().ok() else {
        return;
    };

    let Some(player_actions) = player_actions.get_single().ok() else {
        return;
    };

    player_actions.get_just_pressed().iter().for_each(|action| {
        match action {
            PlayerActions::Up => sokoban.move_entity(player, Dir::Up),
            PlayerActions::Right => sokoban.move_entity(player, Dir::Right),
            PlayerActions::Down => sokoban.move_entity(player, Dir::Down),
            PlayerActions::Left => sokoban.move_entity(player, Dir::Left),
        };
    });
    if player_actions.get_just_pressed().len() > 0 {
        history_events.send(HistoryEvent::Record)
    }
}
