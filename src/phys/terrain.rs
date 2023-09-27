use bevy::{prelude::*, utils::hashbrown::HashSet};
use bevy_xpbd_2d::{math::*, prelude::*};

#[derive(Component)]
pub struct Terrain;

#[derive(Default, Component, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct Pole(pub PoleType);

#[derive(Default, Clone, Copy, Reflect)]
pub enum PoleType {
    #[default]
    Horizontal,
    Vertical,
    Combined,
}

#[derive(Component, Default)]
pub struct Platform(pub HashSet<Entity>);

#[derive(Component)]
pub struct PlatformPass;

pub fn handle_platforms(
    mut platforms: Query<&mut Platform>,
    passers: Query<Option<&PlatformPass>, (With<Collider>, Without<Platform>)>,
    mut collisions: ResMut<Collisions>,
) {
    collisions.retain(|(e1, e2), contacts| {
        fn any_penetrating(contacts: &Contacts) -> bool {
            contacts.manifolds.iter().any(|manifold| {
                manifold
                    .contacts
                    .iter()
                    .any(|contact| contact.penetration > 0.)
            })
        }

        enum RelevantNormal {
            Normal1,
            Normal2,
        }

        let (mut platform, other, normal) = if let Ok(platform) = platforms.get_mut(*e1) {
            (platform, e2, RelevantNormal::Normal1)
        } else if let Ok(platform) = platforms.get_mut(*e2) {
            (platform, e1, RelevantNormal::Normal2)
        } else {
            return true;
        };

        if platform.0.contains(other) {
            if any_penetrating(&contacts) {
                return false;
            } else {
                platform.0.remove(other);
            }
        }

        match passers.get(*other) {
            Ok(_) => {
                if contacts.manifolds.iter().all(|manifold| {
                    let normal = match normal {
                        RelevantNormal::Normal1 => manifold.normal1,
                        RelevantNormal::Normal2 => manifold.normal2,
                    };

                    normal.length() > Scalar::EPSILON && normal.dot(Vector::Y) >= 0.5
                }) {
                    true
                } else if any_penetrating(&contacts) {
                    platform.0.insert(*other);
                    false
                } else {
                    true
                }
            }
            _ => true,
        }
    });
}
