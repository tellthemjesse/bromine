use super::{ffi::c_string, shader::*};
use anyhow::anyhow;
use std::{collections::HashMap, ffi::CStr, ptr};

#[derive(Debug)]
/// Represents OpenGL shader
pub struct GlShader {
    id: u32,
    desc: ShaderDesc,
}

impl GlShader {
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
pub struct GlShaderProgram {
    id: u32,
    pub desc: ShaderProgramDesc,
}

impl GlShaderProgram {
    pub fn id(&self) -> u32 {
        self.id
    }
}

/// Compiles a shader
pub fn compile_shader(source: impl Into<Vec<u8>>, desc: ShaderDesc) -> anyhow::Result<GlShader> {
    let shader: u32;
    let src = c_string(source);
    let mut status = gl::FALSE as i32;

    unsafe {
        shader = gl::CreateShader(desc.stage as u32);
        gl::ShaderSource(shader, 1, &src.as_ptr(), ptr::null());
        gl::CompileShader(shader);
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
    }

    if status != (gl::TRUE as i32) {
        let mut len = 0;
        let mut buf = Vec::with_capacity(len as usize);

        unsafe {
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(
                shader,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut gl::types::GLchar,
            );
        }

        return Err(anyhow!(
            "shader compilation failed: {}",
            str::from_utf8(&buf)?
        ));
    }

    Ok(GlShader { id: shader, desc })
}

/// Links a shader program
///
/// # Note
///
/// Deletes [`GlShader`] objects after done linking
pub fn link_program(shaders: Vec<GlShader>) -> anyhow::Result<GlShaderProgram> {
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
        let mut buf = Vec::with_capacity(len as usize);

        unsafe {
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetProgramInfoLog(
                program,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut gl::types::GLchar,
            );
        }

        return Err(anyhow!(
            "failed to link a program: {}",
            str::from_utf8(&buf)?
        ));
    }

    // store shader descriptors
    let mut shaders_ = Vec::with_capacity(shaders.len());
    for shader in shaders {
        unsafe {
            gl::DeleteShader(shader.id);
        }
        shaders_.push(shader.desc);
    }

    let uniforms = get_uniforms(program)?;

    Ok(GlShaderProgram {
        id: program,
        desc: ShaderProgramDesc::new(shaders_, uniforms),
    })
}

/// Returns a HashMap over active uniforms
///
/// # Dev Note
///
/// Uniform size is unused
fn get_uniforms(program: u32) -> anyhow::Result<HashMap<String, UniformDesc>> {
    let mut active_uniforms = 0;
    let mut buf_size = 0;

    unsafe {
        gl::GetProgramiv(program, gl::ACTIVE_UNIFORMS, &mut active_uniforms);
        gl::GetProgramiv(program, gl::ACTIVE_UNIFORM_MAX_LENGTH, &mut buf_size);
    }

    let mut uniforms = HashMap::with_capacity(active_uniforms as usize);
    let mut buf = Vec::with_capacity(buf_size as usize);

    for index in 0..active_uniforms as u32 {
        let mut uniform_kind = 0;
        // amount of characters written excluding null terminator
        let mut length = 0;

        unsafe {
            gl::GetActiveUniform(
                program,
                index,
                buf_size,
                &mut length,
                ptr::null_mut(),
                &mut uniform_kind,
                buf.as_mut_ptr() as *mut gl::types::GLchar,
            );

            buf.set_len(length as usize + 1);

            // fn call accidentally failed with "data provided is not nul terminated"
            // maybe manually updating the buffer length fixes that ^

            let uniform_name_cstr = CStr::from_bytes_with_nul(&buf)?;
            let uniform_name = uniform_name_cstr.to_str()?.to_string();

            // skip if this is a built-in uniform
            if uniform_name.starts_with("gl_") {
                buf.clear();
                continue;
            }

            let location = gl::GetUniformLocation(program, uniform_name_cstr.as_ptr());

            let uniform_desc = UniformDesc {
                kind: UniformKind::try_from(uniform_kind)?,
                location: location as u32,
            };

            let _ = uniforms.insert(uniform_name, uniform_desc);
        }

        buf.clear();
    }

    Ok(uniforms)
}

/// Enables the shader program
pub fn use_program(program: &GlShaderProgram) {
    unsafe {
        gl::UseProgram(program.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VERTEX_SHADER: &'static str = r"
        #version 450 core

        layout (location = 0) in vec3 v_Position;
        layout (location = 1) in vec2 v_TexCoords;
        layout (location = 0) out vec2 f_TexCoords;

        void main() {
            gl_Position = vec4(v_Position, 1.0);
            f_TexCoords = v_TexCoords;
        }
    ";

    const FRAGMENT_SHADER: &'static str = r"
        #version 450 core

        layout (location = 0) in vec2 f_TexCoords;
        layout (location = 0) out vec4 FragColor;
        layout (location = 0) uniform sampler2D u_Texture;

        void main() {
            FragColor = texture(u_Texture, f_TexCoords);
        }
    ";

    #[test]
    fn test_shader_operations() {
        // it's not possible to load multiple OpenGL contexts due to event loop limitations,
        // so all of the tests should happen here
        let mut tfn = gl_headless::GlHeadless::new(|| {
            let v_desc = ShaderDesc::new("v_test", ShaderStage::Vertex);
            let v_shader = compile_shader(VERTEX_SHADER, v_desc).unwrap();

            let f_desc = ShaderDesc::new("f_test", ShaderStage::Fragment);
            let f_shader = compile_shader(FRAGMENT_SHADER, f_desc).unwrap();

            let program = link_program(vec![v_shader, f_shader]);

            let _ = program.unwrap();
        });
        let _ = tfn.run_once();
    }
}
