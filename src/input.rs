use bevy::prelude::*;

#[derive(Default, Resource, Deref, DerefMut)]
pub struct CursorPos(pub Vec2);

pub fn update_cursor_pos(
    windows: Res<Windows>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_pos: ResMut<CursorPos>,
) {
    let (camera, transform) = match camera_query.get_single() {
        Ok((c, t)) => (c, t),
        Err(e) => return,
    };

    for moved_event in cursor_moved_events.iter() {
        let new = camera
            .viewport_to_world(&transform, moved_event.position)
            .unwrap()
            .origin
            .truncate();
        *cursor_pos = CursorPos(new);
    }
}
