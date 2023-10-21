use bevy::{ecs::system::SystemParam, prelude::*};
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
    pub gizmos: Gizmos<'s>,
    pub editor_actions: Query<'w, 's, &'static ActionState<EditorActions>>,
}

pub fn draw_tile_outline(tile_cursor: Res<TileCursor>, mut gizmos: Gizmos) {
    if let Some(tile_cursor) = **tile_cursor {
        let wpos = tpos_wpos(&tile_cursor);

        for (start, end) in box_lines(wpos, Vec2::new(16., 16.)) {
            gizmos.line_2d(start, end, Color::RED);
        }
    }
}
