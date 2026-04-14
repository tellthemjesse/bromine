//! Low-level implementation for [`buffer_object`](super::buffer_object) module

use super::buffer_object::*;
use super::vertex::Vertex;
use anyhow::bail;
use std::ffi::c_void;

#[derive(Debug)]
/// Represents OpenGL buffer object
pub struct GlBufferObject {
    id: u32,
    desc: BufferObjDesc,
    count: usize,
}

#[derive(Debug)]
/// Represents OpenGL vertex array
pub struct GlVertexArray {
    id: u32,
}

impl GlVertexArray {
    /// Creates new vertex array
    pub fn new() -> Self {
        let mut array = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut array);
        }

        Self { id: array }
    }
    /// Binds this vertex array
    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }
    /// Submits the attributes for given vertex type
    ///
    /// # Safety
    ///
    /// The caller must ensure that this buffer object is active
    pub unsafe fn write<V: Vertex>(&self) -> anyhow::Result<()> {
        unsafe {
            for attr in V::attributes() {
                gl::EnableVertexAttribArray(attr.index);
                gl::VertexAttribPointer(
                    attr.index,
                    attr.size as i32,
                    attr.kind as u32,
                    attr.normalized as u8,
                    attr.stride as i32,
                    attr.offset as *const c_void,
                );

                let err = gl::GetError();
                if err != gl::NO_ERROR {
                    gl::DeleteVertexArrays(1, &mut self.id.clone());
                    gl::BindVertexArray(0);
                    bail!(
                        "failed to make an attribute pointer, err code: {err} ({err:#X})"
                    );
                }
            }
        }

        Ok(())
    }
    /// Unbinds vertex array
    pub fn unbind(&self) {
        unsafe {
            gl::BindVertexArray(0);
        }
    }
    /// Returns the underlying object id
    pub fn id(&self) -> u32 {
        self.id
    }
}

impl GlBufferObject {
    /// Creates new empty buffer object
    pub fn new(desc: BufferObjDesc) -> Self {
        let mut buffer = 0;

        unsafe {
            gl::GenBuffers(1, &mut buffer);
        }

        Self {
            id: buffer,
            desc,
            count: 0,
        }
    }
    /// Binds this buffer
    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(self.desc().kind as u32, self.id);
        }
    }
    /// Submits data to the GPU
    ///
    /// # Safety
    ///
    /// The caller must ensure that this buffer object is active
    pub unsafe fn write<T>(&mut self, data: Vec<T>) -> anyhow::Result<()> {
        if data.is_empty() {
            bail!("cannot submit an empty buffer to the GPU");
        }

        self.count = data.len();

        unsafe {
            gl::BufferData(
                self.desc.kind as u32,
                (size_of::<T>() * self.count) as isize,
                data.as_ptr() as *const c_void,
                self.desc.usage as u32,
            );

            let err = gl::GetError();
            if err != gl::NO_ERROR {
                gl::DeleteBuffers(1, &self.id);
                gl::BindBuffer(self.desc.kind as u32, 0);
                bail!("failed to buffer data, err code: {err} ({err:#X})");
            }
        }

        Ok(())
    }
    /// Unbinds this buffer
    pub fn unbind(&self) {
        unsafe {
            gl::BindBuffer(self.desc().kind as u32, 0);
        }
    }
    /// Returns the underlying object id
    pub fn id(&self) -> u32 {
        self.id
    }
    /// Returns object descriptor
    pub fn desc(&self) -> &BufferObjDesc {
        &self.desc
    }
    /// Returns number of elements stored in this buffer
    pub fn count(&self) -> usize {
        self.count
    }
}

#[cfg(test)]
mod tests {
    use super::super::vertex::{AttributeKind, VertexAttrib};
    use super::*;

    #[repr(transparent)]
    struct Position(pub [f32; 3]);

    impl Vertex for Position {
        fn attributes() -> impl IntoIterator<Item = VertexAttrib> {
            [VertexAttrib {
                index: 0,
                size: 3,
                kind: AttributeKind::Float,
                normalized: false,
                stride: std::mem::size_of::<Position>(),
                offset: 0,
            }]
        }
    }
    
    #[test]
    fn test_vertex_buf() {
        let display = gl_headless::build_display();
        display.load_gl();

        let desc = BufferObjDesc::new(BufferObjKind::Vertex, BufferUsage::StaticDraw);

        let vao = GlVertexArray::new();
        let mut vbo = GlBufferObject::new(desc);

        vao.bind();
        vbo.bind();
        
        unsafe {
            let vbo_write = vbo.write(vec![Position([1.0, 0.0, 1.0])]);
            assert!(vbo_write.is_ok(), "{}", vbo_write.unwrap_err());
            let vao_write = vao.write::<Position>();
            assert!(vao_write.is_ok(), "{}", vao_write.unwrap_err());
        }

        vao.unbind();
        vbo.unbind();        
    }
    
    #[test]
    fn test_empty_buf() {
        let display = gl_headless::build_display();
        display.load_gl();
        
        let desc = BufferObjDesc::new(BufferObjKind::Vertex, BufferUsage::StaticDraw);
  
        let mut vbo = GlBufferObject::new(desc);
        vbo.bind();
        
        let vbo_write = unsafe {
            vbo.write(Vec::<Position>::new())
        };
        
        assert!(vbo_write.is_err())
    }
}
