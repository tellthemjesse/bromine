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
            yaw: -90.0_f32.to_radians(),
            pitch: 0.0,
            roll: 0.0,
        }
    }
}

impl Camera {
    pub fn angles(&self) -> (f32, f32, f32) {
        (self.yaw, self.pitch, self.roll)
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
