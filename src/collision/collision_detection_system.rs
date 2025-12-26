use crate::types::{EcsWorld, Entity, Collider3D};

pub fn run(world: &mut EcsWorld) {
    let mut collider_cache = world.collider_cache.clone();

    let colliders: Vec<(Entity, &Collider3D)> = world
        .query::<(Entity, &Collider3D)>().collect();

    for i in 0..colliders.len() {
        let (entity_ith, collider_ith) = colliders.get(i).unwrap();
        for j in i + 1..colliders.len() {
            let (entity_jth, collider_jth) = colliders.get(j).unwrap();
            if collider_ith.would_collide(collider_jth) {
                collider_cache.insert((*entity_ith, *entity_jth));
            }
        }
    }

    world.collider_cache = collider_cache;
}
