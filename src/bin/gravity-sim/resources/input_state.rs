use std::collections::HashSet;
use winit::keyboard::KeyCode;

#[derive(Debug, Default)]
pub struct InputState {
    /// set of currently pressed keyboard keys
    pub pressed_keys: HashSet<KeyCode>,
    /// mouse delta since last frame
    pub mouse_delta: (f32, f32), // (dx, dy)
}

impl InputState {
    pub fn new() -> Self {
        InputState::default()
    }

    pub fn clear_transient_state(&mut self) {
        self.mouse_delta = (0.0, 0.0);
    }
}
