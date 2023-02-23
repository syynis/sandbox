use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_ecs_tilemap::{
    tiles::{TilePos, TileStorage, TileTextureIndex},
    *,
};
use serde::{Deserialize, Serialize};

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
}

impl<'w, 's> LevelSerializer<'w, 's> {
    pub fn save(&self) -> Option<SerializableLevel> {
        let mut tiles = Vec::new();

        for (pos, id) in self.tiles.iter() {
            tiles.push(SerializableTile { pos: *pos, id: *id });
        }
        Some(SerializableLevel { tiles })
    }
}
