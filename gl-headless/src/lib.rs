#![allow(dead_code)]

use glutin::config::ConfigTemplateBuilder;
use glutin::context::{
    ContextApi, ContextAttributesBuilder, NotCurrentContext, PossiblyCurrentContext, Version,
};
use glutin::display::{Display, GetGlDisplay};
use glutin::prelude::*;
use glutin::surface::{Surface, SurfaceAttributesBuilder, WindowSurface};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasWindowHandle;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::window::{Window, WindowId};

use std::ffi::CString;
use std::num::NonZeroU32;
use std::sync::{Once, RwLock, mpsc::Sender};
use std::thread;

// There is a Wayland version of this extension trait but the X11 version also works on Wayland
#[cfg(windows)]
use winit::platform::windows::EventLoopBuilderExtWindows;
#[cfg(unix)]
use winit::platform::x11::EventLoopBuilderExtX11;

type DisplayRequest = Sender<(Window, NotCurrentContext, Surface<WindowSurface>, Display)>;

struct Tests {}

impl ApplicationHandler<DisplayRequest> for Tests {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        _event: WindowEvent,
    ) {
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, request: DisplayRequest) {
        let window_attributes = Window::default_attributes().with_visible(false);
        let config_template_builder = ConfigTemplateBuilder::new();
        let (window, gl_config) = DisplayBuilder::new()
            .with_window_attributes(Some(window_attributes))
            .build(event_loop, config_template_builder, |mut configs| {
                configs.next().unwrap()
            })
            .unwrap();
        let window = window.unwrap();
        let raw_window_handle = window.window_handle().unwrap().as_raw();

        let api = ContextApi::OpenGl(Some(Version::new(4, 6)));
        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(api)
            .build(Some(raw_window_handle));

        let dispay = gl_config.display();

        let not_current_gl_context = unsafe {
            dispay
                .create_context(&gl_config, &context_attributes)
                .unwrap()
        };

        let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            raw_window_handle,
            NonZeroU32::new(800).unwrap(),
            NonZeroU32::new(600).unwrap(),
        );

        let surface = unsafe { dispay.create_window_surface(&gl_config, &attrs).unwrap() };

        request
            .send((window, not_current_gl_context, surface, dispay))
            .unwrap();
    }
}

pub struct WindowDisplay {
    pub context_surface: ContextSurfacePair,
    pub window: Window,
    pub display: Display,
}

impl WindowDisplay {
    pub fn load_gl(&self) {
        gl::load_with(|addr| self.display.get_proc_address(&CString::new(addr).unwrap()));
    }
}

pub struct ContextSurfacePair {
    context: PossiblyCurrentContext,
    surface: Surface<WindowSurface>,
}

impl ContextSurfacePair {
    pub fn new(context: PossiblyCurrentContext, surface: Surface<WindowSurface>) -> Self {
        Self { context, surface }
    }
}

/// Builds a display
pub fn build_display() -> WindowDisplay {
    static EVENT_LOOP_PROXY: RwLock<Option<EventLoopProxy<DisplayRequest>>> = RwLock::new(None);
    static INIT_EVENT_LOOP: Once = Once::new();

    // initialize event loop in a separate thread and store a proxy
    INIT_EVENT_LOOP.call_once(|| {
        let (sender, receiver) = std::sync::mpsc::channel();

        thread::Builder::new()
            .name("event_loop".into())
            .spawn(move || {
                let event_loop_res = if cfg!(unix) || cfg!(windows) {
                    EventLoop::with_user_event().with_any_thread(true).build()
                } else {
                    EventLoop::with_user_event().build()
                };
                let event_loop = event_loop_res.expect("event loop building");

                sender.send(event_loop.create_proxy()).unwrap();

                let mut app = Tests {};
                event_loop.run_app(&mut app).unwrap();
            })
            .unwrap();

        *EVENT_LOOP_PROXY.write().unwrap() = Some(receiver.recv().unwrap());
    });

    let (sender, receiver) = std::sync::mpsc::channel();

    // request event loop create display building pieces and send them back
    EVENT_LOOP_PROXY
        .read()
        .unwrap()
        .as_ref()
        .unwrap()
        .send_event(sender)
        .unwrap();

    // block until required display building pieces are received
    let (window, not_current_gl_context, surface, display) = receiver.recv().unwrap();

    // now use our surface to make our context current and finally create our display
    let current_context = not_current_gl_context.make_current(&surface).unwrap();

    WindowDisplay {
        context_surface: ContextSurfacePair::new(current_context, surface),
        window,
        display,
    }
}
