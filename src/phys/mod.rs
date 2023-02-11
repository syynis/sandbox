use bevy::prelude::*;

use self::{
    movement::{LinearVelocity, MovementPlugin},
    verlet::VerletPlugin,
};

pub mod movement;
pub mod spatial;
pub mod verlet;

pub enum Gravity {
    Dir(Vec2),
    None,
}

impl Gravity {
    fn acceleration(&self) -> Vec2 {
        match self {
            Gravity::Dir(dir) => *dir,
            Gravity::None => Vec2::ZERO,
        }
    }
}

#[derive(Resource)]
pub struct PhysSettings {
    pub gravity: Gravity,
}

impl Default for PhysSettings {
    fn default() -> Self {
        Self {
            gravity: Gravity::None,
        }
    }
}

fn apply_gravity(
    mut query: Query<&mut LinearVelocity>,
    phys_settings: Res<PhysSettings>,
    time: Res<Time>,
) {
    for mut vel in query.iter_mut() {
        **vel += phys_settings.gravity.acceleration() * time.delta_seconds();
    }
}

pub struct PhysPlugin;

impl Plugin for PhysPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(VerletPlugin)
            .add_plugin(MovementPlugin)
            .add_system(apply_gravity)
            .insert_resource(PhysSettings {
                gravity: Gravity::Dir(Vec2::new(0.0, /*-9.807*/ 0.)),
            });
    }
}
