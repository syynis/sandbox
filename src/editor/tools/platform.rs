use crate::editor::EditorState;
use crate::level::placement::{StorageAccess, TileProperties};
use crate::level::{tpos_wpos, TileCursor};
use crate::util::box_lines;
use bevy::ecs::system::{SystemParam, SystemState};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_prototype_debug_lines::DebugLines;

use super::Tool;

#[derive(SystemParam)]
struct PlatformToolParams<'w, 's> {
    pub tiles: StorageAccess<'w, 's>,
    pub tile_cursor: Res<'w, TileCursor>,
    pub editor_state: ResMut<'w, EditorState>,
    pub lines: ResMut<'w, DebugLines>,
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
            mut tiles,
            tile_cursor,
            mut editor_state,
            ..
        } = self.system_state.get_mut(world);

        let Some(cursor_tile_pos) = **tile_cursor else {
            return;
        };

        tiles.replace(
            &cursor_tile_pos,
            TileProperties {
                id: TileTextureIndex(5),
                flip: TileFlip::default(),
            },
        );

        editor_state.unsaved_changes = true;
        // TODO need to do this in every system, maybe there is some way to hardcode this?
        self.system_state.apply(world);
    }
    fn update(&mut self, world: &mut World) {
        let PlatformToolParams {
            tile_cursor,
            mut lines,
            ..
        } = self.system_state.get_mut(world);

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
