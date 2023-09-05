use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    render::camera::Viewport,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use bevy_prototype_lyon::{prelude::ShapePlugin, shapes};
use sandbox::input::InputPlugin;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins,
        PanCamPlugin::default(),
        DebugLinesPlugin::default(),
        WorldInspectorPlugin::new(),
        ShapePlugin,
        InputPlugin,
    ));

    app.insert_resource(ClearColor(Color::BLACK));

    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (move_portal_camera, sync_portals, debug_portal_camera).chain(),
    );
    app.add_systems(Update, change_focus);
    app.run();
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
            enabled: true,
            ..default()
        },
    ));

    let square_sprite = Sprite {
        color: Color::rgb(0.7, 0.7, 0.8),
        custom_size: Some(Vec2::splat(8.0)),
        ..default()
    };

    cmds.spawn((SpriteBundle {
        sprite: square_sprite.clone(),
        transform: Transform::from_scale(Vec3::new(32., 1., 1.)),
        ..default()
    },));

    cmds.spawn((SpriteBundle {
        sprite: square_sprite.clone(),
        transform: Transform::from_xyz(16., 56., 1.).with_scale(Vec3::new(24., 8., 1.)),
        ..default()
    },));

    cmds.spawn((SpriteBundle {
        sprite: square_sprite.clone(),
        transform: Transform::from_xyz(256., 24., 1.).with_scale(Vec3::new(16., 8., 1.)),
        ..default()
    },));

    let portal_size = UVec2::new(128, 128);

    let portal1 = cmds
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    order: 1,
                    viewport: Some(Viewport {
                        physical_size: portal_size.clone(),
                        ..default()
                    }),
                    ..default()
                },
                camera_2d: Camera2d {
                    clear_color: ClearColorConfig::None,
                },
                ..default()
            },
            Focus,
        ))
        .id();

    let portal2 = cmds
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    order: 2,
                    viewport: Some(Viewport {
                        physical_size: portal_size.clone(),
                        ..default()
                    }),
                    ..default()
                },
                camera_2d: Camera2d {
                    clear_color: ClearColorConfig::None,
                },
                ..default()
            },
            Portal {
                dimension: portal_size,
                other: portal1,
            },
        ))
        .id();
    cmds.entity(portal1).insert(Portal {
        dimension: portal_size,
        other: portal2,
    });
}

fn debug_portal_camera(
    camera_query: Query<(&Transform, Option<&Focus>), With<Portal>>,
    mut lines: ResMut<DebugLines>,
) {
    for (transform, focus) in camera_query.iter() {
        let color = if focus.is_some() {
            Color::BLUE
        } else {
            Color::RED
        };
        lines.line_colored(
            transform.translation - Vec3::new(64., 64., 0.),
            transform.translation + Vec3::new(64., 64., 0.),
            0.,
            color,
        );
    }
}

fn move_portal_camera(
    keys: Res<Input<KeyCode>>,
    mut camera_query: Query<&mut Transform, (With<Portal>, With<Focus>)>,
) {
    let mut transform = camera_query.single_mut();
    if keys.pressed(KeyCode::A) {
        transform.translation -= Vec3::X * 4.;
    }
    if keys.pressed(KeyCode::D) {
        transform.translation += Vec3::X * 4.;
    }
    if keys.pressed(KeyCode::W) {
        transform.translation += Vec3::Y * 4.;
    }
    if keys.pressed(KeyCode::S) {
        transform.translation -= Vec3::Y * 4.;
    }
}

fn change_focus(
    mut cmds: Commands,
    keys: Res<Input<KeyCode>>,
    portal_focus_query: Query<(Entity, &Portal), With<Focus>>,
    portal_query: Query<Entity, With<Portal>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        let (entity, link) = portal_focus_query.single();
        let other = link.other;
        cmds.entity(entity).remove::<Focus>();
        cmds.entity(portal_query.get(other).ok().unwrap())
            .insert(Focus);
    }
}

fn sync_portals(
    portal_query: Query<(Entity, &Portal), With<Focus>>,
    mut transform_q: Query<(&Transform, &mut Camera, &Portal)>,
    main_camera_q: Query<(&GlobalTransform, &Camera), Without<Portal>>,
) {
    let (main_cam_transform, main_cam) = main_camera_q.single();
    for (entity, portal) in portal_query.iter() {
        let other = portal.other;
        if let Some(
            [(transform, mut cam, portal), (other_transform, mut other_cam, other_portal)],
        ) = transform_q.get_many_mut([entity, other]).ok()
        {
            let new_viewport_pos = main_cam
                .world_to_viewport(main_cam_transform, transform.translation)
                .unwrap()
                .as_uvec2();
            let new_other_viewport_pos = main_cam
                .world_to_viewport(main_cam_transform, other_transform.translation)
                .unwrap()
                .as_uvec2();

            let viewport = cam.viewport.as_mut().unwrap();
            let other_viewport = other_cam.viewport.as_mut().unwrap();

            viewport.physical_position = new_other_viewport_pos - other_portal.dimension / 2;
            other_viewport.physical_position = new_viewport_pos - portal.dimension / 2;
        }
    }
}

#[derive(Component)]
pub struct Portal {
    dimension: UVec2,
    other: Entity,
}

#[derive(Component)]
pub struct Focus;

#[derive(Component)]
pub struct Portable;
