use super::Tool;
use crate::{
    editor::EditorState,
    level::{
        placement::{StorageAccess, TileProperties},
        TileCursor,
    },
};
use bevy::{
    ecs::system::{SystemParam, SystemState},
    prelude::*,
};
use bevy_ecs_tilemap::{helpers::square_grid::neighbors::Neighbors, prelude::*};

#[derive(SystemParam)]
struct SlopeToolParams<'w, 's> {
    pub tiles: StorageAccess<'w, 's>,
    pub tile_cursor: Res<'w, TileCursor>,
    pub editor_state: ResMut<'w, EditorState>,
}

pub struct SlopeTool<'w: 'static, 's: 'static> {
    system_state: SystemState<SlopeToolParams<'w, 's>>,
}

impl<'w, 's> Tool for SlopeTool<'w, 's> {
    fn new(world: &mut bevy::prelude::World) -> Self {
        SlopeTool {
            system_state: SystemState::new(world),
        }
    }
    fn apply(&mut self, world: &mut World) {
        let SlopeToolParams {
            mut tiles,
            tile_cursor,
            mut editor_state,
        } = self.system_state.get_mut(world);

        let Some(cursor_tile_pos) = **tile_cursor else {
            return;
        };

        let neighbors = Neighbors::get_square_neighboring_positions(
            &cursor_tile_pos,
            tiles.transform_size().unwrap().1,
            false,
        );

        // TODO more refined testing if tiles are filled / slopes
        let north = neighbors
            .north
            .map_or(false, |pos| tiles.get(&pos).is_some());
        let east = neighbors
            .east
            .map_or(false, |pos| tiles.get(&pos).is_some());
        let south = neighbors
            .south
            .map_or(false, |pos| tiles.get(&pos).is_some());
        let west = neighbors
            .west
            .map_or(false, |pos| tiles.get(&pos).is_some());

        if north && south || east && west {
            return;
        }

        let mut flip = TileFlip::default();

        if west {
            flip.x = true;
        }
        if north {
            flip.y = true;
        }

        tiles.replace(
            &cursor_tile_pos,
            TileProperties {
                id: TileTextureIndex(1),
                flip,
            },
        );
        editor_state.unsaved_changes = true;
        // TODO need to do this in every system, maybe there is some way to hardcode this?
        self.system_state.apply(world);
    }
    fn update(&mut self, world: &mut World) {}
}
