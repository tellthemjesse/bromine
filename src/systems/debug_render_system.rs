use gl::{DEPTH_TEST, Disable, Enable, FILL, FRONT_AND_BACK, LINE, PolygonMode};
use nalgebra_glm::{identity, Mat4, vec3};
use crate::types::Collider3D;
use crate::components::renderable::Renderable;
use crate::components::transform::Transform;
use crate::ecs::EcsWorld;
use crate::tags::DebugTag;

pub fn run(world: &EcsWorld) {
    // Get view matrix
    let view_matrix: Mat4 = world.view_matrix.unwrap_or_else(|| {
        eprintln!("[Warning]: RenderSystem is missing view matrix. Identity matrix will be used");
        identity()
    });
    // Get projection matrix, type annotations are required for identity() fn call
    let projection_matrix: Mat4 = world.projection_matrix.unwrap_or_else(|| {
        eprintln!("[Warning]: RenderSystem is missing projection matrix. Identity matrix will be used");
        identity()
    });

    let resource_manager = &world.resource_manager;
    let query_result = world.query::<(&Renderable, &DebugTag)>().next();
    if let Some((debug_renderable, _)) = query_result {
        let debug_mesh = match resource_manager.get_any_mesh(debug_renderable.mesh) {
            Some(m) => m,
            None => {
                return;
            }
        };

        let shader = match resource_manager.get_shader(debug_renderable.shader) {
            Some(s) => s,
            None => {
                return;
            }
        };

        for (transform, renderable, collider) in world.query::<(&Transform, &Renderable, &Collider3D)>() {
            if !renderable.is_visible {
                continue;
            }

            // 1. Setup Shader
            shader.use_program();
            
            // 5. Set Uniforms (including new ones for GPU calculation)
            shader.set_mat4("view", &view_matrix);
            shader.set_mat4("projection", &projection_matrix);
            shader.set_vec3("debugColor", &vec3(1.0, 0.0, 0.0)); // Keep it red
            shader.set_vec3("colliderCenter", &collider.center);
            shader.set_vec3("colliderSize", &(collider.radius)); // Send full size

            // 6. Set OpenGL state for wireframe drawing & disable depth test
            unsafe {
                // Disable depth test to ensure wireframe is always visible
                Disable(DEPTH_TEST); 
                // Set polygon mode to wireframe
                PolygonMode(FRONT_AND_BACK, LINE);

                // Draw the debug mesh (wireframe box)
                // Pass None for texture, debug mesh doesn't use one
                crate::graphics::draw(debug_mesh, None);

                // Reset OpenGL state
                PolygonMode(FRONT_AND_BACK, FILL); // Back to filled polygons
                Enable(DEPTH_TEST); // Re-enable depth testing
            }
        }
    }
}
