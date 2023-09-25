use bevy::{
    ecs::system::{SystemParam, SystemState},
    prelude::*,
};
use bevy_ecs_tilemap::prelude::*;
use bevy_prototype_debug_lines::DebugLines;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    editor::{EditorActions, EditorState, ToolActions},
    level::{
        placement::{StorageAccess, TileProperties},
        tpos_wpos, TileCursor,
    },
    util::box_lines,
};

use super::Tool;

#[derive(SystemParam)]
struct AreaToolParams<'w, 's> {
    pub tiles: StorageAccess<'w, 's>,
    pub tile_cursor: Res<'w, TileCursor>,
    pub editor_state: ResMut<'w, EditorState>,
    pub editor_actions: Query<'w, 's, &'static ActionState<EditorActions>>,
    pub tool_actions: Query<'w, 's, &'static ActionState<ToolActions>>,
    pub lines: ResMut<'w, DebugLines>,
}

enum Mode {
    Place,
    Delete,
}

impl Mode {
    fn next(&mut self) -> Self {
        use Mode::*;
        match self {
            Place => Delete,
            Delete => Place,
        }
    }
}
pub struct AreaTool<'w: 'static, 's: 'static> {
    system_state: SystemState<AreaToolParams<'w, 's>>,
    start: Option<TilePos>,
    temp_end: Option<TilePos>,
    end: Option<TilePos>,
    mode: Mode,
}

impl<'w, 's> Tool for AreaTool<'w, 's> {
    fn new(world: &mut World) -> Self {
        Self {
            system_state: SystemState::new(world),
            start: None,
            temp_end: None,
            end: None,
            mode: Mode::Place,
        }
    }

    fn apply(&mut self, world: &mut World) {
        let AreaToolParams {
            mut tiles,
            tile_cursor,
            mut editor_state,
            editor_actions,
            ..
        } = self.system_state.get_mut(world);

        let cursor_tile_pos = tile_cursor.or_else(|| self.temp_end);
        let Some(cursor_tile_pos) = cursor_tile_pos else {
            return;
        };

        let Ok(editor_actions) = editor_actions.get_single() else {
            return;
        };

        if editor_actions.just_pressed(EditorActions::ApplyTool) {
            self.start = Some(cursor_tile_pos);
        }

        if editor_actions.pressed(EditorActions::ApplyTool) {
            self.temp_end = Some(cursor_tile_pos);
        }

        if editor_actions.just_released(EditorActions::ApplyTool) {
            if self.start.is_some() {
                self.end = Some(cursor_tile_pos);
            }
        }

        if let (Some(start), Some(end)) = (self.start, self.end) {
            let start = UVec2::from(start);
            let end = UVec2::from(end);
            let (min, max) = (start.min(end), start.max(end));
            (min.x..=max.x).for_each(|x| {
                (min.y..=max.y).for_each(|y| {
                    let pos = TilePos { x, y };
                    match self.mode {
                        Mode::Place => {
                            tiles.replace(
                                &pos,
                                TileProperties {
                                    id: TileTextureIndex(0),
                                    flip: TileFlip::default(),
                                },
                            );
                        }
                        Mode::Delete => {
                            tiles.remove(&pos);
                        }
                    }
                });
            });
            self.start = None;
            self.end = None;
            self.temp_end = None;
            editor_state.unsaved_changes = true;
            self.system_state.apply(world);
        }
    }

    fn update(&mut self, world: &mut World) {
        let AreaToolParams {
            tool_actions,
            editor_actions,
            mut lines,
            tile_cursor,
            ..
        } = self.system_state.get_mut(world);

        let Ok(tool_actions) = tool_actions.get_single() else {
            return;
        };

        let Ok(editor_actions) = editor_actions.get_single() else {
            return;
        };

        if tool_actions.just_pressed(ToolActions::CycleMode) {
            self.mode = self.mode.next();
        }

        if let (Some(start), Some(end)) = (self.start, self.temp_end) {
            let start = UVec2::from(start);
            let end = UVec2::from(end);
            let (min, max) = (start.min(end), start.max(end));

            let min = tpos_wpos(&TilePos::from(min));
            let max = tpos_wpos(&TilePos::from(max));

            for (start, end) in box_lines(min.extend(0.), max - min + 16.) {
                lines.line_colored(start, end, 0., Color::RED);
            }
        }

        if !editor_actions.pressed(EditorActions::ApplyTool) {
            // TODO every system wants this think about best way to factor this out
            if let Some(tile_cursor) = **tile_cursor {
                let wpos = tpos_wpos(&tile_cursor);

                for (start, end) in box_lines(wpos.extend(0.), Vec2::new(16., 16.)) {
                    lines.line_colored(start, end, 0., Color::RED);
                }
            }
        }

        self.system_state.apply(world);
    }
}
