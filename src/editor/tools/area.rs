use bevy::{
    ecs::system::{SystemParam, SystemState},
    prelude::*,
};
use bevy_ecs_tilemap::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    editor::{EditorActions, ToolActions},
    level::{placement::TileProperties, tpos_wpos},
    util::box_lines,
};

use super::{
    util::{draw_tile_outline, CommonToolParams},
    Tool,
};

#[derive(SystemParam)]
struct AreaToolParams<'w, 's> {
    pub common: CommonToolParams<'w, 's>,
    pub tool_actions: Query<'w, 's, &'static ActionState<ToolActions>>,
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
            common:
                CommonToolParams {
                    mut tiles,
                    tile_cursor,
                    mut editor_state,
                    mut lines,
                    editor_actions,
                },
            tool_actions,
        } = self.system_state.get_mut(world);

        let cursor_tile_pos = tile_cursor.or_else(|| self.temp_end);
        let Some(cursor_tile_pos) = cursor_tile_pos else {
            return;
        };

        let Ok(tool_actions) = tool_actions.get_single() else {
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

        if self.start.is_none() {
            draw_tile_outline(tile_cursor, lines);
        }

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
                                editor_state.current_layer,
                            );
                        }
                        Mode::Delete => {
                            tiles.remove(&pos, editor_state.current_layer);
                        }
                    }
                });
            });
            self.start = None;
            self.end = None;
            self.temp_end = None;
            editor_state.unsaved_changes = true;
        }
        self.system_state.apply(world);
    }
}
