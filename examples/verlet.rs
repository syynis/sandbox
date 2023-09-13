use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::*;

use bevy_prototype_debug_lines::DebugLines;
use bevy_prototype_debug_lines::DebugLinesPlugin;
use sandbox::input::CursorPos;
use sandbox::input::InputPlugin;
use sandbox::phys::verlet::*;
use sandbox::phys::Gravity;
use sandbox::phys::PhysPlugin;
use sandbox::phys::PhysSettings;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins,
        PanCamPlugin::default(),
        PhysPlugin,
        DebugLinesPlugin::default(),
        WorldInspectorPlugin::default(),
        InputPlugin,
    ));
    app.insert_resource(PlacementSettings {
        color: Color::WHITE,
        locked_color: Color::RED,
        lock: false,
    })
    .insert_resource(ClearColor(Color::BLACK));

    app.add_systems(Startup, setup);
    app.add_systems(Update, (input, draw_links, build));

    app.run();
}

fn setup(mut cmds: Commands) {
    cmds.spawn((
        Camera2dBundle::default(),
        PanCam {
            grab_buttons: vec![MouseButton::Middle],
            enabled: true,
            ..default()
        },
    ));
}

#[derive(Resource)]
pub struct PlacementSettings {
    color: Color,
    locked_color: Color,
    lock: bool,
}

fn input(
    mut cmds: Commands,
    mut placement: ResMut<PlacementSettings>,
    input: Res<Input<KeyCode>>,
    mut phys_settings: ResMut<PhysSettings>,
) {
    if input.just_pressed(KeyCode::G) {
        phys_settings.gravity = match phys_settings.gravity {
            Gravity::Dir(dir) => Gravity::None,
            Gravity::None => Gravity::Dir(Vec2::new(0.0, -9.81)),
        };
    }

    if input.just_pressed(KeyCode::T) {
        placement.lock = !placement.lock;
    }
}

fn build(
    mut cmds: Commands,
    points: Query<(Entity, &Transform), With<Point>>,
    links: Query<(Entity, &Link)>,
    mouse: Res<Input<MouseButton>>,
    cursor_pos: Res<CursorPos>,
    placement: Res<PlacementSettings>,
    mut egui_ctx: Query<&mut bevy_inspector_egui::bevy_egui::EguiContext>,
    mut selected: Local<Option<Entity>>,
) {
    if let Some(mut egui_ctx) = egui_ctx.get_single_mut().ok() {
        if hover_egui(&mut egui_ctx) {
            return;
        }
    }

    if mouse.just_pressed(MouseButton::Left) {
        let mut spawn = true;
        for (point, point_pos) in points.iter() {
            let point_pos = point_pos.translation.truncate();
            let dist = (**cursor_pos).distance(point_pos);
            if dist < 12.0 {
                spawn = false;

                if let Some(other) = &mut *selected {
                    let other = *other;
                    let other_pos = points.get(other).unwrap().1.translation.truncate();
                    if other != point {
                        cmds.spawn(Link::new(point, other, point_pos.distance(other_pos)));
                    }
                    *selected = None;
                } else {
                    *selected = Some(point);
                }
                break;
            }
        }
        if spawn {
            let color = if placement.lock {
                placement.locked_color
            } else {
                placement.color
            };
            let mut point = cmds.spawn((sprite_bundle(color, **cursor_pos), Point::default()));
            if placement.lock {
                point.insert(Locked);
            }
        }
    }
}

fn hover_egui(egui_ctx: &mut bevy_inspector_egui::bevy_egui::EguiContext) -> bool {
    egui_ctx.get_mut().wants_pointer_input() || egui_ctx.get_mut().wants_keyboard_input()
}

fn sprite_bundle(color: Color, pos: Vec2) -> SpriteBundle {
    SpriteBundle {
        sprite: Sprite {
            color,
            custom_size: Some(Vec2::splat(8.0)),
            ..default()
        },
        transform: Transform::from_xyz(pos.x, pos.y, 0.0),
        ..default()
    }
}

fn draw_links(
    mut lines: ResMut<DebugLines>,
    links_query: Query<&Link>,
    points_query: Query<&Transform, With<Point>>,
) {
    for link in links_query.iter() {
        if let Some((a, b)) = draw_link(link, &points_query) {
            lines.line(a, b, 0.0);
        }
    }
}

fn draw_link(link: &Link, points_query: &Query<&Transform, With<Point>>) -> Option<(Vec3, Vec3)> {
    let a = points_query.get(link.a).ok()?;
    let b = points_query.get(link.b).ok()?;
    Some((a.translation, b.translation))
}
