use bevy::{ecs::system::Command, prelude::*};
use bevy_xpbd_2d::prelude::*;

use crate::lifetime::Lifetime;

use super::holdable::Holdable;

pub struct SpawnPebble {
    pub pos: Vec2,
    pub vel: Vec2,
    pub lifetime: Option<f32>,
}

impl Command for SpawnPebble {
    fn apply(self, world: &mut World) {
        let id = world
            .spawn((
                Position(self.pos),
                LinearVelocity(self.vel),
                RigidBody::Dynamic,
                Collider::ball(4.),
                Holdable,
                SpatialBundle::default(),
                Name::new("Pebble"),
            ))
            .id();
        if let Some(lifetime) = self.lifetime {
            world.entity_mut(id).insert(Lifetime::new(lifetime));
        }
    }
}
