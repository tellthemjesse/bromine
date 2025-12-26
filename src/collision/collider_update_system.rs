use crate::types::{EcsWorld, Transform, Collider3D};

pub fn run(world: &mut EcsWorld) {
    let moving_entities = world
        .query_mut::<(&Transform, &mut Collider3D)>();

    for (transform, collider) in moving_entities {
        collider.update_position(transform.position);
    }
}
