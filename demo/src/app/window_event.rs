use super::application::{ApplicationDemo, FOV_Y};
use crate::ecs::{
    resources::{MouseDelta, PressedKeys, Projection},
    systems::{
        camera::{s_camera_control, s_camera_view},
        render::s_render,
    },
};
use engine::{query_resource, window::game::Game};
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

pub fn resized(app: &mut ApplicationDemo, window_id: WindowId, physical_size: PhysicalSize<u32>) {
    let (_, surface, context) = app.primary_window.as_mut().unwrap().values();

    if let (Some(width), Some(height)) = (
        NonZeroU32::new(physical_size.width),
        NonZeroU32::new(physical_size.height),
    ) {
        surface.resize(context, width, height);
        let aspect_ratio = physical_size.width as f32 / physical_size.height as f32;

        let world = app.world_mut();
        let mut projection_mat = query_resource!(world, mut Projection);
        unsafe {
            *projection_mat = Mat4::perspective_rh_gl(FOV_Y, aspect_ratio, 0.1, 100.0).into();
        }

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

pub fn close_requested(app: &mut ApplicationDemo, ev_loop: &ActiveEventLoop, window_id: WindowId) {
    tracing::info!("close requested for window {window_id:?}");

    if app.primary_window.as_ref().unwrap().window_id == window_id {
        tracing::info!("primary window closed");
        ev_loop.exit();
    }
}

pub fn keyboard_input(app: &mut ApplicationDemo, ev_loop: &ActiveEventLoop, event: KeyEvent) {
    if let PhysicalKey::Code(key_code) = event.physical_key {
        let world = app.world_mut();
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

pub fn redraw_requested(app: &mut ApplicationDemo, window_id: WindowId) {
    let (window, surface, context) = app.primary_window.as_ref().unwrap().values();

    if !context.is_current() {
        if let Err(e) = context.make_current(surface) {
            tracing::error!("failed to make context current for redraw: {e}");
            return;
        }
    }

    let ref mut world = app.world;

    s_camera_control(world);
    s_camera_view(world);
    s_render(world);

    window.pre_present_notify();

    if let Err(e) = surface.swap_buffers(context) {
        tracing::error!("failed to swap buffers for window {window_id:?}: {e}");
    }
}

pub fn focused(app: &mut ApplicationDemo, window_id: WindowId, focused: bool) {
    tracing::info!(
        "window {window_id:?} focus changed: {}",
        if focused { "active" } else { "not active" }
    );
    if !focused {
        let world = app.world_mut();
        let (mut pressed_keys, mut delta) = query_resource!(world, mut PressedKeys, mut MouseDelta);
        pressed_keys.clear();
        delta.clear();
    }
}
