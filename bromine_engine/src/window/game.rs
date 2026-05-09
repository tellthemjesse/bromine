use crate::ecs::World;
use winit::{application::ApplicationHandler, event_loop::ActiveEventLoop, window::WindowId};

pub trait Game: ApplicationHandler {
    /// Creates an OpenGL-compatible window, ready for rendering
    fn make_window(&mut self, event_loop: &ActiveEventLoop) -> ::anyhow::Result<WindowId>;
    fn world(&self) -> &World;
    fn world_mut(&mut self) -> &mut World;
    /// Loads game resources
    fn init_world(&mut self);
    /// Drops game resources
    fn deinit_world(&mut self);
}
