use crate::ecs::Component;
use crate::impl_component;

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub specular: [f32; 3],
    pub diffuse: [f32; 3],
    pub ambient: [f32; 3],
    pub shininess: f32,
}

impl_component!(Material);

impl Material {
    pub fn new(specular: f32, diffuse: f32, ambient: f32, shininess: f32) -> Self {
        Self {
            specular: [specular; 3],
            diffuse: [diffuse; 3],
            ambient: [ambient; 3],
            shininess,
        }
    }

    pub fn new_rgb(
        specular: [f32; 3],
        diffuse: [f32; 3],
        ambient: [f32; 3],
        shininess: f32,
    ) -> Self {
        Self {
            specular,
            diffuse,
            ambient,
            shininess,
        }
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            specular: [1.0; 3],
            diffuse: [1.0; 3],
            ambient: [1.0; 3],
            shininess: 1.0,
        }
    }
}

// todo: implement uniform trait
