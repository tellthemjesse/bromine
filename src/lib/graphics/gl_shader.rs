use std::{collections::HashMap, ffi::{CStr}, ptr};
use anyhow::anyhow;
use super::{ffi::c_string, shader::*};

/// Represents OpenGL shader
pub struct GlShader {
    id: u32,
    pub desc: ShaderDesc,
}

impl GlShader {
    /// Returns the underlying object id
    pub fn id(&self) -> u32 {
        self.id
    }
}

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
    let id: u32;
    let src = c_string(source);
    let mut status = gl::FALSE as i32;
    
    unsafe {
        id = gl::CreateShader(desc.stage as u32);
        gl::ShaderSource(id, 1, &src.as_ptr(), ptr::null());
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut status);
    }
    
    if status != gl::TRUE as i32 {
        // TODO: Query info log for error message
        return Err(anyhow!("shader compilation failed"));
    }
    
    Ok(GlShader { id, desc })
}

/// Links a shader program
/// 
/// # Note
/// 
/// Deletes [`GlShader`] objects after done linking
pub fn link_program(shaders: Vec<GlShader>) -> anyhow::Result<GlShaderProgram> {
    let id: u32;
    let mut is_linked = gl::FALSE as i32;
    
    unsafe { 
        id = gl::CreateProgram();
        for shader in &shaders {
            gl::AttachShader(id, shader.id);
        }
        gl::LinkProgram(id);
        gl::GetProgramiv(id, gl::LINK_STATUS, &mut is_linked);
    }
    
    if is_linked != gl::TRUE as i32 {
        // TODO: Query info log for error message
        return Err(anyhow!("failed to link a program"));
    }
    
    // store shader descriptors
    let mut shaders_ = Vec::with_capacity(shaders.len());
    for shader in shaders {
        unsafe { gl::DeleteShader(shader.id); }
        shaders_.push(shader.desc);
    }
        
   let uniforms = get_uniforms(id)?;
    
    Ok(GlShaderProgram {
        id,
        desc: ShaderProgramDesc::new(shaders_, uniforms)
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
    
        unsafe { 
            gl::GetActiveUniform(
                program, 
                index, 
                buf_size, 
                ptr::null_mut(), 
                ptr::null_mut(), 
                &mut uniform_kind, 
                buf.as_mut_ptr()
            );
            
            let uniform_name_ = CStr::from_ptr(buf.as_ptr());
            let uniform_name = uniform_name_.to_str()?.to_string();
             
            // skip if this is a built-in uniform
            if uniform_name.starts_with("gl_") {
                buf.clear();
                continue; 
            }
            
            let location = gl::GetUniformLocation(program, uniform_name_.as_ptr());
            
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
