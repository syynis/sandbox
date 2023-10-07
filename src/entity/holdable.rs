use bevy::prelude::*;
use bevy_xpbd_2d::{math::Vector, prelude::*};

use crate::phys::movement::LookDir;

use super::pebble::SpawnPebble;

#[derive(Component)]
pub struct Holdable;

#[derive(Component)]
pub struct IsHeld;

#[derive(Component)]
pub struct CanHold;

pub fn pick_up(
    mut cmds: Commands,
    keys: Res<Input<KeyCode>>,
    holdables: Query<Entity, With<Holdable>>,
    held: Query<&IsHeld>,
    holder: Query<(Entity, Option<&Children>, &CollidingEntities), With<CanHold>>,
) {
    let Ok((holder, children, colliding)) = holder.get_single() else {
        return;
    };

    if let Some(children) = children {
        if children.iter().any(|child| held.get(*child).is_ok()) {
            return;
        }
    }

    if keys.just_pressed(KeyCode::H) {
        // Find first colliding enitty that is also a holdable
        if let Some(holdable) = colliding.0.iter().find(|e| holdables.get(**e).is_ok()) {
            // Despawn entity to get rid of all the physics related components
            // TODO when bevy_xpbd supports child colliders think about simply moving the entity to the children list
            // This would hopefully also naturally add velocity inheritance on throwing
            cmds.entity(*holdable).despawn();
            cmds.entity(holder).with_children(|childbuilder| {
                childbuilder.spawn((
                    IsHeld,
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgb(0.7, 0.7, 0.8),
                            custom_size: Some(Vec2::splat(8.0)),
                            ..default()
                        },
                        ..default()
                    },
                ));
            });
        }
    }
}

pub fn throw(
    mut cmds: Commands,
    keys: Res<Input<KeyCode>>,
    holder: Query<(Entity, &Children, &Transform, &LookDir), With<CanHold>>,
    held: Query<(With<IsHeld>, With<Parent>)>,
) {
    let Ok((holder, children, transform, look_dir)) = holder.get_single() else {
        return;
    };

    // Can't throw anything
    let Some(throwable) = children.iter().find(|child| held.get(**child).is_ok()) else {
        return;
    };

    if keys.just_pressed(KeyCode::X) {
        // Remove child throwable entity
        cmds.entity(holder).remove_children(&[*throwable]);
        cmds.entity(*throwable).despawn();

        // Spawn new physics entity
        let pos = transform.translation.truncate() + look_dir.as_vec() * 16.; // Dont hardcode player size
        let vel = look_dir.as_vec() * 512. + Vector::Y * 128.;
        cmds.add(SpawnPebble {
            pos,
            vel,
            lifetime: None,
        });
    }
}
