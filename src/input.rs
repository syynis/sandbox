use std::marker::PhantomData;

use bevy::prelude::*;

#[derive(Default)]
pub struct InputPlugin<T: Component> {
    phantom: PhantomData<T>,
}

impl<T: Component> Plugin for InputPlugin<T> {
    fn build(&self, app: &mut App) {
        app.insert_resource(CursorPos::default());
        app.add_systems(Update, update_cursor_pos::<T>);
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
