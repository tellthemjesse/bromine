use std::sync::OnceLock;
use crate::types::{EcsWorld, Transform, Renderable, RigidBody};
use crate::physics::spacetime_curvature::SpacetimeCurvature;
use crate::tags::SpacetimeMeshTag;
use nalgebra_glm::identity;

#[repr(C, align(16))]
#[derive(Clone, Copy, Default, Debug)]
struct MassData {
    position: [f32; 4],
    mass: f32,
    radius: f32,
    intensity: f32,
    _pad: f32,
}

static UBO: OnceLock<u32> = OnceLock::new();

pub fn run(world: &EcsWorld) {
    let view_matrix = world.view_matrix.unwrap_or(identity());
    let projection_matrix = world.projection_matrix.unwrap_or(identity());
    let resource_manager = &world.resource_manager;

    let (spacetime_renderable, _) = world.query::<(&Renderable, &SpacetimeMeshTag)>().next().unwrap();

    let grid_mesh = resource_manager.get_any_mesh(spacetime_renderable.mesh)
        .unwrap();
    let shader = resource_manager.get_shader(spacetime_renderable.shader)
        .unwrap();

    let influences: Vec<_> = world.query::<(&Transform, &RigidBody, &SpacetimeCurvature)>()
        .map(|(t, rb, sc)| MassData {
            position: [t.position.x, t.position.y, t.position.z, 0.0],
            mass: rb.mass,
            radius: sc.radius,
            intensity: sc.intensity,
            ..Default::default()
        })
        .collect();

    unsafe {
        match UBO.get() {
            Some(ubo) => {
                shader.update_ubo(*ubo, &influences);
            }
            None => {
                let ubo = shader.create_ubo(&influences);
                shader.bind_ubo(ubo, 1);
                let _ = UBO.set(ubo);
            }
        }
        
        shader.use_program();
    
        shader.set_mat4("view", &view_matrix);
        shader.set_mat4("projection", &projection_matrix);
        shader.set_int("mass_count", influences.len() as i32);
        shader.set_float("global_intensity", 5.0);
        shader.set_float("time", world.delta_time);
    
        crate::graphics::draw_lines(grid_mesh, None);
    }
}
