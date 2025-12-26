use bevy_ecs::system::{Query, Res};
use crate::render::ecs::{Renderable, Transform};
use crate::render::ecs::resources::{ProjectionMatrix, ViewMatrix};
// use crate::render::Renderer;

// pub fn render_entities(query: Query<(&Transform, &Renderable)>, renderer: Res<&Renderer>, global: Res<(&ViewMatrix, &ProjectionMatrix)>) {
//     todo!()
// }