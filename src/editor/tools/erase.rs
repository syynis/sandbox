use bevy::{
    ecs::system::{SystemParam, SystemState},
    prelude::*,
};

use crate::{
    editor::EditorState,
    level::{placement::StorageAccess, TileCursor},
};

use super::Tool;

#[derive(SystemParam)]
struct EraseToolParams<'w, 's> {
    pub tiles: StorageAccess<'w, 's>,
    pub tile_cursor: Res<'w, TileCursor>,
    pub editor_state: ResMut<'w, EditorState>,
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
            mut tiles,
            tile_cursor,
            mut editor_state,
        } = self.system_state.get_mut(world);

        let Some(cursor_tile_pos) = **tile_cursor else {
            return;
        };

        tiles.remove(&cursor_tile_pos);
        editor_state.unsaved_changes = true;
        self.system_state.apply(world);
    }
    fn update(&mut self, world: &mut World) {}
}
