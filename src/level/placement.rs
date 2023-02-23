use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_ecs_tilemap::prelude::*;

#[derive(SystemParam)]
pub struct StorageAccess<'w, 's> {
    storage: Query<'w, 's, (Entity, &'static Transform, &'static mut TileStorage)>,
}

impl<'w, 's> StorageAccess<'w, 's> {
    fn set(&mut self, cmds: &mut Commands, pos: &TilePos, id: TileTextureIndex) {
        let (tilemap_entity, map_transform, mut storage) = self.storage.get_single_mut().unwrap();
        if let Some(entity) = storage.get(pos) {
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

    fn get(&self, pos: &TilePos) -> Option<Entity> {
        let (tilemap_entity, map_transform, storage) = self.storage.get_single().ok()?;
        storage.get(pos)
    }
}

#[derive(SystemParam)]
pub struct TilePlacer<'w, 's> {
    cmds: Commands<'w, 's>,
    storage: StorageAccess<'w, 's>,
}

impl<'w, 's> TilePlacer<'w, 's> {
    fn place(&mut self, id: TileTextureIndex, pos: &TilePos) {
        let old = if let Some(existing) = self.get(pos) {
            self.remove(pos);
            Some(existing)
        } else {
            None
        };
    }

    fn get(&self, pos: &TilePos) -> Option<Entity> {
        self.storage.get(pos)
    }

    fn remove(&mut self, pos: &TilePos) {}
}
