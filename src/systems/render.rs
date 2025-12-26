// System responsible for drawing renderable entities
use gl;
use nalgebra_glm::{Mat4, identity};
use obj::TexturedVertex;
use crate::ecs::world::EcsWorld;
use crate::components::transform::Transform;
use crate::components::renderable::Renderable;
use crate::graphics::mesh::{Mesh, Texture};

pub fn run(world: &EcsWorld) {
    // Get base view matrix
    let base_view_matrix: Mat4 = world.view_matrix.unwrap_or_else(|| {
        eprintln!("[Warning]: RenderSystem is missing view matrix. Identity matrix will be used");
        identity()
    });
    // Get projection matrix, type annotations are required for identity() fn call
    let projection_matrix: Mat4 = world.projection_matrix.unwrap_or_else(|| {
        eprintln!("[Warning]: RenderSystem is missing projection matrix. Identity matrix will be used");
        identity()
    });
    
    // Get effect values from CameraState
    let camera_roll = world.camera_state.roll;
    let camera_visual_pitch = world.camera_state.visual_pitch;
    // Add shake later if needed
    // let camera_shake = world.camera_state.shake_offset;

    let resource_manager = &world.resource_manager;

    // Query for renderable entities
    for (transform, renderable) in world.query::<(&Transform, &Renderable)>() {
        if !renderable.is_visible {
            continue;
        }

        //println!("{:?}", renderable);

        // Get the required resources using IDs from Renderable component
        let mesh = match resource_manager.get_any_mesh(renderable.mesh) {
            Some(m) => m,
            None => {
                eprintln!("RenderSystem Error: Missing mesh with ID {}", renderable.mesh);
                continue;
            }
        };
        let shader = match resource_manager.get_shader(renderable.shader) {
            Some(s) => s,
            None => {
                eprintln!("RenderSystem Error: Missing shader with ID {}", renderable.shader);
                continue;
            }
        };

        let mut texture_to_bind: Option<&Texture> = None;
        if let Some(id) = renderable.texture {
            //println!("Got texture with ID: {}", id);
            texture_to_bind = resource_manager.get_texture(id);
        }

        // 1. Setup Shader
        shader.use_program();
        
        // 4. Calculate Model Matrix
        let model_matrix = transform.to_matrix();

        // 5. Set Uniforms
        shader.set_mat4("model", &model_matrix);
        shader.set_mat4("view", &base_view_matrix);
        shader.set_mat4("projection", &projection_matrix);
        shader.set_float("u_cameraRoll", camera_roll);
        shader.set_float("u_cameraVisualPitch", camera_visual_pitch);

        crate::graphics::draw(mesh, texture_to_bind);
    }
} 