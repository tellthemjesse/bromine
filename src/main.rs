#![deny(unused_results)]

mod application;
mod camera;
mod collision;
mod components;
mod constants;
mod ecs;
mod graphics;
mod opengl_backend;
mod physics;
mod resources;
mod systems;
mod tags;
mod types;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_ansi(true)
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let ev_loop = winit::event_loop::EventLoop::builder().build()?;

    let mut app = application::Application::new();

    ev_loop.run_app(&mut app)?;

    Ok(())
}
