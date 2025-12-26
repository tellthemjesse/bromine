use bevy_ecs::resource::Resource;

#[derive(Resource)]
// Separating data and logic means that we'll use a system to update these angles
pub struct CameraStateRes {
    pub yaw: f32,
    pub pitch: f32,
    // Visual effects, that don't really affect camera
    pub roll: f32,
    pub visual_pitch: f32,
}