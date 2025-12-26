use bevy_ecs::resource::Resource;
use glam::Mat4;

#[derive(Resource)]
pub struct ViewMatrix(Mat4);

#[derive(Resource)]
pub struct ProjectionMatrix(Mat4);