use bevy::{
    ecs::system::{SystemParam, SystemState},
    prelude::*,
};
use bevy_ecs_tilemap::prelude::*;

use crate::{editor::EditorActions, level::placement::TileProperties};

use super::{
    util::{draw_tile_outline, CommonToolParams},
    Tool,
};

#[derive(SystemParam)]
struct PoleToolParams<'w, 's> {
    pub common: CommonToolParams<'w, 's>,
}

pub struct PoleTool<'w: 'static, 's: 'static> {
    system_state: SystemState<PoleToolParams<'w, 's>>,
    place_horizontal: bool,
}

impl<'w, 's> Tool for PoleTool<'w, 's> {
    fn new(world: &mut bevy::prelude::World) -> Self {
        Self {
            system_state: SystemState::new(world),
            place_horizontal: false,
        }
    }
    fn apply(&mut self, world: &mut World) {
        let PoleToolParams {
            common:
                CommonToolParams {
                    mut tiles,
                    tile_cursor,
                    gizmos,
                    mut editor_state,
                    editor_actions,
                },
        } = self.system_state.get_mut(world);

        let Some(cursor_tile_pos) = **tile_cursor else {
            return;
        };

        draw_tile_outline(tile_cursor, gizmos);

        let Ok(editor_actions) = editor_actions.get_single() else {
            return;
        };

        if editor_actions.just_pressed(EditorActions::CycleToolMode) {
            self.place_horizontal = !self.place_horizontal;
        }

        if editor_actions.pressed(EditorActions::ApplyTool) {
            let id: i32 = if self.place_horizontal { 3 } else { 2 };
            let id = if tiles
                .get_properties(&cursor_tile_pos, editor_state.current_layer)
                .map_or(false, |prop| {
                    prop.id.0 == 4 || (id - prop.id.0 as i32).abs() == 1
                }) {
                4
            } else {
                id
            };

            tiles.replace(
                &cursor_tile_pos,
                TileProperties {
                    id: TileTextureIndex(id as u32),
                    flip: TileFlip::default(),
                },
                editor_state.current_layer,
            );

            editor_state.unsaved_changes = true;
        }
        self.system_state.apply(world);
    }
}
