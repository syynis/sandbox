use bevy::{
    ecs::system::{SystemParam, SystemState},
    prelude::*,
};
use bevy_ecs_tilemap::{helpers::square_grid::neighbors::Neighbors, prelude::*};

use crate::{editor::EditorActions, level::placement::TileProperties};

use super::{
    util::{draw_tile_outline, CommonToolParams},
    Tool,
};

#[derive(SystemParam)]
struct SlopeToolParams<'w, 's> {
    pub common: CommonToolParams<'w, 's>,
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
            common:
                CommonToolParams {
                    mut tiles,
                    tile_cursor,
                    mut editor_state,
                    lines,
                    editor_actions,
                },
        } = self.system_state.get_mut(world);

        let Some(cursor_tile_pos) = **tile_cursor else {
            return;
        };

        draw_tile_outline(tile_cursor, lines);

        let Ok(editor_actions) = editor_actions.get_single() else {
            return;
        };

        if editor_actions.pressed(EditorActions::ApplyTool) {
            let current_layer = editor_state.current_layer;
            let neighbors = Neighbors::get_square_neighboring_positions(
                &cursor_tile_pos,
                tiles.transform_size(current_layer).unwrap().1,
                false,
            );

            // TODO more refined testing if tiles are filled / slopes
            let north = neighbors
                .north
                .map_or(false, |pos| tiles.get(&pos, current_layer).is_some());
            let east = neighbors
                .east
                .map_or(false, |pos| tiles.get(&pos, current_layer).is_some());
            let south = neighbors
                .south
                .map_or(false, |pos| tiles.get(&pos, current_layer).is_some());
            let west = neighbors
                .west
                .map_or(false, |pos| tiles.get(&pos, current_layer).is_some());

            if north && south || east && west {
                return;
            }

            let mut flip = TileFlip::default();

            if east {
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
                current_layer,
            );
            editor_state.unsaved_changes = true;
        }

        self.system_state.apply(world);
    }
}
