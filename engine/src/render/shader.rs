use std::collections::HashMap;
use super::uniform::UniformDesc;

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
#[repr(u32)]
pub enum ShaderStage {
    Vertex = gl::VERTEX_SHADER,
    Fragment = gl::FRAGMENT_SHADER,
}

#[derive(Debug, Clone)]
pub struct ShaderDesc {
    pub debug_name: String,
    pub stage: ShaderStage,
}

impl ShaderDesc {
    pub fn new(name: impl Into<String>, stage: ShaderStage) -> Self {
        Self {
            debug_name: name.into(),
            stage,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShaderProgramDesc {
    pub shaders: Vec<ShaderDesc>,
    pub uniforms: HashMap<String, UniformDesc>,
}

impl ShaderProgramDesc {
    pub fn new(shaders: Vec<ShaderDesc>, uniforms: HashMap<String, UniformDesc>) -> Self {
        Self { shaders, uniforms }
    }
}
