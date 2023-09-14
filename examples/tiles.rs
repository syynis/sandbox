use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::PanCam;
use bevy_pancam::PanCamPlugin;
use bevy_prototype_debug_lines::DebugLines;
use bevy_prototype_debug_lines::DebugLinesPlugin;
use leafwing_input_manager::prelude::*;
use sandbox::input::InputPlugin;
use sandbox::level::placement::StorageAccess;
use sandbox::level::serialization::LevelSerializer;
use sandbox::level::LevelPlugin;
use sandbox::level::TileCursor;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins.set(ImagePlugin::default_nearest()),
        PanCamPlugin::default(),
        InputPlugin,
        DebugLinesPlugin::default(),
        WorldInspectorPlugin::default(),
        InputManagerPlugin::<EditorActions>::default(),
        LevelPlugin,
    ));
    app.insert_resource(ClearColor(Color::WHITE));
    app.insert_resource(SelectedTileType::default());
    app.add_systems(Startup, (setup, spawn_level));
    app.add_systems(
        Update,
        (apply_editor_actions, render_tilemap_outline, handle_saving),
    );

    app.run();
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum EditorActions {
    PlaceTile,
    RemoveTile,
    CycleMode,
    New,
    Save,
    Load,
}

fn input_map() -> InputMap<EditorActions> {
    use EditorActions::*;
    let mut input_map = InputMap::default();
    input_map.insert(MouseButton::Left, PlaceTile);
    input_map.insert(MouseButton::Right, RemoveTile);
    input_map.insert(KeyCode::C, CycleMode);
    input_map.insert(KeyCode::N, New);
    input_map.insert(KeyCode::S, Save);
    input_map.insert(KeyCode::L, Load);

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

fn spawn_level(mut cmds: Commands, assets_server: Res<AssetServer>) {
    let tiles: Handle<Image> = assets_server.load("tiles2.png");

    let size = TilemapSize { x: 32, y: 32 };
    let storage = TileStorage::empty(size);
    let tilemap_entity = cmds.spawn_empty().id();

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    cmds.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size,
        storage,
        texture: TilemapTexture::Single(tiles),
        tile_size,
        ..default()
    });
}

fn apply_editor_actions(
    actions: Query<&ActionState<EditorActions>>,
    tile_cursor: Res<TileCursor>,
    mut selected_tile: ResMut<SelectedTileType>,
    mut tile_placer: StorageAccess,
) {
    let Some(actions) = actions.get_single().ok() else {
        return;
    };

    if actions.pressed(EditorActions::RemoveTile) {
        if let Some(cursor_tile_pos) = **tile_cursor {
            tile_placer.remove(&cursor_tile_pos);
        }
    }

    if actions.pressed(EditorActions::PlaceTile) {
        if let Some(cursor_tile_pos) = **tile_cursor {
            tile_placer.replace(&cursor_tile_pos, (**selected_tile).into());
        }
    }

    if actions.just_pressed(EditorActions::CycleMode) {
        **selected_tile = selected_tile.next();
    }
}

fn render_tilemap_outline(
    mut lines: ResMut<DebugLines>,
    tilemap_q: Query<(&TilemapSize, &Transform)>,
) {
    for (size, transform) in tilemap_q.iter() {
        let size = Vec2::from(size);
        let size_scaled = size * 16.;

        for (start, end) in box_lines(transform.translation, size_scaled) {
            lines.line_colored(start, end, 0., Color::RED);
        }
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

fn handle_saving(actions: Query<&ActionState<EditorActions>>, mut serializer: LevelSerializer) {
    let Some(actions) = actions.get_single().ok() else {
        return;
    };
    if actions.just_pressed(EditorActions::Save) {
        info!("Saving map");
        serializer.save_to_file();
    }

    if actions.just_pressed(EditorActions::Load) {
        info!("Loading map");
        serializer.load_from_file();
    }

    if actions.just_pressed(EditorActions::New) {
        serializer.storage_access.clear();
    }
}
