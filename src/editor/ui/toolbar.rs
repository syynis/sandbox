use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    editor::EditorState,
    level::layer::ALL_LAYERS,
    ui::{
        widget::{basic_widget, fn_widget, BasicWidget},
        widgets::PanelTitle,
    },
};

#[derive(Default, Clone)]
pub struct EditorToolBar;

impl BasicWidget for EditorToolBar {
    fn new(_: &mut World, _: &egui::Ui) -> Self {
        Self::default()
    }

    fn draw(&mut self, world: &mut World, ui: &mut egui::Ui, id: egui::Id) {
        egui::ScrollArea::vertical()
            .id_source(id.with("vscroll"))
            .show(ui, |ui| {
                basic_widget::<ToolPicker>(world, ui, id.with("tool_picker"));
            });
        ui.separator();
        basic_widget::<LayersPanel>(world, ui, id);
    }
}

#[derive(Default)]
pub struct ToolPicker;

impl BasicWidget for ToolPicker {
    fn new(_: &mut World, _: &egui::Ui) -> Self {
        Self::default()
    }

    fn draw(&mut self, world: &mut World, ui: &mut egui::Ui, _: egui::Id) {
        let mut editor_state = world.resource_mut::<EditorState>();
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

                res.clone().on_hover_text(tool_data.name.clone());
                if res.clone().clicked() {
                    editor_state.active_tool = *tool_id;
                }
            }
        });
    }
}

#[derive(Default)]
pub struct LayersPanel;

impl BasicWidget for LayersPanel {
    fn new(_: &mut World, _: &egui::Ui) -> Self {
        Self::default()
    }

    fn draw(&mut self, world: &mut World, ui: &mut egui::Ui, id: egui::Id) {
        fn_widget::<PanelTitle>(world, ui, id.with("title"), "Layers");
        egui::ScrollArea::vertical()
            .max_height(ui.available_height() - 25.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                basic_widget::<LayersList>(world, ui, id.with("layer_list"));
                ui.allocate_space(ui.available_size());
            });
    }
}

#[derive(Default)]
pub struct LayersList;

impl BasicWidget for LayersList {
    fn new(_: &mut World, _: &egui::Ui) -> Self {
        Self::default()
    }

    fn draw(&mut self, world: &mut World, ui: &mut egui::Ui, _: egui::Id) {
        let state = world.resource::<EditorState>();
        let mut current_layer = state.current_layer;
        let mut changed = false;
        let layout = egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(true);
        ui.with_layout(layout, |ui| {
            for layer in ALL_LAYERS.iter() {
                changed |= ui
                    .selectable_value(&mut current_layer, *layer, layer.name())
                    .changed();
            }
        });

        if changed {
            let mut state = world.resource_mut::<EditorState>();
            state.current_layer = current_layer;
        }
    }
}
