use std::sync::Arc;

use bevy::{ecs::system::SystemState, prelude::*};
use bevy_egui::{egui, EguiUserTextures};
use epaint::TextureId;

use crate::{
    editor::EditorState,
    ui::widget::{basic_widget, BasicWidget},
};

#[derive(Default, Clone)]
pub struct EditorToolBar;

impl BasicWidget for EditorToolBar {
    fn new(world: &mut World, ui: &egui::Ui) -> Self {
        Self::default()
    }

    fn draw(&mut self, world: &mut World, ui: &mut egui::Ui, id: egui::Id) {
        egui::ScrollArea::vertical()
            .id_source(id.with("vscroll"))
            .show(ui, |ui| {
                basic_widget::<ToolPicker>(world, ui, id.with("tool_picker"));
                ui.allocate_space(ui.available_size());
            });
        ui.separator();
    }
}

pub struct ToolPicker<'w: 'static> {
    system_state: SystemState<ResMut<'w, EditorState>>,
}

impl<'w> BasicWidget for ToolPicker<'w> {
    fn new(world: &mut World, ui: &egui::Ui) -> Self {
        Self {
            system_state: SystemState::new(world),
        }
    }

    fn draw(&mut self, world: &mut World, ui: &mut egui::Ui, id: egui::Id) {
        let mut editor_state = self.system_state.get_mut(world);
        ui.add(egui::Button::new("TestTool"));
        let layout = egui::Layout::left_to_right(egui::Align::Min).with_main_wrap(true);

        ui.with_layout(layout, |ui| {
            for tool_id in editor_state.toolset.tool_order.clone().iter() {
                let Some(tool_data) = editor_state.toolset.tools.get(tool_id) else {
                    warn!("Tried to access tool that doesnt exist. Id: {}", tool_id);
                    return;
                };
                let Some(texture_id) = tool_data.egui_texture_id else {
                    continue;
                };
                let image_button = egui::ImageButton::new(texture_id, epaint::Vec2::new(32., 32.));
                let res = ui.add(image_button);
                // TODO how to do this
                // res.on_hover_text(tool_data.name.clone());
                if res.clicked() {
                    println!("{}", tool_data.name.clone());
                    editor_state.active_tool = *tool_id;
                    println!("{}", editor_state.active_tool);
                }
            }
        });
    }
}
