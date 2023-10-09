use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::{PanCam, PanCamPlugin};

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins.set(ImagePlugin::default_nearest()),
        PanCamPlugin::default(),
        WorldInspectorPlugin::default(),
    ));

    app.insert_resource(ClearColor(Color::DARK_GRAY));

    app.add_systems(Startup, setup);
    app.run()
}

fn setup(
    mut cmds: Commands,

    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    cmds.spawn((
        Camera2dBundle::default(),
        PanCam {
            grab_buttons: vec![MouseButton::Middle],
            ..default()
        },
    ));

    let sprite = |light| -> Sprite {
        Sprite {
            color: Color::hsl(0.5, 0.5, light),
            custom_size: Some(Vec2::splat(20.)),
            ..default()
        }
    };

    let mut spawn_voxel = |pos: Vec3, offset: Vec2, flip: bool| {
        cmds.spawn(SpriteBundle {
            sprite: sprite(0.2),
            transform: Transform::from_translation(pos),
            ..default()
        })
        .with_children(|builder| {
            for i in 1..10 {
                builder.spawn(SpriteBundle {
                    sprite: sprite(0.0 + i as f32 / 50.),
                    transform: Transform::from_translation(
                        (offset * ((1 - 2 * flip as i32) as f32)).extend(-1.) * i as f32,
                    ),
                    ..default()
                });
            }
        });
    };

    let map = [
        [
            [0, 0, 0, 0, 0, 0, 0, 0],
            [1, 1, 1, 1, 1, 1, 1, 0],
            [1, 0, 0, 0, 0, 0, 1, 1],
            [1, 0, 0, 0, 0, 0, 1, 1],
            [1, 1, 1, 0, 0, 0, 1, 0],
        ],
        [
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 1, 1, 0, 0, 0, 0, 0],
            [0, 1, 1, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
        ],
        [
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 1, 1, 0, 0, 0],
            [0, 0, 0, 1, 1, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
        ],
    ];
    for (l, layer) in map.iter().enumerate() {
        for (y, row) in layer.iter().rev().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                if *tile == 1 {
                    let l = l as f32;
                    let x = x as f32;
                    let y = y as f32;
                    let offset = Vec2::splat(0.4);
                    let pos = Vec3::new(20. * x, 20. * y, 0.) + ((offset * 9.).extend(-10.) * l);
                    spawn_voxel(pos, Vec2::splat(0.4), false);
                }
            }
        }
    }
}
