use std::{mem, ptr, str};

use gl::types::{GLboolean, GLchar, GLint, GLuint};
use nalgebra_glm::{Vec3, Mat4};

use crate::graphics::context::{get_uniform_location, compile_shader};

#[derive(Debug)]
pub struct Program {
    id: GLuint,
}

impl Program {
    /// Compiles given shaders and links the program
    pub fn new(vertex: &str, fragment: &str, geometry: Option<&str>) -> Self {
        // let mut id: GLuint = 0; // Remove unused initial assignment
        let id: GLuint; // Declare id without initializing

        let vs = compile_shader(vertex, gl::VERTEX_SHADER);
        let fs = compile_shader(fragment, gl::FRAGMENT_SHADER);
        let mut gs: GLuint = 0;

        unsafe {
            id = gl::CreateProgram();

            gl::AttachShader(id, vs);
            gl::AttachShader(id, fs);
            // if geometry shader is present, compile it
            if geometry.is_some() {
                gs = compile_shader(geometry.unwrap(), gl::GEOMETRY_SHADER);
                gl::AttachShader(id, gs);
            }

            gl::LinkProgram(id);

            // getting link status
            let mut status = gl::FALSE as GLint;
            gl::GetProgramiv(id, gl::LINK_STATUS, &mut status);

            // panics if compilation failed
            if status != (gl::TRUE as GLint) {
                let mut len: GLint = 0;
                gl::GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut len);

                let mut buf = Vec::with_capacity(len as usize);
                buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
                gl::GetProgramInfoLog(
                    id,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar,
                );

                panic!("{}", str::from_utf8(&buf).ok().expect("ProgramInfoLog didn't contain valid utf8"));
            }

            // deleting all shaders after the program is successfully linked
            gl::DeleteShader(vs);
            gl::DeleteShader(fs);
            if geometry.is_some() {
                gl::DeleteShader(gs)
            }
        }

        Self {
            id,
        }
    }

    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    pub fn set_float(&self, name: &str, src: f32) {
        unsafe {
            gl::Uniform1f(get_uniform_location(self.id, name), src);
        }
    }

    pub fn set_vec3(&self, name: &str, src: &Vec3) {
        unsafe {
            gl::Uniform3fv(
                get_uniform_location(self.id, name),
                1, src.as_ptr()
            );
        }
    }

    pub fn set_mat4(&self, name: &str, src: &Mat4) {
        unsafe {
            gl::UniformMatrix4fv(
                get_uniform_location(self.id, name),
                1, gl::FALSE as GLboolean,
                src.as_ptr()
            );
        }
    }

    pub fn set_int(&self, name: &str, src: GLint) {
        unsafe {
            gl::Uniform1i(get_uniform_location(self.id, name), src);
        }
    }

    pub unsafe fn create_ubo<T>(&self, data: &[T]) -> GLuint {
        let mut ubo: GLuint = 0;
        gl::GenBuffers(1, &mut ubo);
        gl::BindBuffer(gl::UNIFORM_BUFFER, ubo);
        gl::BufferData(
            gl::UNIFORM_BUFFER,
            (data.len() * mem::size_of::<T>()) as isize,
            data.as_ptr() as *const _,
            gl::DYNAMIC_DRAW // Use GL_STATIC_DRAW for immutable data
        );
        ubo
    }

    pub unsafe fn bind_ubo(&self, ubo: GLuint, binding: u32) {
        gl::BindBufferBase(gl::UNIFORM_BUFFER, binding, ubo);
    }

    pub unsafe fn update_ubo<T>(&self, ubo: GLuint, data: &[T]) {
        gl::BindBuffer(gl::UNIFORM_BUFFER, ubo);
        gl::BufferSubData(
            gl::UNIFORM_BUFFER,
            0,
            (data.len() * mem::size_of::<T>()) as isize,
            data.as_ptr() as *const _
        );
    }
}