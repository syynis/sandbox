use bevy::ecs::system::Command;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_egui::EguiUserTextures;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::PanCam;
use bevy_pancam::PanCamPlugin;
use bevy_prototype_debug_lines::DebugLines;
use bevy_prototype_debug_lines::DebugLinesPlugin;
use leafwing_input_manager::prelude::*;
use sandbox::editor::tools::paint::PaintTool;
use sandbox::editor::tools::slope::SlopeTool;
use sandbox::editor::ui::menu::EditorMenuBar;
use sandbox::editor::ui::toolbar::EditorToolBar;
use sandbox::editor::EditorActions;
use sandbox::editor::EditorEvent;
use sandbox::editor::EditorState;
use sandbox::editor::PickerEvent;
use sandbox::editor::WorldMapExt;
use sandbox::file_picker;
use sandbox::input::InputPlugin;
use sandbox::level::placement::StorageAccess;
use sandbox::level::serialization::LevelSerializer;
use sandbox::level::LevelPlugin;
use sandbox::level::TileCursor;
use sandbox::ui;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins.set(ImagePlugin::default_nearest()),
        PanCamPlugin::default(),
        InputPlugin,
        DebugLinesPlugin::default(),
        WorldInspectorPlugin::default().run_if(enable_inspector),
        InputManagerPlugin::<EditorActions>::default(),
        LevelPlugin,
        file_picker::Plugin::<PickerEvent>::default(),
    ));
    app.insert_resource(ClearColor(Color::WHITE))
        .insert_resource(SelectedTileType::default())
        .insert_resource(EditorState::default());

    app.add_event::<EditorEvent>().add_event::<PickerEvent>();
    app.add_systems(Startup, (setup, load_egui_icons));
    app.add_systems(
        Update,
        (
            apply_editor_actions,
            render_tilemap_outline,
            draw_ui,
            handle_save,
            handle_save_as,
            handle_load,
            handle_close,
            handle_new,
            handle_picker_events,
            toggle_inspector,
        ),
    );

    app.run();
}

fn enable_inspector(state: Res<EditorState>) -> bool {
    state.enabled.inspector
}

fn toggle_inspector(keys: Res<Input<KeyCode>>, mut state: ResMut<EditorState>) {
    if keys.just_pressed(KeyCode::F1) {
        state.enabled.inspector = !state.enabled.inspector;
    }
}

fn input_map() -> InputMap<EditorActions> {
    use EditorActions::*;
    let mut input_map = InputMap::default();
    input_map.insert(MouseButton::Left, PlaceTile);
    input_map.insert(MouseButton::Right, RemoveTile);
    input_map.insert(KeyCode::C, CycleMode);
    input_map.insert(KeyCode::L, Load);

    input_map.insert_chord([KeyCode::ControlLeft, KeyCode::N], EditorActions::New);
    input_map.insert_chord([KeyCode::ControlLeft, KeyCode::S], EditorActions::Save);
    input_map.insert_chord([KeyCode::ControlLeft, KeyCode::C], EditorActions::Close);

    input_map.insert_chord(
        [KeyCode::ControlLeft, KeyCode::ShiftLeft, KeyCode::S],
        EditorActions::SaveAs,
    );

    input_map
}

// NOTE currently needs to be in same order as spritesheet
#[derive(Clone, Copy, Default)]
pub enum TileType {
    #[default]
    Square = 0,
    Ramp,
    PoleV,
    PoleH,
}

impl TileType {
    pub fn next(&self) -> Self {
        use TileType::*;
        match self {
            Square => Ramp,
            Ramp => PoleV,
            PoleV => PoleH,
            PoleH => Square,
        }
    }
}

impl Into<TileTextureIndex> for TileType {
    fn into(self) -> TileTextureIndex {
        let index = self as u32;
        TileTextureIndex(index)
    }
}

#[derive(Resource, Clone, Copy, Default, Deref, DerefMut)]
pub struct SelectedTileType(TileType);

fn setup(mut cmds: Commands) {
    cmds.spawn((
        Camera2dBundle::default(),
        PanCam {
            grab_buttons: vec![MouseButton::Middle],
            enabled: true,
            ..default()
        },
    ));

    cmds.spawn((InputManagerBundle::<EditorActions> {
        input_map: input_map(),
        ..default()
    },));
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
    editor_state.tools = vec![
        Box::new(PaintTool::default()),
        Box::new(SlopeTool::default()),
    ];

    for (idx, tool_id) in editor_state.toolset.tool_order.clone().iter().enumerate() {
        let tool = editor_state.toolset.tools.get_mut(tool_id).unwrap();
        tool.egui_texture_id = Some(ids[idx]);
    }
}

pub struct SpawnMapCommand;

impl Command for SpawnMapCommand {
    fn apply(self, mut world: &mut World) {
        if world.get_map().is_ok() {
            warn!("Tried to spawn world when one already exists");
            return;
        }
        let assets_server = world.resource::<AssetServer>();
        let tiles: Handle<Image> = assets_server.load("tiles2.png");

        let size = TilemapSize { x: 32, y: 32 };
        let storage = TileStorage::empty(size);
        let tilemap_entity = world.spawn_empty().id();

        let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
        let grid_size = tile_size.into();
        let map_type = TilemapType::default();

        world.entity_mut(tilemap_entity).insert(TilemapBundle {
            grid_size,
            map_type,
            size,
            storage,
            texture: TilemapTexture::Single(tiles),
            tile_size,
            ..default()
        });
    }
}

fn apply_editor_actions(
    mut cmds: Commands,
    actions: Query<&ActionState<EditorActions>>,
    tile_cursor: Res<TileCursor>,
    mut selected_tile: ResMut<SelectedTileType>,
    mut tile_placer: StorageAccess,

    mut event_writer: EventWriter<EditorEvent>,
    mut editor_state: ResMut<EditorState>,
) {
    let Some(actions) = actions.get_single().ok() else {
        return;
    };

    if actions.pressed(EditorActions::RemoveTile) {
        if let Some(cursor_tile_pos) = **tile_cursor {
            tile_placer.remove(&cursor_tile_pos);
            editor_state.unsaved_changes = true;
        }
    }

    if actions.pressed(EditorActions::PlaceTile) {
        if let Some(cursor_tile_pos) = **tile_cursor {
            tile_placer.replace(&cursor_tile_pos, (**selected_tile).into());
            editor_state.unsaved_changes = true;
        }
    }

    if actions.just_pressed(EditorActions::CycleMode) {
        **selected_tile = selected_tile.next();
    }

    if actions.just_pressed(EditorActions::Save) {
        if let Some(path) = &editor_state.current_loaded_path {
            event_writer.send(EditorEvent::Save(path.clone()));
        }
    }

    if actions.just_pressed(EditorActions::Load) {
        if let Some(path) = &editor_state.current_loaded_path {
            event_writer.send(EditorEvent::Load(path.clone()));
        } else {
            cmds.spawn(file_picker::Picker::new(PickerEvent::Load(None)).build());
        }
    }

    if actions.just_pressed(EditorActions::Close) {
        event_writer.send(EditorEvent::Close);
    }

    if actions.just_pressed(EditorActions::New) {
        event_writer.send(EditorEvent::New);
    }
}

fn handle_save(
    mut editor_events: EventReader<EditorEvent>,
    mut editor_state: ResMut<EditorState>,
    map: Query<Entity, With<TileStorage>>,
    serializer: LevelSerializer,
) {
    let Ok(_) = map.get_single() else {
        return;
    };
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
    mut editor_events: EventReader<EditorEvent>,
    map: Query<Entity, With<TileStorage>>,
) {
    let Ok(_) = map.get_single() else {
        return;
    };
    for ev in editor_events.iter() {
        if matches!(ev, EditorEvent::SaveAs) {
            cmds.spawn(file_picker::Picker::save_dialog(PickerEvent::Save(None)).build());
        }
    }
}

fn handle_load(mut editor_events: EventReader<EditorEvent>, mut serializer: LevelSerializer) {
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
) {
    for ev in editor_events.iter() {
        if matches!(ev, EditorEvent::Close) {
            let Ok(entity) = map.get_single() else {
                warn!("Can't close. No map loaded");
                return;
            };

            storage.clear();
            cmds.entity(entity).despawn_recursive();
        }
    }
}

fn handle_new(
    mut cmds: Commands,
    mut editor_events: EventReader<EditorEvent>,
    map: Query<Entity, With<TileStorage>>,
    mut storage: StorageAccess,
) {
    for ev in editor_events.iter() {
        if matches!(ev, EditorEvent::New) {
            if let Ok(entity) = map.get_single() {
                storage.clear();
                cmds.entity(entity).despawn_recursive();
            }
            cmds.add(SpawnMapCommand);
        }
    }
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

pub fn draw_ui(world: &mut World) {
    use ui::widget::*;

    ui::with_world_and_egui_context(world, |mut world, ctx| {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            basic_widget::<EditorMenuBar>(world, ui, ui.id().with("menubar"));
        });

        let state = world.resource_mut::<EditorState>();
        egui::SidePanel::right("right_panel")
            .resizable(true)
            .default_width(350.)
            .show_animated(ctx, state.enabled.tool_panel, |ui| {
                basic_widget::<EditorToolBar>(world, ui, ui.id().with("panel"));
            })
    });
}

fn render_tilemap_outline(
    mut lines: ResMut<DebugLines>,
    tilemap_q: Query<(&TilemapSize, &Transform)>,
) {
    let Ok((size, transform)) = tilemap_q.get_single() else {
        return;
    };
    let size = Vec2::from(size);
    let size_scaled = size * 16.;

    for (start, end) in box_lines(transform.translation, size_scaled) {
        lines.line_colored(start, end, 0., Color::RED);
    }
}

fn box_lines(origin: Vec3, size: Vec2) -> [(Vec3, Vec3); 4] {
    let extend = size.extend(0.);
    let min = origin - Vec3::new(8., 8., 0.);
    let max = origin + extend - Vec3::new(8., 8., 0.);

    let bottom_right = (min, min + Vec3::new(size.x, 0., 0.));
    let bottom_up = (min, min + Vec3::new(0., size.y, 0.));
    let top_left = (max, max - Vec3::new(size.x, 0., 0.));
    let top_down = (max, max - Vec3::new(0., size.y, 0.));

    [bottom_right, bottom_up, top_left, top_down]
}
