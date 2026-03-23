use crate::ecs::components::{Camera, Position};
use crate::ecs::resources::{
    MouseDelta, PressedKeys, Projection, Time, TimeDelta, Triangle, TriangleProgram, View,
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
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{DeviceEvent, DeviceId, StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow},
    window::{Theme, Window, WindowAttributes, WindowId},
};

const WINDOW_WIDTH: u32 = 1600;
const WINDOW_HEIGHT: u32 = 900;
const FOV_Y: f32 = 70.0_f32.to_radians();

struct WindowContext {
    window_id: WindowId,
    window: Window,
    surface: Surface<WindowSurface>,
    context: PossiblyCurrentContext,
}

impl WindowContext {
    fn deconstruct(
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
    world: World,
    timer: Instant,
}

impl Application {
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

impl Game for Application {
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
        window.set_cursor_grab(winit::window::CursorGrabMode::None)?;

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

    fn prep_game_world(&mut self) {
        // add projection & view matrix
        {
            let (width, height): (u32, u32) = self.get_window().inner_size().into();
            let aspect_ratio = width as f32 / height as f32;
            let projection_mat =
                Projection::from(Mat4::perspective_rh_gl(FOV_Y, aspect_ratio, 0.0, 100.0));
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
            let vertex_shader: &str = include_str!("shaders/Triangle.vert");
            let fragment_shader: &str = include_str!("shaders/Triangle.frag");

            let v_desc = ShaderDesc::new("v_test", ShaderStage::Vertex);
            let v_shader = compile_shader(vertex_shader, v_desc).unwrap();

            let f_desc = ShaderDesc::new("f_test", ShaderStage::Fragment);
            let f_shader = compile_shader(fragment_shader, f_desc).unwrap();

            let program = link_program(vec![v_shader, f_shader]);

            let prog = program.unwrap();
            self.world.register_resourse(TriangleProgram(prog));
        }
        // add triangle mesh
        {
            struct MyVertex {
                position: [f32; 3],
                color: [f32; 3],
            }

            impl Vertex for MyVertex {
                fn attributes() -> impl IntoIterator<Item = VertexAttrib> {
                    let stride = std::mem::size_of::<MyVertex>();
                    let normalized = false;

                    [
                        VertexAttrib {
                            index: 0,
                            size: 3,
                            kind: AttributeKind::Float,
                            normalized,
                            stride,
                            offset: std::mem::offset_of!(MyVertex, position),
                        },
                        VertexAttrib {
                            index: 1,
                            size: 3,
                            kind: AttributeKind::Float,
                            normalized,
                            stride,
                            offset: std::mem::offset_of!(MyVertex, color),
                        },
                    ]
                }
            }

            let vertices = vec![
                MyVertex {
                    position: [-0.5, -0.5, 0.0],
                    color: [1.0, 0.4, 0.6],
                },
                MyVertex {
                    position: [0.5, -0.5, 0.0],
                    color: [0.6, 1.0, 0.4],
                },
                MyVertex {
                    position: [0.5, 0.5, 0.0],
                    color: [0.1, 0.4, 1.0],
                },
                MyVertex {
                    position: [-0.5, 0.5, 0.0],
                    color: [0.1, 3.0, 0.4],
                },
            ];
            let v_desc = BufferObjDesc::new(BufferObjKind::Vertex, BufferUsage::StaticDraw);

            let elements = vec![0_u32, 1, 2, 3, 2, 0];
            let e_desc = BufferObjDesc::new(BufferObjKind::Element, BufferUsage::StaticDraw);

            let triangle = GlMesh::new::<MyVertex>(vertices, v_desc, Primitive::Triangles)
                .unwrap()
                .with_element_buffer(elements, e_desc)
                .unwrap();
            self.world.register_resourse(Triangle(triangle));
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

    fn drop_game_world(&mut self) {
        tracing::warn!("drop_game_world is not yet implemented");
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

            match self.make_window(ev_loop) {
                Ok(window_id) => tracing::info!("created primary window: {window_id:?}"),
                Err(e) => tracing::error!("failed to create primary window: {e}"),
            }

            self.prep_game_world();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        use window_ev::*;

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
                let world = &mut self.world;
                let mut mouse_delta = query_resource!(world, mut MouseDelta);
                *mouse_delta += delta;
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        let dt = self.timer.elapsed().as_secs_f64();
        self.timer = Instant::now();

        let world = &mut self.world;
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
        tracing::info!("application exiting");
    }
}

mod window_ev {
    use super::Application;
    use super::FOV_Y;
    use crate::ecs::resources::Triangle;
    use crate::ecs::resources::TriangleProgram;
    use crate::ecs::resources::View;
    use crate::ecs::resources::{MouseDelta, PressedKeys, Projection, Time};
    use crate::ecs::systems::camera::s_camera_control;
    use crate::ecs::systems::camera::s_camera_view;
    use engine::query_resource;
    use engine::render::prelude::Renderable;
    use glam::Mat4;
    use glutin::{context::PossiblyCurrentGlContext, prelude::GlSurface};
    use std::num::NonZeroU32;
    use winit::{
        dpi::PhysicalSize,
        event::{ElementState, KeyEvent},
        event_loop::ActiveEventLoop,
        keyboard::{KeyCode, PhysicalKey},
        window::WindowId,
    };

    pub fn resized(app: &mut Application, window_id: WindowId, physical_size: PhysicalSize<u32>) {
        let (_, surface, context) = app.primary_window.as_mut().unwrap().deconstruct();

        if let (Some(width), Some(height)) = (
            NonZeroU32::new(physical_size.width),
            NonZeroU32::new(physical_size.height),
        ) {
            surface.resize(context, width, height);
            let aspect_ratio = physical_size.width as f32 / physical_size.height as f32;

            let world = &mut app.world;
            let mut projection_mat = query_resource!(world, mut Projection);
            *projection_mat = Mat4::perspective_rh_gl(FOV_Y, aspect_ratio, 0.1, 100.0).into();

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

    pub fn close_requested(app: &mut Application, ev_loop: &ActiveEventLoop, window_id: WindowId) {
        tracing::info!("close requested for window {window_id:?}");

        if app.primary_window.as_ref().unwrap().window_id == window_id {
            tracing::info!("primary window closed");
            ev_loop.exit();
        }
    }

    pub fn keyboard_input(app: &mut Application, ev_loop: &ActiveEventLoop, event: KeyEvent) {
        if let PhysicalKey::Code(key_code) = event.physical_key {
            let world = &mut app.world;
            let mut pressed_keys = query_resource!(world, mut PressedKeys);

            match event.state {
                ElementState::Pressed => {
                    if !event.repeat {
                        let _ = pressed_keys.insert(key_code);
                    }
                }
                ElementState::Released => {
                    let _ = pressed_keys.remove(&key_code);
                }
            }
        }
        if event.physical_key == PhysicalKey::Code(KeyCode::Escape)
            && event.state == ElementState::Pressed
        {
            ev_loop.exit();
        }
    }

    pub fn redraw_requested(app: &mut Application, window_id: WindowId) {
        let (window, surface, context) = app.primary_window.as_mut().unwrap().deconstruct();

        if !context.is_current() {
            if let Err(e) = context.make_current(surface) {
                tracing::error!("failed to make context current for redraw: {e}");
                return;
            }
        }

        let world = &mut app.world;

        s_camera_control(world);
        s_camera_view(world);

        let (prog, triangle, time, view, proj) =
            query_resource!(world, TriangleProgram, Triangle, Time, View, Projection);

        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        prog.0.bind();
        prog.0.uniform_value(*view);
        prog.0.uniform_value(*time);
        prog.0.uniform_value(*proj);
        triangle.0.draw();

        window.pre_present_notify();

        if let Err(e) = surface.swap_buffers(context) {
            tracing::error!("failed to swap buffers for window {window_id:?}: {e}");
        }
    }

    pub fn focused(app: &mut Application, window_id: WindowId, focused: bool) {
        tracing::info!(
            "window {window_id:?} focus changed: {}",
            if focused { "active" } else { "not active" }
        );
        if !focused {
            let world = &mut app.world;
            let (mut pressed_keys, mut delta) =
                query_resource!(world, mut PressedKeys, mut MouseDelta);
            pressed_keys.clear();
            delta.clear();
        }
    }
}
