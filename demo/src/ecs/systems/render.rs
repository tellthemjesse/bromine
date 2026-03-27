use crate::ecs::resources::{Projection, Time, Triangle, TriangleProgram, View};
use engine::{ecs::World, query_resource, render::prelude::Renderable};

pub fn s_render(world: &mut World) {
    let (prog, triangle, time, view, proj) =
        query_resource!(world, TriangleProgram, Triangle, Time, View, Projection);

    unsafe {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }

    prog.0.bind();
    prog.0.uniform_value(*view);
    prog.0.uniform_value(*time);
    prog.0.uniform_value(*proj);
    triangle.0.draw();
}
