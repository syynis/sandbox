use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    editor::{EditorEvent, EditorState, PickerEvent, WorldMapExt},
    file_picker,
};

use super::{
    widget::{basic_widget, BasicWidget},
    ConfirmationDialog,
};

#[derive(Default, Clone)]
pub struct EditorMenuBar;

// Inside the ListView widget:
impl BasicWidget for EditorMenuBar {
    fn new(_world: &mut World, _ui: &egui::Ui) -> Self {
        Self::default()
    }
    fn draw(&mut self, world: &mut World, ui: &mut egui::Ui, _id: egui::Id) {
        egui::menu::bar(ui, |ui| {
            egui::menu::menu_button(ui, "File", |ui| {
                let id = ui.id().with("file");
                basic_widget::<New>(world, ui, id.with("map_new"));
                basic_widget::<Open>(world, ui, id.with("map_open"));
                ui.separator();
                basic_widget::<Save>(world, ui, id.with("map_save"));
                basic_widget::<SaveAs>(world, ui, id.with("map_save_as"));
                ui.separator();
                basic_widget::<Close>(world, ui, id.with("map_close"));
                basic_widget::<Quit>(world, ui, id.with("quit"));
            });
            egui::menu::menu_button(ui, "Edit", |ui| {
                let id = ui.id().with("edit");
                basic_widget::<Undo>(world, ui, id.with("undo"));
                basic_widget::<Redo>(world, ui, id.with("redo"));
                ui.separator();
                basic_widget::<Cut>(world, ui, id.with("cut"));
                basic_widget::<Copy>(world, ui, id.with("copy"));
                basic_widget::<Paste>(world, ui, id.with("paste"));
            });
        });
    }
}

#[derive(Default, Clone)]
pub struct New;

impl BasicWidget for New {
    fn new(_world: &mut World, _ui: &egui::Ui) -> Self {
        Self::default()
    }
    fn draw(&mut self, world: &mut World, ui: &mut egui::Ui, _id: egui::Id) {
        if !ui.button("New Map").clicked() {
            return;
        }

        let state = world.resource::<EditorState>();
        if state.unsaved_changes {
            let (save_label, save_event) = match &state.current_loaded_path {
                Some(path) => ("Save", EditorEvent::Save(path.clone())),
                None => ("Save As...", EditorEvent::SaveAs),
            };
            let dialog = ConfirmationDialog::new(
                "Warning: Unsaved Changes",
                "There are unsaved changes to this map.  Would you like to save them?",
            )
            .button("Cancel", None)
            .button("Discard Changes", Some(EditorEvent::New))
            .button(save_label, Some(save_event));

            world.spawn(dialog);
        }

        world.send_event(EditorEvent::New);

        ui.close_menu();
    }
}

#[derive(Default, Clone)]
pub struct Open;

impl BasicWidget for Open {
    fn new(_world: &mut World, _ui: &egui::Ui) -> Self {
        Self::default()
    }
    fn draw(&mut self, world: &mut World, ui: &mut egui::Ui, _id: egui::Id) {
        if ui.button("Open Map...").clicked() {
            world.spawn(file_picker::Picker::new(PickerEvent::Load(None)).build());
            ui.close_menu();
        }
    }
}

#[derive(Default, Clone)]
pub struct Save;

impl BasicWidget for Save {
    fn new(_world: &mut World, _ui: &egui::Ui) -> Self {
        Self::default()
    }
    fn draw(&mut self, world: &mut World, ui: &mut egui::Ui, _id: egui::Id) {
        let state = world.resource::<EditorState>();

        let Some(path) = &state.current_loaded_path else {
            if ui
                .add_enabled(false, egui::Button::new("Save Map"))
                .clicked()
            {
                unreachable!();
            }
            return;
        };

        if ui.button("Save Map").clicked() {
            let event = EditorEvent::Save(path.clone());
            world.send_event(event);
            ui.close_menu();
        }
    }
}

#[derive(Default, Clone)]
pub struct SaveAs;

impl BasicWidget for SaveAs {
    fn new(_world: &mut World, _ui: &egui::Ui) -> Self {
        Self::default()
    }
    fn draw(&mut self, mut world: &mut World, ui: &mut egui::Ui, _id: egui::Id) {
        if world.get_map().is_err() {
            if ui
                .add_enabled(false, egui::Button::new("Save As..."))
                .clicked()
            {
                unreachable!();
            }
            return;
        };

        if ui.button("Save As...").clicked() {
            world.send_event(EditorEvent::SaveAs);
            ui.close_menu();
        }
    }
}

#[derive(Default, Clone)]
pub struct Close;

impl BasicWidget for Close {
    fn new(_world: &mut World, _ui: &egui::Ui) -> Self {
        Self::default()
    }

    fn draw(&mut self, mut world: &mut World, ui: &mut egui::Ui, _id: egui::Id) {
        if world.get_map().is_err() {
            if ui
                .add_enabled(false, egui::Button::new("Close Map"))
                .clicked()
            {
                unreachable!();
            }
            return;
        };

        if !ui.button("Close Map").clicked() {
            return;
        }

        let state = world.resource::<EditorState>();
        if state.unsaved_changes {
            let (save_label, save_event) = match &state.current_loaded_path {
                Some(path) => ("Save", EditorEvent::Save(path.clone())),
                None => ("Save As...", EditorEvent::SaveAs),
            };
            let dialog = ConfirmationDialog::new(
                "Warning: Unsaved Changes",
                "There are unsaved changes to this map.  Would you like to save them?",
            )
            .button("Cancel", None)
            .button("Discard Changes", Some(EditorEvent::Close))
            .button(save_label, Some(save_event));

            world.spawn(dialog);
        }

        world.send_event(EditorEvent::Close);

        ui.close_menu();
    }
}

#[derive(Default, Clone)]
pub struct Quit;

impl BasicWidget for Quit {
    fn new(_world: &mut World, _ui: &egui::Ui) -> Self {
        Self::default()
    }

    fn draw(&mut self, _world: &mut World, ui: &mut egui::Ui, _id: egui::Id) {
        if ui.button("Quit").clicked() {
            debug!("quit");
            ui.close_menu();
            std::process::exit(0);
        }
    }
}

#[derive(Default, Clone)]
pub struct Undo;

impl BasicWidget for Undo {
    fn new(_world: &mut World, _ui: &egui::Ui) -> Self {
        Self::default()
    }

    fn draw(&mut self, _world: &mut World, ui: &mut egui::Ui, _id: egui::Id) {
        if ui.button("Undo").clicked() {
            debug!("undo");
            ui.close_menu();
        }
    }
}

#[derive(Default, Clone)]
pub struct Redo;

impl BasicWidget for Redo {
    fn new(_world: &mut World, _ui: &egui::Ui) -> Self {
        Self::default()
    }

    fn draw(&mut self, _world: &mut World, ui: &mut egui::Ui, _id: egui::Id) {
        if ui.button("Redo").clicked() {
            debug!("redo");
            ui.close_menu();
        }
    }
}

#[derive(Default, Clone)]
pub struct Cut;

impl BasicWidget for Cut {
    fn new(_world: &mut World, _ui: &egui::Ui) -> Self {
        Self::default()
    }

    fn draw(&mut self, _world: &mut World, ui: &mut egui::Ui, _id: egui::Id) {
        if ui.button("Cut").clicked() {
            debug!("cut");
            ui.close_menu();
        }
    }
}

#[derive(Default, Clone)]
pub struct Copy;

impl BasicWidget for Copy {
    fn new(_world: &mut World, _ui: &egui::Ui) -> Self {
        Self::default()
    }

    fn draw(&mut self, _world: &mut World, ui: &mut egui::Ui, _id: egui::Id) {
        if ui.button("Copy").clicked() {
            debug!("copy");
            ui.close_menu();
        }
    }
}

#[derive(Default, Clone)]
pub struct Paste;

impl BasicWidget for Paste {
    fn new(_world: &mut World, _ui: &egui::Ui) -> Self {
        Self::default()
    }

    fn draw(&mut self, _world: &mut World, ui: &mut egui::Ui, _id: egui::Id) {
        if ui.button("Paste").clicked() {
            debug!("paste");
            ui.close_menu();
        }
    }
}
