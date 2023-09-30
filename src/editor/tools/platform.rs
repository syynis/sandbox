use bevy::{
    ecs::system::{SystemParam, SystemState},
    prelude::*,
};
use bevy_ecs_tilemap::tiles::{TileFlip, TileTextureIndex};

use crate::{editor::EditorActions, level::placement::TileProperties};

use super::{
    util::{draw_tile_outline, CommonToolParams},
    Tool,
};

#[derive(SystemParam)]
struct PlatformToolParams<'w, 's> {
    common: CommonToolParams<'w, 's>,
}

pub struct PlatformTool<'w: 'static, 's: 'static> {
    system_state: SystemState<PlatformToolParams<'w, 's>>,
}

impl<'w, 's> Tool for PlatformTool<'w, 's> {
    fn new(world: &mut bevy::prelude::World) -> Self {
        Self {
            system_state: SystemState::new(world),
        }
    }
    fn apply(&mut self, world: &mut World) {
        let PlatformToolParams {
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
            tiles.replace(
                &cursor_tile_pos,
                TileProperties {
                    id: TileTextureIndex(5),
                    flip: TileFlip::default(),
                },
            );

            editor_state.unsaved_changes = true;
        }

        self.system_state.apply(world);
    }
}
