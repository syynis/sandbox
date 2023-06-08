use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{input::CursorPos, nono::Nonogram};

use self::placement::TileUpdateEvent;
use crate::level::serialization::TilePosRef;

pub mod placement;
pub mod serialization;

pub struct LevelPlugin;

#[derive(Component, Debug, Clone, Deref, DerefMut, Serialize, Deserialize)]
pub struct EditableNonogram(pub Nonogram);
#[derive(Component)]
pub struct Editing;

// For areas of multiple tiles this indicates the origin (bottom left) in tile space
#[derive(Component, Deref, Debug, Clone, DerefMut, Serialize, Deserialize)]
pub struct TilePosAnchor {
    #[serde(with = "TilePosRef")]
    pub pos: TilePos,
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct TileCursor(pub Option<TilePos>);

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(TilemapPlugin);
        app.add_startup_system(spawn_level);
        app.insert_resource(TileCursor::default());
        app.add_system(update_tile_cursor);
        app.add_event::<TileUpdateEvent>();
    }
}

pub fn update_tile_cursor(
    world_cursor: Res<CursorPos>,
    mut tile_cursor: ResMut<TileCursor>,
    tile_storage_q: Query<(&Transform, &TilemapSize)>,
) {
    let (map_transform, map_size) = tile_storage_q.get_single().unwrap();
    if world_cursor.is_changed() {
        let cursor_pos = **world_cursor;
        let cursor_in_map_pos: Vec2 = {
            let cursor_pos = Vec4::from((cursor_pos.extend(0.0), 1.0));
            let cursor_in_map_pos = map_transform.compute_matrix().inverse() * cursor_pos;
            cursor_in_map_pos.truncate().truncate()
        };

        **tile_cursor = from_world_pos(&cursor_in_map_pos, &map_size);
    }
}

fn spawn_level(mut cmds: Commands, assets_server: Res<AssetServer>) {
    let tiles: Handle<Image> = assets_server.load("tiles.png");

    let map_size = TilemapSize { x: 32, y: 32 };
    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = cmds.spawn_empty().id();

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    cmds.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(tiles),
        tile_size,
        //transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
        ..default()
    });
}

pub fn world_to_tile_pos(
    pos: Vec2,
    map_transform: &Transform,
    map_size: &TilemapSize,
) -> Option<TilePos> {
    let in_map_pos: Vec2 = {
        let pos = Vec4::from((pos.extend(0.0), 1.0));
        let in_map_pos = map_transform.compute_matrix().inverse() * pos;
        in_map_pos.truncate().truncate()
    };

    from_world_pos(&in_map_pos, &map_size)
}

// Simplified version of TilePos;:from_world_pos with assumptions about tile and grid size
pub fn from_world_pos(world_pos: &Vec2, size: &TilemapSize) -> Option<TilePos> {
    let x = ((world_pos.x / 16.) + 0.5).floor() as i32;
    let y = ((world_pos.y / 16.) + 0.5).floor() as i32;

    TilePos::from_i32_pair(x, y, size)
}
