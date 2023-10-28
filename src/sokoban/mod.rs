use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_ecs_tilemap::tiles::TilePos;
use leafwing_input_manager::prelude::*;

use crate::{grid::Grid, level::tpos_wpos};

use self::{
    history::{HandleHistoryEvents, History, HistoryEvent, HistoryPlugin},
    player::PlayerPlugin,
};

pub mod history;
pub mod player;

pub struct SokobanPlugin;

impl Plugin for SokobanPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PlayerPlugin,
            HistoryPlugin::<Pos>::default(),
            InputManagerPlugin::<SokobanActions>::default(),
        ))
        .register_type::<Pos>().register_type::<History<Pos>>()
        .add_event::<SokobanEvent>()
        .add_systems(Startup, setup)
        .add_systems(Update, handle_sokoban_actions.before(HandleHistoryEvents))
        .add_systems(Update, handle_sokoban_events.run_if(on_event::<SokobanEvent>()))
        .add_systems(PostUpdate, copy_pos_to_transform);
    }
}

#[derive(Actionlike, Clone, Copy, Hash, Debug, PartialEq, Eq, Reflect)]
pub enum SokobanActions {
    Rewind,
}

fn sokoban_actions() -> InputMap<SokobanActions> {
    use SokobanActions::*;
    let mut input_map = InputMap::default();

    input_map.insert(KeyCode::U, Rewind);

    input_map
}

fn setup(mut cmds: Commands) {
    cmds.spawn((
        (InputManagerBundle::<SokobanActions> {
            input_map: sokoban_actions(),
            ..default()
        },),
        Name::new("SokobanActions"),
    ));
}

fn handle_sokoban_actions(
    actions: Query<&ActionState<SokobanActions>>,
    mut history_events: EventWriter<HistoryEvent>,
) {
    let Some(actions) = actions.get_single().ok() else {
        return;
    };
    if actions.just_pressed(SokobanActions::Rewind) {
        history_events.send(HistoryEvent::Rewind)
    }
}

#[derive(Component, Default, Clone, Copy, Debug, PartialEq, Eq, Deref, DerefMut, Reflect)]
pub struct Pos(pub TilePos);

pub fn copy_pos_to_transform(mut query: Query<(&Pos, &mut Transform)>) {
    for (pos, mut transform) in query.iter_mut() {
        transform.translation = tpos_wpos(pos).extend(transform.translation.z);
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Dir {
    Up,
    Right,
    Down,
    Left,
}

impl From<Dir> for IVec2 {
    fn from(direction: Dir) -> IVec2 {
        match direction {
            Dir::Up => IVec2::Y,
            Dir::Left => IVec2::new(-1, 0),
            Dir::Down => IVec2::new(0, -1),
            Dir::Right => IVec2::X,
        }
    }
}

#[derive(Debug, Clone, Event)]
pub enum SokobanEvent {
    Move { entity: Entity, direction: Dir },
}

#[derive(SystemParam)]
pub struct SokobanEvents<'w> {
    writer: EventWriter<'w, SokobanEvent>,
}

impl<'w> SokobanEvents<'w> {
    pub fn move_entity(&mut self, entity: Entity, direction: Dir) {
        self.writer.send(SokobanEvent::Move { entity, direction });
    }
}

fn handle_sokoban_events(
    mut sokoban_entities: Query<(Entity, &mut Pos, &SokobanBlock)>,
    mut sokoban_events: EventReader<SokobanEvent>,
) {
    for ev in sokoban_events.iter() {
        let SokobanEvent::Move { entity, direction } = ev;

        if let Some((entity, mut pos, block)) = sokoban_entities.get_mut(*entity).ok() {
            let dir = IVec2::from(*direction);
            pos.x = pos.x.saturating_add_signed(dir.x);
            pos.y = pos.y.saturating_add_signed(dir.y);
        }
    }
}

#[derive(Debug, Copy, Clone, Component)]
pub enum SokobanBlock {
    Static,
    Dynamic,
}

#[derive(Clone, Default, Debug, Component)]
pub struct Pusher;

#[derive(Debug, Clone)]
pub struct PushEvent {
    pub pusher: Entity,
    pub direction: Dir,
    pub pushed: Vec<Entity>,
}

#[derive(Resource)]
pub struct CollisionMap {
    map: Grid<Option<(Entity, SokobanBlock)>>,
}

impl CollisionMap {
    fn push_collision_map_entry(&mut self, pusher_coords: IVec2, direction: Dir) {
        let Some(e) = self.map.get_mut(pusher_coords) else {
            return;
        };

        match e {
            Some((pusher, SokobanBlock::Dynamic)) => {
                // pusher is dynamic, so we try to push
                let destination = pusher_coords + IVec2::from(direction);
                let val = e.take();
                self.map.set(destination, val);
            }
            Some((_, SokobanBlock::Static)) => {}
            None => {}
        }
    }
}
