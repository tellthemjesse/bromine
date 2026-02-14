use std::collections::HashMap;

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

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
#[repr(u32)]
pub enum UniformKind {
    Float = gl::FLOAT,
    Vec3 = gl::FLOAT_VEC3,
    Mat4 = gl::FLOAT_MAT4,
    Sampler2D = gl::SAMPLER_2D,
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
