// Resource to store current input state

use std::collections::HashSet;
use winit::keyboard::KeyCode; // Use winit's KeyCode

#[derive(Debug, Default)]
pub struct InputState {
    // Set of currently pressed keyboard keys
    pub pressed_keys: HashSet<KeyCode>,
    // Mouse delta since last frame
    pub mouse_delta: (f32, f32), // (dx, dy)
    // TODO: Add mouse button states, scroll wheel, etc. later
}

impl InputState {
    pub fn new() -> Self {
        InputState::default()
    }

    // Call this at the start of each frame to reset transient state like mouse delta
    pub fn clear_transient_state(&mut self) {
        self.mouse_delta = (0.0, 0.0);
    }
} 