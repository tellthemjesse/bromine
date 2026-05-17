use crate::{
    component::{Component, ComponentVec}, entity::Entity, hash::NoOpHash, resource::{Resource, ResourceSlot}
};
use std::{
    any::{TypeId, type_name}, cell::{Ref, RefCell, RefMut}, collections::HashMap
};

pub type TypeIdMap<V> = HashMap<TypeId, V, NoOpHash>;

pub struct World {
    pub entity_index: u32,
    pub resources: TypeIdMap<Box<dyn ResourceSlot>>,
    pub entity_components: TypeIdMap<Box<dyn ComponentVec>>,
    pub typenames: TypeIdMap<&'static str>,
}

impl Default for World {
    fn default() -> Self {
        Self {
            entity_index: 0,
            resources: TypeIdMap::with_hasher(NoOpHash),
            entity_components: TypeIdMap::with_hasher(NoOpHash),
            typenames: TypeIdMap::with_hasher(NoOpHash),
        }
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn spawn_entity(&mut self) -> Entity {
        let index = self.entity_index;
        self.entity_index += 1;
        Entity::from(index)
    }

    pub fn register_resourse<T>(&mut self, resource: T)
    where
        T: Resource,
    {
        let type_id = TypeId::of::<T>();
        let box_ptr = Box::new(RefCell::new(resource));
        if self.resources.insert(type_id, box_ptr).is_none() {
            let _ = self.typenames.insert(type_id, type_name::<T>());
        }
    }

    pub fn register_component<T>(&mut self, entity: Entity, component: T)
    where
        T: Component,
    {
        let index = entity.index() as usize;
        let type_id = TypeId::of::<T>();

        match self.entity_components.get_mut(&type_id) {
            // update existing row
            Some(dynamic_components) => {
                let mut components = dynamic_components
                    .as_any_mut()
                    .downcast_mut::<RefCell<Vec<Option<T>>>>()
                    .unwrap() // can't fail unless downcast type is wrong
                    .borrow_mut();

                // ensure that there's enough slots for new entity
                for _ in components.len()..self.entity_index as usize {
                    components.push(None);
                }

                components[index] = Some(component);
            }
            // register a new component
            None => {
                let mut components = Vec::<Option<T>>::with_capacity(self.entity_index as usize);
                for _ in 0..self.entity_index {
                    components.push(None);
                }

                components[index] = Some(component);
                let box_ptr = Box::new(RefCell::new(components));

                let _ = self.entity_components.insert(type_id, box_ptr);
                let _ = self.typenames.insert(type_id, type_name::<T>());
            }
        }
    }

    /// Mutably borrows a [`Resource`]
    pub fn borrow_resource<'w, T>(&'w self) -> Option<RefMut<'w, T>>
    where
        T: Resource,
    {
        self.resources
            .get(&TypeId::of::<T>())
            .and_then(|box_ptr| box_ptr.as_any().downcast_ref::<RefCell<T>>())
            .map(|cell| cell.borrow_mut())
    }

    /// Immutably borrows a [`Resource`]
    pub fn fetch_resource<'w, T>(&'w self) -> Option<Ref<'w, T>>
    where
        T: Resource,
    {
        self.resources
            .get(&TypeId::of::<T>())
            .and_then(|box_ptr| box_ptr.as_any().downcast_ref::<RefCell<T>>())
            .map(|cell| cell.borrow())
    }

    /// Mutably borrows a [`ComponentVec`]
    pub fn borrow_components<'w, T>(&'w self) -> Option<RefMut<'w, Vec<Option<T>>>>
    where
        T: Component,
    {
        self.entity_components
            .get(&TypeId::of::<T>())
            .and_then(|box_ptr| box_ptr.as_any().downcast_ref::<RefCell<Vec<Option<T>>>>())
            .map(|cell| cell.borrow_mut())
    }

    /// Immutably borrows a [`ComponentVec`]
    pub fn fetch_components<'w, T>(&'w self) -> Option<Ref<'w, Vec<Option<T>>>>
    where
        T: Component,
    {
        self.entity_components
            .get(&TypeId::of::<T>())
            .and_then(|box_ptr| box_ptr.as_any().downcast_ref::<RefCell<Vec<Option<T>>>>())
            .map(|cell| cell.borrow())
    }

    pub fn map_to_entities<T>(&self) -> Option<Vec<Option<usize>>>
    where
        T: Component,
    {
        self.entity_components
            .get(&TypeId::of::<T>())
            .and_then(|box_ptr| box_ptr.as_any().downcast_ref::<RefCell<Vec<Option<T>>>>())
            .map(|cell| {
                cell.borrow()
                    .iter()
                    .enumerate()
                    .map(|(entity, component)| component.as_ref().map(|_| entity))
                    .collect()
            })
    }

    #[allow(unused)]
    pub(crate) fn fetch_entity_component_types(&self, enity: Entity) -> Vec<TypeId> {
        let mut type_ids = Vec::with_capacity(self.entity_components.len());
        for (component_type, box_ptr) in self.entity_components.iter() {
            if box_ptr.peek(enity.index() as usize).is_some() {
                type_ids.push(*component_type);
            }
        }
        type_ids
    }
}
