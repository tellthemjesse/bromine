use std::sync::Arc;
use gl::types::GLuint;
use glam::{Mat4, Vec3};
use crate::opengl_backend::shader::Program;
use crate::render::asset_storage::asset_storage::TextureHandle;

#[derive(Default)]
pub struct RenderCommandQueue {
    commands: Vec<RenderCommand>,
    // Add per-frame data like uniform buffers
}

impl RenderCommandQueue {
    // Submit a command to the queue
    pub fn submit(&mut self, command: RenderCommand) {
        self.commands.push(command);
    }

    // Execute all commands, then clear the queue
    pub fn flush(&mut self) {
        let reversed = self.commands.iter().rev();

        reversed.for_each(|cmd| {
            match cmd {
                RenderCommand::BindShader(_) => {}
                RenderCommand::BindTexture(_) => {}
                RenderCommand::DrawMesh { .. } => {}
                RenderCommand::SetMat4(_) => {}
                RenderCommand::SetVec3(_) => {}
            }
        });

        self.commands = Vec::new();
    }
}

pub enum RenderCommand {
    BindShader(Arc<Program>),
    BindTexture(Arc<TextureHandle>),
    DrawMesh { vao: GLuint, count: usize },
    SetMat4(Arc<Mat4>),
    SetVec3(Arc<Vec3>),
}