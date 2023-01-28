use bevy::prelude::*;

use self::verlet::VerletPlugin;

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

#[derive(Resource, Deref, DerefMut)]
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

pub struct PhysPlugin;

impl Plugin for PhysPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(VerletPlugin)
            .insert_resource(PhysSettings::default());
    }
}
