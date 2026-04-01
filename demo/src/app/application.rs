use super::window_event::*;
use crate::ecs::components::{Camera, Model, Position};
use crate::ecs::resources::{
    MouseDelta, PressedKeys, Projection, SceneProgram, Time, TimeDelta, View,
};
use engine::render::prelude::*;
use engine::{ecs::World, query_resource, window::game::Game};
use glam::{Mat4, Vec3};
use glutin::context::{ContextApi, Version};
use glutin::{
    config::ConfigTemplateBuilder,
    context::{ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext},
    display::{GetGlDisplay, GlDisplay},
    surface::{GlSurface, Surface, SurfaceAttributesBuilder, SwapInterval, WindowSurface},
};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasWindowHandle;
use std::{collections::HashSet, ffi::CString, num::NonZeroU32, time::Instant};
use winit::event::MouseScrollDelta;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{DeviceEvent, DeviceId, StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow},
    window::{Theme, Window, WindowAttributes, WindowId},
};

pub(super) const WINDOW_WIDTH: u32 = 1600;
pub(super) const WINDOW_HEIGHT: u32 = 900;
const FOV_MIN: f32 = 9.0_f32.to_radians();
const FOV_MAX: f32 = 89.0_f32.to_radians();
pub(super) static mut FOV_Y: f32 = 70.0_f32.to_radians();

pub(super) struct WindowContext {
    pub window_id: WindowId,
    pub window: Window,
    pub surface: Surface<WindowSurface>,
    pub context: PossiblyCurrentContext,
}

impl WindowContext {
    pub(super) fn values(&self) -> (&Window, &Surface<WindowSurface>, &PossiblyCurrentContext) {
        (&self.window, &self.surface, &self.context)
    }
}

pub struct ApplicationDemo {
    pub(super) primary_window: Option<WindowContext>,
    pub(super) world: World,
    pub(super) timer: Instant,
}

impl ApplicationDemo {
    pub fn new() -> Self {
        Self {
            primary_window: None,
            world: World::new(),
            timer: Instant::now(),
        }
    }
    /// Panics if [`WindowContext`] is None
    fn get_window(&self) -> &Window {
        &self.primary_window.as_ref().unwrap().window
    }
}

impl Game for ApplicationDemo {
    fn make_window(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<WindowId> {
        let conf_template_builder =
            ConfigTemplateBuilder::new().with_swap_interval(Some(0), Some(1));

        let window_attributes = WindowAttributes::default()
            .with_title("DL3")
            .with_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
            .with_theme(Some(Theme::Dark));

        let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attributes));

        // TODO: HANDLE ERROR PROPERLY
        let (built_window, config) = display_builder
            .build(event_loop, conf_template_builder, |mut configs| {
                configs.next().unwrap()
            })
            .ok()
            .unwrap();

        let window = built_window.unwrap();
        window.set_cursor_visible(false);
        window
            .set_cursor_grab(winit::window::CursorGrabMode::Confined)
            .unwrap();

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

        let context_attributes = ContextAttributesBuilder::new()
            .with_debug(true)
            .with_context_api(ContextApi::OpenGl(Some(Version::new(4, 6))))
            .build(Some(raw_window_handle));

        let not_current_context =
            unsafe { gl_display.create_context(&config, &context_attributes)? };
        let context = not_current_context.make_current(&surface)?;

        gl::load_with(|s| gl_display.get_proc_address(CString::new(s).unwrap().as_c_str()));

        surface.set_swap_interval(&context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))?;

        window.set_cursor_visible(false);
        window.set_cursor_grab(winit::window::CursorGrabMode::Confined)?;

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);

            gl::Enable(gl::DEBUG_OUTPUT);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);

            gl::DebugMessageCallback(Some(debug_callback), std::ptr::null());

            gl::DebugMessageControl(
                gl::DONT_CARE,
                gl::DONT_CARE,
                gl::DONT_CARE,
                0,
                std::ptr::null(),
                gl::TRUE,
            );
        }

        self.primary_window = Some(WindowContext {
            window_id,
            window,
            surface,
            context,
        });

        Ok(window_id)
    }

    fn init_world(&mut self) {
        // add projection & view matrix
        {
            let (width, height): (u32, u32) = self.get_window().inner_size().into();
            let aspect_ratio = width as f32 / height as f32;
            let projection_mat = unsafe {
                Projection::from(Mat4::perspective_rh_gl(FOV_Y, aspect_ratio, 0.0, 100.0))
            };
            self.world.register_resourse(projection_mat);

            let view_mat = View::from(Mat4::IDENTITY);
            self.world.register_resourse(view_mat);
        }
        // add window state
        {
            let pressed_keys = PressedKeys::from(HashSet::new());
            let mouse_delta = MouseDelta::new(0.0, 0.0);
            let time_delta = TimeDelta::from(0.0);
            let time = Time::from(0.0);
            self.world.register_resourse(pressed_keys);
            self.world.register_resourse(mouse_delta);
            self.world.register_resourse(time_delta);
            self.world.register_resourse(time);
        }
        // add shader resource
        {
            let vertex_shader: &str = include_str!("../shaders/Scene.vert");
            let fragment_shader: &str = include_str!("../shaders/Scene.frag");

            let v_desc = ShaderDesc::new("v_test", ShaderStage::Vertex);
            let v_shader = compile_shader(vertex_shader, v_desc).unwrap();

            let f_desc = ShaderDesc::new("f_test", ShaderStage::Fragment);
            let f_shader = compile_shader(fragment_shader, f_desc).unwrap();

            let program = link_program(vec![v_shader, f_shader]);

            let prog = program.unwrap();
            self.world.register_resourse(SceneProgram(prog));
        }
        // add models mesh
        {
            let path = format!(
                "{}/../engine/resources/monkey/scene.gltf",
                std::env!("CARGO_MANIFEST_DIR")
            );

            for gl_model in GlftFile::get_models(path) {
                let entity = self.world.spawn_entity();
                self.world.register_component(entity, Model::from(gl_model));
            }
        }
        // add camera
        {
            let camera = Camera::default();
            let position = Position::from(Vec3::ZERO);

            let entity = self.world.spawn_entity();
            self.world.register_component(entity, camera);
            self.world.register_component(entity, position);
        }
    }

    fn deinit_world(&mut self) {
        tracing::warn!("drop_game_world is not yet implemented");
    }

    fn world(&self) -> &World {
        &self.world
    }

    fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
}

impl ApplicationHandler for ApplicationDemo {
    fn new_events(&mut self, _: &ActiveEventLoop, cause: StartCause) {
        if cause == StartCause::Init {
            tracing::info!("event loop initialized");
        }
        self.timer = Instant::now();
    }

    fn resumed(&mut self, ev_loop: &ActiveEventLoop) {
        if self.primary_window.is_none() {
            ev_loop.set_control_flow(ControlFlow::Poll);

            match self.make_window(ev_loop) {
                Ok(window_id) => tracing::info!("created primary window: {window_id:?}"),
                Err(e) => tracing::error!("failed to create primary window: {e}"),
            }

            self.init_world();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(dimensions) => resized(self, window_id, dimensions),
            WindowEvent::CloseRequested => close_requested(self, event_loop, window_id),
            WindowEvent::RedrawRequested => redraw_requested(self, window_id),
            WindowEvent::KeyboardInput { event, .. } => keyboard_input(self, event_loop, event),
            WindowEvent::Focused(is_focused) => focused(self, window_id, is_focused),
            _ => (),
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                let world = self.world_mut();
                let mut mouse_delta = query_resource!(world, mut MouseDelta);
                *mouse_delta += delta;
            }
            DeviceEvent::MouseWheel { delta } => {
                if let MouseScrollDelta::LineDelta(_, vertical) = delta {
                    let size = self.get_window().inner_size();
                    let (w, h) = (size.width as f32, size.height as f32);

                    let aspect_ratio = w / h;

                    let world = self.world_mut();
                    let mut projection_mat = query_resource!(world, mut Projection);

                    *projection_mat = unsafe {
                        FOV_Y = (FOV_Y + (-vertical) * 0.1).clamp(FOV_MIN, FOV_MAX);
                        Mat4::perspective_rh_gl(FOV_Y, aspect_ratio, 0.1, 100.0).into()
                    };
                }
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        let dt = self.timer.elapsed().as_secs_f64();
        self.timer = Instant::now();

        let world = self.world_mut();
        {
            let (mut mouse_delta, mut time_delta, mut time) =
                query_resource!(world, mut MouseDelta, mut TimeDelta, mut Time);
            mouse_delta.clear();
            *time_delta = dt.into();
            *time += dt as f32;
        }

        self.get_window().request_redraw();
    }

    fn exiting(&mut self, _: &ActiveEventLoop) {
        self.deinit_world();
        tracing::info!("application exiting");
    }
}
