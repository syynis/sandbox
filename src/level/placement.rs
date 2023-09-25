use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_ecs_tilemap::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct TileProperties {
    pub id: TileTextureIndex,
    pub flip: TileFlip,
}

pub enum TileModification {
    Added { old: Option<Entity>, new: Entity },
    Removed { old: Entity },
}

#[derive(Event)]
pub struct TileUpdateEvent {
    pub modification: TileModification,
}

// TODO Layer support
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
    tile_properties: Query<'w, 's, (&'static TileTextureIndex, &'static TileFlip)>,
    tile_update_event_writer: EventWriter<'w, TileUpdateEvent>,
}

impl<'w, 's> StorageAccess<'w, 's> {
    fn set_unchecked(&mut self, pos: &TilePos, tile_properties: TileProperties) -> Option<Entity> {
        let (tilemap_entity, _, _, mut storage) = self.storage.get_single_mut().ok()?;

        let tile_entity = self
            .cmds
            .spawn(TileBundle {
                position: *pos,
                tilemap_id: TilemapId(tilemap_entity),
                texture_index: tile_properties.id,
                flip: tile_properties.flip,
                ..default()
            })
            .id();

        storage.set(pos, tile_entity);
        Some(tile_entity)
    }

    pub fn try_place(&mut self, pos: &TilePos, tile_properties: TileProperties) {
        let Some(_) = self.get(pos) else {
            return;
        };
        if let Some(new) = self.set_unchecked(pos, tile_properties) {
            self.tile_update_event_writer.send(TileUpdateEvent {
                modification: TileModification::Added { old: None, new },
            });
        }
    }

    pub fn replace(&mut self, pos: &TilePos, id: TileProperties) {
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
        let (_, _, _, mut storage) = self.storage.get_single_mut().ok()?;

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

    pub fn get(&self, pos: &TilePos) -> Option<Entity> {
        let storage = self.storage()?;
        storage.get(pos)
    }

    pub fn get_properties(&self, pos: &TilePos) -> Option<TileProperties> {
        let entity = self.get(pos)?;
        let (id, flip) = self.tile_properties.get(entity).ok()?;
        Some(TileProperties {
            id: *id,
            flip: *flip,
        })
    }

    pub fn transform_size(&self) -> Option<(&Transform, &TilemapSize)> {
        let (_, map_transform, size, _) = self.storage.get_single().ok()?;
        Some((map_transform, size))
    }

    pub fn storage(&self) -> Option<&TileStorage> {
        let (_, _, _, storage) = self.storage.get_single().ok()?;
        Some(storage)
    }

    pub fn clear(&mut self) {
        let (_, _, _, mut storage) = self.storage.get_single_mut().unwrap();

        self.tiles.iter().for_each(|tile| {
            if let Some(entity) = storage.get(&tile) {
                storage.remove(&tile);
                self.cmds.entity(entity).despawn_recursive();
            }
        });
    }
}
