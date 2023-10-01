use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_xpbd_2d::{math::*, prelude::*, SubstepSchedule, SubstepSet};
use sandbox::{
    input::{CursorPos, InputPlugin},
    phys::PhysPlugin,
};

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins,
        PanCamPlugin::default(),
        WorldInspectorPlugin::new(),
        InputPlugin::<PanCam>::default(),
        PhysPlugin,
    ));
    app.insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Gravity(Vector::NEG_Y * 9.81 * 32.))
        .insert_resource(SubstepCount(3));

    let substeps = app
        .get_schedule_mut(SubstepSchedule)
        .expect("add SubstepSchedule first");
    substeps.add_systems(
        solve_constraint::<BendConstraint, 2>.in_set(SubstepSet::SolveUserConstraints),
    );

    app.add_systems(Startup, setup);
    app.add_systems(Update, follow_cursor);

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

    let num_points = 12;
    let mut prev = cmds
        .spawn((RigidBody::Static, Sensor, Position(Vector::ZERO)))
        .id();
    let first = prev;
    let last_pos = num_points as Scalar * Vector::Y * 32.;

    let distance_joint = |a, b| -> DistanceJoint {
        DistanceJoint::new(a, b)
            .with_limits(32., 32.)
            .with_rest_length(32.)
            .with_linear_velocity_damping(10000.)
            .with_compliance(1. / 10000000.)
    };
    // TODO Think about also constraining far away parts in the chain
    for i in 1..num_points {
        let pos = i as Scalar * Vector::Y * 32.;
        let curr = cmds
            .spawn((
                TransformBundle::default(),
                RigidBody::Dynamic,
                Position(pos),
                Collider::ball(16.),
                Sensor,
            ))
            .id();
        cmds.spawn((
            distance_joint(prev, curr),
            BendConstraint::new(first, curr, pos),
        ));
        prev = curr;
    }

    let last = cmds
        .spawn((RigidBody::Kinematic, FollowCursor, Sensor))
        .id();
    cmds.spawn((
        distance_joint(prev, last),
        BendConstraint::new(first, last, last_pos),
    ));
}

#[derive(Component)]
struct FollowCursor;

fn follow_cursor(
    cursor_pos: Res<CursorPos>,
    mut follower: Query<&mut Position, With<FollowCursor>>,
) {
    let Ok(mut follower) = follower.get_single_mut() else {
        return;
    };

    follower.0 = cursor_pos.0;
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
struct BendConstraint {
    pub entity1: Entity,
    pub entity2: Entity,
    pub goal: Vector,
    pub lagrange: Scalar,
    pub compliance: Scalar,
}

impl BendConstraint {
    pub fn new(entity1: Entity, entity2: Entity, goal: Vector) -> Self {
        Self {
            entity1,
            entity2,
            goal,
            lagrange: 0.0,
            compliance: 0.00001,
        }
    }
}

impl PositionConstraint for BendConstraint {}

impl XpbdConstraint<2> for BendConstraint {
    fn entities(&self) -> [Entity; 2] {
        [self.entity1, self.entity2]
    }

    fn solve(&mut self, bodies: [&mut RigidBodyQueryItem; 2], dt: Scalar) {
        let [body1, body2] = bodies;

        // Local attachment points at the centers of the bodies for simplicity
        let [r1, r2] = [Vector::ZERO, Vector::ZERO];

        // Distance from entity to goal
        let delta_goal = body2.current_position() - self.goal;
        let length_goal = delta_goal.length();

        // Minimize how far away from goal we are
        let c = -length_goal;

        // Avoid division by zero and unnecessary computation
        if length_goal <= 0.0 || c.abs() < Scalar::EPSILON {
            return;
        }

        // Normalized distance
        let n = delta_goal / length_goal;

        let w1 = self.compute_generalized_inverse_mass(body1, r1, n);
        let w2 = self.compute_generalized_inverse_mass(body2, r2, n);
        let w = [w1, w2];

        // Where should the bodies move to minimize c
        let gradients = [n, -n];

        // Compute magnitude of correction
        let delta_lagrange =
            self.compute_lagrange_update(self.lagrange, c, &gradients, &w, self.compliance, dt);
        self.lagrange += delta_lagrange;

        // Change position
        self.apply_positional_correction(body1, body2, delta_lagrange, n, r1, r2);
    }

    fn clear_lagrange_multipliers(&mut self) {
        self.lagrange = 0.;
    }
}
