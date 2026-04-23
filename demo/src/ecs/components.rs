use engine::ecs::Component;
use engine::impl_component;
use engine::render::GlModel;
use glam::Vec3;
use std::ops::AddAssign;

use crate::impl_newtype;

#[derive(Debug)]
pub struct Camera {
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
        }
    }
}

impl Camera {
    /// Returns normalized forward vector for this camera
    pub fn forward(&self) -> Vec3 {
        Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize()
    }
    /// Returns normalized right vector for this camera
    pub fn right(&self) -> Vec3 {
        self.forward().cross(Vec3::Y).normalize()
    }
    /// Returns normalized up vector for this camera
    pub fn up(&self) -> Vec3 {
        self.right().cross(self.forward()).normalize()
    }
}

impl_component!(Camera);

#[derive(Debug)]
pub struct Position(Vec3);

impl AddAssign<Vec3> for Position {
    fn add_assign(&mut self, rhs: Vec3) {
        self.0 += rhs;
    }
}

impl_newtype!(Position, Vec3);
impl_component!(Position);

pub struct Model(GlModel);
impl_newtype!(Model, GlModel);
impl_component!(Model);
