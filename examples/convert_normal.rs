use bevy::{
    asset::LoadState,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::{PanCam, PanCamPlugin};
use sandbox::editor::palette::{load_palette_images, parse_palette_images, Palettes};

pub fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(ImagePlugin::default_nearest()),
        WorldInspectorPlugin::default(),
        PanCamPlugin::default(),
    ));

    app.add_systems(Startup, (setup, load_palette_images));
    app.add_systems(Update, (convert, parse_palette_images));
    app.add_systems(Update, display.run_if(resource_exists::<Convert>()));

    app.run();
}

#[derive(Resource)]
pub struct Normal(pub Handle<Image>);

#[derive(Resource)]
pub struct Convert(pub Handle<Image>);

#[derive(Resource)]
pub struct ConvertPalette(pub Handle<Image>);

fn setup(mut cmds: Commands, asset_server: Res<AssetServer>) {
    cmds.spawn((
        Camera2dBundle::default(),
        PanCam {
            grab_buttons: vec![MouseButton::Middle],
            enabled: true,
            ..default()
        },
    ));

    let normal = asset_server.load("rocky_normal.png");
    cmds.insert_resource(Normal(normal));
}

fn convert(
    mut cmds: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    assets: Res<Normal>,
    palettes: Option<Res<Palettes>>,
    mut once: Local<bool>,
) {
    if !matches!(asset_server.get_load_state(&assets.0), LoadState::Loaded) {
        return;
    }
    let Some(palettes) = palettes else {
        return;
    };

    let palette = palettes.get(1);
    if *once {
        return;
    }
    *once = true;

    let data = images
        .get(&assets.0)
        .unwrap()
        .data
        .chunks(4)
        .map(|pixel| {
            let (r, g, b, a) = (pixel[0], pixel[1], pixel[2], pixel[3]);
            let r_diff = 255 - r;
            let g_diff = 255 - g;
            let b_diff = b - 128;

            if r_diff < 100 {
                Color::rgba_u8(255, 0, 0, 255)
            } else if g_diff < 100 {
                Color::rgba_u8(0, 0, 255, 255)
            } else {
                Color::rgba_u8(0, 255, 0, 255)
            }
        })
        .flat_map(|color| color.as_rgba_u8())
        .collect::<Vec<u8>>();

    let data_palette = images
        .get(&assets.0)
        .unwrap()
        .data
        .chunks(4)
        .map(|pixel| {
            let (r, g, b, a) = (pixel[0], pixel[1], pixel[2], pixel[3]);
            let r_diff = 255 - r;
            let g_diff = 255 - g;
            let b_diff = b - 128;

            let color = if r_diff < 100 {
                0
            } else if g_diff < 100 {
                2
            } else {
                1
            };

            palette.get_shade_color(color, 0, 0)
        })
        .flat_map(|color| color.as_rgba_u8())
        .collect::<Vec<u8>>();

    let image_size = Extent3d {
        width: 200,
        height: 200,
        ..default()
    };
    let dimension = TextureDimension::D2;
    let image = Image::new(image_size, dimension, data, TextureFormat::Rgba8Unorm);
    let image_palette = Image::new(
        image_size,
        dimension,
        data_palette,
        TextureFormat::Rgba8Unorm,
    );
    let handle = images.add(image);
    let handle_palette = images.add(image_palette);
    cmds.insert_resource(Convert(handle));
    cmds.insert_resource(ConvertPalette(handle_palette))
}

fn display(
    mut cmds: Commands,
    normal: Res<Normal>,
    convert: Res<Convert>,
    convert_palette: Res<ConvertPalette>,
) {
    if convert.is_changed() {
        cmds.spawn(SpriteBundle {
            texture: convert_palette.0.clone(),
            transform: Transform::from_xyz(400., 0., 0.),
            ..default()
        });
        cmds.spawn(SpriteBundle {
            texture: convert.0.clone(),
            transform: Transform::from_xyz(200., 0., 0.),
            ..default()
        });
        cmds.spawn(SpriteBundle {
            texture: normal.0.clone(),
            ..default()
        });
    }
}
