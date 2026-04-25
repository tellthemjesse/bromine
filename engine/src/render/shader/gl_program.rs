//! Low-level implementation for [`shader program`](super::program)

use super::{gl_uniform::GlslUniforms, program::*, uniform::*};
use anyhow::bail;
use std::{
    ffi::{CString, c_char, c_uint}, ptr
};

#[derive(Debug)]
/// Represents OpenGL shader
pub struct GlShader {
    id: u32,
    desc: ShaderDesc,
}

impl GlShader {
    /// Compiles the shader
    pub fn compile(source: impl Into<Vec<u8>>, desc: ShaderDesc) -> anyhow::Result<GlShader> {
        let shader;
        let src = CString::new(source)?;
        let mut status = gl::FALSE as i32;

        unsafe {
            shader = gl::CreateShader(desc.stage as c_uint);
            gl::ShaderSource(shader, 1, &src.as_ptr(), ptr::null());
            gl::CompileShader(shader);
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
        }

        if status != (gl::TRUE as i32) {
            let mut len = 0;

            unsafe {
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            }

            let mut buf: Vec<u8> = Vec::with_capacity(len as usize);

            unsafe {
                gl::GetShaderInfoLog(
                    shader,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut c_char,
                );
                buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            }

            bail!("shader compilation failed: {}", str::from_utf8(&buf)?);
        }

        Ok(GlShader { id: shader, desc })
    }
    /// Returns the underlying object id
    pub fn id(&self) -> u32 {
        self.id
    }
    /// Returns object descriptor
    pub fn desc(&self) -> &ShaderDesc {
        &self.desc
    }
}

#[derive(Debug)]
/// Represents OpenGL shader program
pub struct GlProgram {
    id: u32,
    desc: ProgramDesc,
}

impl GlProgram {
    /// Links the shader program.
    ///
    /// [`GlShader`] objects are deleted after this operation
    pub fn link(shaders: Vec<GlShader>) -> anyhow::Result<Self> {
        let program: u32;
        let mut is_linked = gl::FALSE as i32;

        unsafe {
            program = gl::CreateProgram();
            for shader in &shaders {
                gl::AttachShader(program, shader.id);
            }
            gl::LinkProgram(program);
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut is_linked);
        }

        if is_linked != gl::TRUE as i32 {
            let mut len = 0;

            unsafe {
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            }

            let mut buf: Vec<u8> = Vec::with_capacity(len as usize);

            unsafe {
                gl::GetProgramInfoLog(
                    program,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut gl::types::GLchar,
                );
                buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            }

            bail!("failed to link a program: {}", str::from_utf8(&buf)?);
        }

        // store shader descriptors
        let mut shaders_ = Vec::with_capacity(shaders.len());
        for shader in shaders {
            unsafe {
                gl::DeleteShader(shader.id);
            }
            shaders_.push(shader.desc);
        }

        let uniforms = GlslUniforms(program);
        let globals = uniforms.get_globals()?;
        let blocks = uniforms.get_blocks()?;

        let globals_list = uniforms.globals_to_list(globals);
        let blocks_list = uniforms.blocks_to_list(blocks);

        Ok(Self {
            id: program,
            desc: ProgramDesc::new(shaders_, globals_list, blocks_list),
        })
    }
    /// Returns the underlying object id
    pub fn id(&self) -> u32 {
        self.id
    }
    /// Returns object descriptor
    pub fn desc(&self) -> &ProgramDesc {
        &self.desc
    }
    /// Sets uniform value using data record
    pub fn set_uniform<T>(&self, u: UniformValue<T>) {
        if let Some(variable) = self
            .desc
            .uniforms
            .iter()
            .find(|desc| &desc.name == u.name())
        {
            let location = variable.location as i32;
            unsafe {
                let ptr = u.value_ptr();

                match u.datatype() {
                    GlslDatatype::Float => gl::Uniform1fv(location, 1, ptr as _),
                    GlslDatatype::Vec3 => gl::Uniform3fv(location, 1, ptr as _),
                    GlslDatatype::Mat4 => {
                        gl::UniformMatrix4fv(location, 1, gl::FALSE, ptr as _);
                    }
                    GlslDatatype::Sampler2D => todo!(),
                }
            }
        }
    }
    /// Binds the shader program
    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }
    /// Unbinds the shader program
    pub fn unbind(&self) {
        unsafe {
            gl::UseProgram(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VERTEX_SHADER: &str = r"
        #version 460 core

        layout (location = 0) in vec3 v_Position;
        layout (location = 1) in vec2 v_TexCoords;
        layout (location = 0) out vec2 f_TexCoords;

        void main() {
            gl_Position = vec4(v_Position, 1.0);
            f_TexCoords = v_TexCoords;
        }
    ";

    const FRAGMENT_SHADER: &str = r"
        #version 460 core

        layout (location = 0) in vec2 f_TexCoords;
        layout (location = 0) out vec4 FragColor;
        layout (location = 0) uniform sampler2D u_Texture;

        struct Light { vec3 position; vec3 color; };

        uniform Light u_Light;

        layout (std140) uniform LightBlock { vec3 position; vec3 color; } u_LightBlock;

        void main() {
            vec3 aPos = u_Light.position;
            vec3 aColor = u_Light.color;
            FragColor = texture(u_Texture, f_TexCoords);
        }
    ";

    #[test]
    fn test_program() {
        let display = gl_headless::build_display();
        display.load_gl();

        let vs = GlShader::compile(VERTEX_SHADER, ShaderDesc::vert("v_test"));
        let fs = GlShader::compile(FRAGMENT_SHADER, ShaderDesc::frag("f_test"));

        assert!(
            vs.is_ok(),
            "couldn't compile vertex shader: {}",
            vs.unwrap_err()
        );
        assert!(
            fs.is_ok(),
            "couldn't compile fragment shader: {}",
            fs.unwrap_err()
        );

        let program = GlProgram::link(vec![vs.unwrap(), fs.unwrap()]);

        assert!(
            program.is_ok(),
            "couldn't link program: {}",
            program.unwrap_err()
        );

        let program = program.unwrap();

        assert_eq!(program.desc().uniforms.len(), 3);
        assert_eq!(program.desc().blocks.len(), 1);
    }
}
