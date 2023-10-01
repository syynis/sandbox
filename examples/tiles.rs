use bevy::ecs::system::Command;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_egui::EguiUserTextures;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::PanCam;
use bevy_pancam::PanCamPlugin;
use bevy_prototype_debug_lines::DebugLines;
use bevy_prototype_debug_lines::DebugLinesPlugin;
use bevy_xpbd_2d::math::Vector;
use bevy_xpbd_2d::prelude::*;
use leafwing_input_manager::prelude::*;
use sandbox::editor::tools::area::AreaTool;
use sandbox::editor::tools::erase::EraseTool;
use sandbox::editor::tools::paint::PaintTool;
use sandbox::editor::tools::platform::PlatformTool;
use sandbox::editor::tools::pole::PoleTool;
use sandbox::editor::tools::run_tool;
use sandbox::editor::tools::slope::SlopeTool;
use sandbox::editor::ui::menu::EditorMenuBar;
use sandbox::editor::ui::toolbar::EditorToolBar;
use sandbox::editor::EditorActions;
use sandbox::editor::EditorEvent;
use sandbox::editor::EditorState;
use sandbox::editor::PickerEvent;
use sandbox::editor::ToolActions;
use sandbox::editor::WorldMapExt;
use sandbox::entity::holdable::CanHold;
use sandbox::entity::holdable::Holdable;
use sandbox::entity::holdable::IsHeld;
use sandbox::entity::player::DespawnPlayerCommand;
use sandbox::entity::player::Player;
use sandbox::entity::player::SpawnPlayerCommand;
use sandbox::file_picker;
use sandbox::input::InputPlugin;
use sandbox::level::placement::StorageAccess;
use sandbox::level::serialization::LevelSerializer;
use sandbox::level::tpos_wpos;
use sandbox::level::LevelPlugin;
use sandbox::level::TileCursor;
use sandbox::phys::movement::LookDir;
use sandbox::phys::terrain::Platform;
use sandbox::phys::terrain::Pole;
use sandbox::phys::terrain::PoleType;
use sandbox::phys::terrain::Terrain;
use sandbox::phys::PhysPlugin;
use sandbox::ui;
use sandbox::ui::draw_confirmation_dialog;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins.set(ImagePlugin::default_nearest()),
        PanCamPlugin::default(),
        InputPlugin::<PanCam>::default(),
        DebugLinesPlugin::default(),
        WorldInspectorPlugin::default().run_if(enable_inspector),
        InputManagerPlugin::<EditorActions>::default(),
        InputManagerPlugin::<ToolActions>::default(),
        LevelPlugin,
        file_picker::Plugin::<PickerEvent>::default(),
        PhysPlugin,
    ));
    app.insert_resource(ClearColor(Color::DARK_GRAY))
        .insert_resource(EditorState::default())
        .insert_resource(Gravity(Vector::NEG_Y * 160.0))
        .insert_resource(SubstepCount(3));

    app.register_type::<EditorState>();

    app.add_event::<EditorEvent>().add_event::<PickerEvent>();
    app.add_systems(Startup, (setup, load_egui_icons, setup_cursor));
    app.add_systems(
        Update,
        (
            apply_editor_actions,
            apply_tool,
            render_tilemap_outline,
            draw_ui,
            handle_save.run_if(on_event::<EditorEvent>()),
            handle_save_as.run_if(on_event::<EditorEvent>()),
            handle_load.run_if(on_event::<EditorEvent>()),
            handle_close.run_if(on_event::<EditorEvent>()),
            handle_new.run_if(on_event::<EditorEvent>()),
            handle_picker_events.run_if(on_event::<PickerEvent>()),
            toggle_inspector,
            move_cursor,
            draw_confirmation_dialog::<EditorEvent>,
            spawn_collisions,
            respawn_player,
            draw_look_dir,
            spawn_rock,
            hold,
            throw,
        ),
    );

    app.run();
}

fn respawn_player(mut cmds: Commands, keys: Res<Input<KeyCode>>, tile_cursor: Res<TileCursor>) {
    let Some(tile_cursor) = **tile_cursor else {
        return;
    };
    let pos = tpos_wpos(&tile_cursor);
    if keys.just_pressed(KeyCode::F) {
        cmds.add(DespawnPlayerCommand);
        let size = Vector::new(14., 14.);
        cmds.add(SpawnPlayerCommand::new(pos, size, ()));
    }
}

fn spawn_rock(mut cmds: Commands, keys: Res<Input<KeyCode>>, tile_cursor: Res<TileCursor>) {
    let Some(tile_cursor) = **tile_cursor else {
        return;
    };
    let pos = tpos_wpos(&tile_cursor);
    if keys.just_pressed(KeyCode::G) {
        cmds.spawn((
            Position(pos),
            RigidBody::Dynamic,
            Collider::ball(4.),
            Holdable,
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.7, 0.7, 0.8),
                    custom_size: Some(Vec2::splat(8.0)),
                    ..default()
                },
                ..default()
            },
        ));
    }
}

fn hold(
    mut cmds: Commands,
    keys: Res<Input<KeyCode>>,
    holdables: Query<Entity, With<Holdable>>,
    held: Query<&IsHeld>,
    holder: Query<(Entity, Option<&Children>, &CollidingEntities), With<CanHold>>,
) {
    let Ok((holder, children, colliding)) = holder.get_single() else {
        return;
    };

    if let Some(children) = children {
        if children.iter().any(|child| held.get(*child).is_ok()) {
            return;
        }
    }

    if keys.just_pressed(KeyCode::H) {
        // Find first colliding enitty that is also a holdable
        if let Some(holdable) = colliding.0.iter().find(|e| holdables.get(**e).is_ok()) {
            // Despawn entity to get rid of all the physics related components
            // TODO when bevy_xpbd supports child colliders think about simply moving the entity to the children list
            // This would hopefully also naturally add velocity inheritance on throwing
            cmds.entity(*holdable).despawn();
            cmds.entity(holder).with_children(|childbuilder| {
                childbuilder.spawn((
                    IsHeld,
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgb(0.7, 0.7, 0.8),
                            custom_size: Some(Vec2::splat(8.0)),
                            ..default()
                        },
                        ..default()
                    },
                ));
            });
        }
    }
}

fn throw(
    mut cmds: Commands,
    keys: Res<Input<KeyCode>>,
    holder: Query<(Entity, &Children, &Transform, &LookDir), With<CanHold>>,
    held: Query<(With<IsHeld>, With<Parent>)>,
) {
    let Ok((holder, children, transform, look_dir)) = holder.get_single() else {
        return;
    };

    // Can't throw anything
    let Some(throwable) = children.iter().find(|child| held.get(**child).is_ok()) else {
        return;
    };

    if keys.just_pressed(KeyCode::X) {
        // Remove child throwable entity
        cmds.entity(holder).remove_children(&[*throwable]);
        cmds.entity(*throwable).despawn();

        // Spawn new physics entity
        cmds.spawn((
            LinearVelocity(look_dir.as_vec() * 256. + Vector::Y * 64.),
            Position(transform.translation.truncate() + look_dir.as_vec() * 16.),
            RigidBody::Dynamic,
            Collider::ball(4.),
            Holdable,
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.7, 0.7, 0.8),
                    custom_size: Some(Vec2::splat(8.0)),
                    ..default()
                },
                ..default()
            },
        ));
    }
}

fn draw_look_dir(
    q_player: Query<(&LookDir, &Transform), With<Player>>,
    mut lines: ResMut<DebugLines>,
) {
    if let Some((dir, transform)) = q_player.get_single().ok() {
        match dir {
            LookDir::Left => lines.line_colored(
                transform.translation,
                transform.translation + dir.as_vec().extend(0.) * 16.,
                0.,
                Color::RED,
            ),
            LookDir::Right => lines.line_colored(
                transform.translation,
                transform.translation + dir.as_vec().extend(0.) * 16.,
                0.,
                Color::RED,
            ),
        }
    }
}

fn enable_inspector(state: Res<EditorState>) -> bool {
    state.enabled.inspector
}

fn toggle_inspector(keys: Res<Input<KeyCode>>, mut state: ResMut<EditorState>) {
    if keys.just_pressed(KeyCode::F1) {
        state.enabled.inspector = !state.enabled.inspector;
    }
}

fn editor_actions_map() -> InputMap<EditorActions> {
    use EditorActions::*;
    let mut input_map = InputMap::default();

    input_map.insert(MouseButton::Left, ApplyTool);
    input_map.insert(MouseButton::Right, ApplyTool);
    input_map.insert(KeyCode::C, CycleTool);
    input_map.insert(KeyCode::L, Load);

    input_map.insert_modified(Modifier::Control, MouseButton::Left, EditorActions::Area);
    input_map.insert_chord([KeyCode::ControlLeft, KeyCode::N], EditorActions::New);
    input_map.insert_chord([KeyCode::ControlLeft, KeyCode::S], EditorActions::Save);
    input_map.insert_chord([KeyCode::ControlLeft, KeyCode::C], EditorActions::Close);

    input_map.insert_chord(
        [KeyCode::ControlLeft, KeyCode::ShiftLeft, KeyCode::S],
        EditorActions::SaveAs,
    );

    input_map
}

fn tool_actions_map() -> InputMap<ToolActions> {
    use ToolActions::*;

    let mut input_map = InputMap::default();

    input_map.insert(KeyCode::T, CycleMode);

    input_map
}

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
        input_map: editor_actions_map(),
        ..default()
    },));
    cmds.spawn((InputManagerBundle::<ToolActions> {
        input_map: tool_actions_map(),
        ..default()
    },));
    cmds.add(SpawnMapCommand);
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

pub struct SpawnMapCommand;

impl Command for SpawnMapCommand {
    fn apply(self, mut world: &mut World) {
        if world.get_map().is_ok() {
            warn!("Tried to spawn world when one already exists");
            return;
        }
        let assets_server = world.resource::<AssetServer>();
        let tiles: Handle<Image> = assets_server.load("tiles.png");

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

#[derive(Component)]
struct CustomCursor;

fn setup_cursor(
    mut windows: Query<&mut Window>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let mut window: Mut<Window> = windows.single_mut();
    window.cursor.visible = true;
    let cursor_spawn: Vec3 = Vec3::ZERO;

    commands.spawn((
        ImageBundle {
            image: asset_server.load("cursor.png").into(),
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Auto,
                right: Val::Auto,
                bottom: Val::Auto,
                top: Val::Auto,
                ..default()
            },
            z_index: ZIndex::Global(15),
            transform: Transform::from_translation(cursor_spawn),
            ..default()
        },
        CustomCursor,
    ));
}

fn move_cursor(window: Query<&Window>, mut cursor: Query<&mut Style, With<CustomCursor>>) {
    let window: &Window = window.single();
    if let Some(position) = window.cursor_position() {
        let mut img_style = cursor.single_mut();
        img_style.left = Val::Px(position.x - 8.);
        img_style.top = Val::Px(position.y - 8.);
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

fn apply_editor_actions(
    mut cmds: Commands,
    actions: Query<&ActionState<EditorActions>>,
    mut event_writer: EventWriter<EditorEvent>,
    mut editor_state: ResMut<EditorState>,
) {
    let Some(actions) = actions.get_single().ok() else {
        return;
    };

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

    if actions.just_pressed(EditorActions::CycleTool) {
        editor_state.next_tool();
    }

    if actions.just_pressed(EditorActions::SaveAs) {
        event_writer.send(EditorEvent::SaveAs);
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
    mut editor_state: ResMut<EditorState>,
) {
    for ev in editor_events.iter() {
        if matches!(ev, EditorEvent::Close) {
            let Ok(entity) = map.get_single() else {
                warn!("Can't close. No map loaded");
                return;
            };

            storage.clear();
            cmds.entity(entity).despawn_recursive();
            editor_state.reset_path();
        }
    }
}

fn handle_new(
    mut cmds: Commands,
    mut editor_events: EventReader<EditorEvent>,
    map: Query<Entity, With<TileStorage>>,
    mut storage: StorageAccess,
    mut editor_state: ResMut<EditorState>,
) {
    for ev in editor_events.iter() {
        if matches!(ev, EditorEvent::New) {
            if let Ok(entity) = map.get_single() {
                storage.clear();
                cmds.entity(entity).despawn_recursive();
            }
            cmds.add(SpawnMapCommand);
            editor_state.reset_path();
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
        lines.line_colored(start, end, 0., Color::WHITE);
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

fn spawn_collisions(
    keys: Res<Input<KeyCode>>,
    mut cmds: Commands,
    tiles: StorageAccess,
    tiles_pos: Query<(&TilePos, &TileTextureIndex, &TileFlip)>,
) {
    if keys.just_pressed(KeyCode::Q) {
        tiles.storage().unwrap().iter().for_each(|tile_entity| {
            let Some(tile_entity) = tile_entity else {
                return;
            };

            let Ok((pos, id, flip)) = tiles_pos.get(*tile_entity) else {
                return;
            };

            let make_right_triangle = |corner, size, dir: Vector| -> Collider {
                Collider::triangle(
                    corner + Vector::X * size * dir.x,
                    corner + Vector::Y * size * dir.y,
                    corner,
                )
            };

            let center = tpos_wpos(pos);

            let dir = match (flip.x, flip.y) {
                (false, false) => Vector::new(1., 1.),
                (true, false) => Vector::new(-1., 1.),
                (false, true) => Vector::new(1., -1.),
                (true, true) => Vector::new(-1., -1.),
            };
            let cross = Collider::compound(vec![
                (
                    Position::default(),
                    Rotation::default(),
                    Collider::cuboid(4., 16.),
                ),
                (
                    Position::default(),
                    Rotation::default(),
                    Collider::cuboid(16., 4.),
                ),
            ]);
            let collider = match id.0 {
                0 => Collider::cuboid(16., 16.),
                1 => make_right_triangle(Vector::new(-8., -8.) * dir, 16., dir),
                2 => Collider::cuboid(4., 16.),
                3 => Collider::cuboid(16., 4.),
                4 => cross,
                5 => Collider::cuboid(16., 4.),
                _ => unreachable!(),
            };

            cmds.entity(*tile_entity)
                .insert((RigidBody::Static, collider, Position::from(center)));
            if id.0 == 5 {
                cmds.entity(*tile_entity)
                    .insert((Platform::default(), Position::from(center + Vector::Y * 5.)));
            }

            if id.0 == 2 || id.0 == 3 || id.0 == 4 {
                let pole_type = if id.0 == 2 {
                    PoleType::Vertical
                } else if id.0 == 3 {
                    PoleType::Horizontal
                } else {
                    PoleType::Combined
                };
                cmds.entity(*tile_entity).insert((Sensor, Pole(pole_type)));
            }

            if id.0 == 0 || id.0 == 1 || id.0 == 5 {
                cmds.entity(*tile_entity).insert(Terrain);
            }
        });
    }
}
