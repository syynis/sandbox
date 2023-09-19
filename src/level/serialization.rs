use std::{fs, path::PathBuf};

use bevy::{asset::FileAssetIo, ecs::system::SystemParam, prelude::*};
use bevy_ecs_tilemap::{
    tiles::{TileFlip, TilePos, TileStorage, TileTextureIndex},
    *,
};
use ron::ser;
use serde::{Deserialize, Serialize};

use super::placement::StorageAccess;

#[derive(Serialize, Deserialize)]
#[serde(remote = "TilePos")]
pub struct TilePosRef {
    pub x: u32,
    pub y: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "TileTextureIndex")]
pub struct TileTextureIndexRef(pub u32);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SerializableTile {
    #[serde(with = "TilePosRef")]
    pub pos: TilePos,
    #[serde(with = "TileTextureIndexRef")]
    pub id: TileTextureIndex,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SerializableLevel {
    pub tiles: Vec<SerializableTile>,
}

#[derive(SystemParam)]
pub struct LevelSerializer<'w, 's> {
    tiles: Query<'w, 's, (&'static TilePos, &'static TileTextureIndex)>,
    pub storage_access: StorageAccess<'w, 's>,
}

impl<'w, 's> LevelSerializer<'w, 's> {
    pub fn save(&self) -> Option<SerializableLevel> {
        let mut tiles = Vec::new();

        for (pos, id) in self.tiles.iter() {
            tiles.push(SerializableTile { pos: *pos, id: *id });
        }

        Some(SerializableLevel { tiles })
    }

    pub fn save_to_file(&self, path: PathBuf) {
        if let Some(level) = self.save() {
            let ron =
                ron::ser::to_string_pretty(&level, ron::ser::PrettyConfig::default()).unwrap();
            bevy::log::info!("{}", ron);
            fs::write(path, ron.as_bytes());
        }
    }

    pub fn load_from_file(&mut self, path: PathBuf) {
        if let Some(data) = fs::read_to_string(path).ok() {
            if let Some(level) = ron::from_str::<SerializableLevel>(&data).ok() {
                self.storage_access.clear();
                for tile in level.tiles {
                    self.storage_access.replace(&tile.pos, tile.id);
                }
            }
        }
    }
}
