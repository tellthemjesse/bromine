use nalgebra_glm::{vec3, cross, normalize, look_at};

use crate::types::{EcsWorld, Transform};
use crate::tags::CameraTag;

pub fn run(world: &mut EcsWorld) {
    let camera_transform = world.query::<(&Transform, &CameraTag)>()
        .next()
        .map(|(transform, _tag)| transform);

    let camera_position = if let Some(transform) = camera_transform {
        transform.position
    } else {
        eprintln!("[Error]: Couldn't find camera transform");
        world.view_matrix = None;
        return;
    };

    let yaw = world.camera_state.yaw;
    let pitch = world.camera_state.pitch;

    let forward = normalize(&vec3(
        yaw.cos() * pitch.cos(),
        pitch.sin(),
        yaw.sin() * pitch.cos(),
    ));
    let right = normalize(&cross(&forward, &vec3(0.0, 1.0, 0.0)));
    let final_up = normalize(&cross(&right, &forward));
    let target = camera_position + forward;
    let base_view = look_at(&camera_position, &target, &final_up);

    world.view_matrix = Some(base_view);
}
