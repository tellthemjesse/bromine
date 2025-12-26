use std::num::NonZeroU32;
use std::time::Instant;
use std::error::Error;
use std::path::Path;

use winit::{
    event_loop::{EventLoop, ControlFlow},
    window::{Window, WindowBuilder, Theme},
    dpi::{LogicalSize},
    event::{Event, WindowEvent, DeviceEvent},
    keyboard::{PhysicalKey, KeyCode},
};
use glutin::{
    prelude::*,
    surface::{Surface, WindowSurface, SwapInterval, SurfaceAttributesBuilder},
    context::{PossiblyCurrentContext, ContextAttributesBuilder},
    display::{Display, GetGlDisplay},
    config::{Config, ConfigTemplateBuilder},
};
use glutin_winit::DisplayBuilder;
use gl; // Keep gl crate import
use gl::{Enable, DepthFunc, DEPTH_TEST, LEQUAL, PolygonMode, FRONT_AND_BACK, LINE, TEXTURE_CUBE_MAP_SEAMLESS, LESS};
use gl::types::GLenum;
use obj::{TexturedVertex, Position, load_obj, ObjResult, Obj};
use nalgebra_glm as glm;
use nalgebra_glm::{Mat4, Vec3, vec3, mat4, mat3_to_mat4, mat4_to_mat3, perspective, scale};
// Added glm import

// Re-export needed types from main's modules
use crate::shader::{Program};
use crate::mesh_loader_generics::{Texture, Mesh, draw_instanced, draw};
use crate::realistic_camera::{Camera, Action};
use crate::context as ctx;
use crate::collision::AABB;
use crate::skybox::Skybox;

use raw_window_handle::HasRawWindowHandle;


// Constants from main
const DEFAULT_FS: &str = include_str!("../resources/shaders/default/shader.frag");
const DEFAULT_VS: &str = include_str!("../resources/shaders/default/shader.vert");
const MODEL_SRC: &[u8] = include_bytes!("../resources/models/textured_cube.obj");
const WIDTH: u32 = 800;
const HEIGHT: u32 = 650;

// Constants for debug drawing
const DEBUG_FS: &str = include_str!("../resources/shaders/debug/debug_frag.glsl");
const DEBUG_VS: &str = include_str!("../resources/shaders/debug/debug_vert.glsl");
const SKYBOX_FS: &str = include_str!("../resources/shaders/skybox/skybox.frag");
const SKYBOX_VS: &str = include_str!("../resources/shaders/skybox/skybox.vert");

// Define the Renderable trait
trait Renderable {
    // Method to draw the object
    fn draw(&self, view: &Mat4, projection: &Mat4);

    // Maybe need methods to get AABB or other properties later,
    // but for now, just drawing is main job.
    // We also need a way to get instance translations for physics/AABB update
    fn get_instance_translations(&self) -> Option<&Vec<Vec3>>;
}

struct SceneObject<V, I> {
    mesh: Mesh<V, I>,
    shader: Program, // Each object can have its own shader for now
    transform: Mat4, // Each object has its own position/scale/rotation
    aabb: Option<AABB>, // Optional bounding box
    // For instancing, we might need more data later, but start simple.
    is_instanced: bool,
    num_instances: gl::types::GLsizei,
    instance_translations: Option<Vec<Vec3>>,
}

// Implement Renderable for SceneObject
// We assume draw/draw_instanced in mesh_loader_generics work generically
// based on the Mesh<V, I> type provided.
impl<V, I> Renderable for SceneObject<V, I>
where
    // Add bounds if draw/draw_instanced require them, e.g. V: VertexData, I: IndexType
    // For now, assume Mesh<V, I> is enough context for draw functions.
    Mesh<V, I>: 'static // Example bound, might need adjustment
{
    fn draw(&self, view: &Mat4, projection: &Mat4) {
        self.shader.use_program();
        self.shader.set_mat4("projection", projection);
        self.shader.set_mat4("view", view);
        self.shader.set_mat4("model", &self.transform); // Use the object's specific transform

        if self.is_instanced {
            // Call generic instanced draw function
            draw_instanced(&self.mesh, self.num_instances);
        } else {
            // Call generic non-instanced draw function
            draw(&self.mesh); // Make sure `draw` exists and accepts &Mesh<V, I>
        }
    }

    fn get_instance_translations(&self) -> Option<&Vec<Vec3>> {
        self.instance_translations.as_ref()
    }
}

// Application struct holds all the state
pub struct Application {
    // Window and GL context/surface related fields
    window: Window,
    surface: Surface<WindowSurface>,
    context: PossiblyCurrentContext,
    display: Display, // Keep display for GL loading etc.
    config: Config, // Keep config

    // --- Core Engine State ---
    camera: Camera,
    projection: Mat4,

    // --- Scene Management ---
    // Holds all renderable objects, including instanced groups
    scene_objects: Vec<Box<dyn Renderable>>,
    skybox_shader: Program,
    skybox: Skybox, // Skybox struct holds its own mesh/shader logic

    // --- Timing ---
    timer: Instant,
    delta_time: f32,
}

// Implementation block for Application
impl Application {
    // Constructor - moves setup code here
    pub fn new(event_loop: &EventLoop<()>) -> Result<Self, Box<dyn Error>> {
        // build a window
        let window_builder = WindowBuilder::new()
            .with_title("Dying Light 3")
            .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
            .with_theme(Some(Theme::Dark));
        // create config template with VSync
        let conf_template_builder = ConfigTemplateBuilder::new()
            .with_swap_interval(Some(0), Some(1));
        // pick the first config
        let (window, gl_config) = DisplayBuilder::new()
            .with_window_builder(Some(window_builder))
            .build(event_loop, conf_template_builder, |mut configs| {
                configs.next().expect("No suitable GL config found!")
            })?;
        let window = window.ok_or("Failed to create window")?;
        // configure the window
        window.set_cursor_visible(false);
        window.set_cursor_grab(winit::window::CursorGrabMode::Confined)?;

        let (width, height): (u32, u32) = window.inner_size().into();
        let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            window.raw_window_handle(),
            NonZeroU32::new(width).ok_or("Window width is zero")?,
            NonZeroU32::new(height).ok_or("Window height is zero")?,
        );

        let gl_display = gl_config.display(); // Get display before config is moved

        let surface = unsafe { gl_display.create_window_surface(&gl_config, &attrs)? };
        let context_attributes = ContextAttributesBuilder::new().build(Some(window.raw_window_handle()));
        let context = Some(unsafe {
            gl_display.create_context(&gl_config, &context_attributes)?
        }).ok_or("Failed to create GL context intermediate")?
          .make_current(&surface)?;
        // enable VSync
        let swap_interval = SwapInterval::Wait(NonZeroU32::new(1).ok_or("1 is zero?!")?);
        surface.set_swap_interval(&context, swap_interval)?;

        // Load GL functions using the display
        gl::load_with(|symbol| {
            gl_display.get_proc_address(&ctx::c_string(symbol).as_c_str())
        });

        // Resource Loading
        let shader = Program::new(DEFAULT_VS, DEFAULT_FS, None);
        let skybox_shader = Program::new(SKYBOX_VS, SKYBOX_FS, None);
        let cube_object: ObjResult<Obj<TexturedVertex, u16>> = load_obj(&MODEL_SRC[..]);
        let cube_object = cube_object?;
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        println!("Manifest directory: {}", manifest_dir);
        let texture_path = Path::new(manifest_dir)
            .join("resources")
            .join("textures")
            .join("texture9.jpg");
        let texture = Texture::new(texture_path.to_str().expect("Failed to convert path to string"));

        // Instance Data Calculation
        let mut translations: Vec<Vec3> = vec![];
        let offset = 1.0f32;
        for y in (-10..10_i32).step_by(2) {
            for x in (-10..10_i32).step_by(2) {
                let mut translation = vec3(0.0, 0.0, 0.0_f32);
                translation.x = x as f32 + offset;
                translation.z = y as f32 + offset;
                translations.push(translation);
            }
        }

        let mut cube_aabbs: Vec<AABB> = vec![];
        let scale = 2.0f32;
        for translation in &translations {
            cube_aabbs.push(AABB::new(translation, scale));
        }

        let num_instances = translations.len() as gl::types::GLsizei;

        // Create Mesh
        let mesh = Mesh::<TexturedVertex, u16>::new(cube_object, Some(texture), Some(&translations));

        // --- Load Skybox Resources ---
        let skybox = Skybox::new();

        // Initial GL State
        unsafe {
            Enable(DEPTH_TEST);
            DepthFunc(LESS);
            // PolygonMode(FRONT_AND_BACK, LINE);
            gl::Enable(TEXTURE_CUBE_MAP_SEAMLESS);
        }

        // Initial App State
        let fov = f32::from_bits(width) / f32::from_bits(height);
        let camera = Camera::new();
        let projection = perspective(fov, 50.0f32.to_radians(), 0.1, 100.0);
        let timer = Instant::now();
        let delta_time = 0.0_f32;

        // Return the constructed Application state
        Ok(Self {
            window,
            surface,
            context,
            display,
            config,
            camera,
            projection,
            scene_objects: vec![Box::new(SceneObject {
                mesh,
                shader,
                transform: glm::identity(),
                aabb: None,
                is_instanced: true,
                num_instances,
                instance_translations: Some(translations.clone()),
            })],
            skybox_shader,
            skybox,
            timer,
            delta_time,
        })
    }

    // Main loop runner
    pub fn run(mut self, event_loop: EventLoop<()>) -> Result<(), Box<dyn Error>> {
        // Set control flow policy
        event_loop.set_control_flow(ControlFlow::Poll);

        // Run the event loop
        event_loop.run(move |ev, elwt| {
            match ev {
                Event::WindowEvent { event: window_ev, .. } => {
                    match window_ev {
                        WindowEvent::RedrawRequested => {
                            ctx::clear_color(0.0, 0.0, 0.0, 1.0);
                            ctx::clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                            let view = self.camera.get_view();
                            let projection = self.projection; // Use projection matrix

                            // --- Draw Main Scene --- 
                            for scene_object in &self.scene_objects {
                                scene_object.draw(&view, &projection);
                            }

                            self.skybox_shader.use_program();
                            // Removal of translation from view matrix for skybox happens in shader
                            self.skybox_shader.set_mat4("view", &view);
                            self.skybox_shader.set_mat4("projection", &projection);

                            self.skybox.draw();

                            // --- Finish Frame --- 
                            self.delta_time = self.timer.elapsed().as_secs_f32();
                            self.timer = Instant::now();
                            // Manual error handling for swap_buffers
                            if let Err(e) = self.surface.swap_buffers(&self.context) {
                                eprintln!("Failed to swap buffers: {}", e);
                                elwt.exit(); // Exit loop on swap fail
                            }
                        },
                        WindowEvent::Resized(dimensions) => {
                            // manual resizing is required only on wayland
                            if cfg!(target_os = "linux") {
                                // Manual error handling for NonZeroU32 and resize
                                match (NonZeroU32::new(dimensions.width), NonZeroU32::new(dimensions.height)) {
                                    (Some(width), Some(height)) => {
                                        self.surface.resize(&self.context, width, height);
                                    }
                                    _ => {
                                        eprintln!("Invalid resize dimensions received: {}x{}", dimensions.width, dimensions.height);
                                    }
                                }
                            }
                            // update glViewport and projection matrix respectively
                            unsafe {
                                gl::Viewport(0, 0, dimensions.width as gl::types::GLsizei, dimensions.height as gl::types::GLsizei);
                            }
                            let fov = f32::from_bits(dimensions.width) / f32::from_bits(dimensions.height);
                            self.projection = perspective(fov, 50.0f32.to_radians(), 0.1, 100.0);
                        },
                        WindowEvent::CloseRequested => elwt.exit(),
                        _ => ()
                    }
                },
                Event::DeviceEvent { event: device_event, .. } => match device_event {
                    DeviceEvent::Key(winit::event::RawKeyEvent { physical_key, state }) => {
                        match physical_key {
                            PhysicalKey::Code(KeyCode::KeyW) =>
                                self.camera.dispatch_state(Action::MoveForward, state.is_pressed()),
                            PhysicalKey::Code(KeyCode::KeyA) =>
                                self.camera.dispatch_state(Action::MoveLeft, state.is_pressed()),
                            PhysicalKey::Code(KeyCode::KeyS) =>
                                self.camera.dispatch_state(Action::MoveBackwards, state.is_pressed()),
                            PhysicalKey::Code(KeyCode::KeyD) =>
                                self.camera.dispatch_state(Action::MoveRight, state.is_pressed()),
                            PhysicalKey::Code(KeyCode::ControlLeft) =>
                                self.camera.dispatch_state(Action::Sprint, state.is_pressed()),
                            PhysicalKey::Code(KeyCode::Space) =>
                                self.camera.dispatch_state(Action::MoveUp, state.is_pressed()),
                            PhysicalKey::Code(KeyCode::ShiftLeft) =>
                                self.camera.dispatch_state(Action::MoveDown, state.is_pressed()),
                            PhysicalKey::Code(KeyCode::Escape) => elwt.exit(),
                            PhysicalKey::Code(KeyCode::Tab) =>
                                self.camera.dispatch_state(Action::FlipGravity, state.is_pressed()),
                            _ => ()
                        }
                    },
                    DeviceEvent::MouseMotion { delta } => {
                        self.camera.process_mouse_movement(delta.0 as f32, -delta.1 as f32, true);
                    }
                    _ => ()
                }
                Event::AboutToWait => {
                    self.window.request_redraw();
                },
                _ => ()
            }
            self.camera.process_movement(self.delta_time, &cube_aabbs);
        })?;

        Ok(())
    }
}