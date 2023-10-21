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
    palette::{load_palette_image, parse_palette_image, Palette, PaletteHandles, Palettes},
    render::MapImages,
    tiles::{load_manifests, load_tile_images, load_tiles, Manifest, Manifests, Materials, Tiles},
    tools::{area::ActiveMode, ToolId, ToolSet},
};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<Manifest>::new(&["manifest.ron"]));

        app.register_type::<MapImages>();
        app.register_type::<Palette>();
        app.register_type::<Palettes>();

        app.add_state::<AppState>();
        app.init_resource::<EditorState>();
        app.init_resource::<ActiveMode>();
        app.init_resource::<Manifests>();

        // Loading state
        app.add_systems(
            OnEnter(AppState::Loading),
            (load_palette_image, load_manifests),
        );
        app.add_systems(
            Update,
            (
                parse_palette_image,
                load_tile_images,
                load_tiles,
                finished_loading,
            )
                .run_if(in_state(AppState::Loading)),
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
    palettes: Res<PaletteHandles>,
    tiles: Option<Res<Tiles>>,
    materials: Option<Res<Materials>>,
) {
    let tiles_loaded = tiles.is_some();
    let materials_loaded = materials.is_some();

    let palettes_loaded =
        match asset_server.get_group_load_state(palettes.0.iter().map(|handle| handle.id())) {
            LoadState::Loaded => true,
            _ => false,
        };

    if palettes_loaded && tiles_loaded && materials_loaded {
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
    CyclePalette,
    Load,
    New,
    Save,
    SaveAs,
    ReloadMapDisplay,
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
