use super::buffer_object::*;
use super::vertex::Vertex;
use anyhow::anyhow;
use std::{alloc::Layout, ffi::c_void};

#[derive(Debug)]
/// Represents OpenGL buffer object
pub struct GlBufferObject {
    id: u32,
    desc: BufferObjDesc,
    layout: Layout,
}

#[derive(Debug)]
/// Represents OpenGL vertex array
pub struct GlVertexArray {
    id: u32,
}

impl GlVertexArray {
    /// Returns the underlying object id
    pub fn id(&self) -> u32 {
        self.id
    }
}

impl GlBufferObject {
    /// Returns the underlying object id
    pub fn id(&self) -> u32 {
        self.id
    }
    /// Returns object descriptor
    pub fn desc(&self) -> &BufferObjDesc {
        &self.desc
    }
    /// Returns [`Layout`] of the buffered data, the one obtained with [`Layout::for_value`]
    /// in the [`new_buffer_object`] call
    pub fn layout(&self) -> &Layout {
        &self.layout
    }
}

/// Creates new buffer object from its descriptor and data of a type `T`
pub fn new_buffer_object<T>(data: Vec<T>, desc: BufferObjDesc) -> anyhow::Result<GlBufferObject> {
    let layout = Layout::for_value(&data);
    let mut buffer = 0;

    unsafe {
        gl::GenBuffers(1, &mut buffer);
        gl::BindBuffer(desc.kind as u32, buffer);
        gl::BufferData(
            desc.kind as u32,
            layout.size() as isize,
            data.as_ptr() as *const c_void,
            desc.usage as u32,
        );

        let err = gl::GetError();
        if err != gl::NO_ERROR {
            gl::DeleteBuffers(1, &buffer);
            gl::BindBuffer(desc.kind as u32, 0);
            return Err(anyhow!("failed to buffer data, err code: {err}"));
        }

        gl::BindBuffer(desc.kind as u32, 0);
    }

    Ok(GlBufferObject {
        id: buffer,
        desc,
        layout,
    })
}

/// Creates new vertex array for data type `T` that implementes trait [`Vertex`].
pub fn new_vertex_array<T: Vertex>() -> anyhow::Result<GlVertexArray> {
    let mut array = 0;

    unsafe {
        gl::GenVertexArrays(1, &mut array);
        gl::BindVertexArray(array);

        for attr in T::attributes() {
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
                gl::DeleteVertexArrays(1, &mut array);
                gl::BindVertexArray(0);
                return Err(anyhow!(
                    "failed to make an attribute pointer, err code: {err}"
                ));
            }
        }

        gl::BindVertexArray(0);
    }

    Ok(GlVertexArray { id: array })
}

#[cfg(test)]
mod tests {
    use super::super::vertex::{AttributeKind, VertexAttrib};
    use super::*;

    #[test]
    fn test_buffer_operations() {
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

            let v = MyVertex {
                position: [1.0, 0.0, 1.0],
            };
            let desc = BufferObjDesc::new(BufferObjKind::Vertex, BufferUsage::StaticDraw);

            let _vbo = new_buffer_object(v.position.into(), desc).unwrap();
            let _vao = new_vertex_array::<MyVertex>().unwrap();
        });
        let _ = tfn.run_once();
    }
}
