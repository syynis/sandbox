use bevy::{ecs::system::Command, prelude::*};
use bevy_xpbd_2d::prelude::*;

use super::holdable::Holdable;

pub struct SpawnPebble {
    pub pos: Vec2,
    pub vel: Vec2,
}

impl Command for SpawnPebble {
    fn apply(self, world: &mut World) {
        world.spawn((
            Position(self.pos),
            LinearVelocity(self.vel),
            RigidBody::Dynamic,
            Collider::ball(4.),
            Holdable,
            SpatialBundle::default(),
        ));
    }
}
