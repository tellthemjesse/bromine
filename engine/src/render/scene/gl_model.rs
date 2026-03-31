use crate::render::{gl_mesh::GlMesh, prelude::Renderable};

#[derive(Debug)]
pub struct GlModel {
    pub meshes: Vec<GlMesh>,
}

impl Renderable for GlModel {
    fn draw(&self) {
        self.meshes.iter().for_each(|m| m.draw());
    }
}
