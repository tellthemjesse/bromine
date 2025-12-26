use std::collections::{HashMap, HashSet};
use nalgebra_glm::{Vec3, dot, normalize, reflect, reflect_vec};
use std::ops::{AddAssign, Div, Mul};
use crate::collision::Collider3D;
use crate::ecs::OldWorld;
use crate::components::transform::Transform;
use crate::ecs::entity::Entity;
use crate::physics::rigid_body::RigidBody;

pub fn run(world: &mut OldWorld) {
    let collision_pairs = world.collider_cache.clone();
    let dt = world.delta_time;

    let mut velocity_map: HashMap<Entity, Vec3> = HashMap::new();
    let mut displacement_map: HashMap<Entity, Vec3> = HashMap::new();

    for (entity_1, entity_2) in collision_pairs {
        let query_1 = world
            .query_entity::<(&RigidBody, &Collider3D)>(entity_1);
        let query_2 = world
            .query_entity::<(&RigidBody, &Collider3D)>(entity_2);

        if let (Some((rb_1, c1)), Some((rb_2, c2))) = (query_1, query_2) {
            if let Some((surface_normal, depth)) = c1.get_collision_info(c2) {
                let velocity_1  = rb_1.velocity;
                let velocity_2 = rb_2.velocity;
                let velocity_rel = velocity_2 - velocity_1;
                let velocity_along_normal = dot(&velocity_rel, &surface_normal);

                // If dot product is negative, the objects were moving towards each other
                if velocity_along_normal.lt(&0.0) {
                    // kx = ma => a = kx/m or dv = kx/m dt
                    let impact_restitution = (rb_1.restitution + rb_2.restitution) / 2.0;

                    let velocity_1_unchecked = reflect_vec(&velocity_1, &surface_normal);
                    let velocity_2_unchecked = reflect_vec(&velocity_2, &surface_normal);

                    let velocity_1 = if rb_1.mass.gt(&0.0) {
                        velocity_1_unchecked.mul(impact_restitution).div(rb_1.mass)
                    } else {
                        velocity_1_unchecked
                    };

                    let velocity_2 = if rb_2.mass.gt(&0.0) {
                        velocity_2_unchecked.mul(impact_restitution).div(rb_2.mass)
                    } else {
                        velocity_2_unchecked
                    };

                    velocity_map.insert(entity_1, velocity_1);
                    velocity_map.insert(entity_2, velocity_2);

                    // x = ma/k or x = m/k * dv/dt

                    let displacement_1 =
                        surface_normal.mul(depth);
                    let displacement_2 =
                        surface_normal.mul(depth);

                    displacement_map.entry(entity_1)
                        .and_modify(|x| { *x + displacement_1; })
                        .or_insert(displacement_1);

                    displacement_map.entry(entity_1)
                        .and_modify(|x| { *x + displacement_2; })
                        .or_insert(displacement_2);
                }
            }
        }

        for (entity_id, new_velocity) in &velocity_map {
            if let Some(mut rb) = world.get_component_mut::<RigidBody>(*entity_id) {
                rb.velocity = *new_velocity;
            }
        }
        for (entity_id, pos_change) in &displacement_map {
            if let Some(mut transform) = world.get_component_mut::<Transform>(*entity_id) {
                transform.position -= pos_change;
            }
        }

        // Clear the cache
        world.collider_cache = HashSet::new();
    }
}