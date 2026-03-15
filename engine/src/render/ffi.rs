use std::ffi::CString;
#[cfg(feature = "debug")]
use std::ffi::{CStr, c_void};

#[inline(always)]
pub fn c_string<T>(s: T) -> CString
where
    T: Into<Vec<u8>>,
{
    CString::new(s).unwrap()
}

#[cfg(feature = "debug")]
pub type GlDebugCallback = extern "system" fn(
    source: u32,
    error_type: u32,
    id: u32,
    severity: u32,
    length: i32,
    message: *const i8,
    user_param: *mut c_void,
);

#[cfg(feature = "debug")]
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
            gl::DEBUG_SOURCE_WINDOW_SYSTEM => "WINDOW_SYSTEM",
            gl::DEBUG_SOURCE_SHADER_COMPILER => "SHADER_COMPILER",
            gl::DEBUG_SOURCE_THIRD_PARTY => "THIRD_PARTY",
            gl::DEBUG_SOURCE_APPLICATION => "APPLICATION",
            gl::DEBUG_SOURCE_OTHER => "OTHER",
            _ => "UNKNOWN",
        };

        let type_str = match error_type {
            gl::DEBUG_TYPE_ERROR => "ERROR",
            gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "DEPRECATED_BEHAVIOR",
            gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "UNDEFINED_BEHAVIOR",
            gl::DEBUG_TYPE_PORTABILITY => "PORTABILITY",
            gl::DEBUG_TYPE_PERFORMANCE => "PERFORMANCE",
            gl::DEBUG_TYPE_MARKER => "MARKER",
            gl::DEBUG_TYPE_PUSH_GROUP => "PUSH_GROUP",
            gl::DEBUG_TYPE_POP_GROUP => "POP_GROUP",
            gl::DEBUG_TYPE_OTHER => "OTHER",
            _ => "UNKNOWN",
        };

        let msg = format!(
            "OpenGL Debug: Source={}, Type={}, ID={}, Message={}",
            source_str, type_str, id, msg
        );

        match severity {
            gl::DEBUG_SEVERITY_HIGH => tracing::error!(msg),
            gl::DEBUG_SEVERITY_MEDIUM => tracing::debug!(msg),
            gl::DEBUG_SEVERITY_LOW => tracing::warn!(msg),
            gl::DEBUG_SEVERITY_NOTIFICATION => tracing::info!(msg),
            _ => (),
        };
    }
}
