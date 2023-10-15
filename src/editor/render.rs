use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_ecs_tilemap::tiles::{TileFlip, TilePos, TileTextureIndex};

use crate::level::{layer::Layer, placement::StorageAccess};

use super::{
    palette::Palette,
    tiles::{Tiles, BASE_TILE_SIZE},
};

#[derive(Resource)]
pub struct MapImages {
    pub images: Vec<Handle<Image>>,
    pub offset: Vec2,
    pub layer_offset: [f32; 3],
}

pub fn render_map_images(
    mut cmds: Commands,
    keys: Res<Input<KeyCode>>,
    palette: Option<Res<Palette>>,
    mut images: ResMut<Assets<Image>>,
    tiles: Option<Res<Tiles>>,
    storage: StorageAccess,
    tiles_pos: Query<(&TilePos, &TileTextureIndex, &TileFlip)>,
) {
    if !keys.just_pressed(KeyCode::Z) {
        return;
    }

    let Some(tiles) = tiles else {
        return;
    };

    let Some(palette) = palette else {
        return;
    };

    let Some((_, size)) = storage.transform_size(Layer::World) else {
        return;
    };

    let map_width = size.x as usize;
    let map_height = size.y as usize;
    let mut map = vec![vec![vec![None; map_width]; map_height]; 3];

    storage
        .storage(Layer::World)
        .unwrap()
        .iter()
        .for_each(|tile_entity| {
            let Some(tile_entity) = tile_entity else {
                return;
            };
            let Ok((pos, id, flip)) = tiles_pos.get(*tile_entity) else {
                return;
            };
            map[0][map_height - pos.y as usize - 1][pos.x as usize] = Some(id.0);
        });

    storage
        .storage(Layer::Near)
        .unwrap()
        .iter()
        .for_each(|tile_entity| {
            let Some(tile_entity) = tile_entity else {
                return;
            };
            let Ok((pos, id, flip)) = tiles_pos.get(*tile_entity) else {
                return;
            };
            map[1][map_height - pos.y as usize - 1][pos.x as usize] = Some(id.0);
        });

    storage
        .storage(Layer::Far)
        .unwrap()
        .iter()
        .for_each(|tile_entity| {
            let Some(tile_entity) = tile_entity else {
                return;
            };
            let Ok((pos, id, flip)) = tiles_pos.get(*tile_entity) else {
                return;
            };
            map[2][map_height - pos.y as usize - 1][pos.x as usize] = Some(id.0);
        });

    let mut map_images = Vec::new();
    let width = map_width * BASE_TILE_SIZE;
    let height = map_height * BASE_TILE_SIZE;
    let texture_format_size = 4; // 4 channels each a u8
    let size = Extent3d {
        width: width as u32,
        height: height as u32,
        ..default()
    };
    let dimension = TextureDimension::D2;

    // TODO make this work for different sized tiles
    let tile = tiles.0.get("small_stone").unwrap();
    for (l, layer) in map.iter().enumerate() {
        for idx in 0..10 {
            let mut data: Vec<u8> = vec![0; (texture_format_size * width * height) as usize];
            for (y, row) in layer.iter().enumerate() {
                for (x, tile_id) in row.iter().enumerate() {
                    let start = (x + y * width) * BASE_TILE_SIZE;
                    for vy in 0..BASE_TILE_SIZE {
                        for vx in 0..BASE_TILE_SIZE {
                            let rpos = vx + vy * BASE_TILE_SIZE;
                            let wpos = (start + vx + vy * BASE_TILE_SIZE * map_width)
                                * texture_format_size;

                            let set_color = |d: &mut [u8], color: Color, idx: usize| {
                                let [r, g, b, a] = color.as_rgba_u8();
                                d[idx] = r;
                                d[idx + 1] = g;
                                d[idx + 2] = b;
                                d[idx + 3] = a;
                            };
                            if let Some(tile_id) = tile_id {
                                let dir = tile.get_kind(idx, rpos) as usize;
                                let color = palette.get_sun_color(dir, idx, l);
                                set_color(&mut data, color, wpos);
                            }
                        }
                    }
                }
            }

            let image = Image::new(size, dimension, data, TextureFormat::Rgba8Unorm);
            let handle = images.add(image);

            let offset = Vec2::splat(0.5);
            let layer = match l {
                2 => 10,
                x => x,
            } as f32;
            let pos = (offset * 10.).extend(-10.) * layer;
            cmds.spawn(SpriteBundle {
                texture: handle.clone(),
                transform: Transform::from_translation(
                    Vec3::new(0., 1000., 0.) + pos + offset.extend(-1.) * idx as f32,
                ),
                ..default()
            });

            map_images.push(handle);
        }
    }
    cmds.insert_resource(MapImages {
        images: map_images,
        offset: Vec2::splat(0.5),
        layer_offset: [0., 1., 10.],
    });
}

pub fn clear_map(world: &mut World) {
    let pressed = world.resource::<Input<KeyCode>>().just_pressed(KeyCode::A);
    if pressed {
        world.resource_scope(|world, state: Mut<MapImages>| {
            let mut images = world.resource_mut::<Assets<Image>>();
            for handle in &state.images {
                images.remove(handle);
            }
        });

        world.remove_resource::<MapImages>();
    }
}
