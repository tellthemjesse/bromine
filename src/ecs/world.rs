use crate::ecs::component::Component;
use crate::ecs::entity::Entity;
use crate::resources::manager::TypeErasedResourceMgr;
use crate::resources::input_state::InputState;
use crate::camera::camera_state::CameraState;
use nalgebra_glm::Mat4;
use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use crate::collision::collider_cache::ColliderCache;
use crate::ecs::query::{QueryIter, QueryIterMut, WorldQuery};

pub trait ComponentStorage: Any + Send + Sync + Debug {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn ensure_capacity(&mut self, entity_id: usize);
    fn remove(&mut self, entity_id: usize);
}

impl<T: Component + Debug> ComponentStorage for Vec<Option<T>> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn ensure_capacity(&mut self, entity_id: usize) {
        let required_len = entity_id + 1;
        if self.len() < required_len {
            self.resize_with(required_len, || None);
        }
    }

    fn remove(&mut self, entity_id: usize) {
        if entity_id < self.len() {
            self[entity_id] = None;
        }
    }
}

pub struct EcsWorld {
    next_entity_id: usize,
    pub components: HashMap<TypeId, Box<dyn ComponentStorage>>,
    pub entity_components: HashMap<Entity, Vec<TypeId>>,
    pub resource_manager: TypeErasedResourceMgr,
    pub view_matrix: Option<Mat4>,
    pub projection_matrix: Option<Mat4>,
    pub input_state: InputState,
    pub delta_time: f32,
    pub camera_state: CameraState,
    pub collider_cache: ColliderCache,
}

impl EcsWorld {
    // Create a new, empty World
    pub fn new() -> Self {
        EcsWorld {
            // Entities
            next_entity_id: 0,
            // Components storage
            components: HashMap::new(),
            entity_components: HashMap::new(),
            // Resources
            resource_manager: TypeErasedResourceMgr::new(),
            view_matrix: None,
            projection_matrix: None,
            input_state: InputState::new(),
            delta_time: 0.0,
            camera_state: CameraState::new(),
            collider_cache: HashSet::new(),
        }
    }

    // Create a new entity ID and update entity tracking
    fn allocate_entity_id(&mut self) -> Entity {
        let id = self.next_entity_id;
        self.next_entity_id += 1;
        // Initialize component list for the new entity
        let _ = self.entity_components.insert(id, Vec::new());
        id
    }

    // Create a new entity and return its ID
    pub fn create_entity(&mut self) -> Entity {
        self.allocate_entity_id()
    }

    // --- Generic Component Methods ---

    // Register a component type if it doesn't exist
    fn register_component<T: Component + Debug>(&mut self) {
        let type_id = TypeId::of::<T>();
        let _ = self.components.entry(type_id).or_insert_with(|| {
            // Create new storage for this component type
            Box::new(Vec::<Option<T>>::new())
        });
    }

    // Add any component to an entity
    pub fn add_component<T: Component + Debug>(&mut self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();

        // Ensure the component type is registered and get its storage
        self.register_component::<T>();
        let storage = self.components.get_mut(&type_id).unwrap();

        // Ensure the storage Vec is large enough
        storage.ensure_capacity(entity);

        // Update the entity's component list
        if let Some(components) = self.entity_components.get_mut(&entity) {
            if !components.contains(&type_id) {
                components.push(type_id);
            }
        } else {
            // This case might happen if entity ID was created outside create_entity?
            // Or if entity was deleted and reused. For now, assume create_entity was called.
            // If this happens, we might need to re-insert the entity here.
            eprintln!("Warning: Adding component to untracked entity {}", entity);
            let _ = self.entity_components.insert(entity, vec![type_id]);
        }

        // Downcast the storage to the specific Vec<Option<T>> and insert component
        if let Some(vec) = storage.as_any_mut().downcast_mut::<Vec<Option<T>>>() {
            vec[entity] = Some(component);
        } else {
            // This should theoretically never happen if registration worked
            panic!("Component storage type mismatch after registration!");
        }
    }

    // Get an immutable reference to a component
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.components.get(&type_id)
            .and_then(|storage| storage.as_any().downcast_ref::<Vec<Option<T>>>())
            .and_then(|vec| vec.get(entity))
            .and_then(|opt| opt.as_ref())
    }

    // Get a mutable reference to a component
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.components.get_mut(&type_id)
            .and_then(|storage| storage.as_any_mut().downcast_mut::<Vec<Option<T>>>())
            .and_then(|vec| vec.get_mut(entity))
            .and_then(|opt| opt.as_mut())
    }

    // Remove a component from an entity
    pub fn remove_component<T: Component>(&mut self, entity: Entity) {
        let type_id = TypeId::of::<T>();

        // Remove from the storage vector
        if let Some(storage) = self.components.get_mut(&type_id) {
             storage.remove(entity);
        }

        // Remove from the entity's component list
        if let Some(components) = self.entity_components.get_mut(&entity) {
            components.retain(|&id| id != type_id);
        }
    }

    // Check if an entity has a specific component
    pub fn has_component<T: Component>(&self, entity: Entity) -> bool {
        let type_id = TypeId::of::<T>();
        self.entity_components.get(&entity)
            .map_or(false, |components| components.contains(&type_id))
    }

    // --- Query Methods ---

    // Immutable query method
    pub fn query<'w, Q: WorldQuery<'w>>(&'w self) -> QueryIter<'w, Q> {
        QueryIter::new(self)
    }

    // Mutable query method
    // Returns an iterator yielding mutable references
    pub fn query_mut<'w, Q: WorldQuery<'w>>(&'w mut self) -> QueryIterMut<'w, Q> {
        // Safety: Relies on QueryIterMut::new for safety checks
        QueryIterMut::new(self)
    }

    // --- Query specific entity ---
    pub fn query_entity<'w, Q: WorldQuery<'w>>(&'w self, entity_id: Entity) -> Option<Q::Item> {
        // 1. Check if query itself is valid (no &mut T, &T conflicts within Q)
        Q::check_query_access();
        if Q::IS_MUTABLE {
            panic!("query_entity called with a query that requests mutable components. Use query_entity_mut for that.");
        }

        // 2. Check if the entity actually has all components required by Q
        let required_types = Q::get_component_type_ids();
        if let Some(entity_component_list) = self.entity_components.get(&entity_id) {
            let has_all_components = required_types.iter().all(|required_type| {
                entity_component_list.contains(required_type)
            });

            if has_all_components {
                // 3. If yes, fetch the components
                // Safety: We've checked IS_MUTABLE is false. Entity has components.
                unsafe { Q::fetch(&self.components, entity_id) }
            } else {
                None // Entity doesn't have all required components
            }
        } else {
            None // Entity doesn't exist or has no components registered
        }
    }

    pub fn query_entity_mut<'w, Q: WorldQuery<'w>>(&'w mut self, entity_id: Entity) -> Option<Q::Item> {
        // 1. Check query access
        Q::check_query_access();

        // 2. Check if entity has all components
        let required_types = Q::get_component_type_ids();
        // Immutable borrow for check first, to not conflict with mutable borrow later if entity_id is not found early
        if let Some(entity_component_list) = self.entity_components.get(&entity_id) {
            let has_all_components = required_types.iter().all(|required_type| {
                entity_component_list.contains(required_type)
            });

            if has_all_components {
                // 3. Fetch mutable components
                // Safety: Access is for a single entity. `check_query_access` validated Q.
                // The `&'w mut self` ensures exclusive access to `world` for this call.
                unsafe { Q::fetch_mut(&mut self.components, entity_id) }
            } else {
                None
            }
        } else {
            None
        }
    }

    // Retain the old query_entities for internal use by QueryIter/QueryIterMut
    pub(crate) fn query_entities(&self, component_types: &[TypeId]) -> Vec<Entity> {
        self.entity_components.iter()
            .filter_map(|(&entity_id, entity_component_list)| {
                // Check if the entity has ALL required component types
                if component_types.iter().all(|required_type| {
                    entity_component_list.contains(required_type)
                }) {
                    Some(entity_id)
                } else {
                    None
                }
            })
            .collect()
    }
}

impl std::fmt::Debug for EcsWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "COMPONENTS:\n{:?}\nENTITY_COMPONENTS: \n{:?}\n", self.components, self.entity_components)
    }
}