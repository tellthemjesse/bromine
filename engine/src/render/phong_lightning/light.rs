use crate::ecs::Component;
use crate::impl_component;
use mint::{Point3, Vector3};

#[derive(Debug, Clone, Copy)]
pub struct Light {
    pub specular: [f32; 3],
    pub diffuse: [f32; 3],
    pub ambient: [f32; 3],
}

impl_component!(Light);

impl Light {
    pub fn new(specular: f32, diffuse: f32, ambient: f32) -> Self {
        Self {
            specular: [specular; 3],
            diffuse: [diffuse; 3],
            ambient: [ambient; 3],
        }
    }

    pub fn new_chromatic(specular: [f32; 3], diffuse: [f32; 3], ambient: [f32; 3]) -> Self {
        Self {
            specular,
            diffuse,
            ambient,
        }
    }
}

impl Default for Light {
    fn default() -> Self {
        Self {
            specular: [1.0; 3],
            diffuse: [1.0; 3],
            ambient: [1.0; 3],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PointLight {
    pub light: Light,
    pub position: Point3<f32>,
}

impl PointLight {
    pub fn new(light: Light, position: Point3<f32>) -> Self {
        Self { light, position }
    }
}

impl_component!(PointLight);

#[derive(Debug, Clone, Copy)]
pub struct DirectionLight {
    pub light: Light,
    pub direction: Vector3<f32>,
}

impl DirectionLight {
    pub fn new(light: Light, direction: Vector3<f32>) -> Self {
        Self { light, direction }
    }
}

impl_component!(DirectionLight);

// todo: implement uniform trait
