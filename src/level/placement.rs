use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_ecs_tilemap::prelude::*;

use super::{layer::Layer, tile::texture_name};

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

#[derive(SystemParam)]
pub struct StorageAccess<'w, 's> {
    cmds: Commands<'w, 's>,
    layers: Query<'w, 's, (Entity, &'static mut TileStorage, &'static Layer)>,
    transforms: Query<'w, 's, (&'static Transform, &'static TilemapSize, &'static Layer)>,
    tile_properties: Query<'w, 's, (&'static TileTextureIndex, &'static TileFlip)>,
    tile_update_event_writer: EventWriter<'w, TileUpdateEvent>,
}

impl<'w, 's> StorageAccess<'w, 's> {
    fn set_unchecked(
        &mut self,
        pos: &TilePos,
        tile_properties: TileProperties,
        layer: Layer,
    ) -> Option<Entity> {
        let tilemap_entity = self.tilemap_entity(layer)?;

        let color = match layer {
            Layer::World => Color::rgba_u8(0, 0, 0, 255),
            Layer::Near => Color::rgba_u8(0, 127, 0, 127),
            Layer::Far => Color::rgba_u8(127, 0, 0, 63),
        };
        let tile_entity = self
            .cmds
            .spawn((
                Name::new(texture_name(tile_properties.id)),
                TileBundle {
                    position: *pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: tile_properties.id,
                    flip: tile_properties.flip,
                    color: TileColor(color),
                    ..default()
                },
            ))
            .id();

        // TODO can't put this in a seperate function because we cant return mutable storage reference
        for (_, mut storage, l) in self.layers.iter_mut() {
            if layer == *l {
                storage.set(pos, tile_entity);
            }
        }
        Some(tile_entity)
    }

    pub fn try_place(&mut self, pos: &TilePos, tile_properties: TileProperties, layer: Layer) {
        let Some(_) = self.get(pos, layer) else {
            return;
        };
        if let Some(new) = self.set_unchecked(pos, tile_properties, layer) {
            self.tile_update_event_writer.send(TileUpdateEvent {
                modification: TileModification::Added { old: None, new },
            });
        }
    }

    pub fn replace(&mut self, pos: &TilePos, id: TileProperties, layer: Layer) {
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

    fn despawn(&mut self, pos: &TilePos, layer: Layer) -> Option<Entity> {
        let (_, mut storage, _) = self.layers.iter_mut().find(|(_, _, l)| **l == layer)?;
        let entity = storage.get(pos)?;
        storage.remove(pos);
        self.cmds.entity(entity).despawn_recursive();
        Some(entity)
    }

    pub fn remove(&mut self, pos: &TilePos, layer: Layer) {
        if let Some(old) = self.despawn(pos, layer) {
            self.tile_update_event_writer.send(TileUpdateEvent {
                modification: TileModification::Removed { old },
            });
        }
    }

    pub fn get(&self, pos: &TilePos, layer: Layer) -> Option<Entity> {
        let storage = self.storage(layer)?;
        storage.get(pos)
    }

    pub fn get_properties(&self, pos: &TilePos, layer: Layer) -> Option<TileProperties> {
        let entity = self.get(pos, layer)?;
        let (id, flip) = self.tile_properties.get(entity).ok()?;
        Some(TileProperties {
            id: *id,
            flip: *flip,
        })
    }

    pub fn transform_size(&self, layer: Layer) -> Option<(&Transform, &TilemapSize)> {
        let res = self.transforms.iter().find(|(_, _, l)| **l == layer)?;
        Some((res.0, res.1))
    }

    pub fn storage(&self, layer: Layer) -> Option<&TileStorage> {
        let (_, storage, _) = self.layers.iter().find(|(_, _, l)| **l == layer)?;
        Some(storage)
    }

    pub fn tilemap_entity(&self, layer: Layer) -> Option<Entity> {
        let (entity, _, _) = self.layers.iter().find(|(_, _, l)| **l == layer)?;
        Some(entity)
    }

    pub fn clear(&mut self, layer: Layer) {
        // TODO can't put this in a seperate function because we cant return mutable storage reference
        let Some((_, size)) = self.transform_size(layer) else {
            return;
        };
        let size = size.clone();

        let Some((_, mut storage, _)) = self.layers.iter_mut().find(|(_, _, l)| **l == layer)
        else {
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
