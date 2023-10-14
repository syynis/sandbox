use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::{PanCam, PanCamPlugin};
use rand::{seq::IteratorRandom, thread_rng};
use sandbox::editor::{palette::Palette, tiles::Tiles, AppState, EditorPlugin};

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins.set(ImagePlugin::default_nearest()),
        PanCamPlugin::default(),
        WorldInspectorPlugin::default(),
        EditorPlugin,
    ));

    app.insert_resource(ClearColor(Color::DARK_GRAY));
    app.insert_resource(MapImages::default());

    app.add_systems(Startup, setup);

    // Display state
    app.add_systems(OnEnter(AppState::Display), spawn_map);

    app.run()
}

fn setup(mut cmds: Commands) {
    cmds.spawn((
        Camera2dBundle::default(),
        PanCam {
            grab_buttons: vec![MouseButton::Middle],
            ..default()
        },
    ));
}

#[derive(Default, Resource)]
pub struct MapImages(pub Vec<Handle<Image>>);

fn spawn_map(
    mut cmds: Commands,
    palette: Res<Palette>,
    mut images: ResMut<Assets<Image>>,
    tiles: Res<Tiles>,
    mut map_images: ResMut<MapImages>,
) {
    cmds.insert_resource(ClearColor(palette.meta.skycolor));

    let mut map = vec![vec![0; 64]; 36];
    let map_width = map[0].len();
    let map_height = map.len();
    let mut rng = thread_rng();

    let voxel_size = 20;
    let width = map_width * voxel_size;
    let height = map_height * voxel_size;
    let texture_format_size = 4; // 4 channels each a u8
    let voxel_data_size = voxel_size * texture_format_size;
    let size = Extent3d {
        width: width as u32,
        height: height as u32,
        ..default()
    };
    let dimension = TextureDimension::D2;

    // TODO Probably better to work in pixel coordinates and multiply by texture format size in the end instead of carrying it all the way
    for l in 0..3 {
        (0..(map_width * map_height / 10)).for_each(|_| {
            let row = (0..map_height).choose(&mut rng).unwrap();
            let elem = (0..map_width).choose(&mut rng).unwrap();
            map[row][elem] = 1;
        });
        for idx in 0..10 {
            let (tile_handle, tile_meta) = tiles.0.get("small_stone").unwrap();
            let tile_image = images.get(tile_handle).unwrap();
            let image_offset = tile_meta.get_image_offset(idx);
            let layer_offset = image_offset * voxel_size * voxel_size * texture_format_size;
            let mut data: Vec<u8> = vec![255; (texture_format_size * width * height) as usize];
            for (y, row) in map.iter().enumerate() {
                for (x, tile) in row.iter().enumerate() {
                    let x_part = x * voxel_data_size;
                    let y_part = y * width * voxel_data_size;
                    let start = x_part + y_part;
                    for vy in 0usize..voxel_size {
                        for vx in 0..voxel_size {
                            let rpos = vx * texture_format_size as usize + vy * voxel_data_size;
                            let wpos = start
                                + vx * texture_format_size as usize
                                + vy * voxel_data_size * map_width as usize;

                            if *tile == 1 {
                                let rpos = rpos + layer_offset;
                                let (tr, tg, tb, ta) = (
                                    tile_image.data[rpos],
                                    tile_image.data[rpos + 1],
                                    tile_image.data[rpos + 2],
                                    tile_image.data[rpos + 3],
                                );
                                let dir = match (tr, tg, tb, ta) {
                                    (0, 0, 0, 0) => 3,
                                    (255, 0, 0, 255) => 2,
                                    (0, 255, 0, 255) => 1,
                                    (0, 0, 255, 255) => 0,
                                    _ => unreachable!(),
                                };
                                let [r, g, b, a] = palette.get_sun_color(dir, idx, l).as_rgba_u8();
                                data[wpos] = r;
                                data[wpos + 1] = g;
                                data[wpos + 2] = b;
                                data[wpos + 3] = a;
                            } else if *tile == 0 {
                                data[wpos + 3] = 0;
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
                transform: Transform::from_translation(pos + offset.extend(-1.) * idx as f32),
                ..default()
            });

            map_images.0.push(handle);
        }
    }
}
