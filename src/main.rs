#![allow(unsafe_op_in_unsafe_fn)]

// --- ECS Structure Modules ---
mod ecs;
mod components;
mod resources;
mod systems;
mod constants;
mod application_ecs;

// --- Utility Modules ---
mod graphics; 
mod collision;
mod tags;
mod player;
mod physics;
mod camera;
mod render;
mod opengl_backend;

mod types;

use std::error::Error;
use winit::event_loop::EventLoop;
use application_ecs::ApplicationECS;

fn main() -> Result<(), Box<dyn Error>> {
    // 1. Create the event loop
    let event_loop = EventLoop::builder().build()?;

    // 2. Create the application state
    let mut app = match ApplicationECS::new() {
        Ok(app) => app,
        Err(e) => {
            eprintln!("Failed to initialize application state: {}", e);
            return Err(e);
        }
    };

    // 3. Run the application
    event_loop.run_app(&mut app)?;

    Ok(())
}

/* TODO: Check
    https://crates.io/crates/tobj,
    https://docs.rs/ahash/latest/ahash/index.html,
    https://crates.io/crates/cgmath
    https://www.khronos.org/files/gltf20-reference-guide.pdf
*/
