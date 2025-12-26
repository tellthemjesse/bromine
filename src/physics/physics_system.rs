use crate::ecs::world::OldWorld;
use crate::tags::PhysicsTag;
use crate::physics::rigid_body::RigidBody;
use crate::components::transform::Transform;

pub fn run(world: &mut OldWorld) {
    let dt = world.delta_time;
    // Query for entities with PhysicsTag, RigidBody, and Transform components
    // Use the new generic query method!
    // We need mutable access to Transform and RigidBody
    for (transform, rigid_body, _tag) in world.query_mut::<(&mut Transform, &mut RigidBody, &PhysicsTag)>() {
        // Update velocity based on acceleration
        rigid_body.velocity += rigid_body.acceleration * dt;
        
        // Update position based on velocity
        //if rigid_body.velocity.abs().gt(&vec3(1e-6, 1e-6, 1e-6f32)) {
            transform.position += rigid_body.velocity * dt;
        //}
    }
}   
