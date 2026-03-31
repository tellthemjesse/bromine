use crate::ecs::{components::Model, resources::{Projection, Time, SceneProgram, View}};
use engine::{ecs::World, query, query_resource, render::prelude::Renderable};

pub fn s_render(world: &mut World) {
    let (prog, time, view, proj) =
        query_resource!(world, SceneProgram, Time, View, Projection);

    unsafe {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }

    prog.0.bind();
    prog.0.uniform_value(*view);
    prog.0.uniform_value(*time);
    prog.0.uniform_value(*proj);
    
    for model_opt in query!(world, Model).iter() {
        if let Some(model) = model_opt {
            model.draw();
        }
    }
}
