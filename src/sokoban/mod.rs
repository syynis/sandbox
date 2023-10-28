use bevy::{ecs::system::SystemParam, prelude::*};

use crate::grid::Grid;

#[derive(Debug, Copy, Clone)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl From<Direction> for IVec2 {
    fn from(direction: Direction) -> IVec2 {
        match direction {
            Direction::Up => IVec2::Y,
            Direction::Left => IVec2::new(-1, 0),
            Direction::Down => IVec2::new(0, -1),
            Direction::Right => IVec2::X,
        }
    }
}

#[derive(Debug, Clone, Event)]
pub enum SokobanCommand {
    Move {
        entity: Entity,
        direction: Direction,
    },
}

#[derive(SystemParam)]
pub struct SokobanCommands<'w> {
    writer: EventWriter<'w, SokobanCommand>,
}

impl<'w> SokobanCommands<'w> {
    pub fn move_block(&mut self, entity: Entity, direction: Direction) {
        self.writer.send(SokobanCommand::Move { entity, direction });
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
    pub direction: Direction,
    pub pushed: Vec<Entity>,
}

#[derive(Resource)]
pub struct CollisionMap {
    map: Grid<Option<(Entity, SokobanBlock)>>,
}

impl CollisionMap {
    fn push_collision_map_entry(&mut self, pusher_coords: IVec2, direction: Direction) {
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
