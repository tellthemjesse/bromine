use super::{
    ffi::c_string,
    shader::*,
};

/// Represents OpenGL shader
pub struct GlShader {
    pub id: u32,
    pub desc: ShaderDesc,
}

/// Represents OpenGL shader program
pub struct GlShaderProgram {
    pub id: u32,
    pub desc: ShaderProgramDesc,
}

/// Compiles a shader
pub fn compile_shader(source: impl Into<Vec<u8>>, desc: ShaderDesc) -> anyhow::Result<GlShader> {
    todo!()
}

/// Links a shader program
pub fn link_program(shaders: Vec<GlShader>) -> anyhow::Result<GlShaderProgram> {
    todo!()
}

/// Enables the shader program
pub fn use_program(program: &GlShaderProgram) {
    unsafe { gl::UseProgram(program.id); }
}