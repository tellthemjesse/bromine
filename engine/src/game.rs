pub trait Game: winit::application::ApplicationHandler {
    /// Creates an OpenGL-compatible window, ready for rendering
    fn make_window(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> anyhow::Result<winit::window::WindowId>;
    /// Loads game resources
    fn prep_game_world(&mut self);
    /// Drops game resources
    fn drop_game_world(&mut self);
}
