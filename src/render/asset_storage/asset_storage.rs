use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use bevy_ecs::prelude::{Component, Resource};
use crate::render::asset_storage::mesh::Mesh;
use crate::render::asset_storage::texture::Texture;

pub type AssetID = u32;

// Handles are unique ID's that Renderable component holds
#[derive(Component)]
pub struct MeshHandle(AssetID);

#[derive(Component)]
pub struct TextureHandle(AssetID);

#[derive(Resource)]
pub struct AssetStorage {
    meshes: HashMap<AssetID, Mesh>,
    textures: HashMap<AssetID, Texture>,
    next_id: AtomicU32,
}

impl AssetStorage {
    fn insert_mesh(&mut self, mesh: Mesh) -> MeshHandle {
        todo!();
        let id = self.next_id.fetch_add(1, Ordering::Relaxed) as AssetID;
        // self.meshes.push(mesh);
        MeshHandle(id)
    }

    fn insert_texture(&mut self, texture: Texture) -> TextureHandle {
        todo!();
        let id = self.next_id.fetch_add(1, Ordering::Relaxed) as AssetID;
        // self.textures.push(texture);
        TextureHandle(id)
    }

    fn get_mesh(&self, handle: &MeshHandle) -> Option<&Mesh> {
        self.meshes.get(&handle.0)
    }

    fn get_texture(&self, handle: &TextureHandle) -> Option<&Texture> {
        self.textures.get(&handle.0)
    }
}

