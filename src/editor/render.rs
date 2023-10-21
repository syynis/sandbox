use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_ecs_tilemap::{
    helpers::square_grid::neighbors::{Neighbors, SquareDirection},
    tiles::{TileFlip, TilePos, TileTextureIndex},
};
use leafwing_input_manager::prelude::ActionState;

use crate::{
    editor::tiles::TilePixel,
    level::{
        layer::{Layer, ALL_LAYERS},
        placement::StorageAccess,
        tile::TileKind,
    },
};

use super::{
    palette::Palette,
    tiles::{Materials, Tiles, TILE_SIZE},
    EditorActions,
};

#[derive(Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct MapImages {
    pub images: Vec<Handle<Image>>,
    pub offset: Vec2,
}

pub fn render_map_images(
    mut cmds: Commands,
    palette: Res<Palette>,
    mut images: ResMut<Assets<Image>>,
    tiles: Res<Tiles>,
    materials: Res<Materials>,
    storage: StorageAccess,
    tiles_pos: Query<(&TilePos, &TileTextureIndex, &TileFlip)>,
    map_images: Option<ResMut<MapImages>>,
    editor_actions: Query<&ActionState<EditorActions>>,
) {
    let Ok(editor_actions) = editor_actions.get_single() else {
        return;
    };
    if !editor_actions.just_pressed(EditorActions::ReloadMapDisplay) {
        return;
    }
    let Some((_, map_size)) = storage.transform_size(Layer::World) else {
        return;
    };

    if let Some(map_images) = map_images {
        for handle in &map_images.images {
            images.remove(handle);
        }
    }

    let map_width = map_size.x as usize;
    let map_height = map_size.y as usize;

    let width = map_width * TILE_SIZE;
    let height = map_height * TILE_SIZE;
    let center = Vec2::new(width as f32 / 2., height as f32 / 2.);
    let texture_format_size = 4; // 4 channels each a u8

    // TODO make this work for different sized tiles
    let mut map_images = Vec::new();
    let tile = tiles.0.get("small_stone").unwrap();
    let material = materials.0.get("standard").unwrap();
    for (l, layer) in ALL_LAYERS.iter().enumerate() {
        let map = storage.storage(*layer).unwrap();
        for sub_layer in 0..10 {
            let mut data: Vec<u8> = vec![0; (texture_format_size * width * height) as usize];
            for (pos, id, flip) in map.iter().filter_map(|tile_entity| {
                let Some(tile_entity) = tile_entity else {
                    return None;
                };
                tiles_pos.get(*tile_entity).ok()
            }) {
                let (x, y) = (pos.x as usize, map_height - pos.y as usize - 1);
                let tile_start = (x + y * width) * TILE_SIZE;
                let tile_center = Vec2::new(x as f32 + 0.5, y as f32 + 0.5) * TILE_SIZE as f32;

                let base_offset = (tile_center - center) / center * Vec2::new(2., 2.);
                let layer_offset = base_offset * 10. * l as f32;
                let offset = layer_offset + base_offset * sub_layer as f32;
                let offset_rounded = -offset.round();
                let offset_x = offset_rounded.x as i32;
                let offset_y = offset_rounded.y as i32;
                let offset_idx = offset_x + offset_y * TILE_SIZE as i32 * map_width as i32;
                let tile_start = tile_start as i32 + offset_idx;
                let tile_start = tile_start as usize;

                (0..TILE_SIZE)
                    .flat_map(move |ty| (0..TILE_SIZE).map(move |tx| (tx, ty)))
                    .for_each(|(tx, ty)| {
                        let rpos = tx + ty * TILE_SIZE;
                        let wpos =
                            (tile_start + tx + ty * TILE_SIZE * map_width) * texture_format_size;

                        let set_color = |d: &mut [u8], color: Color, idx: usize| {
                            let [r, g, b, a] = color.as_rgba_u8();
                            d[idx] = r;
                            d[idx + 1] = g;
                            d[idx + 2] = b;
                            d[idx + 3] = a;
                        };
                        let dir = match TileKind::from(*id) {
                            TileKind::Square => {
                                let neighbors = Neighbors::get_square_neighboring_positions(
                                    pos, map_size, true,
                                );
                                use SquareDirection::*;
                                let directions: [SquareDirection; 9] = [
                                    West, NorthWest, North, NorthEast, East, SouthEast, South,
                                    SouthWest, West,
                                ];
                                let neighbors: Vec<bool> = directions
                                    .iter()
                                    .map(|dir| match neighbors.get(*dir) {
                                        Some(npos) => map.get(npos).is_some(),
                                        None => false,
                                    })
                                    .collect();
                                material.block.get_pixel(sub_layer, rpos, &neighbors)
                            }
                            TileKind::Slope => {
                                let is_solid = |dir: Option<&TilePos>| -> bool {
                                    dir.map_or(false, |pos| {
                                        storage.get_properties(&pos, *layer).map_or(
                                            false,
                                            |tile_properties| {
                                                TileKind::from(tile_properties.id).is_solid()
                                            },
                                        )
                                    })
                                };
                                let cardinal = Neighbors::get_square_neighboring_positions(
                                    pos, map_size, false,
                                );

                                use SquareDirection::*;
                                let directions = match (flip.x, flip.y) {
                                    (false, false) => (West, South),
                                    (true, false) => (East, South),
                                    (false, true) => (West, North),
                                    (true, true) => (East, North),
                                };
                                let horizontal = is_solid(cardinal.get(directions.0));
                                let vertical = is_solid(cardinal.get(directions.1));
                                let neighbors = vec![horizontal, vertical];
                                material
                                    .slope
                                    .get_pixel(sub_layer, rpos, flip.clone(), &neighbors)
                            }
                            TileKind::Pole(_) => TilePixel::Neutral,
                            TileKind::Platform => TilePixel::Neutral,
                        } as usize;

                        // let dir = tile.get_pixel(sub_layer, rpos) as usize;
                        let color = palette.get_sun_color(dir, sub_layer, l);
                        /*
                        let color = match dir {
                            0 => Color::BLUE,
                            1 => Color::GREEN,
                            2 => Color::RED,
                            3 => Color::NONE,
                            _ => unreachable!(),
                        };
                        */
                        set_color(&mut data, color, wpos);
                    });
            }

            let image_size = Extent3d {
                width: width as u32,
                height: height as u32,
                ..default()
            };
            let dimension = TextureDimension::D2;
            let image = Image::new(image_size, dimension, data, TextureFormat::Rgba8Unorm);
            let handle = images.add(image);
            map_images.push(handle);
        }
    }
    cmds.insert_resource(MapImages {
        images: map_images,
        offset: Vec2::splat(0.5),
    });
}

#[derive(Component)]
pub struct MapDisplay;
pub fn setup_display(mut cmds: Commands) {
    cmds.spawn((
        MapDisplay,
        SpatialBundle {
            transform: Transform::from_translation(Vec3::new(0., 1000., 0.)),
            ..default()
        },
    ));
}

pub fn display_images(
    mut cmds: Commands,
    map_display: Query<Entity, With<MapDisplay>>,
    map_images: Res<MapImages>,
) {
    let Ok(entity) = map_display.get_single() else {
        return;
    };
    if map_images.is_changed() {
        cmds.entity(entity).despawn_descendants();
        for (idx, image) in map_images.images.iter().enumerate() {
            let sub_layer = idx % 10;
            let layer = idx / 10;
            cmds.entity(entity).with_children(|child_builder| {
                child_builder.spawn(SpriteBundle {
                    texture: image.clone(),
                    transform: Transform::from_translation(
                        Vec3::NEG_Z * sub_layer as f32 + Vec3::NEG_Z * (10 * layer) as f32,
                    ),
                    ..default()
                });
            });
        }
    }
}
