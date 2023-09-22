use anyhow::Context;
use anyhow::Result;
use bevy_ecs_tilemap::tiles::TileStorage;
use std::path::PathBuf;

use bevy::prelude::*;
use leafwing_input_manager::Actionlike;

pub mod tools;
pub mod ui;

use crate::file_picker;

use self::tools::Tool;
use self::tools::ToolId;
use self::tools::ToolSet;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorState>();
    }
}

#[derive(Resource)]
pub struct EditorState {
    pub enabled: EnabledUiElements,
    pub toolset: ToolSet,
    pub active_tool: ToolId,
    pub current_loaded_path: Option<PathBuf>,
    pub unsaved_changes: bool,
    // TODO Layers
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            enabled: EnabledUiElements::default(),
            toolset: ToolSet::default(),
            active_tool: 0,
            current_loaded_path: None,
            unsaved_changes: false,
        }
    }
}

#[derive(Debug)]
pub struct EnabledUiElements {
    pub inspector: bool,
    pub tool_panel: bool,
    pub egui_debug: bool,
}

impl Default for EnabledUiElements {
    fn default() -> Self {
        Self {
            inspector: true,
            tool_panel: true,
            egui_debug: false,
        }
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum EditorActions {
    PlaceTile,
    RemoveTile,
    CycleMode,
    New,
    Close,
    Save,
    SaveAs,
    Load,
}

#[derive(Debug, Clone, Event)]
pub enum EditorEvent {
    New,
    Close,
    Save(PathBuf),
    SaveAs,
    Load(PathBuf),
}

#[derive(Debug, Event)]
pub enum PickerEvent {
    Save(Option<PathBuf>),
    Load(Option<PathBuf>),
}

impl file_picker::PickerEvent for PickerEvent {
    fn set_result(&mut self, result: Vec<PathBuf>) {
        use PickerEvent::*;

        *self = match *self {
            Save(_) => Save(Some(result[0].clone())),
            Load(_) => Load(Some(result[0].clone())),
        };
    }
}

// TODO more principled way to check if we are currently editing a tilemap
pub trait WorldMapExt: Sized {
    fn get_map(&mut self) -> Result<&TileStorage>;
}

impl WorldMapExt for &mut World {
    fn get_map(&mut self) -> Result<&TileStorage> {
        let mut q = self.query::<&TileStorage>();
        q.get_single(self)
            .context("Failed to get single map entity")
    }
}
