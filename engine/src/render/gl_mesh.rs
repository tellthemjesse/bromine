use anyhow::anyhow;

use super::{
    buffer_object::{BufferObjDesc, BufferObjKind},
    element::*,
    gl_buffer_object::*,
    vertex::Vertex,
};

#[derive(Debug)]
/// Represents OpenGL element buffer
pub struct GlElementBuffer {
    buf: GlBufferObject,
    repr: u32,
}

impl GlElementBuffer {
    /// Creates new element buffer object from its descriptor and data of a type `E`
    pub fn new<E: Element>(data: Vec<E>, desc: BufferObjDesc) -> anyhow::Result<Self> {
        if desc.kind != BufferObjKind::Element {
            return Err(anyhow!(
                "buffer kind mismatch, expected {:?}, got {:?}",
                BufferObjKind::Element,
                desc.kind
            ));
        }

        let buf = GlBufferObject::new(data, desc)?;
        Ok(Self {
            buf,
            repr: E::repr(),
        })
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
    vbo: GlBufferObject,
    primitive: Primitive,
    vao: Option<GlVertexArray>,
    ebo: Option<GlElementBuffer>,
}

impl GlMesh {
    /// Creates new Mesh with specified vertex data
    pub fn new<T>(data: Vec<T>, desc: BufferObjDesc, primitive: Primitive) -> anyhow::Result<Self> {
        if desc.kind != BufferObjKind::Vertex {
            return Err(anyhow!(
                "buffer kind mismatch, expected {:?}, got {:?}",
                BufferObjKind::Vertex,
                desc.kind
            ));
        }

        let vbo = GlBufferObject::new(data, desc)?;
        Ok(Self {
            vbo,
            primitive,
            vao: None,
            ebo: None,
        })
    }
    /// Creates new Mesh with specified vertex attributes
    pub fn with_vertex_attributes<V: Vertex>(mut self) -> anyhow::Result<Self> {
        let vao = GlVertexArray::new::<V>()?;
        self.vao = Some(vao);
        Ok(self)
    }
    /// Creates new Mesh with specified element data
    pub fn with_element_buffer<E: Element>(
        mut self,
        data: Vec<E>,
        desc: BufferObjDesc,
    ) -> anyhow::Result<Self> {
        let ebo = GlElementBuffer::new(data, desc)?;
        self.ebo = Some(ebo);
        Ok(self)
    }
    /// Returns a reference to the VBO
    pub fn vbo(&self) -> &GlBufferObject {
        &self.vbo
    }
    /// Returns the primitive type used for drawing
    pub fn primitive(&self) -> Primitive {
        self.primitive
    }
    /// Returns a reference to the VAO
    pub fn vao(&self) -> Option<&GlVertexArray> {
        self.vao.as_ref()
    }
    /// Returns a reference to the EBO
    pub fn ebo(&self) -> Option<&GlElementBuffer> {
        self.ebo.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::super::{buffer_object::*, vertex::*};
    use super::*;

    #[test]
    fn test_mesh() {
        let mut tfn = gl_headless::GlHeadless::new(|| {
            struct MyVertex {
                position: [f32; 3],
            }

            impl Vertex for MyVertex {
                fn attributes() -> Vec<VertexAttrib> {
                    vec![VertexAttrib {
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
                    position: [0.54, 0.21, -0.43],
                },
                MyVertex {
                    position: [0.33, -0.12, 0.94],
                },
            ];
            let v_desc = BufferObjDesc::new(BufferObjKind::Vertex, BufferUsage::StaticDraw);

            let elements = vec![1, 3, 2_u32];
            let e_desc = BufferObjDesc::new(BufferObjKind::Element, BufferUsage::StaticDraw);

            let _ = GlMesh::new(vertices, v_desc, Primitive::Triangles)
                .unwrap()
                .with_vertex_attributes::<MyVertex>()
                .unwrap()
                .with_element_buffer(elements, e_desc)
                .unwrap();
        });
        let _ = tfn.run_once();
    }
}
