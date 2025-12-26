use crate::ecs::world::EcsWorld;
use crate::tags::PhysicsTag;
use crate::physics::rigid_body::RigidBody;
use crate::components::transform::Transform;
use crate::constants::G_SIM;
use nalgebra_glm::normalize;

const TINY_NUMBER: f32 = 1e-6;

pub fn run(world: &mut EcsWorld) {
    let mut physics_entities: Vec<(&Transform, &mut RigidBody)> = world.query_mut::<(&Transform, &mut RigidBody, &PhysicsTag)>()
        .map(|(transform, rb, _)| (transform, rb))
        .collect();

    for (_, rigid_body) in physics_entities.iter_mut() {
        rigid_body.clear_acceleration();
    }

    // split_at_mut for safe pairwise mutable borrowing
    let mut i = 0;
    while i < physics_entities.len() {
        let (left_slice, right_slice) = physics_entities.split_at_mut(i + 1);
        let (transform_i, rigid_body_i) = left_slice.last_mut().unwrap();
        let mass_i = rigid_body_i.mass;
        let pos_i = transform_i.position;

        for (transform_j, rigid_body_j) in right_slice.iter_mut() {
            let mass_j = rigid_body_j.mass;
            let pos_j = transform_j.position;

            let vec_ij = pos_j - pos_i;
            let dist_sq = vec_ij.magnitude_squared() + TINY_NUMBER;
            let force_magnitude = G_SIM * mass_i * mass_j / dist_sq;
            let force_direction = normalize(&vec_ij);

            // Apply force as acceleration (a = F/m)
            let accel_change = force_direction * force_magnitude;

            rigid_body_i.acceleration += accel_change / mass_i;
            rigid_body_j.acceleration -= accel_change / mass_j;
        }
        i += 1;
    }
}
