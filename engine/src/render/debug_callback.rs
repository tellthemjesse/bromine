#![cfg(feature = "debug")]
//! Debug callback implementation. 
//! Available on crate feature `debug` 

use std::ffi::{CStr, c_void};

pub type GlDebugCallback = extern "system" fn(
    source: u32,
    error_type: u32,
    id: u32,
    severity: u32,
    length: i32,
    message: *const i8,
    user_param: *mut c_void,
);

pub extern "system" fn debug_callback(
    source: u32,
    error_type: u32,
    id: u32,
    severity: u32,
    _length: i32,
    message: *const i8,
    _user_param: *mut c_void,
) {
    unsafe {
        let msg = CStr::from_ptr(message).to_string_lossy();

        let source_str = match source {
            gl::DEBUG_SOURCE_API => "API",
            gl::DEBUG_SOURCE_WINDOW_SYSTEM => "window system",
            gl::DEBUG_SOURCE_SHADER_COMPILER => "shader compiler",
            gl::DEBUG_SOURCE_THIRD_PARTY => "third party",
            gl::DEBUG_SOURCE_APPLICATION => "application",
            gl::DEBUG_SOURCE_OTHER => "other",
            _ => "unknown",
        };

        let type_str = match error_type {
            gl::DEBUG_TYPE_ERROR => "error",
            gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "deprecated behavior",
            gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "undefined behavior",
            gl::DEBUG_TYPE_PORTABILITY => "portability",
            gl::DEBUG_TYPE_PERFORMANCE => "performance",
            gl::DEBUG_TYPE_MARKER => "marker",
            gl::DEBUG_TYPE_PUSH_GROUP => "push group",
            gl::DEBUG_TYPE_POP_GROUP => "pop group",
            gl::DEBUG_TYPE_OTHER => "other",
            _ => "unknown",
        };

        let message = format!("{type_str} notice from {source_str} [id = {id}]: {msg}");

        match severity {
            gl::DEBUG_SEVERITY_HIGH => tracing::error!(message),
            gl::DEBUG_SEVERITY_MEDIUM => tracing::debug!(message),
            gl::DEBUG_SEVERITY_LOW => tracing::warn!(message),
            gl::DEBUG_SEVERITY_NOTIFICATION => tracing::info!(message),
            _ => (),
        };
    }
}
