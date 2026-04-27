use crate::render::{
    gl_mesh::GlMesh, gl_model::GlModel, prelude::{BufferObjDesc, BufferObjKind, BufferUsage}, vertex::{AttributeKind, Vertex, VertexAttrib}
};
use std::path::Path;

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

pub struct GlftFile;

impl GlftFile {
    pub fn get_models<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<GlModel>> {
        let (documnet, buffers, _) = gltf::import(path)?;

        let mut models = vec![];

        for node in documnet.nodes() {
            if let Some(model) = node.mesh() {
                let mut gl_model = GlModel { meshes: vec![] };

                for mesh in model.primitives() {
                    let reader = mesh.reader(|buffer| Some(&buffers[buffer.index()]));

                    let positions = reader
                        .read_positions()
                        .expect("model did not contain positions");
                    let normals = reader
                        .read_normals()
                        .expect("model did not contain normals");

                    let mut vertices = Vec::with_capacity(positions.len());

                    positions.zip(normals).for_each(|(position, normal)| {
                        vertices.push(RawVertex { position, normal });
                    });

                    let primitive = mesh.mode().as_gl_enum().try_into()?;
                    let desc = BufferObjDesc::new(BufferObjKind::Vertex, BufferUsage::StaticDraw);

                    let mut mesh = GlMesh::new(vertices, desc, primitive)?;

                    if let Some(indices_data) = reader.read_indices() {
                        mesh = mesh.with_element_buffer(
                            indices_data.into_u32().collect(),
                            BufferObjDesc::new(BufferObjKind::Element, BufferUsage::StaticDraw),
                        )?;
                    }

                    gl_model.meshes.push(mesh);
                }

                models.push(gl_model);
            }
        }

        Ok(models)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model() -> Result<(), anyhow::Error> {
        let display = gl_headless::build_display();
        display.load_gl();

        let path = format!(
            "{}/resources/monkey/scene.gltf",
            std::env!("CARGO_MANIFEST_DIR")
        );

        GlftFile::get_models(path)?;

        Ok(())
    }
}
