use super::Tool;
use bevy::prelude::*;

pub struct SlopeTool;

impl Tool for SlopeTool {
    fn new(world: &mut bevy::prelude::World) -> Self {
        SlopeTool
    }
    fn apply(&mut self, world: &mut World) {
        println!("apply slope");
    }
}
