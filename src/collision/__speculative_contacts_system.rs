use nalgebra_glm::{Vec3, dot, normalize};
use crate::components::transform::Transform;
use crate::collision::Collider3D;
use crate::physics::rigid_body::RigidBody;
use crate::ecs::World;

// Time threshold for collision prediction (in seconds)
const PREDICTION_TIME_THRESHOLD: f32 = 0.05;
// Coefficient to scale the correction velocity (between 0 and 1)
const VELOCITY_CORRECTION_FACTOR: f32 = 0.9;

pub fn run(world: &mut World) {
    let dt = world.delta_time;

    // Collect all entities with transform, collider and rigid body
    let mut entity_data: Vec<(usize, &Transform, &Collider3D, &mut RigidBody)> = 
        world.query_mut::<(&Transform, &Collider3D, &mut RigidBody)>()
            .enumerate()
            .map(|(idx, (transform, collider, rigid_body))| {
                (idx, transform, collider, rigid_body)
            })
            .collect();
    
    let entity_count = entity_data.len();
    
    // Check each pair of entities for potential collisions
    for i in 0..entity_count {
        let (_entity_id_i, transform_i, collider_i, _) = &entity_data[i];
        
        // Get velocity from rigid_body, but we can't modify it yet due to borrow rules
        let velocity = {
            let (_, _, _, rigid_body_i) = &entity_data[i];
            rigid_body_i.velocity
        };
        
        // Skip if object has no velocity
        if velocity.magnitude() < f32::EPSILON {
            continue;
        }
        
        // Clone the velocity so we can modify it
        let mut velocity_i = velocity;
        let pos_i = transform_i.position;
        
        for j in 0..entity_count {
            // Skip self-comparison
            if i == j {
                continue;
            }
            
            let (_, transform_j, collider_j, rigid_body_j) = &entity_data[j];
            let pos_j = transform_j.position;
            
            // Calculate predicted positions
            let predicted_pos_i = pos_i + velocity_i * dt;
            
            // Clone the collider so we can freely modify it
            let mut predicted_collider_i = Collider3D::new(predicted_pos_i, collider_i.radius * 2.0);
            
            // Check if predicted position would collide
            if predicted_collider_i.would_collide(collider_j) {
                // Calculate displacement vector from object i to object j
                let displacement = pos_j - pos_i;
                
                // If objects are already overlapping, push them apart immediately
                if collider_i.would_collide(collider_j) {
                    let penetration_depth = (collider_i.radius + collider_j.radius).magnitude() - displacement.magnitude();
                    let push_direction = if displacement.magnitude() > 0.001 {
                        normalize(&displacement)
                    } else {
                        // If objects are at exactly same position, push in random direction
                        Vec3::new(1.0, 0.0, 0.0)
                    };
                    
                    // Only adjust velocity of the first object for simplicity
                    // In a more sophisticated system, you'd distribute based on mass
                    let correction = push_direction * -1.0 * penetration_depth;
                    velocity_i = correction / dt;
                    
                    // Exit early as we've already handled this collision
                    break;
                }
                
                // Calculate relative velocity
                let relative_velocity = velocity_i - rigid_body_j.velocity;
                
                // Check if objects are moving toward each other
                if dot(&relative_velocity, &displacement) < 0.0 {
                    // Calculate time to collision (approximate)
                    let distance = displacement.magnitude() - (collider_i.radius + collider_j.radius).magnitude();
                    let approach_speed = -dot(&relative_velocity, &normalize(&displacement));
                    
                    if approach_speed > 0.0 {
                        let time_to_collision = distance / approach_speed;
                        
                        // If collision will happen soon (within threshold)
                        if time_to_collision < PREDICTION_TIME_THRESHOLD {
                            // Calculate collision normal
                            let normal = normalize(&displacement);
                            
                            // Calculate velocity along normal
                            let velocity_along_normal = dot(&velocity_i, &normal);
                            
                            // Calculate impulse (simplified - ignoring mass differences)
                            let impulse = -velocity_along_normal * VELOCITY_CORRECTION_FACTOR;
                            
                            // Apply impulse to velocity
                            velocity_i = velocity_i + normal * impulse;
                        }
                    }
                }
            }
        }
        
        // Now we can update the velocity of entity i
        // Get a mutable reference to the rigid body
        if let Some((_, _, _, rigid_body)) = entity_data.get_mut(i) {
            rigid_body.velocity = velocity_i;
        }
    }
} 