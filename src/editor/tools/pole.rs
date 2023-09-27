use bevy::{
    ecs::system::{SystemParam, SystemState},
    prelude::*,
};
use bevy_ecs_tilemap::prelude::*;
use bevy_prototype_debug_lines::DebugLines;
use leafwing_input_manager::prelude::*;

use crate::{
    editor::{EditorState, ToolActions},
    level::{
        placement::{StorageAccess, TileProperties},
        tpos_wpos, TileCursor,
    },
    util::box_lines,
};

use super::Tool;

#[derive(SystemParam)]
struct PoleToolParams<'w, 's> {
    pub tiles: StorageAccess<'w, 's>,
    pub tile_cursor: Res<'w, TileCursor>,
    pub editor_state: ResMut<'w, EditorState>,
    pub action_state: Query<'w, 's, &'static ActionState<ToolActions>>,
    pub lines: ResMut<'w, DebugLines>,
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
            mut tiles,
            tile_cursor,
            mut editor_state,
            ..
        } = self.system_state.get_mut(world);

        let Some(cursor_tile_pos) = **tile_cursor else {
            return;
        };

        let id: i32 = if self.place_horizontal { 3 } else { 2 };
        let id = if tiles
            .get_properties(&cursor_tile_pos)
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
        );

        editor_state.unsaved_changes = true;
        // TODO need to do this in every system, maybe there is some way to hardcode this?
        self.system_state.apply(world);
    }

    fn update(&mut self, world: &mut World) {
        let PoleToolParams {
            action_state,
            tile_cursor,
            mut lines,
            ..
        } = self.system_state.get_mut(world);

        let Ok(action_state) = action_state.get_single() else {
            return;
        };

        if action_state.just_pressed(ToolActions::CycleMode) {
            self.place_horizontal = !self.place_horizontal;
        }

        // TODO every system wants this think about best way to factor this out
        if let Some(tile_cursor) = **tile_cursor {
            let wpos = tpos_wpos(&tile_cursor);

            for (start, end) in box_lines(wpos.extend(0.), Vec2::new(16., 16.)) {
                lines.line_colored(start, end, 0., Color::RED);
            }
        }
        self.system_state.apply(world);
    }
}
