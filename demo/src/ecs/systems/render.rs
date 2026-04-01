use crate::ecs::{components::{Camera, Position, Model}, resources::{Projection, Time, SceneProgram, View}};
use engine::{ecs::World, query, query_resource, render::prelude::{Renderable, Uniform, UniformKind}};

pub fn s_render(world: &mut World) {
    let (prog, time, view, proj) =
        query_resource!(world, SceneProgram, Time, View, Projection);
    
    let query;
    let index;
    query!(world, Position, with Camera, out(query), entity(index));
    
    let position = query[index].as_ref().unwrap();
    let u_view = Uniform {
        name: "u_ViewPos".to_string(),
        kind: UniformKind::Vec3,
        value_ptr: position.as_ref().as_ptr(),
    };
    
    unsafe {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }

    prog.0.bind();
    prog.0.uniform_value_t(*view);
    prog.0.uniform_value_t(*time);
    prog.0.uniform_value_t(*proj);
    prog.0.uniform_value_s(u_view);
    
    for model_opt in query!(world, Model).iter() {
        if let Some(model) = model_opt {
            model.draw();
        }
    }
    
    prog.0.unbind();
}
