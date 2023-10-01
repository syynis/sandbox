use std::marker::PhantomData;

use bevy::prelude::*;

#[derive(Default)]
pub struct InputPlugin<T: Component> {
    phantom: PhantomData<T>,
}

impl<T: Component> Plugin for InputPlugin<T> {
    fn build(&self, app: &mut App) {
        app.insert_resource(CursorPos::default());
        app.add_systems(Startup, setup_cursor);
        app.add_systems(Update, (update_cursor_pos::<T>, move_cursor));
        app.register_type::<CursorPos>();
    }
}

#[derive(Default, Resource, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
pub struct CursorPos(pub Vec2);

pub fn update_cursor_pos<T: Component>(
    camera_query: Query<(&Camera, &GlobalTransform), With<T>>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_pos: ResMut<CursorPos>,
) {
    let Ok((camera, transform)) = camera_query.get_single() else {
        return;
    };

    for moved_event in cursor_moved_events.iter() {
        let Some(new) = camera
            .viewport_to_world(&transform, moved_event.position)
            .map(|ray| ray.origin.truncate())
        else {
            return;
        };
        cursor_pos.0 = new;
    }
}

#[derive(Component)]
struct CustomCursor;

fn setup_cursor(
    mut windows: Query<&mut Window>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let mut window: Mut<Window> = windows.single_mut();
    window.cursor.visible = true;
    let cursor_spawn: Vec3 = Vec3::ZERO;

    commands.spawn((
        ImageBundle {
            image: asset_server.load("cursor.png").into(),
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Auto,
                right: Val::Auto,
                bottom: Val::Auto,
                top: Val::Auto,
                ..default()
            },
            z_index: ZIndex::Global(15),
            transform: Transform::from_translation(cursor_spawn),
            ..default()
        },
        CustomCursor,
    ));
}

fn move_cursor(window: Query<&Window>, mut cursor: Query<&mut Style, With<CustomCursor>>) {
    let window: &Window = window.single();
    if let Some(position) = window.cursor_position() {
        let mut img_style = cursor.single_mut();
        img_style.left = Val::Px(position.x - 8.);
        img_style.top = Val::Px(position.y - 8.);
    }
}
