use std::mem;
use nalgebra_glm::identity;
use crate::components::renderable::Renderable;
use crate::components::transform::Transform;
use crate::ecs::OldWorld;
use crate::physics::mass_influence::MassInfluence;
use crate::physics::rigid_body::RigidBody;
use crate::physics::spacetime_curvature::SpacetimeCurvature;
use crate::tags::SpacetimeMeshTag;
const MAX_MASSES: usize = 1024;

#[repr(C, align(16))]
#[derive(Clone, Copy, Default, Debug)]
struct MassData {
    position: [f32; 4],
    mass: f32,
    radius: f32,
    intensity: f32,
    _padding: f32, // Ensure 32-byte alignment
}

pub fn run(world: &OldWorld) {
    let view_matrix = world.view_matrix.unwrap_or(identity());
    let projection_matrix = world.projection_matrix.unwrap_or(identity());
    let resource_manager = &world.resource_manager;

    let (spacetime_renderable, _) = world.query::<(&Renderable, &SpacetimeMeshTag)>().next().unwrap();

    let grid_mesh = resource_manager.get_any_mesh(spacetime_renderable.mesh)
        .expect("Grid mesh not found");
    let shader = resource_manager.get_shader(spacetime_renderable.shader)
        .expect("Curvature shader not found");

    let influences: Vec<_> = world.query::<(&Transform, &RigidBody, &SpacetimeCurvature)>()
        .map(|(t, rb, sc)| MassData {
            position: [t.position.x, t.position.y, t.position.z, 0.0],
            mass: rb.mass,
            radius: sc.radius,
            intensity: sc.intensity,
            _padding: 0.0
        })
        .collect();

    //println!("{:?}", influences);

    /*assert_eq!(
        mem::size_of::<MassData>(),
        32, // 4 (vec4) + 4 (3 floats) + 4 (padding) = 32 bytes
        "UBO struct has incorrect size"
    );*/

    // 2. Create and bind UBO
    unsafe {
        // Create UBO with binding index 1
        let ubo = shader.create_ubo(1, &influences);

        // Bind UBO to binding point 1
        shader.bind_ubo(ubo, 1);
    }

    // 3. Set global uniforms
    shader.use_program();

    shader.set_mat4("view", &view_matrix);
    shader.set_mat4("projection", &projection_matrix);
    shader.set_int("mass_count", influences.len() as i32);
    shader.set_float("global_intensity", 5.0); // Example value
    shader.set_float("time", world.delta_time);

    // 4. Render global grid
    crate::graphics::draw_lines(grid_mesh, None);
}