use bevy::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CursorPos::default());
        app.add_systems(Update, update_cursor_pos);
        app.register_type::<CursorPos>();
    }
}

#[derive(Default, Resource, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
pub struct CursorPos(pub Vec2);

pub fn update_cursor_pos(
    camera_query: Query<(&Camera, &GlobalTransform)>,
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
