use std::f32::consts::TAU;
const PITCH_LIMIT: f32 = 1.55334; // 89 degrees
use nutype::nutype;

#[nutype(sanitize(with = |val| val.clamp(-PITCH_LIMIT, PITCH_LIMIT)))]
struct Yaw(f32);
struct Pitch(f32);
struct Roll(f32);

#[derive(Debug)]
pub struct CameraState {
    // yaw corresponds to rotation around Y axis
    pub yaw: f32,
    // pitch corresponds to rotation around X axis
    pub pitch: f32,
    pub roll: f32,
    pub visual_pitch: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        CameraState {
            // looking straight ahead along -Z
            yaw: -90.0f32.to_radians(),
            pitch: 0.0,
            roll: 0.0,
            visual_pitch: 0.0,
        }
    }
}

impl CameraState {
    pub fn new() -> Self {
        CameraState::default()
    }

    pub fn update_angles(&mut self, dx: f32, dy: f32, sensitivity: f32) {
        self.yaw += dx.to_radians() * sensitivity;
        self.pitch += dy.to_radians() * sensitivity;

        // clamp pitch
        self.pitch = self.pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);
        // reduce yaw angle
        self.yaw = self.yaw % TAU;
    }
}
