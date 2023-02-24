use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_ecs_tilemap::prelude::*;

pub enum TileModification {
    Added { old: Option<Entity>, new: Entity },
    Removed { old: Entity },
}

pub struct TileUpdateEvent {
    pub modification: TileModification,
}

#[derive(SystemParam)]
pub struct StorageAccess<'w, 's> {
    storage: Query<
        'w,
        's,
        (
            Entity,
            &'static Transform,
            &'static TilemapSize,
            &'static mut TileStorage,
        ),
    >,
}

impl<'w, 's> StorageAccess<'w, 's> {
    fn set(&mut self, cmds: &mut Commands, pos: &TilePos, id: TileTextureIndex) {
        let (tilemap_entity, map_transform, size, mut storage) =
            self.storage.get_single_mut().unwrap();
        if storage.get(pos).is_none() {
            storage.set(
                pos,
                cmds.spawn(TileBundle {
                    position: *pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: id,
                    ..default()
                })
                .id(),
            );
        }
    }

    fn despawn(&mut self, cmds: &mut Commands, pos: &TilePos) {
        let (tilemap_entity, map_transform, size, mut storage) =
            self.storage.get_single_mut().unwrap();

        if let Some(entity) = storage.get(pos) {
            cmds.entity(entity).despawn_recursive();
            storage.remove(pos);
        }
    }

    fn get(&self, pos: &TilePos) -> Option<Entity> {
        let (tilemap_entity, map_transform, size, storage) = self.storage.get_single().ok()?;
        storage.get(pos)
    }

    pub fn transform_size(&self) -> (&Transform, &TilemapSize) {
        let (tilemap_entity, map_transform, size, storage) = self.storage.get_single().unwrap();
        (map_transform, size)
    }

    pub fn storage(&self) -> &TileStorage {
        let (tilemap_entity, map_transform, size, storage) = self.storage.get_single().unwrap();
        storage
    }
}

#[derive(SystemParam)]
pub struct TilePlacer<'w, 's> {
    cmds: Commands<'w, 's>,
    pub storage: StorageAccess<'w, 's>,
    tile_update_event_writer: EventWriter<'w, 's, TileUpdateEvent>,
}

impl<'w, 's> TilePlacer<'w, 's> {
    pub fn try_place(&mut self, pos: &TilePos, id: TileTextureIndex) {
        if let Some(existing) = self.get(pos) {
            return;
        } else {
            self.storage.set(&mut self.cmds, pos, id);
        }
    }
    pub fn replace(&mut self, pos: &TilePos, id: TileTextureIndex) {
        let old = if let Some(existing) = self.get(pos) {
            self.remove(pos);
            Some(existing)
        } else {
            None
        };
        self.storage.set(&mut self.cmds, pos, id)
    }

    pub fn get(&self, pos: &TilePos) -> Option<Entity> {
        self.storage.get(pos)
    }

    pub fn remove(&mut self, pos: &TilePos) {
        self.storage.despawn(&mut self.cmds, pos)
    }
}
