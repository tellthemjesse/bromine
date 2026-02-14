use std::alloc::Layout;

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
#[repr(u32)]
pub enum BufferObjectKind {
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
pub struct BufferObjectDesc {
    pub kind: BufferObjectKind,
    pub usage: BufferUsage,
    pub layout: Layout,
}
