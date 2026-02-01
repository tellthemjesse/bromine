use std::collections::HashSet;
use winit::keyboard::KeyCode;

use nalgebra_glm::{cross, normalize, vec3, Vec3};

use crate::constants::{
    CAMERA_SPEED, INTERPOLATION_SPEED, MOUSE_SENSITIVITY, TARGET_PITCH_ANGLE, TARGET_ROLL_ANGLE,
};
use crate::tags::CameraTag;
use crate::types::{EcsWorld, RigidBody, Transform};

pub fn run(world: &mut EcsWorld) {
    let dt = world.delta_time;
    let mouse_delta = world.input_state.mouse_delta;
    let pressed_keys = world.input_state.pressed_keys.clone();

    // apply mouse movement
    world
        .camera_state
        .update_angles(mouse_delta.0, -mouse_delta.1, MOUSE_SENSITIVITY);
    let yaw = world.camera_state.yaw;
    let pitch = world.camera_state.pitch;

    // get roll direction
    let target_roll = sgn_roll(&pressed_keys) * TARGET_ROLL_ANGLE;

    let roll_diff = target_roll - world.camera_state.roll;
    world.camera_state.roll += roll_diff * INTERPOLATION_SPEED * dt;

    // get pitch direction
    let target_visual_pitch = sgn_pitch(&pressed_keys) * TARGET_PITCH_ANGLE;

    let visual_pitch_diff = target_visual_pitch - world.camera_state.visual_pitch;
    world.camera_state.visual_pitch += visual_pitch_diff * INTERPOLATION_SPEED * dt;

    // clear input state
    world.input_state.clear_transient_state();

    // find camera entity components
    let mut camera_components_opt = None;
    for (transform, rigid_body, _tag) in
        world.query_mut::<(&mut Transform, &mut RigidBody, &CameraTag)>()
    {
        camera_components_opt = Some((transform, rigid_body));
        break;
    }

    if let Some((transform, rigid_body)) = camera_components_opt {
        // calculate direction vectors
        let move_forward = vec3(
            yaw.cos() * pitch.cos(),
            pitch.sin(),
            yaw.sin() * pitch.cos(),
        );
        let move_forward = normalize(&move_forward);
        let move_right = normalize(&cross(&move_forward, &vec3(0.0, 1.0, 0.0)));

        // determine desired movement direction
        let mut desired_movement_direction = Vec3::zeros();

        if pressed_keys.contains(&KeyCode::KeyW) {
            desired_movement_direction += move_forward;
        }
        if pressed_keys.contains(&KeyCode::KeyS) {
            desired_movement_direction -= move_forward;
        }
        if pressed_keys.contains(&KeyCode::KeyA) {
            desired_movement_direction -= move_right;
        }
        if pressed_keys.contains(&KeyCode::KeyD) {
            desired_movement_direction += move_right;
        }
        if pressed_keys.contains(&KeyCode::Space) {
            desired_movement_direction += vec3(0.0, 1.0, 0.0);
        }
        if pressed_keys.contains(&KeyCode::ShiftLeft) {
            desired_movement_direction -= vec3(0.0, 1.0, 0.0);
        }

        // update velocity
        if desired_movement_direction != Vec3::zeros() {
            rigid_body.velocity = normalize(&desired_movement_direction) * CAMERA_SPEED;
        } else {
            rigid_body.velocity = Vec3::zeros();
        }

        // update position
        transform.position += rigid_body.velocity * dt;
    } else {
        eprintln!("[Error]: Couldn't find camera transform");
    }
}

fn sgn_pitch(keys: &HashSet<KeyCode>) -> f32 {
    if keys.contains(&KeyCode::KeyW) {
        -1.0
    } else if keys.contains(&KeyCode::KeyS) {
        1.0
    } else {
        0.0
    }
}

fn sgn_roll(keys: &HashSet<KeyCode>) -> f32 {
    if keys.contains(&KeyCode::KeyD) {
        -1.0
    } else if keys.contains(&KeyCode::KeyA) {
        1.0
    } else {
        0.0
    }
}
