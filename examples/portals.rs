use bevy::{
    core_pipeline::clear_color::ClearColorConfig, prelude::*, render::camera::Viewport,
    sprite::Mesh2dHandle,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_prototype_debug_lines::DebugLinesPlugin;
use bevy_prototype_lyon::prelude::ShapePlugin;
use sandbox::{
    entity::player::{DespawnPlayerCommand, Player, SpawnPlayerCommand},
    input::{CursorPos, InputPlugin},
    phys::{movement::Control, terrain::handle_platforms, PhysPlugin},
};

use bevy_xpbd_2d::{math::*, parry::bounding_volume::Aabb};
use bevy_xpbd_2d::{prelude::*, PostProcessCollisions};

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins,
        PanCamPlugin::default(),
        DebugLinesPlugin::default(),
        WorldInspectorPlugin::new(),
        PhysPlugin,
        ShapePlugin,
        InputPlugin::<PanCam>::default(),
    ));

    app.insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Gravity(Vector::NEG_Y * 320.0));

    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (
            (move_portal_camera, sync_portals).chain(),
            change_focus,
            player_in_portal,
            respawn_player,
            sync_link_pos,
            disable_gravity,
        ),
    );
    app.add_systems(
        PostProcessCollisions,
        sync_link_collisions.after(handle_platforms),
    );

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
            transform: Transform::from_xyz(16., 48., 1.).with_scale(Vec3::new(16., 4., 1.)),
            ..default()
        },
        RigidBody::Static,
        Collider::cuboid(16. * 16., 4. * 16.),
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
                transform: Transform::from_xyz(-176., 80., 1.),
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
            Position(Vector::new(-176., 80.)),
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
                transform: Transform::from_xyz(80., 80., 1.),
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
            Position(Vector::new(80., 80.)),
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

fn respawn_player(mut cmds: Commands, keys: Res<Input<KeyCode>>, cursor_pos: Res<CursorPos>) {
    let pos = **cursor_pos;
    if keys.just_pressed(KeyCode::F) {
        cmds.add(DespawnPlayerCommand);
        let size = Vector::new(14., 14.);
        cmds.add(SpawnPlayerCommand::new(
            pos,
            size,
            CollisionLayers::new([Layer::Normal], [Layer::Normal]),
        ));
    }
}

fn disable_gravity(
    keys: Res<Input<KeyCode>>,
    mut gravity: ResMut<Gravity>,
    mut disabled: Local<bool>,
) {
    if keys.just_pressed(KeyCode::G) {
        if *disabled {
            gravity.0 = Vector::NEG_Y * 320.;
        } else {
            gravity.0 = Vector::ZERO;
        }
        *disabled = !(*disabled);
    }
}

fn sync_link_collisions(
    player: Query<Entity, With<Player>>,
    player_link: Query<Entity, With<PlayerLink>>,
    mut collisions: ResMut<Collisions>,
) {
    let Ok(player) = player.get_single() else {
        return;
    };
    let Ok(link) = player_link.get_single() else {
        return;
    };

    for contacts in collisions.collisions_with_entity_mut(link) {
        let Contacts {
            entity1,
            entity2,
            manifolds,
            ..
        } = contacts;

        println!(
            "before: entiy1: {}, entity2: {}",
            entity1.index(),
            entity2.index()
        );
        if *entity1 == link {
            contacts.entity1 = player;
        } else if *entity2 == link {
            contacts.entity2 = player;
        }
        println!(
            "after: entiy1: {}, entity2: {}",
            contacts.entity1.index(),
            contacts.entity2.index()
        );

        for manifold in manifolds.iter_mut() {
            if manifold.entity1 == link {
                manifold.entity1 = player;
            } else {
                manifold.entity2 = player;
            }
        }
    }

    //collisions.retain(|(e1, e2), _| *e1 != link && *e2 != link);
}

fn sync_link_pos(
    player: Query<(&Position, &ColliderAabb), (With<Player>, Without<PlayerLink>, Without<Portal>)>,
    mut player_link: Query<&mut Position, (With<PlayerLink>, Without<Player>, Without<Portal>)>,
    q_portal: Query<(&Portal, &Position, &ColliderAabb)>,
) {
    let Ok((player_pos, player_aabb)) = player.get_single() else {
        return;
    };
    let Ok(mut link) = player_link.get_single_mut() else {
        return;
    };

    let Some((portal, portal_pos, _)) = q_portal
        .iter()
        .find(|(_, _, portal_aabb)| player_aabb.intersection(portal_aabb).is_some())
    else {
        return;
    };

    let (_, other_portal_pos, _) = q_portal.get(portal.other).ok().unwrap();

    let diff = **portal_pos - **player_pos;
    link.0 = **other_portal_pos - diff;
}

#[derive(PhysicsLayer, Default)]
enum Layer {
    #[default]
    Normal,
    Portal,
}

#[derive(Component)]
pub struct PlayerLink;

fn player_in_portal(
    mut cmds: Commands,
    q_player: Query<(&Position, &ColliderAabb), With<Player>>,
    q_player_link: Query<Entity, With<PlayerLink>>,
    q_portal: Query<(&Portal, &Position, &ColliderAabb)>,
    mut spawned: Local<bool>,
) {
    let Ok((pos, player_aabb)) = q_player.get_single() else {
        return;
    };
    let Some((portal, portal_pos, _)) = q_portal
        .iter()
        .find(|(_, _, portal_aabb)| player_aabb.intersection(portal_aabb).is_some())
    else {
        if let Some(player_link) = q_player_link.get_single().ok() {
            println!("despawn");
            cmds.entity(player_link).despawn_recursive();
            *spawned = false;
        }
        return;
    };

    let (_, other_portal_pos, _) = q_portal.get(portal.other).ok().unwrap();
    if q_player_link.get_single().is_err() {
        let diff = **portal_pos - **pos;
        if !(*spawned) {
            cmds.spawn((
                PlayerLink,
                RigidBody::Dynamic,
                Collider::cuboid(16.0, 16.0),
                Position(**other_portal_pos - diff),
                LockedAxes::new().lock_rotation(),
                Friction::new(0.),
                Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
                Control::default(),
                CollisionLayers::new([Layer::Normal], [Layer::Normal]),
                GravityScale(0.0),
            ));
            *spawned = true;
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
    if keys.just_pressed(KeyCode::T) {
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
