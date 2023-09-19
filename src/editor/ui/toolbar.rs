use bevy::prelude::*;
use bevy_egui::egui;

use crate::ui::widget::{basic_widget, BasicWidget};

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

#[derive(Default, Clone)]
pub struct ToolPicker;

impl BasicWidget for ToolPicker {
    fn new(world: &mut World, ui: &egui::Ui) -> Self {
        Self::default()
    }

    fn draw(&mut self, world: &mut World, ui: &mut egui::Ui, id: egui::Id) {
        let mock_tools = vec!["place", "remove", "area", "ramp"];
        ui.add(egui::Button::new("TestTool"));
        let layout = egui::Layout::left_to_right(egui::Align::Min).with_main_wrap(true);
        let drag = egui::LayerId::new(egui::Order::Tooltip, id.with("dragging"));

        ui.with_layout(layout, |ui| {
            for (idx, tool) in mock_tools.iter().enumerate() {
                let button = egui::Button::new(tool.to_owned().clone());

                let res = ui.add(button);

                if res.clicked() {
                    println!("{}", tool.to_owned().clone());
                }
            }
        });
    }
}
