use bevy::{
    ecs::system::{SystemParam, SystemState},
    prelude::*,
};
use bevy_ecs_tilemap::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    editor::{EditorActions, EditorState, ToolActions},
    level::{
        placement::{StorageAccess, TileProperties},
        TileCursor,
    },
};

use super::Tool;

#[derive(SystemParam)]
struct AreaToolParams<'w, 's> {
    pub tiles: StorageAccess<'w, 's>,
    pub tile_cursor: Res<'w, TileCursor>,
    pub editor_state: ResMut<'w, EditorState>,
    pub editor_actions: Query<'w, 's, &'static ActionState<EditorActions>>,
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
    end: Option<TilePos>,
    mode: Mode,
}

impl<'w, 's> Tool for AreaTool<'w, 's> {
    fn new(world: &mut World) -> Self {
        Self {
            system_state: SystemState::new(world),
            start: None,
            end: None,
            mode: Mode::Place,
        }
    }

    fn apply(&mut self, world: &mut World) {
        let AreaToolParams {
            mut tiles,
            tile_cursor,
            mut editor_state,
            editor_actions: action_state,
            ..
        } = self.system_state.get_mut(world);

        let Some(cursor_tile_pos) = **tile_cursor else {
            return;
        };

        let Ok(action_state) = action_state.get_single() else {
            return;
        };

        if action_state.just_pressed(EditorActions::ApplyTool) {
            self.start = Some(cursor_tile_pos);
        }

        if action_state.just_released(EditorActions::ApplyTool) {
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
            editor_state.unsaved_changes = true;
            self.system_state.apply(world);
        }
    }

    fn update(&mut self, world: &mut World) {
        let AreaToolParams {
            tool_actions: action_state,
            ..
        } = self.system_state.get_mut(world);

        let Ok(action_state) = action_state.get_single() else {
            return;
        };

        if action_state.just_pressed(ToolActions::CycleMode) {
            self.mode = self.mode.next();
        }
        self.system_state.apply(world);
    }
}
