use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_flycam::{MovementSettings, PlayerPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use lazy_static::lazy_static;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        //.add_plugin(PlayerPlugin)
        .add_plugin(WorldInspectorPlugin);

    app.insert_resource(MovementSettings {
        sensitivity: 0.00005,
        speed: 6.0,
    });
    app.insert_resource(SolutionStep(0));
    app.add_startup_system(spawn_iso_cam);
    //app.add_startup_system(setup);
    app.add_system(place);
    app.add_system(clear);

    app.run();
}

#[derive(Resource, Deref, DerefMut)]
struct SolutionStep(pub usize);

lazy_static! {
    pub static ref SOMA_TABLE: Vec<(Vec<Vec3>, Color)> =
        vec![
        (vec![Vec3::new(0., 0., 0.), Vec3::new(0., 0., 1.), Vec3::new(1., 0., 1.)], Color::RED), // V
        (vec![Vec3::new(0., 0., 0.), Vec3::new(0., 0., 1.), Vec3::new(1., 0., 1.), Vec3::new(2., 0., 1.)], Color::BLUE), // L
        (vec![Vec3::new(0., 0., 1.), Vec3::new(1., 0., 0.), Vec3::new(1., 0., 1.), Vec3::new(2., 0., 1.)], Color::BLACK), // T
        (vec![Vec3::new(0., 0., 0.), Vec3::new(1., 0., 0.), Vec3::new(1., 0., 1.), Vec3::new(2., 0., 1.)], Color::YELLOW), // Z
        (vec![Vec3::new(0., 0., 0.), Vec3::new(1., 0., 0.), Vec3::new(1., 0., 1.), Vec3::new(1., 1., 1.)], Color::PURPLE), // A
        (vec![Vec3::new(0., 0., 1.), Vec3::new(1., 0., 1.), Vec3::new(1., 0., 0.), Vec3::new(1., 1., 0.)], Color::GREEN), // B
        (vec![Vec3::new(0., 0., 1.), Vec3::new(1., 0., 1.), Vec3::new(1., 0., 0.), Vec3::new(1., 1., 1.)], Color::WHITE), // P
    ];

    pub static ref SOMA_TABLE_SPECIFIC: Vec<(Vec<Vec3>, Color)> =
        vec![
        (vec![Vec3::new(1., 0., 1.), Vec3::new(2., 0., 0.), Vec3::new(2., 0., 1.), Vec3::new(2., 0., 2.)], Color::BEIGE), // T
        (vec![Vec3::new(2., 1., 1.), Vec3::new(2., 1., 2.), Vec3::new(2., 2., 2.)], Color::GRAY), // V
        (vec![Vec3::new(1., 0., 2.), Vec3::new(1., 1., 2.), Vec3::new(1., 1., 1.), Vec3::new(1., 2., 1.)], Color::RED), // Z
        (vec![Vec3::new(0., 0., 0.), Vec3::new(0., 0., 1.), Vec3::new(0., 0., 2.), Vec3::new(1., 0., 0.)], Color::ORANGE), // L
        (vec![Vec3::new(1., 1., 0.), Vec3::new(2., 1., 0.), Vec3::new(2., 2., 0.), Vec3::new(2., 2., 1.)], Color::YELLOW), // A
        (vec![Vec3::new(0., 1., 1.), Vec3::new(0., 1., 2.), Vec3::new(0., 2., 2.), Vec3::new(1., 2., 2.)], Color::PINK), // B
        (vec![Vec3::new(0., 1., 0.), Vec3::new(0., 2., 0.), Vec3::new(0., 2., 1.), Vec3::new(1., 2., 0.)], Color::GREEN), // P
    ];
}

fn spawn_iso_cam(mut cmds: Commands) {
    // camera
    cmds.spawn(Camera3dBundle {
        projection: OrthographicProjection {
            scale: 5.0,
            scaling_mode: ScalingMode::FixedVertical(2.0),
            ..default()
        }
        .into(),
        transform: Transform::from_xyz(-5.0, 7.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    // light
    cmds.spawn(PointLightBundle {
        transform: Transform::from_xyz(-5.0, 10.0, -5.0),
        ..default()
    });
}

fn clear(
    mut cmds: Commands,
    query: Query<Entity, With<Anchor>>,
    key: Res<Input<KeyCode>>,
    mut solution_step: ResMut<SolutionStep>,
) {
    if key.just_pressed(KeyCode::C) {
        for entity in query.iter() {
            cmds.entity(entity).despawn_recursive();
        }
        **solution_step = 0;
    }
}
fn place(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    key: Res<Input<KeyCode>>,
    mut solution_step: ResMut<SolutionStep>,
) {
    if **solution_step > 6 {
        return;
    }

    if key.just_pressed(KeyCode::F) {
        cmds.spawn((Anchor, SpatialBundle::default()))
            .with_children(|parent| {
                let (cubes, color) = &SOMA_TABLE_SPECIFIC[**solution_step];
                let material = materials.add((*color).into());
                for pos in cubes {
                    let pos = *pos;
                    parent.spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                        material: material.clone(),
                        transform: Transform {
                            translation: pos,
                            ..default()
                        },
                        ..default()
                    });
                }
            });

        **solution_step += 1;
    }
}

#[derive(Component)]
struct Anchor;

fn setup(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let x_offset = Vec3::Y * 3.0;
    for i in 0..*&SOMA_TABLE.len() {
        let (cubes, color) = &SOMA_TABLE[i];

        let material = materials.add((*color).into());
        cmds.spawn((
            Anchor,
            SpatialBundle {
                transform: Transform {
                    translation: Vec3::ZERO + i as f32 * x_offset,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            for pos in cubes {
                let pos = *pos;
                parent.spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                    material: material.clone(),
                    transform: Transform {
                        translation: pos,
                        ..default()
                    },
                    ..default()
                });
            }
        });
    }
}
