use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    editor::{
        tools::area::{ActiveMode, ALL_MODES},
        EditorState,
    },
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
        basic_widget::<LayersPanel>(world, ui, id.with("layers"));
        ui.separator();
        basic_widget::<AreaToolPanel>(world, ui, id.with("area_tool"));
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
                let image_button = egui::ImageButton::new(texture_id, epaint::Vec2::new(32., 32.))
                    .selected(*tool_id == editor_state.active_tool);

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
            .id_source("layers_scroll")
            .auto_shrink([false, true])
            .show(ui, |ui| {
                basic_widget::<LayersList>(world, ui, id.with("layer_list"));
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

#[derive(Default)]
pub struct AreaToolPanel;

impl BasicWidget for AreaToolPanel {
    fn new(_: &mut World, _: &egui::Ui) -> Self {
        Self::default()
    }

    fn draw(&mut self, world: &mut World, ui: &mut egui::Ui, id: egui::Id) {
        fn_widget::<PanelTitle>(world, ui, id.with("title"), "Area Tool Mode");
        egui::ScrollArea::vertical()
            .id_source("area_tool")
            .auto_shrink([false, true])
            .show(ui, |ui| {
                let mode = world.resource::<ActiveMode>();
                let mut current_mode = **mode;
                let mut changed = false;
                let layout = egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(true);
                ui.with_layout(layout, |ui| {
                    for mode in ALL_MODES.iter() {
                        changed |= ui
                            .selectable_value(&mut current_mode, *mode, mode.name())
                            .changed();
                    }
                });

                if changed {
                    let mut mode = world.resource_mut::<ActiveMode>();
                    mode.0 = current_mode;
                }
                ui.allocate_space(ui.available_size());
            });
    }
}
