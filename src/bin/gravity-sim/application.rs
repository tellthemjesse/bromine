use std::num::NonZeroU32;
use std::path::Path;
use std::time::Instant;

use crate::constants::*;
use crate::ecs::entity::EntityConstructor;
use crate::graphics::{mesh::Mesh, mesh::Texture};
use crate::opengl_backend::shader::Program;
use crate::physics::spacetime_curvature::SpacetimeCurvature;
use crate::resources::manager::TypeErasedResourceMgr;
use crate::tags::{CameraTag, PhysicsTag};
use crate::tags::{DebugTag, MovingObjectTag, SpacetimeMeshTag};
use crate::types::{Collider3D, EcsWorld, Renderable, RigidBody, Transform};
use glutin::{
    config::ConfigTemplateBuilder,
    context::{ContextAttributesBuilder, PossiblyCurrentContext},
    display::GetGlDisplay,
    prelude::*,
    surface::{Surface, SurfaceAttributesBuilder, SwapInterval, WindowSurface},
};
use glutin_winit::DisplayBuilder;
use nalgebra_glm::{perspective, vec3, Vec3};
use obj::{load_obj, Position, TexturedVertex};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{DeviceEvent, DeviceId, StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow},
    raw_window_handle::HasWindowHandle,
    window::{Theme, Window, WindowAttributes, WindowId},
};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

const FRAGMENT_SHADER: &str = include_str!("../../../game_resources/shaders/default/shader.frag");
const VERTEX_SHADER: &str = include_str!("../../../game_resources/shaders/default/shader.vert");

const DFRAGMENT_SHADER: &str = include_str!("../../../game_resources/shaders/debug/shader.frag");
const DVERTEX_SHADER: &str = include_str!("../../../game_resources/shaders/debug/shader.vert");

const CURVE_FRAG: &str = include_str!("../../../game_resources/shaders/spacetimeCurve.frag");
const CURVE_VERT: &str = include_str!("../../../game_resources/shaders/spacetimeCurve.vert");

const TEXTURED_CUBE: &[u8] = include_bytes!("../../../game_resources/models/sphere.obj");
const POSITIONED_CUBE: &[u8] = include_bytes!("../../../game_resources/models/positioned_cube.obj");

pub struct WindowContext {
    window_id: WindowId,
    window: Window,
    surface: Surface<WindowSurface>,
    context: PossiblyCurrentContext,
}

impl WindowContext {
    pub fn deconstruct(
        &mut self,
    ) -> (
        &mut Window,
        &mut Surface<WindowSurface>,
        &mut PossiblyCurrentContext,
    ) {
        (&mut self.window, &mut self.surface, &mut self.context)
    }
}

pub struct Application {
    primary_window: Option<WindowContext>,
    world: EcsWorld,
    timer: Instant,
}

impl Application {
    pub fn new() -> Self {
        Application {
            primary_window: None,
            world: EcsWorld::new(),
            timer: Instant::now(),
        }
    }

    fn create_gl_window(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<WindowId, Box<dyn std::error::Error>> {
        let conf_template_builder =
            ConfigTemplateBuilder::new().with_swap_interval(Some(0), Some(1));

        let window_attributes = WindowAttributes::default()
            .with_title("DL3")
            .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
            .with_theme(Some(Theme::Dark));

        let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attributes));

        let (built_window, config) =
            display_builder.build(event_loop, conf_template_builder, |mut configs| {
                configs.next().unwrap()
            })?;

        let window = built_window.unwrap();

        let gl_display = config.display();

        let window_id = window.id();
        let (width, height): (u32, u32) = window.inner_size().into();

        let window_handle = window.window_handle()?;
        let raw_window_handle = window_handle.as_raw();

        let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            raw_window_handle,
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        );
        let surface = unsafe { gl_display.create_window_surface(&config, &attrs)? };

        let context_attributes = ContextAttributesBuilder::new().build(Some(raw_window_handle));

        let not_current_context =
            unsafe { gl_display.create_context(&config, &context_attributes)? };
        let context = not_current_context.make_current(&surface)?;

        gl::load_with(|symbol| {
            gl_display.get_proc_address(&crate::graphics::context::c_string(symbol).as_c_str())
        });

        surface.set_swap_interval(&context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))?;

        window.set_cursor_visible(false);
        window.set_cursor_grab(winit::window::CursorGrabMode::Confined)?;

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
        }

        let aspect_ratio = width as f32 / height as f32;
        self.world.projection_matrix =
            Some(perspective(aspect_ratio, 70.0f32.to_radians(), 0.1, 200.0));

        self.primary_window = Some(WindowContext {
            window_id,
            window,
            surface,
            context,
        });

        Ok(window_id)
    }

    fn init_world_state(&mut self) {
        tracing::info!("loading resources...");
        let resource_manager = &mut self.world.resource_manager;

        let resources_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("game_resources");
        let textures_path = resources_path.join("textures");

        let texture_path0 = textures_path.join("texture9.jpg");
        let texture0 = Texture::new_texture_2d(texture_path0.to_str().unwrap());
        let tex0_id = resource_manager.add_texture(texture0);

        let texture_path1 = textures_path.join("texture10.jpg");
        let texture1 = Texture::new_texture_2d(texture_path1.to_str().unwrap());
        let tex1_id = resource_manager.add_texture(texture1);

        let obj_data = load_obj::<TexturedVertex, &[u8], u16>(TEXTURED_CUBE).expect("Blablabla");
        let debug_cube_data = load_obj::<Position, &[u8], u16>(POSITIONED_CUBE).expect("Blablabla");

        let cube_mesh_obj = Mesh::<TexturedVertex, u16>::new(obj_data.clone(), None);
        let debug_cube_mesh = Mesh::<Position, u16>::new(debug_cube_data);

        let cube_mesh_id = resource_manager.add_mesh(cube_mesh_obj);
        let debug_cube_id = resource_manager.add_mesh(debug_cube_mesh);
        let curve_mesh = TypeErasedResourceMgr::create_curvature_grid(500, 250);
        let spacetime_mesh_id = resource_manager.add_mesh(curve_mesh);

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

        tracing::info!("creating initial entities...");

        let collider_camera = Collider3D::new(vec3(0.0, 0.0, 10.0), vec3(0.5, 0.5, 0.5));

        EntityConstructor::new(self.world.create_entity())
            .with(Transform::identity().with_position(vec3(2.0, 0.0, 10.0)))
            .with(CameraTag::default())
            .with(MovingObjectTag::default())
            .with(collider_camera)
            .with(RigidBody::new(0.0).with_restitution(0.0))
            .apply(&mut self.world);

        EntityConstructor::new(self.world.create_entity())
            .with(Transform::identity())
            .with(Renderable::new(debug_cube_id, dshader_id, None).with_visibility_flag(false))
            .with(DebugTag::default())
            .apply(&mut self.world);

        let base_mass = 1.2e6f32;

        let spacetime_curvature = SpacetimeCurvature {
            radius: 0.5,
            intensity: 1.0,
        };

        let center = Vec3::zeros();
        let scale = vec3(1.5, 1.5, 1.5);
        EntityConstructor::new(self.world.create_entity())
            .with(
                Transform::identity()
                    .with_position(center)
                    .with_scale(scale),
            )
            .with(Renderable::new(
                cube_mesh_id,
                default_shader_id,
                Some(tex0_id),
            ))
            .with(RigidBody::new(base_mass).with_restitution(0.5))
            .with(PhysicsTag::default())
            .with(MovingObjectTag::default())
            .with(Collider3D::new(center, scale))
            .with(spacetime_curvature)
            .apply(&mut self.world);

        let position = vec3(40.0, 0.0, -20.0);
        let scale = vec3(1.3, 1.3, 1.3);
        let velocity = get_tangent_velocity(position, center, base_mass);
        let velocity = vec3(velocity.x + 0.1, velocity.y + 0.1, velocity.z + 0.1);

        EntityConstructor::new(self.world.create_entity())
            .with(
                Transform::identity()
                    .with_position(position)
                    .with_scale(scale),
            )
            .with(Renderable::new(
                cube_mesh_id,
                default_shader_id,
                Some(tex0_id),
            ))
            .with(
                RigidBody::new(base_mass / 10.0)
                    .with_restitution(0.5)
                    .with_velocity(velocity),
            )
            .with(PhysicsTag::default())
            .with(MovingObjectTag::default())
            .with(Collider3D::new(position, scale))
            .with(spacetime_curvature)
            .apply(&mut self.world);

        let position = vec3(-40.0, 0.0, 20.0);
        let scale = vec3(1.0, 1.0, 1.0);
        let velocity = get_tangent_velocity(position, center, base_mass);

        EntityConstructor::new(self.world.create_entity())
            .with(
                Transform::identity()
                    .with_position(position)
                    .with_scale(scale),
            )
            .with(Renderable::new(
                cube_mesh_id,
                default_shader_id,
                Some(tex1_id),
            ))
            .with(
                RigidBody::new(base_mass / 10.0)
                    .with_restitution(0.5)
                    .with_velocity(velocity),
            )
            .with(PhysicsTag::default())
            .with(MovingObjectTag::default())
            .with(Collider3D::new(position, scale))
            .with(spacetime_curvature)
            .apply(&mut self.world);

        tracing::info!("initial entities successfully created");

        tracing::info!("{:?}", self.world);
    }
}

impl ApplicationHandler for Application {
    fn new_events(&mut self, _: &ActiveEventLoop, cause: StartCause) {
        if cause == StartCause::Init {
            tracing::info!("event loop initialized");
        }
        self.timer = Instant::now();
    }

    fn resumed(&mut self, ev_loop: &ActiveEventLoop) {
        if self.primary_window.is_none() {
            ev_loop.set_control_flow(ControlFlow::Poll);

            match self.create_gl_window(ev_loop) {
                Ok(window_id) => tracing::info!("created primary window: {window_id:?}"),
                Err(e) => tracing::error!("failed to create primary window: {e}"),
            }

            self.init_world_state();
        }
    }

    fn window_event(&mut self, ev_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::Resized(dimensions) => window_ev::resized(self, window_id, dimensions),
            WindowEvent::CloseRequested => window_ev::close_requested(self, ev_loop, window_id),
            WindowEvent::RedrawRequested => window_ev::redraw_requested(self, window_id),
            WindowEvent::KeyboardInput { event, .. } => {
                window_ev::keyboard_input(self, ev_loop, event)
            }
            WindowEvent::Focused(focused) => window_ev::focused(self, window_id, focused),
            _ => (),
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.world.input_state.mouse_delta.0 += delta.0 as f32;
                self.world.input_state.mouse_delta.1 += delta.1 as f32;
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        let delta = self.timer.elapsed().as_secs_f32();
        self.world.delta_time = delta;
        self.timer = Instant::now();

        self.world.input_state.clear_transient_state();
        self.primary_window
            .as_ref()
            .unwrap()
            .window
            .request_redraw();
    }

    fn exiting(&mut self, _: &ActiveEventLoop) {
        tracing::info!("application exiting");
    }
}

fn get_tangent_velocity(pos: Vec3, center: Vec3, mass: f32) -> Vec3 {
    let rel_pos = pos - center;
    let r = rel_pos.magnitude();
    let velocity_mag = (G_SIM * mass / r).sqrt();

    let tangent_dir = Vec3::new(-rel_pos.z, 0.0, rel_pos.x).normalize();

    tangent_dir * velocity_mag
}

mod window_ev {
    use super::Application;
    use crate::{camera, collision, physics, systems};
    use glutin::context::PossiblyCurrentGlContext;
    use glutin::prelude::GlSurface;
    use nalgebra_glm::perspective;
    use std::num::NonZeroU32;
    use winit::dpi::PhysicalSize;
    use winit::event::{ElementState, KeyEvent};
    use winit::event_loop::ActiveEventLoop;
    use winit::keyboard::{KeyCode, PhysicalKey};
    use winit::window::WindowId;

    /// Cosmetic type, that wraps mutable reference
    pub type AppRefMut<'a> = &'a mut Application;

    pub fn resized(app: AppRefMut, window_id: WindowId, physical_size: PhysicalSize<u32>) {
        let (_, surface, context) = app.primary_window.as_mut().unwrap().deconstruct();

        if let (Some(width), Some(height)) = (
            NonZeroU32::new(physical_size.width),
            NonZeroU32::new(physical_size.height),
        ) {
            surface.resize(context, width, height);
            let aspect_ratio = physical_size.width as f32 / physical_size.height as f32;

            app.world.projection_matrix =
                Some(perspective(aspect_ratio, 50.0f32.to_radians(), 0.1, 100.0));
            unsafe {
                gl::Viewport(
                    0,
                    0,
                    physical_size.width as i32,
                    physical_size.height as i32,
                );
            }
            tracing::info!("window {window_id:?} resized to {physical_size:?}");
        }
    }

    pub fn close_requested(app: AppRefMut, ev_loop: &ActiveEventLoop, window_id: WindowId) {
        tracing::info!("close requested for window {window_id:?}");

        if app.primary_window.as_ref().unwrap().window_id == window_id {
            tracing::info!("primary window closed");
            ev_loop.exit();
        }
    }

    pub fn keyboard_input(app: AppRefMut, ev_loop: &ActiveEventLoop, event: KeyEvent) {
        if let PhysicalKey::Code(key_code) = event.physical_key {
            match event.state {
                ElementState::Pressed => {
                    if !event.repeat {
                        let _ = app.world.input_state.pressed_keys.insert(key_code);
                    }
                }
                ElementState::Released => {
                    let _ = app.world.input_state.pressed_keys.remove(&key_code);
                }
            }
        }
        if event.physical_key == PhysicalKey::Code(KeyCode::Escape)
            && event.state == ElementState::Pressed
        {
            ev_loop.exit();
        }
    }

    pub fn redraw_requested(app: AppRefMut, window_id: WindowId) {
        let (window, surface, context) = app.primary_window.as_mut().unwrap().deconstruct();

        if !context.is_current() {
            if let Err(e) = context.make_current(surface) {
                tracing::error!("failed to make context current for redraw: {e}");
                return;
            }
        }

        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        physics::gravity_system::run(&mut app.world);
        physics::physics_system::run(&mut app.world);
        camera::camera_control_system::run(&mut app.world);
        camera::camera_system::run(&mut app.world);

        collision::collider_update_system::run(&mut app.world);
        collision::collision_detection_system::run(&mut app.world);
        collision::collision_handle_system::run(&mut app.world);

        systems::render::run(&app.world);
        systems::debug_render_system::run(&app.world);
        systems::curvature_render::run(&app.world);

        window.pre_present_notify();

        if let Err(e) = surface.swap_buffers(context) {
            tracing::error!("failed to swap buffers for window {window_id:?}: {e}");
        }
    }

    pub fn focused(app: AppRefMut, window_id: WindowId, focused: bool) {
        tracing::info!(
            "window {window_id:?} focus changed: {}",
            if focused { "active" } else { "not active" }
        );
        if !focused {
            app.world.input_state.pressed_keys.clear();
            app.world.input_state.clear_transient_state();
        }
    }
}
