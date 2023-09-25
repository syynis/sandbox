use crate::editor::EditorState;
use crate::level::placement::{StorageAccess, TileProperties};
use crate::level::TileCursor;
use bevy::ecs::system::{SystemParam, SystemState};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use super::Tool;

#[derive(SystemParam)]
struct PaintToolParams<'w, 's> {
    pub tiles: StorageAccess<'w, 's>,
    pub tile_cursor: Res<'w, TileCursor>,
    pub editor_state: ResMut<'w, EditorState>,
}

pub struct PaintTool<'w: 'static, 's: 'static> {
    system_state: SystemState<PaintToolParams<'w, 's>>,
}

impl<'w, 's> Tool for PaintTool<'w, 's> {
    fn new(world: &mut bevy::prelude::World) -> Self {
        Self {
            system_state: SystemState::new(world),
        }
    }
    fn apply(&mut self, world: &mut World) {
        let PaintToolParams {
            mut tiles,
            tile_cursor,
            mut editor_state,
        } = self.system_state.get_mut(world);

        let Some(cursor_tile_pos) = **tile_cursor else {
            return;
        };

        tiles.replace(
            &cursor_tile_pos,
            TileProperties {
                id: TileTextureIndex(0),
                flip: TileFlip::default(),
            },
        );

        editor_state.unsaved_changes = true;
        // TODO need to do this in every system, maybe there is some way to hardcode this?
        self.system_state.apply(world);
    }
    fn update(&mut self, world: &mut World) {}
}
