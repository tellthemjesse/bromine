use crate::ecs::{
    components::{Camera, Position},
    resources::{MouseDelta, PressedKeys, TimeDelta, View},
};
use engine::{ecs::World, query, query_resource};
use glam::{Mat4, Vec3, Vec3Swizzles};
use std::f32::consts::TAU;
use winit::keyboard::KeyCode;

pub fn s_camera_control(world: &mut World) {
    let mut query;
    let index;
    query!(world, Camera, and Position, out(query), entity(index));
    let dt = query_resource!(world, TimeDelta).as_f32();
    let mouse = query_resource!(world, MouseDelta);

    let camera = query.0[index].as_mut().unwrap();
    {
        camera.yaw += mouse.dx().to_radians() as f32 * 0.1;
        camera.pitch -= mouse.dy().to_radians() as f32 * 0.1;

        camera.pitch = camera
            .pitch
            .clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());
        camera.yaw %= TAU;
    }

    let (yaw, pitch, _roll) = camera.angles();
    let position = query.1[index].as_mut().unwrap();
    let keys = query_resource!(world, PressedKeys);

    let mut direction = Vec3::ZERO;

    let forward = Vec3::new(
        yaw.cos() * pitch.cos(),
        pitch.sin(),
        yaw.sin() * pitch.cos(),
    )
    .normalize();
    let right = forward.cross(Vec3::Y).normalize();

    if keys.contains(&KeyCode::KeyW) {
        direction += forward;
    }
    if keys.contains(&KeyCode::KeyS) {
        direction -= forward;
    }
    if keys.contains(&KeyCode::Space) {
        direction += Vec3::Y;
    }
    if keys.contains(&KeyCode::ShiftLeft) {
        direction -= Vec3::Y;
    }
    if keys.contains(&KeyCode::KeyD) {
        direction += right
    }
    if keys.contains(&KeyCode::KeyA) {
        direction -= right;
    }

    let velocity = direction.normalize_or_zero() * 4.0;

    *position += velocity * dt;
}

pub fn s_camera_view(world: &mut World) {
    let query;
    let index;
    query!(world, Camera, and Position, out(query), entity(index));
    let mut view = query_resource!(world, mut View);

    let camera = query.0[index].as_ref().unwrap();
    let position = query.1[index].as_ref().unwrap().xyz();

    let yaw = camera.yaw;
    let pitch = camera.pitch;

    let forward = Vec3::new(
        yaw.cos() * pitch.cos(),
        pitch.sin(),
        yaw.sin() * pitch.cos(),
    )
    .normalize();
    let right = forward.cross(Vec3::Y).normalize();
    let up = right.cross(forward).normalize();
    let target = position + forward;

    *view = Mat4::look_at_rh(position, target, up).into();
}
