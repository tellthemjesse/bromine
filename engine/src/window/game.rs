use winit::{
    application::ApplicationHandler,
    event_loop::ActiveEventLoop,
    window::WindowId,
};

pub trait Game: ApplicationHandler {
    /// Creates an OpenGL-compatible window, ready for rendering
    fn make_window(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> ::anyhow::Result<WindowId>;
    /// Loads game resources
    fn prep_game_world(&mut self);
    /// Drops game resources
    fn drop_game_world(&mut self);
}
