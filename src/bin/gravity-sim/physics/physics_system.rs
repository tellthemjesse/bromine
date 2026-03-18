use crate::tags::PhysicsTag;
use crate::types::{EcsWorld, RigidBody, Transform};

/// applies acceleration & velocity to the entity transform
pub fn run(world: &mut EcsWorld) {
    let dt = world.delta_time;
    for (transform, rigid_body, _tag) in
        world.query_mut::<(&mut Transform, &mut RigidBody, &PhysicsTag)>()
    {
        rigid_body.velocity += rigid_body.acceleration * dt;
        transform.position += rigid_body.velocity * dt;
    }
}
