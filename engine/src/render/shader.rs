//! High-level representation of shaders

use super::uniform::UniformDesc;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
#[repr(u32)]
/// Represents a shader stage
pub enum ShaderStage {
    Vertex = gl::VERTEX_SHADER,
    Fragment = gl::FRAGMENT_SHADER,
}

#[derive(Debug, Clone)]
/// Shader descriptor
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
    /// Creates new shader descriptor with stage set to [`ShaderStage::Vertex`]
    pub fn vert(name: impl Into<String>) -> Self {
        Self {
            debug_name: name.into(),
            stage: ShaderStage::Vertex,
        }
    }
    /// Creates new shader descriptor with stage set to [`ShaderStage::Fragment`]
    pub fn frag(name: impl Into<String>) -> Self {
        Self {
            debug_name: name.into(),
            stage: ShaderStage::Fragment,
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
