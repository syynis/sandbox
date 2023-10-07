use std::time::Duration;

use bevy::prelude::*;

pub struct LifetimePlugin;

impl Plugin for LifetimePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, handle_lifetimes);
    }
}

#[derive(Component)]
pub struct Lifetime {
    pub lifetime: Timer,
}

impl Lifetime {
    pub fn new(lifetime: f32) -> Self {
        Self {
            lifetime: Timer::new(Duration::from_secs_f32(lifetime), TimerMode::Once),
        }
    }
}

pub fn handle_lifetimes(
    mut cmds: Commands,
    mut lifetimes: Query<(Entity, &mut Lifetime)>,
    time: Res<Time>,
) {
    // TODO
    for (entity, mut lifetime) in lifetimes.iter_mut() {
        lifetime.lifetime.tick(time.delta());
        if lifetime.lifetime.finished() {
            cmds.entity(entity).despawn_recursive();
        }
    }
}
