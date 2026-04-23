use crate::render::{
    buffer_object::{BufferObjDesc, BufferObjKind},
    element::*,
    gl_buffer_object::*,
    renderable::Renderable,
    vertex::Vertex,
};
use anyhow::anyhow;
use std::ptr;

#[derive(Debug)]
/// Represents OpenGL element buffer
pub struct GlElementBuffer {
    buf: GlBufferObject,
    repr: u32,
}

impl GlElementBuffer {
    /// Creates new element buffer object
    pub fn new(desc: BufferObjDesc) -> anyhow::Result<Self> {
        if desc.kind != BufferObjKind::Element {
            return Err(anyhow!(
                "buffer kind mismatch, expected {:?}, got {:?}",
                BufferObjKind::Element,
                desc.kind
            ));
        }

        let buf = GlBufferObject::new(desc);
        Ok(Self { buf, repr: 0 })
    }
    /// Binds this buffer
    pub fn bind(&self) {
        self.buf.bind();
    }
    /// Submits data to the GPU
    ///
    /// # Safety
    ///
    /// The caller must ensure that this buffer object is active
    pub unsafe fn write<E: Element>(&mut self, data: Vec<E>) -> anyhow::Result<()> {
        self.repr = E::repr();
        unsafe { self.buf.write(data) }
    }
    /// Unbinds this buffer
    pub fn unbind(&self) {
        self.buf.unbind();
    }
    /// Returns the underlying buffer
    pub fn buf(&self) -> &GlBufferObject {
        &self.buf
    }
    /// Returns the indices type used for drawing
    pub fn repr(&self) -> u32 {
        self.repr
    }
}

#[derive(Debug)]
/// Composition of geometry that can be drawn
pub struct GlMesh {
    vao: GlVertexArray,
    vbo: GlBufferObject,
    primitive: Primitive,
    ebo: Option<GlElementBuffer>,
}

impl GlMesh {
    /// Creates new Mesh with specified vertex data
    pub fn new<V: Vertex>(
        data: Vec<V>,
        desc: BufferObjDesc,
        primitive: Primitive,
    ) -> anyhow::Result<Self> {
        if desc.kind != BufferObjKind::Vertex {
            return Err(anyhow!(
                "buffer kind mismatch, expected {:?}, got {:?}",
                BufferObjKind::Vertex,
                desc.kind
            ));
        }

        let vao = GlVertexArray::generate();
        let mut vbo = GlBufferObject::new(desc);

        vao.bind();
        vbo.bind();
        unsafe {
            vbo.write(data)?;
            vao.write::<V>()?;
        }
        vao.unbind();
        vbo.unbind();

        Ok(Self {
            vao,
            vbo,
            primitive,
            ebo: None,
        })
    }
    /// Creates new Mesh with specified element data
    pub fn with_element_buffer<E: Element>(
        mut self,
        data: Vec<E>,
        desc: BufferObjDesc,
    ) -> anyhow::Result<Self> {
        let mut ebo = GlElementBuffer::new(desc)?;

        self.vao.bind();
        ebo.bind();
        unsafe {
            ebo.write(data)?;
        }
        self.vao.unbind();
        ebo.unbind();

        self.ebo = Some(ebo);
        Ok(self)
    }
    /// Returns a reference to the VAO
    pub fn vao(&self) -> &GlVertexArray {
        &self.vao
    }
    /// Returns a reference to the VBO
    pub fn vbo(&self) -> &GlBufferObject {
        &self.vbo
    }
    /// Returns the primitive type used for drawing
    pub fn primitive(&self) -> Primitive {
        self.primitive
    }
    /// Returns a reference to the EBO
    pub fn ebo(&self) -> Option<&GlElementBuffer> {
        self.ebo.as_ref()
    }
}

impl Renderable for GlMesh {
    fn draw(&self) {
        self.vao.bind();

        unsafe {
            if let Some(ref ebo) = self.ebo {
                gl::DrawElements(
                    self.primitive as u32,
                    ebo.buf.count() as i32,
                    ebo.repr,
                    ptr::null(),
                );
            } else {
                gl::DrawArrays(self.primitive as u32, 0, self.vbo.count() as i32);
            }
        }

        self.vao.unbind();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::{buffer_object::*, vertex::*};

    #[test]
    fn test_mesh() {
        let display = gl_headless::build_display();
        display.load_gl();

        struct MyVertex {
            position: [f32; 3],
        }

        impl Vertex for MyVertex {
            fn attributes() -> impl IntoIterator<Item = VertexAttrib> {
                [VertexAttrib {
                    index: 0,
                    size: 3,
                    kind: AttributeKind::Float,
                    normalized: true,
                    stride: std::mem::size_of::<MyVertex>(),
                    offset: std::mem::offset_of!(MyVertex, position),
                }]
            }
        }

        let vertices = vec![
            MyVertex {
                position: [0.54, 0.21, -0.43],
            },
            MyVertex {
                position: [0.54, 0.66, -0.43],
            },
            MyVertex {
                position: [0.33, -0.12, 0.94],
            },
        ];
        let v_desc = BufferObjDesc::new(BufferObjKind::Vertex, BufferUsage::StaticDraw);

        let elements = vec![0, 2, 1_u32];
        let e_desc = BufferObjDesc::new(BufferObjKind::Element, BufferUsage::StaticDraw);

        let _ = GlMesh::new::<MyVertex>(vertices, v_desc, Primitive::Triangles)
            .unwrap()
            .with_element_buffer(elements, e_desc)
            .unwrap();
    }
}
