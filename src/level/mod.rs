use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(TilemapPlugin);
        app.add_startup_system(spawn_level);
    }
}

fn spawn_level(mut cmds: Commands, assets_server: Res<AssetServer>) {
    let tiles: Handle<Image> = assets_server.load("tiles.png");

    let map_size = TilemapSize { x: 64, y: 64 };
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
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
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
