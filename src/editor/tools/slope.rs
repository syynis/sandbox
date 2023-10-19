use bevy::{
    ecs::system::{SystemParam, SystemState},
    prelude::*,
};
use bevy_ecs_tilemap::{helpers::square_grid::neighbors::Neighbors, prelude::*};

use crate::{
    editor::EditorActions,
    level::{placement::TileProperties, tile::TileKind},
};

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
            let is_solid = |dir: Option<TilePos>| -> bool {
                dir.map_or(false, |pos| {
                    tiles
                        .get_properties(&pos, current_layer)
                        .map_or(false, |tile_properties| {
                            TileKind::from(tile_properties.id).is_solid()
                        })
                })
            };

            let north = is_solid(neighbors.north);
            let east = is_solid(neighbors.east);
            let south = is_solid(neighbors.south);
            let west = is_solid(neighbors.west);

            let count = north as u8 + east as u8 + south as u8 + west as u8;
            if !(count == 1 || count == 2) {
                return;
            }
            if north && south || east && west {
                return;
            }

            // TODO figure out better control flow
            let mut skip = false;
            if count == 1 {
                if let Some(properties) = tiles.get_properties(&cursor_tile_pos, current_layer) {
                    if TileKind::from(properties.id).is_slope() {
                        if editor_actions.just_pressed(EditorActions::ApplyTool) {
                            let old_flip = properties.flip;
                            let new_flip = if north || south {
                                TileFlip {
                                    x: !old_flip.x,
                                    ..old_flip
                                }
                            } else {
                                TileFlip {
                                    y: !old_flip.y,
                                    ..old_flip
                                }
                            };

                            tiles.replace(
                                &cursor_tile_pos,
                                TileProperties {
                                    flip: new_flip,
                                    ..properties
                                },
                                current_layer,
                            );
                        }
                        skip = true;
                    }
                }
            }

            if !skip {
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
            }
            editor_state.unsaved_changes = true;
        }

        self.system_state.apply(world);
    }
}
