use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use bevy_ecs::resource::Resource;
use crate::opengl_backend::shader::Program;

pub type ProgramID = u32;

pub struct ShaderHandle(ProgramID);

#[derive(Resource)]
pub struct ShaderStorage {
    shaders: HashMap<ProgramID, Program>,
    next_id: AtomicU32
}

impl ShaderStorage {
    pub fn insert_shader(&mut self, shader: Program) -> ProgramID {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed) as ProgramID;
        self.shaders.insert(id, shader);
        id
    }

    pub fn get_shader(&self, shader: &ShaderHandle) -> Option<&Program> {
        self.shaders.get(&shader.0)
    }
}