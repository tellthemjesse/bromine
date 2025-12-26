use std::{
    ffi::CString,
    ptr, str,
};
use gl::GetUniformLocation;
use gl::types::{GLenum, GLuint, GLint, GLchar, GLbitfield, GLfloat};

pub fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    let shader: GLuint;
    unsafe {
        shader = gl::CreateShader(ty);
        let c_str = c_string(src);
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);
        // get compile status
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // panic if compilation failed
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);

            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(
                shader,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );

            panic!("{}", str::from_utf8(&buf).ok().expect("ShaderInfoLog didn't contain valid utf8"));
        }

        shader
    }
}

pub fn get_uniform_location(program: GLuint, name: &str) -> GLint {
    let c_str = c_string(name);
    unsafe {
        GetUniformLocation(program, c_str.as_ptr())
    }
}

pub fn clear_color(red: GLfloat, green: GLfloat, blue: GLfloat, alpha: GLfloat) {
    unsafe {
        gl::ClearColor(red, green, blue, alpha);
    }
}

pub fn clear(mask: GLbitfield) {
    unsafe {
        gl::Clear(mask);
    }
}

pub fn c_string(src: &str) -> CString {
    CString::new(src).expect("Encountered a 0 byte in str slice")
}