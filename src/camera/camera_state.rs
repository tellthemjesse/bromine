// Resource to store camera orientation state (yaw and pitch)

// --- Constants ---
use std::f32::consts::PI;
const PITCH_LIMIT: f32 = 1.55334; // 89 degrees as radian


// --- Camera State ---
#[derive(Debug)] // Make it debuggable
pub struct CameraState {
    // Yaw angle (rotation around Y axis) in radians
    pub yaw: f32,
    // Pitch angle (rotation around X axis) in radians
    pub pitch: f32,
    pub roll: f32,
    pub visual_pitch: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        CameraState {
            // Initial state: looking straight ahead along -Z
            yaw: -90.0f32.to_radians(), // Yaw of -90 degrees faces -Z
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

    // Helper to update state based on mouse delta, clamping pitch
    pub fn update_angles(&mut self, dx: f32, dy: f32, sensitivity: f32) {
        self.yaw += dx.to_radians() * sensitivity;
        self.pitch += dy.to_radians() * sensitivity;

        // Clamp pitch to avoid looking straight up/down or flipping
        self.pitch = self.pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);

        // Keep yaw within standard range (optional, but can prevent large numbers)
        self.yaw = self.yaw % (2.0 * PI);
    }
} 