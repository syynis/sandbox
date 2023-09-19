use crate::ui::widget::BasicWidget;
use bevy_egui::egui;

use super::Tool;

#[derive(Debug, Default)]
pub struct SlopeTool;

impl Tool for SlopeTool {
    fn apply(&mut self, world: &mut bevy::prelude::World) {
        println!("apply slope");
    }
}
