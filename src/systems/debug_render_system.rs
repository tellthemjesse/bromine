use crate::types::{EcsWorld, Transform, Renderable, Collider3D};
use nalgebra_glm::{Mat4, vec3};
use crate::tags::DebugTag;

pub fn run(world: &EcsWorld) {
    let view_matrix = world.view_matrix.unwrap_or_else(|| {
        tracing::warn!("view matrix is missing");
        Mat4::identity()
    });
    let projection_matrix = world.projection_matrix.unwrap_or_else(|| {
        tracing::warn!("view matrix is missing");
        Mat4::identity()
    });

    let resource_manager = &world.resource_manager;
    let query_result = world.query::<(&Renderable, &DebugTag)>().next();
    if let Some((debug_renderable, _)) = query_result {
        let debug_mesh = resource_manager.get_any_mesh(debug_renderable.mesh);
        let shader = resource_manager.get_shader(debug_renderable.shader);

        if debug_mesh.is_none() || shader.is_none() {
            tracing::warn!("debug renderer is used, but resources are not present");
            return;
        }

        let debug_mesh = debug_mesh.unwrap();
        let shader = shader.unwrap();

        for (_, renderable, collider) in world.query::<(&Transform, &Renderable, &Collider3D)>() {
            if !renderable.is_visible {
                continue;
            }

            shader.use_program();

            shader.set_mat4("view", &view_matrix);
            shader.set_mat4("projection", &projection_matrix);
            shader.set_vec3("debugColor", &vec3(1.0, 0.0, 0.0));
            shader.set_vec3("colliderCenter", &collider.center);
            shader.set_vec3("colliderSize", &(collider.radius));

            unsafe {
                gl::Disable(gl::DEPTH_TEST);
                gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

                crate::graphics::draw(debug_mesh, None);

                gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
                gl::Enable(gl::DEPTH_TEST);
            }
        }
    }
}
