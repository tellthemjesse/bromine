use nalgebra_glm::{Vec3, Vec4};

pub struct MassInfluence {
    pub position: Vec3,
    pub mass: f32,
    pub radius: f32,
    pub intensity: f32,
}

impl MassInfluence {
    fn as_vec4(&self) -> Vec4 {
        Vec4::new(
            self.position.x,
            self.position.y,
            self.position.z,
            self.mass
        )
    }
}