use engine::{ecs::World, game::Game, query_resource};
use glam::Mat4;
use glutin::{
    config::ConfigTemplateBuilder,
    context::{ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext},
    display::{GetGlDisplay, GlDisplay},
    surface::{GlSurface, Surface, SurfaceAttributesBuilder, SwapInterval, WindowSurface},
};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasWindowHandle;
use std::{collections::HashSet, ffi::CString, mem::forget, num::NonZeroU32, str::FromStr, time::Instant};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{DeviceEvent, DeviceId, StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow},
    window::{Theme, Window, WindowAttributes, WindowId},
};

use crate::ecs::resources::{MouseDelta, PressedKeys, Projection};

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

        let context_attributes = ContextAttributesBuilder::new().build(Some(raw_window_handle));

        let not_current_context =
            unsafe { gl_display.create_context(&config, &context_attributes)? };
        let context = not_current_context.make_current(&surface)?;

        gl::load_with(|symbol| {
            gl_display.get_proc_address(CString::from_str(symbol).unwrap().as_c_str())
        });

        surface.set_swap_interval(&context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))?;

        window.set_cursor_visible(false);
        window.set_cursor_grab(winit::window::CursorGrabMode::Confined)?;

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
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
        // add projection matrix
        {
            let (width, height): (u32, u32) = self.get_window().inner_size().into();
            let aspect_ratio = width as f32 / height as f32;
            let projection_mat =
                Projection::from(Mat4::perspective_rh_gl(FOV_Y, aspect_ratio, 0.0, 100.0));
            self.world.register_resourse(projection_mat);
        }
        // add input state 
        {
            let pressed_keys = PressedKeys::from(HashSet::new());
            let mouse_delta = MouseDelta::new(0.0, 0.0);
            self.world.register_resourse(pressed_keys);
            self.world.register_resourse(mouse_delta);
        }

        tracing::warn!("prep_game_world is not yet implemented");
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
        let world = &mut self.world;
        {
            let (mut pressed_keys, mut mouse_delta) =
                query_resource!(world, mut PressedKeys, mut MouseDelta);
            pressed_keys.clear();
            mouse_delta.clear();
        }

        let dt = self.timer.elapsed().as_secs_f32();
        //self.world.delta_time = delta;
        self.timer = Instant::now();

        self.get_window().request_redraw();
    }

    fn exiting(&mut self, _: &ActiveEventLoop) {
        tracing::info!("application exiting");
    }
}

mod window_ev {
    use super::Application;
    use super::FOV_Y;
    use crate::ecs::resources::{MouseDelta, PressedKeys, Projection};
    use engine::query_resource;
    use glam::Mat4;
    use glutin::context::PossiblyCurrentGlContext;
    use glutin::prelude::GlSurface;
    use std::num::NonZeroU32;
    use winit::dpi::PhysicalSize;
    use winit::event::{ElementState, KeyEvent};
    use winit::event_loop::ActiveEventLoop;
    use winit::keyboard::{KeyCode, PhysicalKey};
    use winit::window::WindowId;

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

        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

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
