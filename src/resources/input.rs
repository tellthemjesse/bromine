use std::collections::HashSet;
use bevy_ecs::prelude::Resource;
use winit::keyboard::KeyCode;

use glam::Vec2;

#[derive(Resource)]
#[derive(Default)]
pub struct InputState {
    pub pressed_keys: HashSet<KeyCode>,
    // Mouse delta since last frame
    pub mouse_delta: Vec2, // (dx, dy)
}

impl InputState {
    pub fn new() -> Self {
        InputState::default()
    }

    pub fn clear_delta(&mut self) {
        self.mouse_delta = Vec2::ZERO;
    }
}