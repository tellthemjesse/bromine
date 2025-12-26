use bevy_ecs::system::{Query, Res, ResMut};
use crate::camera::resources::CameraStateRes;
use crate::resources::input::InputState;
use std::f32::consts::TAU;

const PITCH_LIMIT: f32 = 1.55334;

pub fn update_camera_angles(input_res: Res<InputState>, mut camera_res: ResMut<CameraStateRes>) {
    let mouse_position_change = input_res.mouse_delta;
    // x = yaw, y = pitch
    let mut yaw = camera_res.yaw + mouse_position_change.x;
    let mut pitch = camera_res.pitch + mouse_position_change.y;

    yaw = yaw % TAU;
    pitch = pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);

    camera_res.yaw = yaw;
    camera_res.pitch = pitch;
}

pub fn apply_movement(/* mut query: Query<>, */ input_res: Res<InputState>, mut camera_res: ResMut<CameraStateRes>) {

}