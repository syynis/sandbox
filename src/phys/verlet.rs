use bevy::prelude::*;

pub struct VerletPlugin;

#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone)]
struct PointLabel;
#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone)]
struct LinkLabel;

impl Plugin for VerletPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (update_points.in_set(PointLabel)),
                (update_links.in_set(LinkLabel).after(PointLabel)),
                (handle_links.after(LinkLabel)),
            ),
        );
        app.register_type::<Point>()
            .register_type::<Locked>()
            .register_type::<Link>()
            .register_type::<Tension>();
        bevy::log::info!("Loaded verlet plugin");
    }
}

pub enum Gravity {
    Dir(Vec2),
    None,
}

impl Gravity {
    fn acceleration(&self) -> Vec2 {
        match self {
            Gravity::Dir(dir) => *dir,
            Gravity::None => Vec2::ZERO,
        }
    }
}

#[derive(Resource)]
pub struct PhysSettings {
    pub gravity: Gravity,
}

impl Default for PhysSettings {
    fn default() -> Self {
        Self {
            gravity: Gravity::None,
        }
    }
}

#[derive(Default, Debug, Reflect, Component)]
pub struct Point {
    pub old: Option<Vec2>,
}

#[derive(Debug, Reflect, Component)]
pub struct Link {
    pub a: Entity,
    pub b: Entity,
    dist: f32,
}

#[derive(Debug, Reflect, Component)]
pub struct Tension(pub f32);

#[derive(Debug, Reflect, Component)]
pub struct Locked;

impl Link {
    pub fn new(a: Entity, b: Entity, dist: f32) -> Self {
        Self { a, b, dist }
    }
}

fn update_point(transform: &mut Transform, point: &mut Point, acc: Vec2, friction: f32) {
    let pos = transform.translation.truncate();
    let vel = point.old.map_or(Vec2::ZERO, |old| pos - old);
    transform.translation += (vel * friction + acc).extend(0.0);
    point.old = Some(pos);
}

pub fn update_points(
    mut points_query: Query<(&mut Transform, &mut Point), Without<Locked>>,
    time: Res<Time>,
    phys_settings: Res<PhysSettings>,
) {
    let gravity = phys_settings.gravity.acceleration() * time.delta_seconds();
    let friction = 0.998;
    for (mut transform, mut point) in points_query.iter_mut() {
        update_point(&mut transform, &mut point, gravity, friction);
    }
}

pub fn update_links(
    links_query: Query<&Link>,
    mut points_query: Query<(&mut Transform, Option<&Locked>), With<Point>>,
) {
    (0..10).for_each(|_| {
        for link in links_query.iter() {
            let [(mut a_transform, a_locked), (mut b_transform, b_locked)] =
                match points_query.get_many_mut([link.a, link.b]) {
                    Ok(l) => l,
                    Err(e) => {
                        bevy::log::error!("Entity for link does not exist: {}", e);
                        continue;
                    }
                };
            let (a_locked, b_locked) = (a_locked.is_some(), b_locked.is_some());
            if a_locked && b_locked {
                continue;
            }
            let (a_pos, b_pos) = (
                a_transform.translation.truncate(),
                b_transform.translation.truncate(),
            );
            let center = (a_pos + b_pos) / 2.0;
            let dir = (a_pos - b_pos).normalize_or_zero() * link.dist / 2.0;

            if !a_locked {
                let new = if b_locked {
                    b_transform.translation.truncate() + dir * 2.0
                } else {
                    center + dir
                };
                a_transform.translation = new.extend(a_transform.translation.z);
            }

            if !b_locked {
                let new = if a_locked {
                    a_transform.translation.truncate() - dir * 2.0
                } else {
                    center - dir
                };
                b_transform.translation = new.extend(b_transform.translation.z);
            }
        }
    });
}

fn handle_tension(
    entity: Entity,
    link: &Link,
    max_tension: f32,
    points_query: &Query<&Transform, With<Point>>,
) -> Option<Entity> {
    let a = points_query.get(link.a).ok()?;
    let b = points_query.get(link.b).ok()?;
    let dist = a.translation.distance_squared(b.translation);
    if dist > link.dist * link.dist * max_tension {
        Some(entity)
    } else {
        None
    }
}

pub fn handle_links(
    mut commands: Commands,
    links_query: Query<(Entity, &Link, &Tension)>,
    points_query: Query<&Transform, With<Point>>,
) {
    for (entity, link, max_tension) in links_query.iter() {
        if let Some(link) = handle_tension(entity, link, max_tension.0, &points_query) {
            commands.entity(link).despawn_recursive();
        }
    }
}
