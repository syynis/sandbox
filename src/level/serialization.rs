use std::{fs, path::PathBuf};

use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_ecs_tilemap::tiles::{TileFlip, TilePos, TileTextureIndex};
use serde::{Deserialize, Serialize};

use super::placement::{StorageAccess, TileProperties};

#[derive(Serialize, Deserialize)]
#[serde(remote = "TilePos")]
pub struct TilePosRef {
    pub x: u32,
    pub y: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "TileTextureIndex")]
pub struct TileTextureIndexRef(pub u32);

#[derive(Serialize, Deserialize)]
#[serde(remote = "TileFlip")]
pub struct TileFlipRef {
    pub x: bool,
    pub y: bool,
    pub d: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SerializableTile {
    #[serde(with = "TilePosRef")]
    pub pos: TilePos,
    #[serde(with = "TileTextureIndexRef")]
    pub id: TileTextureIndex,
    #[serde(with = "TileFlipRef", default, skip_serializing_if = "is_default_flip")]
    pub flip: TileFlip,
}

fn is_default_flip(flip: &TileFlip) -> bool {
    !flip.x && !flip.y && !flip.d
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SerializableLevel {
    pub tiles: Vec<SerializableTile>,
}

#[derive(SystemParam)]
pub struct LevelSerializer<'w, 's> {
    tiles: Query<
        'w,
        's,
        (
            &'static TilePos,
            &'static TileTextureIndex,
            &'static TileFlip,
        ),
    >,
    pub storage_access: StorageAccess<'w, 's>,
}

impl<'w, 's> LevelSerializer<'w, 's> {
    pub fn save(&self) -> Option<SerializableLevel> {
        let mut tiles = Vec::new();

        for (pos, id, flip) in self.tiles.iter() {
            tiles.push(SerializableTile {
                pos: *pos,
                id: *id,
                flip: *flip,
            });
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
                    self.storage_access.replace(
                        &tile.pos,
                        TileProperties {
                            id: tile.id,
                            flip: tile.flip,
                        },
                    );
                }
            }
        }
    }
}
