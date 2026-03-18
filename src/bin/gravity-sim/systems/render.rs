use crate::graphics::mesh::Texture;
use crate::types::{EcsWorld, Renderable, Transform};
use nalgebra_glm::Mat4;

pub fn run(world: &EcsWorld) {
    let base_view_matrix = world.view_matrix.unwrap_or_else(|| {
        tracing::warn!("view matrix is missing");
        Mat4::identity()
    });
    let projection_matrix = world.projection_matrix.unwrap_or_else(|| {
        tracing::warn!("projection matrix is missing");
        Mat4::identity()
    });

    let camera_roll = world.camera_state.roll;
    let camera_visual_pitch = world.camera_state.visual_pitch;

    let resource_manager = &world.resource_manager;

    for (transform, renderable) in world.query::<(&Transform, &Renderable)>() {
        if !renderable.is_visible {
            continue;
        }

        let mesh = match resource_manager.get_any_mesh(renderable.mesh) {
            Some(m) => m,
            None => {
                tracing::error!("missing mesh with ID {}", renderable.mesh);
                continue;
            }
        };
        let shader = match resource_manager.get_shader(renderable.shader) {
            Some(s) => s,
            None => {
                tracing::error!("missing shader with ID {}", renderable.shader);
                continue;
            }
        };

        let mut texture_to_bind: Option<&Texture> = None;
        if let Some(id) = renderable.texture {
            texture_to_bind = resource_manager.get_texture(id);
        }

        shader.use_program();

        let model_matrix = transform.to_matrix();

        shader.set_mat4("model", &model_matrix);
        shader.set_mat4("view", &base_view_matrix);
        shader.set_mat4("projection", &projection_matrix);
        shader.set_float("u_cameraRoll", camera_roll);
        shader.set_float("u_cameraVisualPitch", camera_visual_pitch);

        crate::graphics::draw(mesh, texture_to_bind);
    }
}
