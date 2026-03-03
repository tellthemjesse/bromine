#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
#[repr(u32)]
pub enum BufferObjKind {
    Vertex = gl::ARRAY_BUFFER,
    Index = gl::ELEMENT_ARRAY_BUFFER,
    Uniform = gl::UNIFORM_BUFFER,
}

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
#[repr(u32)]
pub enum BufferUsage {
    StaticDraw = gl::STATIC_DRAW,
    DynamicDraw = gl::DYNAMIC_DRAW,
}

#[derive(Debug, Clone, Copy)]
/// Describes the necessary info to create a buffer object
pub struct BufferObjDesc {
    pub kind: BufferObjKind,
    pub usage: BufferUsage,
}

impl BufferObjDesc {
    pub fn new(kind: BufferObjKind, usage: BufferUsage) -> Self {
        Self { kind, usage }
    }
}
