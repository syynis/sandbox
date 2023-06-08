use std::fs;

use bevy::{asset::FileAssetIo, ecs::system::SystemParam, prelude::*};
use bevy_ecs_tilemap::{
    tiles::{TilePos, TileStorage, TileTextureIndex},
    *,
};
use ron::ser;
use serde::{Deserialize, Serialize};

use crate::nono::{Cell, Nonogram};

use super::{placement::TilePlacer, EditableNonogram, TilePosAnchor};

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
    pub nonograms: Vec<(EditableNonogram, TilePosAnchor)>,
}

#[derive(SystemParam)]
pub struct LevelSerializer<'w, 's> {
    tiles: Query<'w, 's, (&'static TilePos, &'static TileTextureIndex)>,
    nonograms: Query<'w, 's, (&'static EditableNonogram, &'static TilePosAnchor)>,
    tile_placer: TilePlacer<'w, 's>,
}

impl<'w, 's> LevelSerializer<'w, 's> {
    pub fn save(&self) -> Option<SerializableLevel> {
        let mut tiles = Vec::new();

        for (pos, id) in self.tiles.iter() {
            tiles.push(SerializableTile { pos: *pos, id: *id });
        }

        let mut nonograms = Vec::new();
        for (nonogram, anchor) in self.nonograms.iter() {
            nonograms.push((nonogram.clone(), anchor.clone()));
        }

        Some(SerializableLevel { tiles, nonograms })
    }

    pub fn save_to_file(&self) {
        if let Some(level) = self.save() {
            let path = FileAssetIo::get_base_path().join("assets/map.ron");
            let ron =
                ron::ser::to_string_pretty(&level, ron::ser::PrettyConfig::default()).unwrap();
            bevy::log::info!("{}", ron);
            fs::write(path, ron.as_bytes());
        }
    }

    pub fn load_from_file(&mut self) {
        let path = FileAssetIo::get_base_path().join("assets/map.ron");
        if let Some(data) = fs::read_to_string(path).ok() {
            if let Some(level) = ron::from_str::<SerializableLevel>(&data).ok() {
                self.tile_placer.clear();
                for tile in level.tiles {
                    self.tile_placer.replace(&tile.pos, tile.id);
                }
            }
        }
    }
}
