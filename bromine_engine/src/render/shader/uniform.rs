//! High-level uniform representation

use anyhow::bail;

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

#[derive(Debug, Clone)]
/// Uniform variable descriptor
///
/// todo: include size (implementing arrays)
pub struct UniformVarDesc {
    pub name: String,
    pub datatype: GlslDatatype,
    pub location: u32,
}

impl UniformVarDesc {
    pub fn new(name: String, datatype: GlslDatatype, location: u32) -> Self {
        Self {
            name,
            datatype,
            location,
        }
    }
}

#[derive(Debug, Clone)]
/// Uniform block descriptor
pub struct UniformBlockDesc {
    pub name: String,
    pub binding: u32,
}

impl UniformBlockDesc {
    pub fn new(name: String, binding: u32) -> Self {
        Self { name, binding }
    }
}

/// Global identifier + location
pub(crate) type GlobalIdentifier = (UniformVar, u32);
/// Block identitifier + binding
pub(crate) type BlockIdentifier = (String, u32);

#[derive(Debug, Clone)]
pub(crate) struct UniformVar {
    pub name: String,
    pub datatype: GlslDatatype,
}

/// Represents uniform value with data type `T` that matches specified [`GlslDatatype`].
///
/// Use [`GlProgram::set_uniform()`](crate::render::prelude::GlProgram::set_uniform()) to set the value
pub struct UniformValue<T> {
    name: String,
    datatype: GlslDatatype,
    value_ptr: *const T,
}

impl<T> UniformValue<T> {
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
