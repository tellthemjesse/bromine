//! High-level uniform representation

use anyhow::bail;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
#[repr(u32)]
/// OpenGL Shading Language data type
pub enum GlslDatatype {
    Float = gl::FLOAT,
    Vec3 = gl::FLOAT_VEC3,
    Mat4 = gl::FLOAT_MAT4,
    Sampler2D = gl::SAMPLER_2D,
}

impl TryFrom<u32> for GlslDatatype {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            gl::FLOAT => Ok(GlslDatatype::Float),
            gl::FLOAT_VEC3 => Ok(GlslDatatype::Vec3),
            gl::FLOAT_MAT4 => Ok(GlslDatatype::Mat4),
            gl::SAMPLER_2D => Ok(GlslDatatype::Sampler2D),
            _ => bail!("unsupported gl_enum: {value} ({value:#X})"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
/// Uniform variable descriptor
///
/// Name property is not a part of this structure because [`GlShaderProgram`](super::gl_shader::GlShaderProgram)
/// owns a `HashMap<K, V>` over all active uniforms (`K` = *Name*, `V` = *Descriptor*)
pub struct UniformVarDesc {
    pub datatype: GlslDatatype,
    pub location: u32,
}

impl UniformVarDesc {
    pub fn new(datatype: GlslDatatype, location: u32) -> Self {
        Self { datatype, location }
    }
}

#[derive(Debug, Clone)]
/// Uniform block descriptor
///
/// Name property is not a part of this structure because [`GlShaderProgram`](super::gl_shader::GlShaderProgram)
/// owns a `HashMap<K, V>` over all active uniform blocks (`K` = *Name*, `V` = *Descriptor*)
pub struct UniformBlockDesc {
    pub binding: u32,
    pub fields: HashMap<String, GlslDatatype>,
}

impl UniformBlockDesc {
    pub fn new(binding: u32, fields: HashMap<String, GlslDatatype>) -> Self {
        Self { binding, fields } 
    }
}

#[derive(Debug, Clone)]
/// Type of a uniform variable within shader program
/// 
/// todo: atomics
pub(crate) enum UniformVarType {
    /// Global identifier (variable) + location
    Global(UniformVarMeta, u32),
    /// Scoped identifier (field name inside a structure) + block index
    Scoped(UniformVarMeta, u32),
    /// Glsl variable (prefixed with gl_)
    Builtin,
}

#[derive(Debug, Clone)]
pub(crate) struct UniformVarMeta {
    pub name: String,
    pub datatype: GlslDatatype,
}

#[derive(Debug, Clone)]
pub(crate) struct UniformBlockMeta {
    pub name: String,
    pub binding: u32,
    pub index: u32,
}

/// Represents uniform value with underlying data type `T`
///
/// Implement this trait when it makes sense for the type to be directly used as uniform
///
/// Use [`GlShaderProgram::uniform_value_t()`](crate::render::prelude::GlShaderProgram::uniform_value_t) to set the value
pub trait UniformValue<T> {
    fn datatype(&self) -> GlslDatatype;
    fn name(&self) -> &str;
    fn value_ptr(&self) -> *const T;
}

/// Represents uniform value with underlying data type `T`
///
/// Use [`GlShaderProgram::uniform_value_s()`](crate::render::prelude::GlShaderProgram::uniform_value_s) to set the value
pub struct UniformVariable<T> {
    name: String,
    datatype: GlslDatatype,
    value_ptr: *const T,
}

impl<T> UniformVariable<T> {
    pub fn new(name: String, datatype: GlslDatatype, value_ptr: *const T) -> Self {
        Self {
            name,
            datatype,
            value_ptr,
        }
    }
    /// Uniform name
    pub fn name(&self) -> &String {
        &self.name
    }
    /// Uniform data type
    pub fn datatype(&self) -> GlslDatatype {
        self.datatype
    }
    /// Value pointer
    pub fn value_ptr(&self) -> *const T {
        self.value_ptr
    }
}
