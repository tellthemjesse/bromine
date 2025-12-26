use gl::types::GLuint;
use crate::render::asset_storage::vertex::{GLVertexFormat, VertexType};

pub struct Mesh {
    pub vertices: Vec<VertexType>,
    pub indices: Vec<IndexType>,
    pub vertex_format: GLVertexFormat,
}

pub trait MeshBackend {
    fn upload_vertices() {

    }
}

pub enum IndexType {
    U16(u16),
    U32(u32),
}
