use anyhow::Context;
use anyhow::Result;
use bevy::{ecs::system::Command, prelude::*};
use bevy_ecs_tilemap::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{input::CursorPos, nono::Nonogram};

use self::{layer::ALL_LAYERS, placement::TileUpdateEvent};
use crate::level::serialization::TilePosRef;

pub mod layer;
pub mod placement;
pub mod serialization;
pub mod tile;

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
        app.add_plugins(TilemapPlugin);
        app.insert_resource(TileCursor::default());
        app.add_systems(Update, update_tile_cursor);
        app.add_event::<TileUpdateEvent>();
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

pub struct SpawnMapCommand {
    size: u32,
    tile_size: u32,
}

impl SpawnMapCommand {
    pub fn new(size: u32, tile_size: u32) -> Self {
        Self { size, tile_size }
    }
}

impl Command for SpawnMapCommand {
    fn apply(self, world: &mut World) {
        for layer in ALL_LAYERS.iter() {
            let assets_server = world.resource::<AssetServer>();
            let tiles: Handle<Image> = assets_server.load("tiles.png");

            let size = TilemapSize::from(UVec2::splat(self.size));
            let storage = TileStorage::empty(size);
            let tilemap_entity = world.spawn_empty().id();

            let tile_size = TilemapTileSize::from(Vec2::splat(self.tile_size as f32));
            let grid_size = tile_size.into();
            let map_type = TilemapType::Square;

            world.entity_mut(tilemap_entity).insert((
                TilemapBundle {
                    grid_size,
                    map_type,
                    size,
                    storage,
                    texture: TilemapTexture::Single(tiles),
                    tile_size,
                    transform: Transform::from_xyz(0., 0., layer.z_index()),
                    ..default()
                },
                layer.clone(),
                Name::new(layer.name()),
            ));
        }
    }
}

pub fn update_tile_cursor(
    world_cursor: Res<CursorPos>,
    mut tile_cursor: ResMut<TileCursor>,
    tile_storage_q: Query<(&Transform, &TilemapSize)>,
) {
    // FIXME We should only query the currently focused layer,
    // this is especially important if at some point layers have different transforms
    for (map_transform, map_size) in tile_storage_q.iter() {
        if world_cursor.is_changed() {
            let cursor_pos = **world_cursor;
            let cursor_in_map_pos: Vec2 = {
                let cursor_pos = Vec4::from((cursor_pos.extend(0.0), 1.0));
                let cursor_in_map_pos = map_transform.compute_matrix().inverse() * cursor_pos;
                cursor_in_map_pos.truncate().truncate()
            };

            **tile_cursor = from_world_pos(&cursor_in_map_pos, &map_size);
        }
        return;
    }
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

pub fn tpos_wpos(tpos: &TilePos) -> Vec2 {
    tpos.center_in_world(&TilemapGridSize { x: 16., y: 16. }, &TilemapType::Square)
}
