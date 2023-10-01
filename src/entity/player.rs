use bevy::{
    ecs::{query::WorldQuery, system::Command},
    prelude::*,
};
use bevy_xpbd_2d::{math::Vector, prelude::*};

use crate::phys::{
    movement::{Control, LookDir},
    terrain::PlatformPass,
};

use super::holdable::CanHold;

#[derive(WorldQuery)]
pub struct PlayerQuery {
    entity: Entity,
    player: &'static Player,
}

#[derive(Component)]
pub struct Player;

pub struct SpawnPlayerCommand<B: Bundle> {
    pub pos: Vector,
    pub size: Vector,
    pub extra: B,
}

impl<B: Bundle> SpawnPlayerCommand<B> {
    pub fn new(pos: Vector, size: Vector, extra: B) -> Self {
        Self { pos, size, extra }
    }
}

impl<B: Bundle> Command for SpawnPlayerCommand<B> {
    fn apply(self, world: &mut World) {
        world.spawn((
            Player,
            Position(self.pos),
            Collider::cuboid(self.size.x, self.size.y),
            RigidBody::Dynamic,
            LockedAxes::new().lock_rotation(),
            TransformBundle::default(),
            ShapeCaster::new(
                Collider::cuboid(self.size.x - 0.5, self.size.y - 0.5),
                Vector::NEG_Y * 0.05,
                0.,
                Vector::NEG_Y,
            )
            .with_ignore_origin_penetration(true) // Don't count player's collider
            .with_max_time_of_impact(0.2)
            .with_max_hits(1),
            Control::default(),
            Friction::new(0.),
            LookDir::Right,
            PlatformPass,
            GravityScale(1.),
            CanHold,
            VisibilityBundle::default(),
            self.extra,
        ));
    }
}

pub struct DespawnPlayerCommand;

impl Command for DespawnPlayerCommand {
    fn apply(self, world: &mut World) {
        if let Ok(q) = world.query::<PlayerQuery>().get_single(world) {
            // TODO despawn children
            world.despawn(q.entity);
        };
    }
}
