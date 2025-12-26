// --- Module responsible for collision detection & handling ---
pub mod collision_detection_system;
pub mod collider_update_system;
pub mod collision_handle_system;
//pub mod __speculative_contacts_system;

mod collider_component;
pub mod collider_cache;

pub use collider_component::Collider3D;