use std::path::PathBuf;

use bevy::prelude::*;
use leafwing_input_manager::Actionlike;

pub mod tools;
pub mod ui;

use crate::file_picker;
use crate::level::layer::Layer;

use self::tools::{ToolId, ToolSet};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorState>();
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct EditorState {
    pub enabled: EnabledUiElements,
    pub toolset: ToolSet,
    pub active_tool: ToolId,
    pub current_loaded_path: Option<PathBuf>,
    pub unsaved_changes: bool,
    pub current_layer: Layer,
    // TODO Layers
}

impl EditorState {
    pub fn reset_path(&mut self) {
        self.current_loaded_path = None;
        self.unsaved_changes = false;
    }

    pub fn next_tool(&mut self) {
        self.active_tool = (self.active_tool + 1) % self.toolset.tools.len();
    }

    pub fn next_layer(&mut self) {
        self.current_layer = self.current_layer.next();
    }
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            enabled: EnabledUiElements::default(),
            toolset: ToolSet::default(),
            active_tool: 0,
            current_loaded_path: None,
            unsaved_changes: false,
            current_layer: Layer::World,
        }
    }
}

#[derive(Debug, Reflect)]
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
    ApplyTool,
    Area,
    Close,
    CycleTool,
    CycleLayer,
    Load,
    New,
    Save,
    SaveAs,
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum ToolActions {
    CycleMode,
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
