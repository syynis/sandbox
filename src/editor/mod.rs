use std::path::PathBuf;

use bevy::{asset::LoadState, prelude::*};
use bevy_common_assets::ron::RonAssetPlugin;
use leafwing_input_manager::Actionlike;

pub mod palette;
pub mod render;
pub mod tiles;
pub mod tools;
pub mod ui;

use crate::file_picker;
use crate::level::layer::Layer;

use self::{
    palette::{load_palette_image, parse_palette_image, Palette, PaletteHandle, PaletteRows},
    tiles::{load_manifests, load_tiles, Manifests, TileManifest, Tiles},
    tools::{area::ActiveMode, ToolId, ToolSet},
};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<TileManifest>::new(&["manifest.ron"]));
        app.register_type::<Palette>();
        app.register_type::<PaletteRows>();

        app.add_state::<AppState>();
        app.init_resource::<EditorState>();
        app.init_resource::<ActiveMode>();
        app.init_resource::<Manifests>();
        app.init_resource::<Tiles>();
        app.init_resource::<Palette>();

        // Loading state
        app.add_systems(
            OnEnter(AppState::Loading),
            (load_palette_image, load_manifests),
        );
        app.add_systems(
            Update,
            (parse_palette_image, load_tiles, finished_loading).run_if(in_state(AppState::Loading)),
        );
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Loading,
    Display,
}

fn finished_loading(
    mut next_state: ResMut<NextState<AppState>>,
    asset_server: Res<AssetServer>,
    tiles: Res<Tiles>,
    palette: Res<PaletteHandle>,
) {
    let tiles_loaded =
        match asset_server.get_group_load_state(tiles.0.values().map(|handle| handle.0.id())) {
            LoadState::Loaded => true,
            LoadState::Failed => {
                bevy::log::error!("Failed to load tile asset");
                false
            }
            _ => false,
        };

    let palette_loaded = match asset_server.get_load_state(palette.0.id()) {
        LoadState::Loaded => true,
        LoadState::Failed => {
            bevy::log::error!("Failed to load palette image");
            false
        }
        _ => false,
    };

    if palette_loaded && tiles_loaded {
        next_state.set(AppState::Display);
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
    CycleToolMode,
    Load,
    New,
    Save,
    SaveAs,
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
