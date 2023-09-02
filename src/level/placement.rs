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
}

impl<'w, 's> StorageAccess<'w, 's> {
    fn set_unchecked(
        &mut self,
        cmds: &mut Commands,
        pos: &TilePos,
        id: TileTextureIndex,
    ) -> Option<Entity> {
        let (tilemap_entity, map_transform, size, mut storage) =
            self.storage.get_single_mut().ok()?;

        let tile_entity = cmds
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
    fn set(&mut self, cmds: &mut Commands, pos: &TilePos, id: TileTextureIndex) -> Option<Entity> {
        let (tilemap_entity, map_transform, size, mut storage) =
            self.storage.get_single_mut().ok()?;

        if let Some(old) = storage.get(pos) {
            storage.remove(pos);
        }

        let tile_entity = cmds
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

    fn despawn(&mut self, cmds: &mut Commands, pos: &TilePos) -> Option<Entity> {
        let (tilemap_entity, map_transform, size, mut storage) =
            self.storage.get_single_mut().ok()?;

        if let Some(entity) = storage.get(pos) {
            cmds.entity(entity).despawn_recursive();
            storage.remove(pos);
            Some(entity)
        } else {
            None
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

    pub fn clear(&mut self, cmds: &mut Commands) {
        let (tilemap_entity, map_transform, size, mut storage) =
            self.storage.get_single_mut().unwrap();

        self.tiles.iter().for_each(|tile| {
            if let Some(entity) = storage.get(&tile) {
                storage.remove(&tile);
                cmds.entity(entity).despawn_recursive();
            }
        });
    }
}

#[derive(SystemParam)]
pub struct TilePlacer<'w, 's> {
    cmds: Commands<'w, 's>,
    pub storage: StorageAccess<'w, 's>,
    tile_update_event_writer: EventWriter<'w, TileUpdateEvent>,
}

impl<'w, 's> TilePlacer<'w, 's> {
    pub fn try_place(&mut self, pos: &TilePos, id: TileTextureIndex) {
        if let Some(existing) = self.get(pos) {
            return;
        } else {
            if let Some(new) = self.storage.set_unchecked(&mut self.cmds, pos, id) {
                self.tile_update_event_writer.send(TileUpdateEvent {
                    modification: TileModification::Added { old: None, new },
                });
            }
        }
    }
    pub fn replace(&mut self, pos: &TilePos, id: TileTextureIndex) {
        let old = self.get(pos);
        if let Some(new) = self.storage.set(&mut self.cmds, pos, id) {
            self.tile_update_event_writer.send(TileUpdateEvent {
                modification: TileModification::Added { old, new },
            });
        }
    }

    pub fn get(&self, pos: &TilePos) -> Option<Entity> {
        self.storage.get(pos)
    }

    pub fn remove(&mut self, pos: &TilePos) {
        if let Some(old) = self.storage.despawn(&mut self.cmds, pos) {
            self.tile_update_event_writer.send(TileUpdateEvent {
                modification: TileModification::Removed { old },
            });
        }
    }

    pub fn clear(&mut self) {
        self.storage.clear(&mut self.cmds);
    }
}
