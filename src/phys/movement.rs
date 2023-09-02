use bevy::prelude::*;

pub struct MovementPlugin;

#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone)]
pub struct VelocityLabel;
#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone)]
pub struct ForceLabel;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (apply_force, apply_torque).in_set(ForceLabel))
            .add_systems(
                Update,
                (apply_velocity, apply_angular_velocity)
                    .in_set(VelocityLabel)
                    .after(ForceLabel),
            );
        app.register_type::<LinearVelocity>()
            .register_type::<AngularVelocity>()
            .register_type::<Force>()
            .register_type::<Torque>();

        bevy::log::info!("Loaded movement plugin");
    }
}

#[derive(Component, Deref, DerefMut, Reflect, Clone, Default)]
pub struct LinearVelocity(pub Vec2);
#[derive(Component, Deref, DerefMut, Reflect, Clone, Default)]
pub struct AngularVelocity(pub f32);

#[derive(Component, Deref, DerefMut, Reflect, Clone, Default)]
pub struct Force(pub Vec2);
#[derive(Component, Deref, DerefMut, Reflect, Clone, Default)]
pub struct Torque(pub f32);

pub fn apply_velocity(mut velocities: Query<(&mut Transform, &LinearVelocity)>, time: Res<Time>) {
    for (mut transform, dir) in &mut velocities.iter_mut() {
        transform.translation += dir.extend(0.0) * time.delta_seconds();
    }
}

pub fn apply_angular_velocity(
    mut velocities: Query<(&mut Transform, &AngularVelocity)>,
    time: Res<Time>,
) {
    for (mut transform, torque) in &mut velocities.iter_mut() {
        transform.rotate(Quat::from_rotation_z(**torque * time.delta_seconds()));
    }
}

pub fn apply_force(mut forces: Query<(&mut LinearVelocity, &Force)>, time: Res<Time>) {
    for (mut vel, force) in &mut forces.iter_mut() {
        **vel += **force * time.delta_seconds();
    }
}

pub fn apply_torque(mut forces: Query<(&mut AngularVelocity, &Torque)>, time: Res<Time>) {
    for (mut vel, torque) in &mut forces.iter_mut() {
        **vel += **torque * time.delta_seconds();
    }
}
