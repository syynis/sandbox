use bevy::log::*;
use bevy::prelude::*;

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::PanCam;
use bevy_pancam::PanCamPlugin;
use bevy_prototype_debug_lines::DebugLines;
use bevy_prototype_debug_lines::DebugLinesPlugin;
use sandbox::input::update_cursor_pos;
use sandbox::input::CursorPos;
use sandbox::phys;
use sandbox::phys::movement::Force;
use sandbox::phys::movement::LinearVelocity;
use sandbox::phys::movement::MovementPlugin;
use sandbox::phys::verlet::*;
use sandbox::phys::Gravity;
use sandbox::phys::PhysPlugin;
use sandbox::phys::PhysSettings;

const PPM: f32 = 32.0;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugin(PanCamPlugin::default())
        .add_plugin(PhysPlugin)
        .add_plugin(DebugLinesPlugin::default())
        .add_plugin(WorldInspectorPlugin);
    app.insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Ground::default());

    app.insert_resource(CursorPos::default())
        .add_system(update_cursor_pos);
    app.add_startup_system(setup);
    app.add_system(draw_ground)
        .add_system(movement)
        .add_system(player_ground_collision);

    app.run();
}

#[derive(Resource, Reflect, Deref, DerefMut, Clone)]
struct Ground(pub Vec<(f32, f32)>);

impl Ground {
    fn height_at(&self, x: f32) -> f32 {
        if x < 0. {
            return self.0.first().unwrap_or(&(0., 0.)).1;
        }
        if let Some([(sx, sy), (ex, ey)]) = self.0.windows(2).find(|c| match c {
            [(sx, sy), (ex, ey)] => (sx..ex).contains(&&x),
            _ => false,
        }) {
            let t = (x - sx) / (ex - sx);
            return sy * (1.0 - t) + ey * t;
        } else {
            return self.0.last().unwrap_or(&(0., 0.0)).1;
        };
    }
}

#[derive(Component)]
struct Player;

impl Default for Ground {
    fn default() -> Self {
        Self(
            Vec::from([
                (0., 0.),
                (15., 0.),
                (35., 5.),
                (50., 5.),
                (55., 6.),
                (70., 6.5),
            ])
            .iter()
            .map(|(x, y)| (x * PPM, y * PPM))
            .collect::<Vec<(f32, f32)>>(),
        )
    }
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

    cmds.spawn((
        LinearVelocity::default(),
        Force::default(),
        Player,
        SpriteBundle {
            sprite: Sprite {
                color: Color::RED,
                custom_size: Some(Vec2::splat(32.0)),
                anchor: bevy::sprite::Anchor::BottomLeft,
                ..default()
            },
            transform: Transform::from_xyz(32.0, 32.0, 0.0),
            ..default()
        },
    ));
}

fn movement(input: Res<Input<KeyCode>>, mut query: Query<&mut LinearVelocity, With<Player>>) {
    if let Some(mut vel) = query.get_single_mut().ok() {
        let mut any_pressed = false;
        if input.pressed(KeyCode::A) {
            any_pressed = true;
            **vel -= Vec2::new(4., 0.);
        }
        if input.pressed(KeyCode::D) {
            any_pressed = true;
            **vel += Vec2::new(4., 0.);
        }
        if !any_pressed {
            **vel = Vec2::new(0., vel.y);
        }
    }
}

fn player_ground_collision(ground: Res<Ground>, mut query: Query<&mut Transform, With<Player>>) {
    if let Some(mut transform) = query.get_single_mut().ok() {
        transform.translation.y = ground.height_at(transform.translation.x + 16.);
    }
}

fn draw_ground(ground: Res<Ground>, mut lines: ResMut<DebugLines>) {
    let ground = ground.0.clone();
    for i in 0..ground.len() - 1 {
        let start = Vec2::new(ground[i].0, ground[i].1).extend(0.0);
        let end = Vec2::new(ground[i + 1].0, ground[i + 1].1).extend(0.0);
        lines.line(start, end, 0.0);
    }
}
