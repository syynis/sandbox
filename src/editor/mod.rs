use std::path::PathBuf;

use bevy::{asset::LoadState, ecs::system::SystemState, prelude::*};
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_ecs_tilemap::tiles::TileStorage;
use bevy_egui::EguiUserTextures;
use leafwing_input_manager::{prelude::*, user_input::Modifier, Actionlike, InputManagerBundle};

use crate::{
    file_picker,
    level::{
        layer::ALL_LAYERS, placement::StorageAccess, serialization::LevelSerializer,
        SpawnMapCommand,
    },
    util::box_lines,
};
use crate::{level::layer::Layer, ui::draw_confirmation_dialog};

use self::{
    palette::{load_palette_image, parse_palette_image, Palette, PaletteHandles, Palettes},
    render::{display_images, render_map_images, MapImages},
    tiles::{load_manifests, load_tile_images, load_tiles, Manifest, Manifests, Materials, Tiles},
    tools::{
        area::{ActiveMode, AreaTool},
        erase::EraseTool,
        paint::PaintTool,
        platform::PlatformTool,
        pole::PoleTool,
        run_tool,
        slope::SlopeTool,
        ToolId, ToolSet,
    },
    ui::draw_ui,
};

pub mod palette;
pub mod render;
pub mod tiles;
pub mod tools;
pub mod ui;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            InputManagerPlugin::<EditorActions>::default(),
            RonAssetPlugin::<Manifest>::new(&["manifest.ron"]),
            file_picker::Plugin::<PickerEvent>::default(),
        ));

        app.register_type::<MapImages>();
        app.register_type::<Palette>();
        app.register_type::<Palettes>();
        app.register_type::<EditorState>();

        app.add_state::<AppState>();
        app.init_resource::<EditorState>();
        app.init_resource::<ActiveMode>();
        app.init_resource::<Manifests>();

        app.add_event::<EditorEvent>().add_event::<PickerEvent>();

        app.add_systems(Startup, |mut cmds: Commands| {
            cmds.spawn((
                (InputManagerBundle::<EditorActions> {
                    input_map: editor_actions_map(),
                    ..default()
                },),
                Name::new("EditorActions"),
            ));
        });
        // Loading state
        app.add_systems(
            OnEnter(AppState::Loading),
            (load_palette_image, load_manifests, load_egui_icons),
        );
        app.add_systems(
            Update,
            (
                (
                    parse_palette_image,
                    load_tile_images,
                    load_tiles,
                    finished_loading,
                )
                    .run_if(in_state(AppState::Loading)),
                (
                    apply_editor_actions,
                    render_map_images,
                    display_images.run_if(resource_exists::<MapImages>()),
                )
                    .run_if(in_state(AppState::Display)),
                (
                    handle_save,
                    handle_save_as,
                    handle_load,
                    handle_close,
                    handle_new,
                )
                    .run_if(on_event::<EditorEvent>()),
                draw_ui,
                apply_tool,
                draw_confirmation_dialog::<EditorEvent>,
                handle_picker_events.run_if(on_event::<PickerEvent>()),
                render_tilemap_outline,
            ),
        );
    }
}

fn render_tilemap_outline(mut gizmo: Gizmos, storage: StorageAccess) {
    let Some((transform, size)) = storage.transform_size(Layer::World) else {
        return;
    };
    let size = Vec2::from(size);
    let size_scaled = size * 16.;

    for (start, end) in box_lines(transform.translation.truncate(), size_scaled) {
        gizmo.line_2d(start, end, Color::WHITE);
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Loading,
    Display,
}

fn load_egui_icons(
    asset_server: Res<AssetServer>,
    mut editor_state: ResMut<EditorState>,
    mut egui_user_textures: ResMut<EguiUserTextures>,
) {
    // TODO figure out a way to make this more principled
    let tools = asset_server.load_folder("tools");
    let mut ids = Vec::new();
    for tool in tools.unwrap().iter() {
        let tool_id = egui_user_textures.add_image(tool.clone().typed());
        // TODO handle cases where folders or non images are in tools folder
        let file_name = asset_server
            .get_handle_path(tool)
            .unwrap()
            .path()
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        println!("file: {}", file_name);
        editor_state.toolset.add(&file_name);
        ids.push(tool_id);
    }

    for (idx, tool_id) in editor_state.toolset.tool_order.clone().iter().enumerate() {
        let tool = editor_state.toolset.tools.get_mut(tool_id).unwrap();
        tool.egui_texture_id = Some(ids[idx]);
    }
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
        self.current_layer = self.current_layer.wrapping_next();
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

fn apply_tool(world: &mut World, system_param: &mut SystemState<Res<EditorState>>) {
    let editor_state = system_param.get(world);
    let active_tool_id = editor_state.active_tool;
    match active_tool_id {
        0 => run_tool::<PlatformTool>(world, active_tool_id),
        1 => run_tool::<AreaTool>(world, active_tool_id),
        2 => run_tool::<PaintTool>(world, active_tool_id),
        3 => run_tool::<PoleTool>(world, active_tool_id),
        4 => run_tool::<SlopeTool>(world, active_tool_id),
        5 => run_tool::<EraseTool>(world, active_tool_id),
        _ => {}
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

fn editor_actions_map() -> InputMap<EditorActions> {
    use EditorActions::*;
    let mut input_map = InputMap::default();

    input_map.insert(MouseButton::Left, ApplyTool);
    input_map.insert(MouseButton::Right, ApplyTool);
    input_map.insert(KeyCode::C, CycleTool);
    input_map.insert(KeyCode::L, Load);
    input_map.insert(KeyCode::T, CycleToolMode);
    input_map.insert(KeyCode::Z, ReloadMapDisplay);
    input_map.insert(KeyCode::P, CyclePalette);

    input_map.insert_modified(Modifier::Control, MouseButton::Left, EditorActions::Area);
    input_map.insert_modified(Modifier::Shift, KeyCode::C, EditorActions::CycleLayer);
    input_map.insert_chord([KeyCode::ControlLeft, KeyCode::N], EditorActions::New);
    input_map.insert_chord([KeyCode::ControlLeft, KeyCode::S], EditorActions::Save);
    input_map.insert_chord([KeyCode::ControlLeft, KeyCode::C], EditorActions::Close);

    input_map.insert_chord(
        [KeyCode::ControlLeft, KeyCode::ShiftLeft, KeyCode::S],
        EditorActions::SaveAs,
    );

    input_map
}

fn apply_editor_actions(
    mut cmds: Commands,
    actions: Query<&ActionState<EditorActions>>,
    mut event_writer: EventWriter<EditorEvent>,
    mut editor_state: ResMut<EditorState>,
    mut palettes: ResMut<Palettes>,
) {
    let Some(actions) = actions.get_single().ok() else {
        return;
    };

    actions
        .get_just_pressed()
        .iter()
        .for_each(|action| match action {
            EditorActions::ApplyTool => {}
            EditorActions::Area => {}
            EditorActions::Close => {
                event_writer.send(EditorEvent::Close);
            }
            EditorActions::CycleTool => {
                editor_state.next_tool();
            }
            EditorActions::CycleLayer => {
                editor_state.next_layer();
            }
            EditorActions::CycleToolMode => todo!(),
            EditorActions::CyclePalette => {
                palettes.cycle();
            }
            EditorActions::Load => {
                if let Some(path) = &editor_state.current_loaded_path {
                    event_writer.send(EditorEvent::Load(path.clone()));
                } else {
                    cmds.spawn(file_picker::Picker::new(PickerEvent::Load(None)).build());
                }
            }
            EditorActions::New => {
                event_writer.send(EditorEvent::New);
            }
            EditorActions::Save => {
                if let Some(path) = &editor_state.current_loaded_path {
                    event_writer.send(EditorEvent::Save(path.clone()));
                }
            }
            EditorActions::SaveAs => {
                event_writer.send(EditorEvent::SaveAs);
            }
            EditorActions::ReloadMapDisplay => {}
        });
}

#[derive(Debug, Clone, Event)]
pub enum EditorEvent {
    New,
    Close,
    Save(PathBuf),
    SaveAs,
    Load(PathBuf),
}

fn handle_save(
    mut editor_events: EventReader<EditorEvent>,
    mut editor_state: ResMut<EditorState>,
    map: Query<Entity, With<TileStorage>>,
    serializer: LevelSerializer,
) {
    for ev in editor_events.iter() {
        if let EditorEvent::Save(path) = ev {
            println!("{}", path.as_path().to_str().unwrap());
            editor_state.unsaved_changes = false;
            serializer.save_to_file(path.clone());
        }
    }
}

fn handle_save_as(
    mut cmds: Commands,
    map: Query<Entity, With<TileStorage>>,
    mut editor_events: EventReader<EditorEvent>,
) {
    for ev in editor_events.iter() {
        if matches!(ev, EditorEvent::SaveAs) {
            cmds.spawn(file_picker::Picker::save_dialog(PickerEvent::Save(None)).build());
        }
    }
}

fn handle_load(
    mut editor_events: EventReader<EditorEvent>,
    map: Query<Entity, With<TileStorage>>,
    mut serializer: LevelSerializer,
) {
    for ev in editor_events.iter() {
        if let EditorEvent::Load(path) = ev {
            println!("{}", path.as_path().to_str().unwrap());
            serializer.load_from_file(path.clone());
        }
    }
}

fn handle_close(
    mut cmds: Commands,
    mut editor_events: EventReader<EditorEvent>,
    map: Query<Entity, With<TileStorage>>,
    mut storage: StorageAccess,
    mut editor_state: ResMut<EditorState>,
) {
    for ev in editor_events.iter() {
        if matches!(ev, EditorEvent::Close) {
            let Ok(entity) = map.get_single() else {
                warn!("Can't close. No map loaded");
                return;
            };

            for layer in ALL_LAYERS.iter() {
                storage.clear(*layer);
            }
            cmds.entity(entity).despawn_recursive();
            editor_state.reset_path();
        }
    }
}

fn handle_new(
    mut cmds: Commands,
    mut editor_events: EventReader<EditorEvent>,
    map: Query<Entity, (With<TileStorage>, With<Layer>)>,
    mut storage: StorageAccess,
    mut editor_state: ResMut<EditorState>,
) {
    for ev in editor_events.iter() {
        if matches!(ev, EditorEvent::New) {
            if let Ok(entity) = map.get_single() {
                for layer in ALL_LAYERS.iter() {
                    storage.clear(*layer);
                }
                cmds.entity(entity).despawn_recursive();
            }
            cmds.add(SpawnMapCommand::new(UVec2::new(64, 32), 16));
            editor_state.reset_path();
        }
    }
}

#[derive(Debug, Event)]
pub enum PickerEvent {
    Save(Option<PathBuf>),
    Load(Option<PathBuf>),
}

fn handle_picker_events(
    mut picker_events: EventReader<PickerEvent>,
    mut state: ResMut<EditorState>,
    mut editor_events: EventWriter<EditorEvent>,
) {
    for event in picker_events.iter() {
        match event {
            PickerEvent::Save(path) => {
                let Some(path) = path else { continue };
                if state.current_loaded_path.is_none() {
                    state.current_loaded_path = Some(path.clone());
                }

                editor_events.send(EditorEvent::Save(path.clone()));
            }
            PickerEvent::Load(path) => {
                let Some(path) = path else { continue };
                if state.current_loaded_path.is_none() {
                    state.current_loaded_path = Some(path.clone());
                }

                editor_events.send(EditorEvent::Load(path.clone()));
            }
        }
    }
    picker_events.clear();
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
