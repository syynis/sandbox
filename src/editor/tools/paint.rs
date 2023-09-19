use crate::ui::widget::BasicWidget;
use bevy_egui::egui;

use super::Tool;

#[derive(Debug, Default)]
pub struct PaintTool;

impl Tool for PaintTool {
    fn apply(&mut self, world: &mut bevy::prelude::World) {
        println!("apply paint");
    }
}
