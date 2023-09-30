use bevy::{
    ecs::system::{SystemParam, SystemState},
    prelude::*,
};

use crate::editor::EditorActions;

use super::{
    util::{draw_tile_outline, CommonToolParams},
    Tool,
};

#[derive(SystemParam)]
struct EraseToolParams<'w, 's> {
    pub common: CommonToolParams<'w, 's>,
}

pub struct EraseTool<'w: 'static, 's: 'static> {
    system_state: SystemState<EraseToolParams<'w, 's>>,
}

impl<'w, 's> Tool for EraseTool<'w, 's> {
    fn new(world: &mut World) -> Self {
        Self {
            system_state: SystemState::new(world),
        }
    }

    fn apply(&mut self, world: &mut World) {
        let EraseToolParams {
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
            tiles.remove(&cursor_tile_pos);
            editor_state.unsaved_changes = true;
        }
        self.system_state.apply(world);
    }
}
