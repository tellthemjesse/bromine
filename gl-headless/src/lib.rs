use glutin::{
    config::ConfigTemplateBuilder,
    context::{ContextAttributesBuilder, PossiblyCurrentContext},
    display::GetGlDisplay,
    prelude::{GlDisplay, NotCurrentGlContext, PossiblyCurrentGlContext},
    surface::{Surface, SurfaceAttributesBuilder, WindowSurface},
};
use glutin_winit::DisplayBuilder;
use std::{ffi::CString, num::NonZeroU32};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    platform::windows::{EventLoopBuilderExtWindows, WindowExtWindows},
    window::{WindowAttributes, WindowId},
};

pub struct GlHeadless {
    run_fn: fn(),
}

impl GlHeadless {
    /// OpenGL functions are loaded once an event loop is active(e.g., by using [`run_once`] method).
    /// Until then, this structure just wraps a closure
    pub const fn new(run_fn: fn()) -> Self {
        Self { run_fn }
    }

    /// Creates and runs an event loop that exits after closure execution.
    /// That means no buffers are swapped
    pub fn run_once(&mut self) -> anyhow::Result<()> {
        let event_loop = EventLoop::builder().with_any_thread(true).build()?;
        event_loop.run_app(self)?;
        
        Ok(())
    }
}

impl ApplicationHandler for GlHeadless {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let context = GlContext::new(&event_loop).unwrap();

        if !context.is_current() { context.make_current(); }

        (self.run_fn)();
        event_loop.exit();
    }
    fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, _: WindowEvent) {}
}

struct GlContext {
    context: PossiblyCurrentContext,
    surface: Surface<WindowSurface>,
}

impl GlContext {
   fn new(event_loop: &ActiveEventLoop) -> anyhow::Result<Self> {
        let window_attr = WindowAttributes::default()
            .with_title("GlHeadless window")
            .with_visible(false);

        let template_builder = ConfigTemplateBuilder::default();
        let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attr));

        let (window, config) = display_builder
            .build(event_loop, template_builder, |mut configs| {
                configs.next().unwrap()
            })
            .ok()
            .unwrap();

        let window = window.unwrap();
        let (w, h) = window.inner_size().into();

        let window_handle = unsafe { window.window_handle_any_thread()? };
        let rwh = window_handle.as_raw();

        let surface_attr = unsafe {
            SurfaceAttributesBuilder::<WindowSurface>::new().build(
                rwh,
                NonZeroU32::new_unchecked(w),
                NonZeroU32::new_unchecked(h),
            )
        };

        let context_attr = ContextAttributesBuilder::new().build(Some(rwh));

        let display = config.display();

        let not_current_ctx = unsafe { display.create_context(&config, &context_attr)? };

        let surface = unsafe { display.create_window_surface(&config, &surface_attr)? };

        let context = not_current_ctx.make_current(&surface)?;

        gl::load_with(|addr| display.get_proc_address(&CString::new(addr).unwrap()));

        Ok(GlContext { context, surface })
    }

    fn is_current(&self) -> bool {
        self.context.is_current()
    }

    fn make_current(&self) {
       self.context.make_current(&self.surface).unwrap()
    }
}
