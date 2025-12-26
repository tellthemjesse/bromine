use bevy_ecs::prelude::Component;
use glam::{Mat4, Quat, Vec3, vec3};
use crate::render::asset_storage::asset_storage::{AssetID, MeshHandle, TextureHandle};
use crate::render::shader_storage::ShaderHandle;

// TODO: Implement materials
pub struct Material;
pub struct MaterialHandle(AssetID);

#[derive(Component)]
pub struct Renderable {
    pub mesh: MeshHandle,
    pub texture: Option<TextureHandle>,
    pub material: Option<MaterialHandle>,
    pub shader_to_use: Option<ShaderHandle>,
    pub is_visible: bool
}

impl Renderable {
    fn make_visible(&mut self) {
        self.is_visible = true;
    }

    fn make_invisible(&mut self) {
        self.is_visible = false;
    }

    fn new(mesh: MeshHandle) -> Self {
        Renderable {
            mesh,
            texture: None,
            material: None,
            shader_to_use: None,
            is_visible: true
        }
    }

    fn with_texture(mut self, texture: TextureHandle) -> Self {
        self.texture = Some(texture);
        self
    }

    fn with_material(mut self, material: MaterialHandle) -> Self {
        self.material = Some(material);
        self
    }

    fn with_shader(mut self, shader: ShaderHandle) -> Self {
        self.shader_to_use = Some(shader);
        self
    }
}

#[derive(Component)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    pub fn new(translation: Vec3) -> Self {
        Transform {
            translation,
            ..Default::default()
        }
    }

    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }

    pub fn matrix(&self) -> Mat4 {
        let translation = Mat4::from_translation(self.translation);
        let rotation = Mat4::from_quat(self.rotation);
        let scale = Mat4::from_scale(self.scale);

        translation * rotation * scale
    }
}