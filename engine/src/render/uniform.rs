//! High-level uniform representation

use anyhow::anyhow;

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
#[repr(u32)]
/// OpenGL uniform kind
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
            _ => Err(anyhow!("unknown uniform kind: {value} ({value:#X})")),
        }
    }
}

#[derive(Debug, Clone, Copy)]
/// Uniform descriptor
pub struct UniformDesc {
    pub kind: UniformKind,
    pub location: u32,
}

#[derive(Debug, Clone, Copy)]
/// Uniform block descriptor
pub struct UniformBlockDesc {
    pub kind: UniformKind,
    pub binding: u32,
}

/// Represents uniform value with underlying data type `T`
///
/// Use [`uniform_value_t`](crate::render::prelude::GlShaderProgram::uniform_value_t) fn to buffer data
pub trait UniformValue<T> {
    fn name(&self) -> &str;
    fn kind(&self) -> &UniformKind;
    unsafe fn value_ptr(&self) -> *const T;
}

/// Represents uniform value with underlying data type `T`
///
/// Use [`uniform_value_s`](crate::render::prelude::GlShaderProgram::uniform_value_s) fn to buffer data
pub struct Uniform<T> {
    pub name: String,
    pub kind: UniformKind,
    pub value_ptr: *const T,
}
