use std::any::TypeId;

pub enum StorageType {
    State,
    Cache,
}
// Storage is basically a component, that needs to be accessed via query, so why separate those ?
pub trait Storage {
    const STORAGE_TYPE: StorageType;
    //const UNDERLYING_TAG: TypeId;
}

impl Storage for crate::resources::input_state::InputState {
    const STORAGE_TYPE: StorageType = StorageType::State;
}

impl Storage for crate::camera::camera_state::CameraState {
    const STORAGE_TYPE: StorageType = StorageType::State;
}

impl Storage for crate::collision::collider_cache::ColliderCache {
    const STORAGE_TYPE: StorageType = StorageType::Cache;
}