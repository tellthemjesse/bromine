use winit::keyboard::KeyCode;
use nalgebra_glm::{normalize, vec3, cross, Vec3};

use crate::ecs::OldWorld;
use crate::components::transform::Transform;
use crate::tags::CameraTag;
use crate::constants::*;
use crate::physics::rigid_body::RigidBody;

pub fn run(world: &mut OldWorld) {
    let dt = world.delta_time;
    let mouse_delta = world.input_state.mouse_delta;
    let pressed_keys = world.input_state.pressed_keys.clone(); // Clone needed for iteration

    // --- Update Camera Angles (Yaw/Pitch from mouse) ---
    world.camera_state.update_angles(mouse_delta.0, -mouse_delta.1, MOUSE_SENSITIVITY);
    let yaw = world.camera_state.yaw;
    let pitch = world.camera_state.pitch;
    
    // --- Determine Target Roll & Interpolate --- (Uses constants)
    let target_roll = if pressed_keys.contains(&KeyCode::KeyD) {
        -TARGET_ROLL_ANGLE
    } else if pressed_keys.contains(&KeyCode::KeyA) {
        TARGET_ROLL_ANGLE
    } else {
        0.0 
    };
    let roll_diff = target_roll - world.camera_state.roll;
    world.camera_state.roll += roll_diff * INTERPOLATION_SPEED * dt;

    // --- Determine Target Temporary Pitch & Interpolate (Uses constants) ---
    let target_visual_pitch = if pressed_keys.contains(&KeyCode::KeyW) {
        -TARGET_PITCH_ANGLE
    } else if pressed_keys.contains(&KeyCode::KeyS) {
        TARGET_PITCH_ANGLE
    } else {
        0.0
    };
    let visual_pitch_diff = target_visual_pitch - world.camera_state.visual_pitch;
    world.camera_state.visual_pitch += visual_pitch_diff * INTERPOLATION_SPEED * dt;

    // Clear transient input state *after* using it for everything
    world.input_state.clear_transient_state();

    // Find the camera entity and get its mutable transform and rigid body
    let mut camera_components_opt = None;
    for (transform, rigid_body, _tag) in world.query_mut::<(&mut Transform, &mut RigidBody, &CameraTag)>() {
        camera_components_opt = Some((transform, rigid_body));
        break; 
    }
    
    if let Some((transform, rigid_body)) = camera_components_opt {
        // --- Calculate Direction Vectors (Yaw/Pitch Only) for Movement ---
        let move_forward = vec3(
            yaw.cos() * pitch.cos(),
            pitch.sin(),
            yaw.sin() * pitch.cos(),
        );
        let move_forward = normalize(&move_forward);
        let move_right = normalize(&cross(&move_forward, &vec3(0.0, 1.0, 0.0)));

        // --- Determine Desired Movement Direction from Input ---
        let mut desired_movement_direction = Vec3::zeros();

        if pressed_keys.contains(&KeyCode::KeyW) { desired_movement_direction += move_forward; }
        if pressed_keys.contains(&KeyCode::KeyS) { desired_movement_direction -= move_forward; }
        if pressed_keys.contains(&KeyCode::KeyA) { desired_movement_direction -= move_right; }
        if pressed_keys.contains(&KeyCode::KeyD) { desired_movement_direction += move_right; }
        if pressed_keys.contains(&KeyCode::Space) { desired_movement_direction += vec3(0.0, 1.0, 0.0); }
        if pressed_keys.contains(&KeyCode::ShiftLeft) { desired_movement_direction -= vec3(0.0, 1.0, 0.0); }

        // --- Update RigidBody's Velocity based on Input ---
        if desired_movement_direction != Vec3::zeros() {
            // Normalize the direction and scale by CAMERA_SPEED to set the target velocity
            rigid_body.velocity = normalize(&desired_movement_direction) * CAMERA_SPEED;
        } else {
            // No input, so camera should stop
            rigid_body.velocity = Vec3::zeros();
        }

        // --- Update Camera Position using RigidBody's Velocity ---
        transform.position += rigid_body.velocity * dt;
    } else {
        eprintln!("CameraControlSystem Error: No entity found with Transform, RigidBody, and CameraTag!");
    }
} 