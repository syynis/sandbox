use bevy::prelude::*;

use crate::{
    editor::{
        ui::{menu::EditorMenuBar, toolbar::EditorToolBar},
        EditorState,
    },
    ui,
};

pub mod menu;
pub mod toolbar;

pub fn draw_ui(world: &mut World) {
    use ui::widget::*;

    ui::with_world_and_egui_context(world, |world, ctx| {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            basic_widget::<EditorMenuBar>(world, ui, ui.id().with("menubar"));
        });

        let state = world.resource_mut::<EditorState>();
        egui::SidePanel::right("right_panel")
            .resizable(true)
            .default_width(250.)
            .show_animated(ctx, state.enabled.tool_panel, |ui| {
                basic_widget::<EditorToolBar>(world, ui, ui.id().with("panel"));
            })
    });
}
