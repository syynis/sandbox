use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_ecs_tilemap::prelude::*;

use super::layer::{FarLayer, Layer, LayerId, NearLayer, WorldLayer};

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

// TODO This is extremely clunky find a way to fix this
#[derive(SystemParam)]
pub struct LayerStorage<'w, 's, T: Component + Layer, O1: Component + Layer> {
    storage: Query<'w, 's, (Entity, &'static mut TileStorage), (With<T>, Without<O1>)>,
    // TODO Maybe enforce that each tilemap has the same size and transform
    transform: Query<'w, 's, (&'static Transform, &'static TilemapSize), (With<T>, Without<O1>)>,
}

#[derive(SystemParam)]
pub struct StorageAccess<'w, 's> {
    cmds: Commands<'w, 's>,
    world: LayerStorage<'w, 's, WorldLayer, NearLayer>,
    near: LayerStorage<'w, 's, NearLayer, FarLayer>,
    far: LayerStorage<'w, 's, FarLayer, WorldLayer>,
    tile_properties: Query<'w, 's, (&'static TileTextureIndex, &'static TileFlip)>,
    tile_update_event_writer: EventWriter<'w, TileUpdateEvent>,
}

impl<'w, 's> StorageAccess<'w, 's> {
    fn set_unchecked(
        &mut self,
        pos: &TilePos,
        tile_properties: TileProperties,
        layer: LayerId,
    ) -> Option<Entity> {
        let tilemap_entity = self.tilemap_entity(layer)?;

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

        // TODO can't put this in a seperate function because we cant return mutable storage reference
        let (_, mut storage) = match layer {
            LayerId::World => self.world.storage.get_single_mut().ok(),
            LayerId::Near => self.near.storage.get_single_mut().ok(),
            LayerId::Far => self.far.storage.get_single_mut().ok(),
        }?;
        storage.set(pos, tile_entity);
        Some(tile_entity)
    }

    pub fn try_place(&mut self, pos: &TilePos, tile_properties: TileProperties, layer: LayerId) {
        let Some(_) = self.get(pos, layer) else {
            return;
        };
        if let Some(new) = self.set_unchecked(pos, tile_properties, layer) {
            self.tile_update_event_writer.send(TileUpdateEvent {
                modification: TileModification::Added { old: None, new },
            });
        }
    }

    pub fn replace(&mut self, pos: &TilePos, id: TileProperties, layer: LayerId) {
        let old = self.get(pos, layer);
        if old.is_some() {
            self.remove(pos, layer);
        }
        if let Some(new) = self.set_unchecked(pos, id, layer) {
            self.tile_update_event_writer.send(TileUpdateEvent {
                modification: TileModification::Added { old, new },
            });
        }
    }

    fn despawn(&mut self, pos: &TilePos, layer: LayerId) -> Option<Entity> {
        // TODO can't put this in a seperate function because we cant return mutable storage reference
        let (_, mut storage) = match layer {
            LayerId::World => self.world.storage.get_single_mut().ok(),
            LayerId::Near => self.near.storage.get_single_mut().ok(),
            LayerId::Far => self.far.storage.get_single_mut().ok(),
        }?;

        let entity = storage.get(pos)?;
        storage.remove(pos);
        self.cmds.entity(entity).despawn_recursive();
        Some(entity)
    }

    pub fn remove(&mut self, pos: &TilePos, layer: LayerId) {
        if let Some(old) = self.despawn(pos, layer) {
            self.tile_update_event_writer.send(TileUpdateEvent {
                modification: TileModification::Removed { old },
            });
        }
    }

    pub fn get(&self, pos: &TilePos, layer: LayerId) -> Option<Entity> {
        let storage = self.storage(layer)?;
        storage.get(pos)
    }

    pub fn get_properties(&self, pos: &TilePos, layer: LayerId) -> Option<TileProperties> {
        let entity = self.get(pos, layer)?;
        let (id, flip) = self.tile_properties.get(entity).ok()?;
        Some(TileProperties {
            id: *id,
            flip: *flip,
        })
    }

    pub fn transform_size(&self, layer: LayerId) -> Option<(&Transform, &TilemapSize)> {
        let transform = match layer {
            LayerId::World => self.world.transform.get_single().ok(),
            LayerId::Near => self.near.transform.get_single().ok(),
            LayerId::Far => self.far.transform.get_single().ok(),
        }?;
        Some(transform)
    }

    pub fn storage(&self, layer: LayerId) -> Option<&TileStorage> {
        let (_, storage) = match layer {
            LayerId::World => self.world.storage.get_single().ok(),
            LayerId::Near => self.near.storage.get_single().ok(),
            LayerId::Far => self.far.storage.get_single().ok(),
        }?;
        Some(storage)
    }

    pub fn tilemap_entity(&self, layer: LayerId) -> Option<Entity> {
        let (entity, _) = match layer {
            LayerId::World => self.world.storage.get_single().ok(),
            LayerId::Near => self.near.storage.get_single().ok(),
            LayerId::Far => self.far.storage.get_single().ok(),
        }?;
        Some(entity)
    }

    pub fn clear(&mut self, layer: LayerId) {
        // TODO can't put this in a seperate function because we cant return mutable storage reference
        let Some((_, size)) = self.transform_size(layer) else {
            return;
        };
        let size = size.clone();

        let Ok((_, mut storage)) = (match layer {
            LayerId::World => self.world.storage.get_single_mut(),
            LayerId::Near => self.near.storage.get_single_mut(),
            LayerId::Far => self.far.storage.get_single_mut(),
        }) else {
            return;
        };
        let mut pos_to_remove = Vec::new();
        storage.iter_mut().enumerate().for_each(|(idx, tile)| {
            pos_to_remove.push(TilePos {
                x: idx as u32 % size.x,
                y: idx as u32 / size.x,
            });
            if let Some(tile) = tile {
                self.cmds.entity(*tile).despawn_recursive();
            }
        });
        pos_to_remove.iter().for_each(|pos| storage.remove(&pos));
    }
}
