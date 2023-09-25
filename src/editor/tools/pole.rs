use bevy::{
    ecs::system::{SystemParam, SystemState},
    prelude::*,
};
use bevy_ecs_tilemap::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::{
    editor::{EditorState, ToolActions},
    level::{
        placement::{StorageAccess, TileProperties},
        TileCursor,
    },
};

use super::Tool;

#[derive(SystemParam)]
struct PoleToolParams<'w, 's> {
    pub tiles: StorageAccess<'w, 's>,
    pub tile_cursor: Res<'w, TileCursor>,
    pub editor_state: ResMut<'w, EditorState>,
    pub action_state: Query<'w, 's, &'static ActionState<ToolActions>>,
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

        let id: u32 = if self.place_horizontal { 3 } else { 2 };
        let id = if tiles
            .get_properties(&cursor_tile_pos)
            .map_or(false, |prop| id.saturating_sub(prop.id.0) == 1)
        {
            5
        } else {
            id
        };

        tiles.replace(
            &cursor_tile_pos,
            TileProperties {
                id: TileTextureIndex(id),
                flip: TileFlip::default(),
            },
        );

        editor_state.unsaved_changes = true;
        // TODO need to do this in every system, maybe there is some way to hardcode this?
        self.system_state.apply(world);
    }

    fn update(&mut self, world: &mut World) {
        let PoleToolParams { action_state, .. } = self.system_state.get_mut(world);

        let Ok(action_state) = action_state.get_single() else {
            return;
        };

        if action_state.just_pressed(ToolActions::CycleMode) {
            self.place_horizontal = !self.place_horizontal;
        }
        self.system_state.apply(world);
    }
}
