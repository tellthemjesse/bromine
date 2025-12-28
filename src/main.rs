#![allow(unsafe_op_in_unsafe_fn)]

mod ecs;
mod components;
mod resources;
mod systems;
mod constants;
mod application_ecs;

mod graphics; 
mod collision;
mod tags;
mod physics;
mod camera;
mod opengl_backend;

mod types;

use winit::event_loop::EventLoop;
use application_ecs::Application;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_ansi(true)
        .init();
    
    let ev_loop = EventLoop::builder().build()?;

    let mut app = Application::new();

    ev_loop.run_app(&mut app)?;

    Ok(())
}
