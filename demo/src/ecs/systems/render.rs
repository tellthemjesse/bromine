use crate::ecs::{
    components::{Camera, Model, Position},
    resources::{Projection, SceneProgram, Time, View},
};
use engine::{
    ecs::World,
    query, query_resource,
    render::prelude::{GlslDatatype, Renderable, UniformValue},
};

pub fn s_render(world: &mut World) {
    let (prog, time, view, proj) = query_resource!(world, SceneProgram, Time, View, Projection);

    let query;
    let index;
    query!(world, Position, with Camera, out(query), entity(index));

    let position = query[index].as_ref().unwrap();
    let u_view_pos = UniformValue::new(
        "u_ViewPos".to_string(),
        GlslDatatype::Vec3,
        position.as_ref().as_ptr(),
    );
    let u_time = UniformValue::new("u_Time".to_string(), GlslDatatype::Float, time.value());
    let u_view = UniformValue::new(
        "u_View".to_string(),
        GlslDatatype::Mat4,
        view.value().as_ref().as_ptr(),
    );
    let u_proj = UniformValue::new(
        "u_Proj".to_string(),
        GlslDatatype::Mat4,
        proj.value().as_ref().as_ptr(),
    );

    unsafe {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }

    prog.0.bind();
    prog.0.set_uniform(u_view);
    prog.0.set_uniform(u_time);
    prog.0.set_uniform(u_proj);
    prog.0.set_uniform(u_view_pos);

    for model in query!(world, Model).iter().flatten() {
        model.draw();
    }

    prog.0.unbind();
}
