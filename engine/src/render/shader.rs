use std::collections::HashMap;
use anyhow::anyhow;

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
        Self { debug_name: name.into(), stage, }
    }
}

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
#[repr(u32)]
pub enum UniformKind {
    Float = gl::FLOAT,
    Vec3 = gl::FLOAT_VEC3,
    Mat4 = gl::FLOAT_MAT4,
    Sampler2D = gl::SAMPLER_2D,
}

impl TryFrom<u32> for UniformKind {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            gl::FLOAT => Ok(UniformKind::Float),
            gl::FLOAT_VEC3 => Ok(UniformKind::Vec3),
            gl::FLOAT_MAT4 => Ok(UniformKind::Mat4),
            gl::SAMPLER_2D => Ok(UniformKind::Sampler2D),
            _ => Err(anyhow!("unknown uniform kind: {value} (0x{value:X})"))
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UniformDesc {
    pub kind: UniformKind,
    pub location: u32,
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