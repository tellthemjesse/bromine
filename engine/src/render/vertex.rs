//! Declares [`Vertex`] trait. Provides [`VertexAttrib`] struct
//!
//! [OpenGL refernce page](https://registry.khronos.org/OpenGL-Refpages/gl4/html/glVertexAttribPointer.xhtml)

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
#[repr(u32)]
/// Represents data types, accepted by [`gl::VertexAttribPointer()`]
pub enum AttributeKind {
    I8 = gl::BYTE,
    U8 = gl::UNSIGNED_BYTE,
    I16 = gl::SHORT,
    U16 = gl::UNSIGNED_SHORT,
    I32 = gl::INT,
    U32 = gl::UNSIGNED_INT,
    Float = gl::FLOAT,
    Double = gl::DOUBLE,
}

#[derive(Debug, Clone, Copy)]
pub struct VertexAttrib {
    pub index: u32,
    /// number of components per attribute
    pub size: usize,
    pub kind: AttributeKind,
    pub normalized: bool,
    /// stride is typically [`std::mem::size_of`] `T`
    pub stride: usize,
    /// [`std::mem::offset_of`] the field inside `T`
    pub offset: usize,
}

/// This trait is required for submitting data to the vertex buffer
pub trait Vertex {
    /// A list of vertex attributes
    fn attributes() -> impl IntoIterator<Item = VertexAttrib>;
}
