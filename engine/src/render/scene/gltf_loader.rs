use crate::render::{
    gl_mesh::GlMesh,
    gl_model::GlModel,
    prelude::{BufferObjDesc, BufferObjKind, BufferUsage},
    vertex::{AttributeKind, Vertex, VertexAttrib},
};

/// Vertex with position and normal, position location = 0, normal location = 1
#[derive(Debug)]
pub struct RawVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

impl Vertex for RawVertex {
    fn attributes() -> impl IntoIterator<Item = VertexAttrib> {
        let stride = std::mem::size_of::<RawVertex>();
        let normalized = false;

        [
            VertexAttrib {
                index: 0,
                size: 3,
                kind: AttributeKind::Float,
                normalized,
                stride,
                offset: std::mem::offset_of!(RawVertex, position),
            },
            VertexAttrib {
                index: 1,
                size: 3,
                kind: AttributeKind::Float,
                normalized,
                stride,
                offset: std::mem::offset_of!(RawVertex, normal),
            },
        ]
    }
}

pub struct GlftFile {}

impl GlftFile {
    pub fn get_models<P: AsRef<std::path::Path>>(path: P) -> Vec<GlModel> {
        let (documnet, buffers, _) = gltf::import(path).unwrap();

        let mut models = vec![];

        for node in documnet.nodes() {
            if let Some(model) = node.mesh() {
                let mut gl_model = GlModel { meshes: vec![] };

                for mesh in model.primitives() {
                    let reader = mesh.reader(|buffer| Some(&buffers[buffer.index()]));

                    let positions = reader.read_positions().unwrap();
                    let normals = reader.read_normals().unwrap();

                    let mut vertices = vec![];

                    positions.zip(normals).for_each(|pair| {
                        vertices.push(RawVertex {
                            position: pair.0,
                            normal: pair.1,
                        });
                    });

                    let primitive = mesh.mode().as_gl_enum().try_into().unwrap();
                    let desc = BufferObjDesc::new(BufferObjKind::Vertex, BufferUsage::StaticDraw);

                    let mut mesh = GlMesh::new(vertices, desc, primitive).unwrap();

                    if let Some(indices_data) = reader.read_indices() {
                        mesh = mesh
                            .with_element_buffer(
                                indices_data.into_u32().collect(),
                                BufferObjDesc::new(BufferObjKind::Element, BufferUsage::StaticDraw),
                            )
                            .unwrap();
                    }

                    gl_model.meshes.push(mesh);
                }

                models.push(gl_model);
            }
        }

        models
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model() {
        let display = gl_headless::build_display();
        display.load_gl();
        
        let path = format!(
            "{}/resources/monkey/scene.gltf",
            std::env!("CARGO_MANIFEST_DIR")
        );

        let _models = GlftFile::get_models(path);
    }
}
