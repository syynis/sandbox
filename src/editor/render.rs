use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_ecs_tilemap::tiles::{TileFlip, TilePos, TileTextureIndex};

use crate::level::{
    layer::{Layer, ALL_LAYERS},
    placement::StorageAccess,
};

use super::{
    palette::Palette,
    tiles::{Tiles, TILE_SIZE},
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

    let width = map_width * TILE_SIZE;
    let height = map_height * TILE_SIZE;
    let texture_format_size = 4; // 4 channels each a u8
    let size = Extent3d {
        width: width as u32,
        height: height as u32,
        ..default()
    };
    let dimension = TextureDimension::D2;

    // TODO make this work for different sized tiles
    let mut map_images = Vec::new();
    let tile = tiles.0.get("small_stone").unwrap();
    for (l, layer) in ALL_LAYERS.iter().enumerate() {
        let mapp = storage.storage(*layer).unwrap();
        for sub_layer in 0..10 {
            let mut data: Vec<u8> = vec![0; (texture_format_size * width * height) as usize];
            for (pos, id, flip) in mapp.iter().filter_map(|tile_entity| {
                let Some(tile_entity) = tile_entity else {
                    return None;
                };
                tiles_pos.get(*tile_entity).ok()
            }) {
                let (x, y) = (pos.x as usize, map_height - pos.y as usize - 1);
                let start = (x + y * width) * TILE_SIZE;

                (0..TILE_SIZE)
                    .flat_map(move |vy| (0..TILE_SIZE).map(move |vx| (vx, vy)))
                    .for_each(|(vx, vy)| {
                        let rpos = vx + vy * TILE_SIZE;
                        let wpos = (start + vx + vy * TILE_SIZE * map_width) * texture_format_size;

                        let set_color = |d: &mut [u8], color: Color, idx: usize| {
                            let [r, g, b, a] = color.as_rgba_u8();
                            d[idx] = r;
                            d[idx + 1] = g;
                            d[idx + 2] = b;
                            d[idx + 3] = a;
                        };
                        let dir = tile.get_pixel(sub_layer, rpos) as usize;
                        let color = palette.get_sun_color(dir, sub_layer, l);
                        set_color(&mut data, color, wpos);
                    });
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
                    Vec3::new(0., 1000., 0.) + pos + offset.extend(-1.) * sub_layer as f32,
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
