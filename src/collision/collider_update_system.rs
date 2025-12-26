use crate::collision::Collider3D;
use crate::tags::MovingObjectTag;
use crate::components::transform::Transform;
use crate::ecs::OldWorld;

pub fn run(world: &mut OldWorld) {
    let mut moving_entities = world
        .query_mut::<(&Transform, &mut Collider3D)>();

    for (transform, collider) in moving_entities {
        collider.update_position(transform.position);
    }
}