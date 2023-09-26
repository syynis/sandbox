use bevy::{
    core_pipeline::clear_color::ClearColorConfig, prelude::*, render::camera::Viewport,
    sprite::Mesh2dHandle,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_prototype_debug_lines::DebugLinesPlugin;
use bevy_prototype_lyon::prelude::ShapePlugin;
use sandbox::{
    entity::player::Player,
    input::InputPlugin,
    phys::{
        movement::{Control, MovementPlugin},
        PhysPlugin,
    },
};

use bevy_xpbd_2d::prelude::*;
use bevy_xpbd_2d::{math::*, parry::bounding_volume::Aabb};

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins,
        PanCamPlugin::default(),
        DebugLinesPlugin::default(),
        WorldInspectorPlugin::new(),
        PhysPlugin,
        ShapePlugin,
        InputPlugin,
        MovementPlugin,
    ));

    app.insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Gravity(Vector::NEG_Y * 320.0));

    app.add_systems(Startup, setup);
    app.add_systems(Update, (move_portal_camera, sync_portals).chain());
    app.add_systems(Update, (change_focus, player_in_portal));
    app.add_systems(PostUpdate, sync_velocities);

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
        custom_size: Some(Vec2::splat(16.0)),
        ..default()
    };

    cmds.spawn((
        SpriteBundle {
            sprite: square_sprite.clone(),
            transform: Transform::from_scale(Vec3::new(32., 1., 1.)),
            ..default()
        },
        RigidBody::Static,
        Collider::cuboid(32. * 16., 1. * 16.),
        CollisionLayers::new([Layer::Normal], [Layer::Normal]),
    ));

    cmds.spawn((
        SpriteBundle {
            sprite: square_sprite.clone(),
            transform: Transform::from_xyz(16., 48., 1.).with_scale(Vec3::new(16., 4., 1.)),
            ..default()
        },
        RigidBody::Static,
        Collider::cuboid(16. * 16., 4. * 16.),
        CollisionLayers::new([Layer::Normal], [Layer::Normal]),
    ));

    cmds.spawn((
        SpriteBundle {
            sprite: square_sprite.clone(),
            transform: Transform::from_xyz(512., 32., 1.).with_scale(Vec3::new(16., 16., 1.)),
            ..default()
        },
        RigidBody::Static,
        Collider::cuboid(16. * 16., 16. * 16.),
        CollisionLayers::new([Layer::Normal], [Layer::Normal]),
    ));

    let portal_size = Vec2::new(128., 128.);
    let portal_mesh: Mesh2dHandle = meshes
        .add(shape::Box::new(portal_size.x, portal_size.y, 1.0).into())
        .into();
    let portal_material = materials.add(ColorMaterial::from(Color::rgba(0.8, 0.1, 0.1, 0.05)));

    let mut aabb = Aabb::new_invalid();
    aabb.mins.x = 128.;
    aabb.mins.y = 128.;
    aabb.maxs.x = 256.;
    aabb.maxs.y = 256.;
    let portal1 = cmds
        .spawn((
            portal_mesh.clone(),
            portal_material.clone(),
            VisibilityBundle::default(),
            Camera2dBundle {
                transform: Transform::from_xyz(-48., 160., 1.),
                camera: Camera {
                    order: 1,
                    viewport: Some(Viewport {
                        physical_size: portal_size.as_uvec2().clone(),
                        ..default()
                    }),
                    ..default()
                },
                camera_2d: Camera2d {
                    clear_color: ClearColorConfig::None,
                },
                ..default()
            },
            RigidBody::Kinematic,
            Position(Vector::new(-48., 160.)),
            Collider::cuboid(portal_size.x, portal_size.y),
            CollisionLayers::new([Layer::Portal], [Layer::Portal]),
            ColliderAabb(aabb),
            Focus,
        ))
        .id();

    let portal2 = cmds
        .spawn((
            portal_mesh.clone(),
            portal_material.clone(),
            VisibilityBundle::default(),
            Camera2dBundle {
                transform: Transform::from_xyz(80., 160., 1.),
                camera: Camera {
                    order: 2,
                    viewport: Some(Viewport {
                        physical_size: portal_size.as_uvec2().clone(),
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
            RigidBody::Kinematic,
            Position(Vector::new(80., 160.)),
            Collider::cuboid(portal_size.x, portal_size.y),
            CollisionLayers::new([Layer::Portal], [Layer::Portal]),
            ColliderAabb::from_shape(Collider::cuboid(portal_size.x, portal_size.y).get_shape()),
        ))
        .id();
    cmds.entity(portal1).insert(Portal {
        dimension: portal_size,
        other: portal2,
    });
}

#[derive(PhysicsLayer, Default)]
enum Layer {
    #[default]
    Normal,
    Portal,
}

fn approx_equal(a: f32, b: f32, dp: u8) -> bool {
    let p = 10f32.powi(-(dp as i32));
    (a - b).abs() < p
}

fn sync_velocities(
    mut q_player: Query<&mut LinearVelocity, (With<Player>, Without<PlayerLink>)>,
    mut q_link: Query<&mut LinearVelocity, (With<PlayerLink>, Without<Player>)>,
) {
    if let Some(mut vel) = q_player.get_single_mut().ok() {
        if let Some(mut link_vel) = q_link.get_single_mut().ok() {
            let vel_cp = vel.clone();
            let link_vel_cp = link_vel.clone();

            let mut new_vel = vel_cp.min(link_vel_cp.0);

            if approx_equal(0.0, link_vel_cp.x, 10) {
                new_vel.x = 0.;
            }

            if approx_equal(0.0, link_vel_cp.y, 10) {
                new_vel.y = 0.;
            }

            vel.0 = new_vel;
            link_vel.0 = new_vel;
        }
    }
}

#[derive(Component)]
pub struct PlayerLink;

fn player_in_portal(
    mut cmds: Commands,
    q_player: Query<(&Position, &LinearVelocity, &ColliderAabb), With<Player>>,
    q_player_link: Query<Entity, With<PlayerLink>>,
    q_portal: Query<(&Portal, &Position, &ColliderAabb)>,
    mut spawned: Local<bool>,
) {
    if let Some((pos, linvel, player_aabb)) = q_player.get_single().ok() {
        let res = q_portal
            .iter()
            .find(|(_, _, portal_aabb)| player_aabb.intersection(portal_aabb).is_some());

        if let Some((portal, portal_pos, _)) = res {
            let (_, other_portal_pos, _) = q_portal.get(portal.other).ok().unwrap();
            if q_player_link.get_single().ok().is_none() {
                let diff = **portal_pos - **pos;
                if !(*spawned) {
                    cmds.spawn((
                        PlayerLink,
                        LinearVelocity::from(linvel.clone()),
                        RigidBody::Dynamic,
                        Collider::cuboid(16.0, 16.0),
                        Position(**other_portal_pos - diff),
                        LockedAxes::new().lock_rotation(),
                        Friction::new(0.),
                        Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
                        Control::default(),
                        CollisionLayers::new([Layer::Normal], [Layer::Normal]),
                    ));
                    *spawned = true;
                }
            }
        } else {
            if let Some(player_link) = q_player_link.get_single().ok() {
                println!("despawn");
                cmds.entity(player_link).despawn_recursive();
                *spawned = false;
            }
        }
    }
}

fn move_portal_camera(
    keys: Res<Input<KeyCode>>,
    mut camera_query: Query<&mut Position, (With<Portal>, With<Focus>)>,
) {
    let mut pos = camera_query.single_mut();
    if keys.just_pressed(KeyCode::Left) {
        **pos -= Vec2::X * 16.;
    }
    if keys.just_pressed(KeyCode::Right) {
        **pos += Vec2::X * 16.;
    }
    if keys.just_pressed(KeyCode::Up) {
        **pos += Vec2::Y * 16.;
    }
    if keys.just_pressed(KeyCode::Down) {
        **pos -= Vec2::Y * 16.;
    }
}

fn change_focus(
    mut cmds: Commands,
    keys: Res<Input<KeyCode>>,
    portal_focus_query: Query<(Entity, &Portal), With<Focus>>,
    portal_query: Query<Entity, With<Portal>>,
) {
    if keys.just_pressed(KeyCode::F) {
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
            [(portal_transform, mut cam, portal), (other_portal_transform, mut other_cam, other_portal)],
        ) = transform_q.get_many_mut([entity, other]).ok()
        {
            let new_viewport_pos = main_cam
                .world_to_viewport(main_cam_transform, portal_transform.translation)
                .unwrap()
                .as_uvec2();
            let new_other_viewport_pos = main_cam
                .world_to_viewport(main_cam_transform, other_portal_transform.translation)
                .unwrap()
                .as_uvec2();

            let viewport = cam.viewport.as_mut().unwrap();
            let other_viewport = other_cam.viewport.as_mut().unwrap();

            viewport.physical_position =
                new_other_viewport_pos - other_portal.dimension.as_uvec2() / 2;
            other_viewport.physical_position = new_viewport_pos - portal.dimension.as_uvec2() / 2;
        }
    }
}

#[derive(Component)]
pub struct Portal {
    dimension: Vec2,
    other: Entity,
}

#[derive(Component)]
pub struct Focus;

#[derive(Component)]
pub struct Portable;
