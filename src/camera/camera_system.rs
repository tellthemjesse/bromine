// System responsible for calculating the view matrix based on the camera entity and CameraState resource
use nalgebra_glm::{vec3, cross, normalize, look_at};

use crate::ecs::OldWorld;
use crate::components::transform::Transform;
use crate::tags::CameraTag;

pub fn run(world: &mut OldWorld) {
    // Find the camera entity using the new query system
    let camera_transform = world.query::<(&Transform, &CameraTag)>()
        .next()
        .map(|(transform, _tag)| transform);
        
    let camera_position = if let Some(transform) = camera_transform {
        transform.position
    } else {
        eprintln!("CameraSystem Error: No entity found with Transform and CameraTag!");
        world.view_matrix = None;
        return;
    };

    // Get CORE yaw and pitch from CameraState resource
    // We IGNORE roll and temp_pitch here!
    let yaw = world.camera_state.yaw;
    let pitch = world.camera_state.pitch;

    // 1. Calculate forward direction from CORE yaw and pitch
    let forward = vec3(
        yaw.cos() * pitch.cos(),
        pitch.sin(),
        yaw.sin() * pitch.cos(),
    );
    let forward = normalize(&forward);

    // 2. Calculate right vector using cross product with world up
    let right = normalize(&cross(&forward, &vec3(0.0, 1.0, 0.0)));

    // 3. Calculate the camera's ACTUAL up vector (no roll applied here)
    let final_up = normalize(&cross(&right, &forward));

    // 4. Calculate the target point
    let target = camera_position + forward;

    // 5. Calculate the BASE view matrix using only core orientation
    let base_view = look_at(&camera_position, &target, &final_up);

    // Store the BASE view matrix in the World resource
    world.view_matrix = Some(base_view);
} 