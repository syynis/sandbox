use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_prototype_debug_lines::DebugLines;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    editor::{EditorActions, EditorState},
    level::{placement::StorageAccess, tpos_wpos, TileCursor},
    util::box_lines,
};

#[derive(SystemParam)]
pub struct CommonToolParams<'w, 's> {
    pub tiles: StorageAccess<'w, 's>,
    pub tile_cursor: Res<'w, TileCursor>,
    pub editor_state: ResMut<'w, EditorState>,
    pub lines: ResMut<'w, DebugLines>,
    pub editor_actions: Query<'w, 's, &'static ActionState<EditorActions>>,
}

pub fn draw_tile_outline(tile_cursor: Res<TileCursor>, mut lines: ResMut<DebugLines>) {
    if let Some(tile_cursor) = **tile_cursor {
        let wpos = tpos_wpos(&tile_cursor);

        for (start, end) in box_lines(wpos.extend(0.), Vec2::new(16., 16.)) {
            lines.line_colored(start, end, 0., Color::RED);
        }
    }
}
