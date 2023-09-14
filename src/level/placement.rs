use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_ecs_tilemap::prelude::*;

pub enum TileModification {
    Added { old: Option<Entity>, new: Entity },
    Removed { old: Entity },
}

#[derive(Event)]
pub struct TileUpdateEvent {
    pub modification: TileModification,
}

#[derive(SystemParam)]
pub struct StorageAccess<'w, 's> {
    cmds: Commands<'w, 's>,
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
    tiles: Query<'w, 's, &'static TilePos>,
    tile_update_event_writer: EventWriter<'w, TileUpdateEvent>,
}

impl<'w, 's> StorageAccess<'w, 's> {
    fn set_unchecked(&mut self, pos: &TilePos, id: TileTextureIndex) -> Option<Entity> {
        let (tilemap_entity, map_transform, size, mut storage) =
            self.storage.get_single_mut().ok()?;

        let tile_entity = self
            .cmds
            .spawn(TileBundle {
                position: *pos,
                tilemap_id: TilemapId(tilemap_entity),
                texture_index: id,
                ..default()
            })
            .id();

        storage.set(pos, tile_entity);
        Some(tile_entity)
    }

    pub fn try_place(&mut self, pos: &TilePos, id: TileTextureIndex) {
        if let Some(existing) = self.get(pos) {
            return;
        } else {
            if let Some(new) = self.set_unchecked(pos, id) {
                self.tile_update_event_writer.send(TileUpdateEvent {
                    modification: TileModification::Added { old: None, new },
                });
            }
        }
    }

    pub fn replace(&mut self, pos: &TilePos, id: TileTextureIndex) {
        let old = self.get(pos);
        if old.is_some() {
            self.remove(pos);
        }
        if let Some(new) = self.set_unchecked(pos, id) {
            self.tile_update_event_writer.send(TileUpdateEvent {
                modification: TileModification::Added { old, new },
            });
        }
    }

    fn despawn(&mut self, pos: &TilePos) -> Option<Entity> {
        let (tilemap_entity, map_transform, size, mut storage) =
            self.storage.get_single_mut().ok()?;

        if let Some(entity) = storage.get(pos) {
            self.cmds.entity(entity).despawn_recursive();
            storage.remove(pos);
            Some(entity)
        } else {
            None
        }
    }

    pub fn remove(&mut self, pos: &TilePos) {
        if let Some(old) = self.despawn(pos) {
            self.tile_update_event_writer.send(TileUpdateEvent {
                modification: TileModification::Removed { old },
            });
        }
    }

    fn get(&self, pos: &TilePos) -> Option<Entity> {
        let (tilemap_entity, map_transform, size, storage) = self.storage.get_single().ok()?;
        storage.get(pos)
    }

    pub fn transform_size(&self) -> Option<(&Transform, &TilemapSize)> {
        let (tilemap_entity, map_transform, size, storage) = self.storage.get_single().ok()?;
        Some((map_transform, size))
    }

    pub fn storage(&self) -> Option<&TileStorage> {
        let (tilemap_entity, map_transform, size, storage) = self.storage.get_single().ok()?;
        Some(storage)
    }

    pub fn clear(&mut self) {
        let (tilemap_entity, map_transform, size, mut storage) =
            self.storage.get_single_mut().unwrap();

        self.tiles.iter().for_each(|tile| {
            if let Some(entity) = storage.get(&tile) {
                storage.remove(&tile);
                self.cmds.entity(entity).despawn_recursive();
            }
        });
    }
}
