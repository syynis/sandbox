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
    editor::tiles::{MaterialTileKind, TilePixel},
    level::{
        layer::{Layer, ALL_LAYERS},
        placement::StorageAccess,
        tile::TileKind,
    },
};

use super::{
    palette::Palettes,
    tiles::{Materials, Tiles, TILE_SIZE},
    EditorActions,
};

#[derive(Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct MapTexture {
    pub texture: Handle<Image>,
    pub colored: Handle<Image>,
    pub depth: Vec<u8>,
}

pub fn make_image(width: u32, height: u32, data: Vec<u8>) -> Image {
    let image_size = Extent3d {
        width,
        height,
        ..default()
    };
    let dimension = TextureDimension::D2;
    Image::new(image_size, dimension, data, TextureFormat::Rgba8Unorm)
}

pub fn render_map_images(
    mut cmds: Commands,
    palette: Res<Palettes>,
    mut images: ResMut<Assets<Image>>,
    tiles: Res<Tiles>,
    materials: Res<Materials>,
    storage: StorageAccess,
    tiles_pos: Query<(&TilePos, &TileTextureIndex, &TileFlip)>,
    map_texture: Option<ResMut<MapTexture>>,
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

    if let Some(map_texture) = map_texture {
        images.remove(&map_texture.texture);
    }

    let map_width = map_size.x as usize;
    let map_height = map_size.y as usize;

    let width = map_width * TILE_SIZE;
    let height = map_height * TILE_SIZE;
    let map_center = Vec2::new(width as f32 / 2., height as f32 / 2.);
    let texture_format_size = 4; // 4 channels each a u8

    // TODO make this work for different sized tiles
    let tile = tiles.0.get("small_stone").unwrap();
    let material = materials.0.get("frame").unwrap();
    let mut data: Vec<u8> = vec![0; texture_format_size * width * height];
    let mut depth: Vec<u8> = vec![30; width * height];
    for (l, layer) in ALL_LAYERS.iter().enumerate() {
        let map = storage.storage(*layer).unwrap();
        for sub_layer in 0..1 {
            for (pos, id, flip) in map.iter().filter_map(|tile_entity| {
                let Some(tile_entity) = tile_entity else {
                    return None;
                };
                tiles_pos.get(*tile_entity).ok()
            }) {
                let (x, y) = (pos.x as usize, map_height - pos.y as usize - 1);
                let tile_start = (x + y * width) * TILE_SIZE;
                let tile_center = Vec2::new(x as f32 + 0.5, y as f32 + 0.5) * TILE_SIZE as f32;

                // TODO simplify this
                // How much do we move per depth
                let base_offset = Vec2::new(2., 2.);
                // Distance towards center normalized
                let center_dir = (tile_center - map_center) / map_center;
                // Offset calculated from depth, dir and base_offset
                let offset = center_dir * (10 * l + sub_layer) as f32 * base_offset;
                // Needed for image indexing
                // Negate because for image indexing y = 0 -> top row
                // TODO think about doing this differently, would get rid of i32 casts
                let offset_rounded = -offset.round();
                let offset_x = offset_rounded.x as i32;
                let offset_y = offset_rounded.y as i32;
                let offset_idx = offset_x + offset_y * TILE_SIZE as i32 * map_width as i32;
                let tile_start_offset = (tile_start as i32 + offset_idx) as usize;

                (0..TILE_SIZE)
                    .flat_map(move |ty| (0..TILE_SIZE).map(move |tx| (tx, ty)))
                    .for_each(|(tx, ty)| {
                        let rpos = tx + ty * TILE_SIZE;
                        let wpos = tile_start_offset + tx + ty * TILE_SIZE * map_width;
                        let wpos_texture = wpos * texture_format_size;
                        let layer_idx = (sub_layer + l * 10) as u8;
                        // dont overdraw
                        if depth[wpos] < layer_idx {
                            return;
                        }

                        let dir = match TileKind::from(*id) {
                            TileKind::Square => {
                                let neighbors = Neighbors::get_square_neighboring_positions(
                                    pos, map_size, true,
                                );
                                // Cyclic in last quadrant we want to check West, SouthWest and South
                                // TODO a bit hacky think about doing this in a principled way
                                use SquareDirection::*;
                                let directions: [SquareDirection; 9] = [
                                    West, NorthWest, North, NorthEast, East, SouthEast, South,
                                    SouthWest, West,
                                ];
                                // Which neighbors contain a tile
                                // TODO check if neighbor is solid
                                let neighbors: Vec<bool> = directions
                                    .iter()
                                    .map(|dir| {
                                        neighbors
                                            .get(*dir)
                                            .map_or(false, |npos| map.get(npos).is_some())
                                    })
                                    .collect();
                                material.block.get_pixel(sub_layer, rpos, &neighbors)
                            }
                            TileKind::Slope => {
                                let is_solid = |neighbor: Option<&TilePos>| -> bool {
                                    neighbor.map_or(false, |pos| {
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

                                // Slope can have at most two neighbors that need to be solid
                                use SquareDirection::*;
                                let neighbors = match (flip.x, flip.y) {
                                    (false, false) => vec![West, South],
                                    (true, false) => vec![East, South],
                                    (false, true) => vec![West, North],
                                    (true, true) => vec![East, North],
                                }
                                .iter()
                                .map(|dir| is_solid(cardinal.get(*dir)))
                                .collect();
                                material
                                    .slope
                                    .get_pixel(sub_layer, rpos, flip.clone(), &neighbors)
                            }
                            TileKind::Pole(_) => TilePixel::Neutral,
                            TileKind::Platform => TilePixel::Neutral,
                        };

                        let tpos = tile_start + tx + ty * TILE_SIZE * map_width;
                        let texture_pos = UVec2::new((tpos % width) as u32, (tpos / width) as u32);
                        let dir = material
                            .texture
                            .as_ref()
                            .map_or(dir, |texture| texture.get_pixel(texture_pos));

                        // draw to the depth buffer unless pixel is transparent
                        if !matches!(dir, TilePixel::None) {
                            depth[wpos] = layer_idx;
                        }

                        let [r, g, b, a] = match dir {
                            TilePixel::Up => Color::BLUE,
                            TilePixel::Neutral => Color::GREEN,
                            TilePixel::Down => Color::RED,
                            TilePixel::None => Color::NONE,
                            _ => unreachable!(),
                        }
                        .as_rgba_u8();
                        data[wpos_texture] = r;
                        data[wpos_texture + 1] = g;
                        data[wpos_texture + 2] = b;
                        data[wpos_texture + 3] = a;
                    });
            }
        }
    }

    let mut colored = data.clone();
    colored.chunks_mut(4).enumerate().for_each(|(idx, chunk)| {
        let depth = depth[idx] as usize;
        let sub_layer = depth % 10;
        let layer = depth / 10;
        let (r, g, b, a) = (chunk[0], chunk[1], chunk[2], chunk[3]);
        let dir = match (r, g, b, a) {
            (0, 0, 0, 0) => TilePixel::None,
            (255, 0, 0, 255) => TilePixel::Up,
            (0, 255, 0, 255) => TilePixel::Neutral,
            (0, 0, 255, 255) => TilePixel::Down,
            _ => unreachable!(),
        };
        let [r, g, b, a] = palette
            .get_active()
            .get_shade_color(dir, sub_layer, layer)
            .as_rgba_u8();
        chunk[0] = r;
        chunk[1] = g;
        chunk[2] = b;
        chunk[3] = a;
    });

    let texture = images.add(make_image(width as u32, height as u32, data));
    let colored = images.add(make_image(width as u32, height as u32, colored));

    cmds.insert_resource(MapTexture {
        texture,
        colored,
        depth,
    });
}

#[derive(Component)]
pub struct MapDisplay;
pub fn setup_display(mut cmds: Commands) {
    cmds.spawn((
        MapDisplay,
        SpatialBundle {
            transform: Transform::from_translation(Vec3::new(480., 1000., 0.)),
            ..default()
        },
    ));
}

pub fn display_images(
    mut cmds: Commands,
    map_display: Query<Entity, With<MapDisplay>>,
    map_texture: Res<MapTexture>,
) {
    let Ok(entity) = map_display.get_single() else {
        return;
    };
    if map_texture.is_changed() {
        cmds.entity(entity).despawn_descendants();
        cmds.entity(entity).with_children(|builder| {
            builder.spawn(SpriteBundle {
                texture: map_texture.texture.clone(),
                ..default()
            });
            builder.spawn(SpriteBundle {
                texture: map_texture.colored.clone(),
                transform: Transform::from_translation(Vec3::Y * 1000.),
                ..default()
            });
        });
    }
}
