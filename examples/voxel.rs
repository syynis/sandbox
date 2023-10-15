use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::{PanCam, PanCamPlugin};
use rand::{seq::IteratorRandom, thread_rng};
use sandbox::editor::{
    palette::Palette,
    tiles::{Tiles, BASE_TILE_SIZE},
    AppState, EditorPlugin,
};

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

    (0..3).for_each(|_| {
        (0..(map_width * map_height / 10)).for_each(|_| {
            let row = (0..map_height).choose(&mut rng).unwrap();
            let elem = (0..map_width).choose(&mut rng).unwrap();
            map[row][elem] = 1;
        });
    });

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
    for l in 0..3 {
        for idx in 0..10 {
            let mut data: Vec<u8> = vec![0; (texture_format_size * width * height) as usize];
            for (y, row) in map.iter().enumerate() {
                for (x, tile_id) in row.iter().enumerate() {
                    let start = (x + y * width) * BASE_TILE_SIZE;
                    for vy in 0..BASE_TILE_SIZE {
                        for vx in 0..BASE_TILE_SIZE {
                            let rpos = vx + vy * BASE_TILE_SIZE;
                            let wpos = (start + vx + vy * BASE_TILE_SIZE * map_width as usize)
                                * texture_format_size;

                            let set_color = |d: &mut [u8], color: Color, idx: usize| {
                                let [r, g, b, a] = color.as_rgba_u8();
                                d[idx] = r;
                                d[idx + 1] = g;
                                d[idx + 2] = b;
                                d[idx + 3] = a;
                            };
                            if *tile_id == 1 {
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
                transform: Transform::from_translation(pos + offset.extend(-1.) * idx as f32),
                ..default()
            });

            map_images.0.push(handle);
        }
    }
}
