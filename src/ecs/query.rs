use crate::ecs::component::Component;
use crate::ecs::entity::Entity;
use crate::ecs::world::EcsWorld;
use std::any::TypeId;
use std::marker::PhantomData;
use std::ptr::NonNull; // For mutable borrows
use std::collections::HashMap;
use crate::ecs::world::ComponentStorage;

// TODO: Define query traits and iterator structs here 

// --- QueryItem: Represents a single fetchable item in a query --- 

// Helper trait to define the actual data type returned by a QueryItem
pub trait QueryItemData<'w> {
    type Item;
}

// Trait for fetching a single component type or metadata
// Needs lifetime 'w bound to the World borrow
pub unsafe trait QueryItem<'w>: QueryItemData<'w> {
    // Check if this item borrows mutably
    const IS_MUTABLE: bool;
    // Get the TypeId if this item refers to a component
    fn get_component_type_id() -> Option<TypeId>;
    
    // Fetch IMMUTABLE data. 
    // Panics if called on a mutable item type like &mut T.
    unsafe fn fetch(
        storages: &'w HashMap<TypeId, Box<dyn ComponentStorage>>,
        entity_id: Entity
    ) -> Option<Self::Item>;
    
    // Fetch potentially MUTABLE data using raw pointer.
    // Safety: Caller (iterator) must ensure pointer validity and exclusive access if IS_MUTABLE is true.
    unsafe fn fetch_mut(
        storages_ptr: *mut HashMap<TypeId, Box<dyn ComponentStorage>>,
        entity_id: Entity
    ) -> Option<Self::Item>;

    // Called by the WorldQuery to check access conflicts (e.g., multiple &mut T)
    fn check_access(type_ids: &mut Vec<TypeId>);
}

// --- Implementations for common QueryItems --- 

// Implementation for immutable borrow: &T
impl<'w, T: Component> QueryItemData<'w> for &'w T {
    type Item = &'w T;
}

unsafe impl<'w, T: Component> QueryItem<'w> for &'w T {
    const IS_MUTABLE: bool = false;
    
    fn get_component_type_id() -> Option<TypeId> {
        Some(TypeId::of::<T>())
    }
    
    unsafe fn fetch(
        storages: &'w HashMap<TypeId, Box<dyn ComponentStorage>>,
        entity_id: Entity
    ) -> Option<Self::Item> {
        let type_id = TypeId::of::<T>();
        storages.get(&type_id)
            .and_then(|storage| storage.as_any().downcast_ref::<Vec<Option<T>>>())
            .and_then(|vec| vec.get(entity_id))
            .and_then(|opt| opt.as_ref())
    }
    
    unsafe fn fetch_mut(
        storages_ptr: *mut HashMap<TypeId, Box<dyn ComponentStorage>>,
        entity_id: Entity
    ) -> Option<Self::Item> {
        // Safely dereference the mutable pointer to get immutable access
        let storages = &*storages_ptr;
        // Call the regular fetch
        Self::fetch(storages, entity_id)
    }
    
    fn check_access(type_ids: &mut Vec<TypeId>) {
        // Immutable borrows are generally fine, just record the access
        if let Some(id) = Self::get_component_type_id() {
            type_ids.push(id);
        }
    }
}

// Implementation for mutable borrow: &mut T
impl<'w, T: Component> QueryItemData<'w> for &'w mut T {
    type Item = &'w mut T;
}

unsafe impl<'w, T: Component> QueryItem<'w> for &'w mut T {
    const IS_MUTABLE: bool = true;

    fn get_component_type_id() -> Option<TypeId> {
        Some(TypeId::of::<T>())
    }

    // This fetch CANNOT provide &mut T from &HashMap. Panic!
    unsafe fn fetch(
        _storages: &'w HashMap<TypeId, Box<dyn ComponentStorage>>,
        _entity_id: Entity
    ) -> Option<Self::Item> {
        panic!("Attempted to fetch mutable component {} using immutable query. Use query_mut().", std::any::type_name::<T>());
    }
    
    // fetch_mut CAN provide &mut T using the *mut HashMap pointer.
    unsafe fn fetch_mut(
        storages_ptr: *mut HashMap<TypeId, Box<dyn ComponentStorage>>,
        entity_id: Entity
    ) -> Option<Self::Item> {
        let type_id = TypeId::of::<T>();
        // Safely get mutable access from the pointer
        let mutable_storages = &mut *storages_ptr;

        // Now get mutable access to the specific Box<dyn ComponentStorage>
        mutable_storages.get_mut(&type_id)
            .and_then(|storage_box| storage_box.as_any_mut().downcast_mut::<Vec<Option<T>>>())
            .and_then(|vec| vec.get_mut(entity_id))
            .and_then(|opt| opt.as_mut())
            // Crucially, ensure the lifetime matches 'w
            .map(|comp_ref| &mut *(comp_ref as *mut T))
    }
    
    fn check_access(type_ids: &mut Vec<TypeId>) {
         if let Some(id) = Self::get_component_type_id() {
            // Check for conflicts: another borrow (mut or immut) of the same type
            if type_ids.contains(&id) {
                panic!("Query attempted to borrow component type {:?} multiple times (at least one mutably).", id);
            }
            type_ids.push(id);
        }
    }
}

// --- Implementations for Entity ID ---
impl<'w> QueryItemData<'w> for Entity {
    type Item = Entity;
}

unsafe impl<'w> QueryItem<'w> for Entity {
    const IS_MUTABLE: bool = false;

    fn get_component_type_id() -> Option<TypeId> {
        None // Entity is not a component
    }

    unsafe fn fetch(
        _storages: &'w HashMap<TypeId, Box<dyn ComponentStorage>>,
        entity_id: Entity
    ) -> Option<Self::Item> {
        Some(entity_id)
    }

    unsafe fn fetch_mut(
        _storages_ptr: *mut HashMap<TypeId, Box<dyn ComponentStorage>>,
        entity_id: Entity
    ) -> Option<Self::Item> {
        // Entity ID is not mutable, so this is same as fetch
        Some(entity_id)
    }

    fn check_access(_type_ids: &mut Vec<TypeId>) {
        // Accessing Entity ID does not conflict with component borrows
    }
}

// --- WorldQuery: Represents the entire query (often a tuple of QueryItems) ---

// Helper trait to define the combined Item type for a WorldQuery
pub trait WorldQueryData<'w> {
    type Item;
}

// Trait to define a valid query over the World
// Unsafe because fetch potentially uses unsafe QueryItem fetches
pub unsafe trait WorldQuery<'w>: WorldQueryData<'w> {
    // Is any item in this query mutable?
    const IS_MUTABLE: bool;
    // Gather all component TypeIds required by this query
    fn get_component_type_ids() -> Vec<TypeId>;
    
    // Check for access conflicts within the query itself (e.g., (&mut T, &T))
    fn check_query_access();

    // Fetch the combined item data for an entity (immutable context)
    // Safety: Caller must ensure entity has all required components.
    unsafe fn fetch(
        storages: &'w HashMap<TypeId, Box<dyn ComponentStorage>>,
        entity_id: Entity
    ) -> Option<Self::Item>;
    
    // Fetch the combined item data for an entity (mutable context)
    // Safety: Caller must ensure entity has all required components and pointer is valid.
    unsafe fn fetch_mut(
        storages_ptr: *mut HashMap<TypeId, Box<dyn ComponentStorage>>,
        entity_id: Entity
    ) -> Option<Self::Item>;
}

// --- Implement WorldQuery for Tuples --- 

// Macro to implement WorldQuery for tuples of QueryItems
// This avoids writing repetitive code for different tuple sizes
macro_rules! impl_world_query_tuple {
    ( $( $name:ident ),* ) => {
        // Implement WorldQueryData for the tuple
        impl<'w, $($name: QueryItem<'w>),*> WorldQueryData<'w> for ( $( $name, )* ) {
            type Item = ( $( $name::Item, )* );
        }

        // Implement WorldQuery for the tuple
        #[allow(non_snake_case)] // Allow variable names like T1, T2
        #[allow(unused_variables)] // Allow unused `storages` if tuple is empty
        unsafe impl<'w, $($name: QueryItem<'w>),*> WorldQuery<'w> for ( $( $name, )* ) {
            // Calculate if query is mutable based on its items
            // Use correct OR logic for boolean consts
            const IS_MUTABLE: bool = $( $name::IS_MUTABLE || )* false;
            
            fn get_component_type_ids() -> Vec<TypeId> {
                let mut type_ids = Vec::new();
                $( 
                    if let Some(id) = $name::get_component_type_id() {
                        type_ids.push(id);
                    }
                )*
                type_ids
            }
            
            fn check_query_access() {
                 let mut accessed_types: Vec<TypeId> = Vec::new();
                 $( 
                     $name::check_access(&mut accessed_types);
                 )*
            }

            // Immutable fetch
            unsafe fn fetch(
                storages: &'w HashMap<TypeId, Box<dyn ComponentStorage>>,
                entity_id: Entity
            ) -> Option<Self::Item> {
                // Fetch each item in the tuple using QueryItem::fetch
                $( 
                    let $name = $name::fetch(storages, entity_id)?;
                )*
                // Combine into a tuple and return
                Some(( $( $name, )* ))
            }
            
            // Mutable fetch
            unsafe fn fetch_mut(
                storages_ptr: *mut HashMap<TypeId, Box<dyn ComponentStorage>>,
                entity_id: Entity
            ) -> Option<Self::Item> {
                 // Fetch each item in the tuple using QueryItem::fetch_mut
                 $( 
                    let $name = $name::fetch_mut(storages_ptr, entity_id)?;
                )*
                // Combine into a tuple and return
                Some(( $( $name, )* ))
            }
        }
    };
}

// Implement WorldQuery for tuples 
//impl_world_query_tuple!(); // Empty tuple
// Leave it like that, it's valid. The macro already generates tuple is comma
impl_world_query_tuple!(Q1); 
impl_world_query_tuple!(Q1, Q2);
impl_world_query_tuple!(Q1, Q2, Q3);
impl_world_query_tuple!(Q1, Q2, Q3, Q4);
impl_world_query_tuple!(Q1, Q2, Q3, Q4, Q5);

// --- Query Iterator --- 

pub struct QueryIter<'w, Q: WorldQuery<'w>> {
    world_storages: &'w HashMap<TypeId, Box<dyn ComponentStorage>>,
    matching_entities: std::vec::IntoIter<Entity>, // Iterator over IDs that match the query
    _phantom: PhantomData<Q>, // Mark that we logically borrow Q
}

impl<'w, Q: WorldQuery<'w>> QueryIter<'w, Q> {
    // Internal constructor called by World::query
    pub(crate) fn new(world: &'w EcsWorld) -> Self {
        // First, check for incompatible accesses within the query itself
        Q::check_query_access(); 
        
        // PANIC if QueryIter is used to request mutable components
        if Q::IS_MUTABLE {
            panic!("Attempted to create immutable QueryIter for a query that requests mutable components. Use query_mut() instead.");
        }
        
        let required_types = Q::get_component_type_ids();
        let matching_entities = world.query_entities(&required_types).into_iter();
        
        Self {
            world_storages: &world.components, 
            matching_entities,
            _phantom: PhantomData,
        }
    }
}

// Implement the Iterator trait
impl<'w, Q: WorldQuery<'w>> Iterator for QueryIter<'w, Q> {
    type Item = Q::Item;

    fn next(&mut self) -> Option<Self::Item> {
        // Loop through matching entity IDs
        for entity_id in self.matching_entities.by_ref() {
            // Fetch the data for this entity using the WorldQuery implementation
            // Safety: 
            // 1. `new` checked for conflicting borrows within Q.
            // 2. `WorldQuery::fetch` relies on `QueryItem::fetch`.
            // 3. `QueryItem<&mut T>::fetch` uses unsafe pointer casts, BUT the iterator structure 
            //    ensures we process one entity at a time. Rust's borrow rules prevent the *iterator itself* 
            //    from being used in a way that would cause multiple live mutable borrows *of the same component* 
            //    across different `next()` calls simultaneously within the same scope. 
            //    The `unsafe` block handles aliasing *during* the single `fetch` call.
            // 4. We assume `world.query_entities` correctly identified entities that *have* the required components.
            // 5. We checked IS_MUTABLE is false in QueryIter::new, so this fetch is safe.
            let item = unsafe { Q::fetch(self.world_storages, entity_id) };
            
            // If fetch succeeded (it should if query_entities was correct), return it
            if item.is_some() {
                return item;
            }
            // If fetch failed (e.g., component unexpectedly missing), log error and try next entity
            else {
                 eprintln!("Error: Query failed to fetch components for entity {}, expected components: {:?}", entity_id, Q::get_component_type_ids());
            }
        }
        // No more matching entities found
        None
    }
}

// --- Mutable Query Iterator --- 

// Very similar to QueryIter, but holds mutable access to storages
pub struct QueryIterMut<'w, Q: WorldQuery<'w>> {
    // Needs exclusive access to the component storage
    world_storages: NonNull<HashMap<TypeId, Box<dyn ComponentStorage>>>, // Use NonNull for variance
    matching_entities: std::vec::IntoIter<Entity>, 
    _phantom: PhantomData<&'w mut Q>, // Phantom data uses mutable borrow lifetime
}

impl<'w, Q: WorldQuery<'w>> QueryIterMut<'w, Q> {
    // Internal constructor called by World::query_mut
    // Safety: Caller must ensure World is not borrowed elsewhere incompatibly.
    pub(crate) fn new(world: &'w mut EcsWorld) -> Self {
        Q::check_query_access(); 
        let required_types = Q::get_component_type_ids();
        // Need to borrow immutably first to call query_entities, then get mutable pointer
        let matching_entities = world.query_entities(&required_types).into_iter();
        let world_storages_ptr = NonNull::from(&mut world.components);
        
        Self {
            world_storages: world_storages_ptr,
            matching_entities,
            _phantom: PhantomData,
        }
    }
    
    // Helper to safely get mutable reference to storages
    // Safety: Must ensure this iterator has exclusive access via 'w
    unsafe fn storages_mut(&mut self) -> &'w mut HashMap<TypeId, Box<dyn ComponentStorage>> {
        &mut *self.world_storages.as_ptr()
    }
    
    // Helper to safely get immutable reference to storages
    // Safety: Must ensure this iterator has exclusive access via 'w
    unsafe fn storages(&self) -> &'w HashMap<TypeId, Box<dyn ComponentStorage>> {
        &*self.world_storages.as_ptr()
    }
}

// Implement the Iterator trait for QueryIterMut
impl<'w, Q: WorldQuery<'w>> Iterator for QueryIterMut<'w, Q> {
    type Item = Q::Item; // Should yield mutable references if Q requests them

    fn next(&mut self) -> Option<Self::Item> {
        // Loop until we find a valid entity or exhaust the iterator
        loop {
            // Get the next entity ID *without* holding onto the mutable borrow of self.matching_entities
            let entity_id = self.matching_entities.next()?;

            // Get the mutable pointer to storage
            let storages_ptr = self.world_storages.as_ptr();
            
            // Call the mutable fetch method
            match unsafe { Q::fetch_mut(storages_ptr, entity_id) } {
                Some(item) => return Some(item), // Found item, return it
                None => {
                    // Fetch failed for this entity (should be rare if query_entities is correct)
                    eprintln!("Error: QueryMut failed to fetch components for entity {}, expected components: {:?}", entity_id, Q::get_component_type_ids());
                    // Continue loop to try the next entity_id
                    continue; 
                }
            }
        }
    }
}

// TODO: Query Iterator struct 