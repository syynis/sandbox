use bevy::prelude::*;
use bevy_xpbd_2d::{prelude::*, PostProcessCollisions};

use self::{movement::MovementPlugin, terrain::handle_platforms};

pub mod movement;
pub mod spatial;
pub mod terrain;
pub mod verlet;

pub struct PhysPlugin;

impl Plugin for PhysPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MovementPlugin, PhysicsPlugins::default()));
        app.add_systems(PostProcessCollisions, handle_platforms);
    }
}
