// --- Imports ---
// Standard Library
use std::num::NonZeroU32;
use std::time::Instant;
use std::error::Error;
use std::path::Path;
use std::collections::HashMap;
use bevy_ecs::prelude::{Schedule, World};
//use crate::opengl_bindings as gl;

// Crates
use gl::{COLOR_BUFFER_BIT, DEPTH_BUFFER_BIT, DEPTH_TEST, DepthFunc, Enable, LESS, Viewport};

use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{DeviceEvent, DeviceId, ElementState, StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow},
    keyboard::{KeyCode, PhysicalKey},
    raw_window_handle::HasWindowHandle,
    window::{Theme, Window, WindowAttributes, WindowId}
};
use glutin::{
    config::{Config, ConfigTemplateBuilder},
    context::{ContextAttributesBuilder, PossiblyCurrentContext},
    display::GetGlDisplay,
    prelude::*,
    surface::{Surface, SurfaceAttributesBuilder, SwapInterval, WindowSurface},
};
use glutin_winit::DisplayBuilder;
use nalgebra_glm::{perspective, vec3, Vec3};
use obj::{load_obj, Position, TexturedVertex};
// Workspace Crates
use crate::ecs::world::OldWorld;
use crate::components::{
    renderable::Renderable,
    transform::Transform
};
use crate::collision::Collider3D;
use crate::tags::{DebugTag, MovingObjectTag, SpacetimeMeshTag};
use crate::systems;
use crate::graphics::{
    context as ctx,
    mesh::Mesh,
    mesh::Texture,
};
use crate::ecs::entity::EntityConstructor;
use crate::constants::*;
use crate::opengl_backend::shader::Program;
use crate::physics::rigid_body::RigidBody;
use crate::physics::spacetime_curvature::SpacetimeCurvature;
use crate::resources::manager::NewResourceManager;
use crate::tags::CameraTag;
use crate::tags::PhysicsTag;

fn get_tangent_velocity(pos: Vec3, center: Vec3, mass: f32) -> Vec3 {
    let rel_pos = pos - center;
    let r = rel_pos.magnitude();
    let velocity_mag = (G_SIM * mass / r).sqrt();

    // Get a direction perpendicular to the position vector in the XZ-plane
    let tangent_dir = Vec3::new(-rel_pos.z, 0.0, rel_pos.x).normalize();

    tangent_dir * velocity_mag
}

// --- Constants ---
const WIDTH: u32 = 800;
const HEIGHT: u32 = 650;
const FRAGMENT_SHADER: &str = include_str!("../resources/shaders/default/shader.frag");
const VERTEX_SHADER: &str = include_str!("../resources/shaders/default/shader.vert");

const DFRAGMENT_SHADER: &str = include_str!("../resources/shaders/debug/shader.frag");
const DVERTEX_SHADER: &str = include_str!("../resources/shaders/debug/shader.vert");

const CURVE_FRAG: &str = include_str!("../resources/shaders/spacetimeCurve.frag");
const CURVE_VERT: &str = include_str!("../resources/shaders/spacetimeCurve.vert");

const TEXTURED_CUBE: &[u8] = include_bytes!("../resources/models/sphere.obj");
const POSITIONED_CUBE: &[u8] = include_bytes!("../resources/models/positioned_cube.obj");

// --- Application Struct ---
pub struct ApplicationECS {
    windows: HashMap<WindowId, (Window, Surface<WindowSurface>, PossiblyCurrentContext)>,
    opengl_conf: Option<Config>,
    world: OldWorld,
    timer: Instant,
    ecs_world: World,
    schedule: Schedule,
}

// --- Implementation ---
impl ApplicationECS {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            windows: HashMap::new(),
            opengl_conf: None,
            world: OldWorld::new(),
            timer: Instant::now(),
            ecs_world: World::new(),
            schedule: Schedule::default(),
        })
    }

    // Create window and associated GL resources
    fn create_gl_window(&mut self, event_loop: &ActiveEventLoop) -> Result<WindowId, Box<dyn Error>> {
        // Get GL Config
        let conf_template_builder = ConfigTemplateBuilder::new()
            .with_swap_interval(Some(0), Some(1));

        let window_attributes = WindowAttributes::default()
            .with_title("Dying Light 3")
            .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
            .with_theme(Some(Theme::Dark));

        let display_builder = DisplayBuilder::new()
            .with_window_attributes(Some(window_attributes));

        let (built_window, config) = display_builder.build(event_loop, conf_template_builder, |mut configs| {
            configs.next().expect("No suitable GL config found!")
        }).unwrap();

        let window = built_window.unwrap();

        // Store GL configuration
        self.opengl_conf = Some(config.clone());

        // To avoid repeating .unwrap() calls use config that was just created
        let gl_display = config.display();

        let window_id = window.id();
        let (width, height): (u32, u32) = window.inner_size().into();

        // Get WindowHandle first
        let window_handle = window.window_handle()?;
        let raw_window_handle = window_handle.as_raw();

        let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            raw_window_handle, // Pass the RawWindowHandle directly
            NonZeroU32::new(width).ok_or("Window width is zero")?,
            NonZeroU32::new(height).ok_or("Window height is zero")?,
        );
        let surface = unsafe { gl_display.create_window_surface(&config, &attrs)? };

        let context_attributes = ContextAttributesBuilder::new()
            .build(Some(raw_window_handle)); // Pass RawWindowHandle here too

        let not_current_context = unsafe { gl_display.create_context(&config, &context_attributes)? };
        let context = not_current_context.make_current(&surface)?;

        // --- Load gl functions ---
        gl::load_with(|symbol| {
            gl_display.get_proc_address(&crate::graphics::context::c_string(symbol).as_c_str())
        });

        // Prefer VSync for now
        surface.set_swap_interval(&context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))?;

        window.set_cursor_visible(false);
        window.set_cursor_grab(winit::window::CursorGrabMode::Confined)?;

        // -- Define initial GL state --
        unsafe {
            Enable(DEPTH_TEST);
            DepthFunc(LESS);
        }

        let aspect_ratio = width as f32 / height as f32;
        self.world.projection_matrix = Some(perspective(aspect_ratio, 70.0f32.to_radians(), 0.1, 200.0));

        self.windows.insert(window_id, (window, surface, context));

        Ok(window_id)
    }

    fn init_world_state(&mut self) {
        println!("Loading resources...");
        let mut resource_manager = &mut self.world.resource_manager;

        let resources_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("resources");
        let textures_path = resources_path.join("textures");

        // --- Load textures ---
        let texture_path0 = textures_path.join("texture9.jpg");
        let texture0 = Texture::new_texture_2d(texture_path0.to_str().unwrap());
        let tex0_id = resource_manager.add_texture(texture0);

        let texture_path1 = textures_path.join("texture10.jpg");
        let texture1 = Texture::new_texture_2d(texture_path1.to_str().unwrap());
        let tex1_id = resource_manager.add_texture(texture1);

        // --- Load cube obj data ---
        let obj_data = load_obj::<TexturedVertex, &[u8], u16>(TEXTURED_CUBE).expect("Blablabla");
        let debug_cube_data = load_obj::<Position, &[u8], u16>(POSITIONED_CUBE).expect("Blablabla");

        let cube_mesh_obj = Mesh::<TexturedVertex, u16>::new(obj_data.clone(), None);
        let cube_mesh_obj1 = Mesh::<TexturedVertex, u16>::new(obj_data, None);
        let debug_cube_mesh = Mesh::<Position, u16>::new(debug_cube_data);

        // Add the created Mesh object to the ResourceManager
        let cube_mesh_id = resource_manager.add_mesh(cube_mesh_obj);
        let cube_mesh_id1 = resource_manager.add_mesh(cube_mesh_obj1);
        let debug_cube_id = resource_manager.add_mesh(debug_cube_mesh);
        let curve_mesh = NewResourceManager::create_curvature_grid(500, 250);
        //println!("curve m {:?}", curve_mesh);
        let spacetime_mesh_id = resource_manager.add_mesh(curve_mesh);
        //println!("Loaded cube mesh with ID: {}", cube_mesh_id);

        // Load Default Shader
        let shader_program = Program::new(VERTEX_SHADER, FRAGMENT_SHADER, None);
        let dshader_program = Program::new(DVERTEX_SHADER, DFRAGMENT_SHADER, None);
        let curve_shader = Program::new(CURVE_VERT, CURVE_FRAG, None);
        let default_shader_id = resource_manager.add_shader(shader_program);
        let dshader_id = resource_manager.add_shader(dshader_program);
        let curve_shader_id = resource_manager.add_shader(curve_shader);

        EntityConstructor::new(self.world.create_entity())
            .with(Renderable::new(spacetime_mesh_id, curve_shader_id, None))
            .with(SpacetimeMeshTag::default())
            .apply(&mut self.world);

        // TODO: File that stores all entities data -> parse it and fill the world
        println!("Creating initial entities...");

        let collider_camera = Collider3D::new(vec3(0.0, 0.0, 10.0), vec3(0.5, 0.5, 0.5));

        // --- Camera Entity ---
        EntityConstructor::new(self.world.create_entity())
            .with(Transform::identity().with_position(vec3(2.0, 0.0, 10.0)))
            .with(CameraTag::default())
            .with(MovingObjectTag::default())
            .with(collider_camera)
            .with(RigidBody::new(0.0)
                .with_restitution(0.0))
            .apply(&mut self.world);

        // -- Debug Cube Entity ---
        EntityConstructor::new(self.world.create_entity())
            .with(Transform::identity())
            .with(Renderable::new(debug_cube_id, dshader_id, None)
                .with_visibility_flag(false))
            .with(DebugTag::default())
            .apply(&mut self.world);

        let based_mass = 1.2e6f32; // Base mass for calculations
        let grav_constant = G_SIM; // Gravitational constant = 0.25

        let spacetime_curvature = SpacetimeCurvature {
            radius: 1.0,
            intensity: 1.0,
        };

        // --- Center of solar system ---
        let center = Vec3::zeros();
        let scale = vec3(1.5, 1.5, 1.5);
        EntityConstructor::new(self.world.create_entity())
            .with(Transform::identity()
                .with_position(center)
                .with_scale(scale))
            .with(Renderable::new(cube_mesh_id, default_shader_id, Some(tex0_id)))
            .with(RigidBody::new(based_mass)
                .with_restitution(0.5))
            .with(PhysicsTag::default())
            .with(MovingObjectTag::default())
            .with(Collider3D::new(center, scale))
            .with(spacetime_curvature)
            .apply(&mut self.world);

        // -- Second object --
        let position = vec3(40.0, 0.0, -20.0);
        let scale = vec3(1.3, 1.3, 1.3);
        let velocity = get_tangent_velocity(position, center, based_mass);
        let velocity = vec3(velocity.x + 0.1, velocity.y + 0.1, velocity.z + 0.1);

        EntityConstructor::new(self.world.create_entity())
            .with(Transform::identity()
                .with_position(position)
                .with_scale(scale))
            .with(Renderable::new(cube_mesh_id, default_shader_id, Some(tex0_id)))
            .with(RigidBody::new(based_mass / 10.0)
                .with_restitution(0.5)
                .with_velocity(velocity))
            .with(PhysicsTag::default())
            .with(MovingObjectTag::default())
            .with(Collider3D::new(position, scale))
            .with(spacetime_curvature)
            .apply(&mut self.world);

        // -- Third object --
        let position = vec3(-40.0, 0.0, 20.0);
        let scale = vec3(1.0, 1.0, 1.0);
        let velocity = get_tangent_velocity(position, center, based_mass);

        EntityConstructor::new(self.world.create_entity())
            .with(Transform::identity()
                .with_position(position)
                .with_scale(scale))
            .with(Renderable::new(cube_mesh_id, default_shader_id, Some(tex1_id)))
            .with(RigidBody::new(based_mass / 10.0)
                .with_restitution(0.5)
                .with_velocity(velocity))
            .with(PhysicsTag::default())
            .with(MovingObjectTag::default())
            .with(Collider3D::new(position, scale))
            .with(spacetime_curvature)
            .apply(&mut self.world);

        println!("Initial entities successfully created");
    }
}

// --- ApplicationHandler Implementation ---
impl ApplicationHandler for ApplicationECS {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        // Reset delta time timer at the start of a new event phase
        // Or handle specific StartCause if needed (e.g., ResumeTimeReached)
        if cause == StartCause::Init {
            println!("Event Loop Initialized.");
        }
        // Reset timer for delta time calculation before processing events for this iteration
        self.timer = Instant::now();
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("Application Resumed");
        // --- Create the initial window when the application or starts
        if self.windows.is_empty() {
            // We set control flow only ONCE, then might change if FPS values are used
            // Poll for VSync
            // WaitUntil for constant FPS value
            event_loop.set_control_flow(ControlFlow::Poll);

            match self.create_gl_window(event_loop) {
                Ok(window_id) => println!("Created initial window with ID: {:?}", window_id),
                Err(e) => eprintln!("Failed to create initial window: {}", e),
            }

            // -- Initialize world state AFTER OpenGL functions are loaded
            self.init_world_state();
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _event: ()) {
        // Handle custom user events if you use event_loop.create_proxy()
        // Examples in Git Repo, not needed now
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        // Get the specific window, surface, and context for this event
        let window_entry = self.windows.get_mut(&window_id);

        if window_entry.is_none() { return; }
        let (window, surface, context) = window_entry.unwrap();

        match event {
            WindowEvent::Resized(physical_size) => {
                println!("Window {:?} resized to {:?}", window_id, physical_size);
                 if let (Some(width), Some(height)) =
                     (NonZeroU32::new(physical_size.width), NonZeroU32::new(physical_size.height)) {
                     // Use resize directly on the surface
                     surface.resize(context, width, height);
                     let aspect_ratio = physical_size.width as f32 / physical_size.height as f32;

                     self.world.projection_matrix = Some(perspective(aspect_ratio, 50.0f32.to_radians(), 0.1, 100.0));
                     unsafe { Viewport(0, 0, physical_size.width as i32, physical_size.height as i32); }
                } else {
                     eprintln!("Invalid resize dimensions received: {}x{}", physical_size.width, physical_size.height);
                }
            }
            WindowEvent::CloseRequested => {
                println!("Close requested for window {:?}", window_id);
                self.windows.remove(&window_id);
                if self.windows.is_empty() {
                    println!("Last window closed, exiting.");
                    event_loop.exit();
                }
            }
            WindowEvent::RedrawRequested => {
                let current_delta = self.world.delta_time;

                // Make context current
                if !context.is_current() {
                    if let Err(e) = context.make_current(surface) {
                         eprintln!("Failed to make context current for redraw: {}", e);
                         return;
                    }
                }

                ctx::clear_color(0.0, 0.0, 0.0, 1.0);
                ctx::clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);

                // DEBUG Log (using info obtained before mutable borrow)
                static mut FRAME_COUNT: u64 = 0;
                const LOG_INTERVAL: u64 = 360;
                unsafe {
                    FRAME_COUNT += 1;
                    if FRAME_COUNT % LOG_INTERVAL == 0 {
                        println!(
                            "[Runner thread] Frame: {}, Delta: {:.4}s, Entities: {}, Components Stores: {}",
                            FRAME_COUNT,
                            current_delta,
                            self.world.entity_components.len(), // This is immutable borrow of world, fine
                            self.world.components.len(),      // This is immutable borrow of world, fine
                        );
                    }
                }

                use crate::{camera, collision, physics};

                // --- Run Systems ---

                physics::gravity_system::run(&mut self.world);
                physics::physics_system::run(&mut self.world);
                camera::camera_control_system::run(&mut self.world);
                camera::camera_system::run(&mut self.world);

                collision::collider_update_system::run(&mut self.world);
                collision::collision_detection_system::run(&mut self.world);
                collision::collision_handle_system::run(&mut self.world);


                systems::render::run(&self.world);
                systems::debug_render_system::run(&self.world);
                systems::curvature_render::run(&self.world);

                // --- Finish Frame (Swap Buffers) ---
                if let Err(e) = surface.swap_buffers(context) {
                    eprintln!("Failed to swap buffers for window {:?}: {}", window_id, e);
                }
                // -- Since we're using VSync, it doesn't matter where to place redraw request
                // => How to implement constant FPS value?
                // POSSIBLE SOLUTION: Split app main thread into render thread and runner thread
                // Make render thread sleep after swap until set FPS timing, while runner thread handles everything else
                window.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                 // Update key state in World resource
                 if let PhysicalKey::Code(key_code) = event.physical_key {
                     match event.state {
                         ElementState::Pressed => {
                             if !event.repeat { // Only insert if not a repeat press
                                 self.world.input_state.pressed_keys.insert(key_code);
                             }
                         }
                         ElementState::Released => {
                             self.world.input_state.pressed_keys.remove(&key_code);
                         }
                     }
                 }
                 // Handle Escape key directly
                 if event.physical_key == PhysicalKey::Code(KeyCode::Escape) && event.state == ElementState::Pressed {
                     event_loop.exit();
                 }
            }
            WindowEvent::CursorMoved { position: _, .. } => {
                 // This position is relative to the window
            }
            WindowEvent::MouseWheel { delta: _, phase: _, .. } => {
                 // Handle mouse wheel input
            }
            WindowEvent::MouseInput { state: _, button: _, .. } => {
                 // Handle mouse button input
            }
            WindowEvent::CursorEntered { .. } => {
                 // Handle cursor entering the window
            }
            WindowEvent::CursorLeft { .. } => {
                 // Handle cursor leaving the window
            }
            WindowEvent::Focused(focused) => {
                println!("Window {:?} focus changed: {}", window_id, focused);
                if focused {
                    // Maybe re-confine cursor? Check winit docs/examples
                    // window.set_cursor_grab(winit::window::CursorGrabMode::Confined).ok();
                } else {
                    // Release cursor grab?
                    // window.set_cursor_grab(winit::window::CursorGrabMode::None).ok();
                    // Clear input state maybe?
                    self.world.input_state.pressed_keys.clear();
                    self.world.input_state.clear_transient_state();
                }
            }
            _ => (),
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: DeviceId, event: DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => {

                self.world.input_state.mouse_delta.0 += delta.0 as f32; // Cast to f32
                self.world.input_state.mouse_delta.1 += delta.1 as f32; // Cast to f32
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Calculate delta time here before waiting
        let delta = self.timer.elapsed().as_secs_f32();
        self.world.delta_time = delta;
        self.timer = Instant::now(); // Reset timer for next frame/iteration ???

        // Reset mouse delta, can remove it from cam_control_system ?
        self.world.input_state.clear_transient_state();

        // Request redraw for all windows that need it (typically all for games)
        /*for (window, _, _) in self.windows.values() {
            window.request_redraw();
        }*/
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        println!("Application Suspended");
    }


    // TODO: Safe game state before exit
    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        println!("Application Exiting");
    }

    fn memory_warning(&mut self, _event_loop: &ActiveEventLoop) {
        println!("Memory Warning Received!");
    }
}