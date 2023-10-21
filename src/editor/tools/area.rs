use bevy::{
    ecs::system::{SystemParam, SystemState},
    prelude::*,
};
use bevy_ecs_tilemap::prelude::*;

use crate::{
    editor::EditorActions,
    level::{layer::ALL_LAYERS, placement::TileProperties, tpos_wpos},
    util::box_lines,
};

use super::{
    util::{draw_tile_outline, CommonToolParams},
    Tool,
};

pub const ALL_MODES: [Mode; 5] = [
    Mode::PlaceLayer,
    Mode::DeleteLayer,
    Mode::PlaceAllLayers,
    Mode::DeleteAllLayers,
    Mode::CopyBack,
];

#[derive(Default, PartialEq, Copy, Clone)]
pub enum Mode {
    #[default]
    PlaceLayer,
    DeleteLayer,
    PlaceAllLayers,
    DeleteAllLayers,
    CopyBack,
}

#[derive(Default, Resource, Deref, DerefMut)]
pub struct ActiveMode(pub Mode);

impl Mode {
    pub fn next(&self) -> Self {
        use Mode::*;
        match self {
            PlaceLayer => DeleteLayer,
            DeleteLayer => PlaceAllLayers,
            PlaceAllLayers => DeleteAllLayers,
            DeleteAllLayers => CopyBack,
            CopyBack => PlaceLayer,
        }
    }

    pub fn name(&self) -> &str {
        use Mode::*;
        match self {
            PlaceLayer => "Place",
            DeleteLayer => "Delete",
            PlaceAllLayers => "Place All",
            DeleteAllLayers => "Delete All",
            CopyBack => "Copy Back",
        }
    }
}

#[derive(SystemParam)]
struct AreaToolParams<'w, 's> {
    pub common: CommonToolParams<'w, 's>,
    pub current_mode: ResMut<'w, ActiveMode>,
}

pub struct AreaTool<'w: 'static, 's: 'static> {
    system_state: SystemState<AreaToolParams<'w, 's>>,
    start: Option<TilePos>,
    temp_end: Option<TilePos>,
    end: Option<TilePos>,
}

impl<'w, 's> Tool for AreaTool<'w, 's> {
    fn new(world: &mut World) -> Self {
        Self {
            system_state: SystemState::new(world),
            start: None,
            temp_end: None,
            end: None,
        }
    }

    fn apply(&mut self, world: &mut World) {
        let AreaToolParams {
            common:
                CommonToolParams {
                    mut tiles,
                    tile_cursor,
                    mut editor_state,
                    mut gizmos,
                    editor_actions,
                },
            mut current_mode,
        } = self.system_state.get_mut(world);

        let cursor_tile_pos = tile_cursor.or_else(|| self.temp_end);
        let Some(cursor_tile_pos) = cursor_tile_pos else {
            return;
        };

        if let (Some(start), Some(end)) = (self.start, self.temp_end) {
            let start = UVec2::from(start);
            let end = UVec2::from(end);
            let (min, max) = (start.min(end), start.max(end));

            let min = tpos_wpos(&TilePos::from(min));
            let max = tpos_wpos(&TilePos::from(max));

            for (start, end) in box_lines(min, max - min + 16.) {
                gizmos.line_2d(start, end, Color::RED);
            }
        }

        if self.start.is_none() {
            draw_tile_outline(tile_cursor, gizmos);
        }

        let Ok(editor_actions) = editor_actions.get_single() else {
            return;
        };

        if editor_actions.just_pressed(EditorActions::CycleToolMode) {
            current_mode.0 = current_mode.next();
        }

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
                    match **current_mode {
                        Mode::PlaceLayer => {
                            tiles.replace(
                                &pos,
                                TileProperties {
                                    id: TileTextureIndex(0),
                                    flip: TileFlip::default(),
                                },
                                editor_state.current_layer,
                            );
                        }
                        Mode::DeleteLayer => {
                            tiles.remove(&pos, editor_state.current_layer);
                        }

                        Mode::PlaceAllLayers => ALL_LAYERS.iter().for_each(|layer| {
                            tiles.replace(
                                &pos,
                                TileProperties {
                                    id: TileTextureIndex(0),
                                    flip: TileFlip::default(),
                                },
                                *layer,
                            )
                        }),

                        Mode::DeleteAllLayers => ALL_LAYERS
                            .iter()
                            .for_each(|layer| tiles.remove(&pos, *layer)),
                        Mode::CopyBack => {
                            if tiles.get(&pos, editor_state.current_layer).is_some() {
                                tiles.replace(
                                    &pos,
                                    TileProperties {
                                        id: TileTextureIndex(0),
                                        flip: TileFlip::default(),
                                    },
                                    editor_state.current_layer.next(),
                                )
                            }
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
