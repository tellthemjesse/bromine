use std::{
    any::{Any, TypeId},
    cell::{RefCell, RefMut},
    collections::HashMap,
};

/// Components represent entity related data
pub trait Component: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[macro_export]
macro_rules! impl_component {
    ($type:ty) => {
        impl Component for $type {
            fn as_any(&self) -> &dyn Any { self }
            fn as_any_mut(&mut self) -> &mut dyn Any { self }
        }
    };
}

/// What's in the box ???
pub type BoxedComponent = Box<dyn Component>;

/// Resources are singleton data containers
pub trait Resource: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[macro_export]
macro_rules! impl_resource {
    ($type:ty) => {
        impl Resource for $type {
            fn as_any(&self) -> &dyn Any { self }
            fn as_any_mut(&mut self) -> &mut dyn Any { self }
        }
    };
}

/// What's in the box ???
pub type BoxedResource = Box<dyn Resource>;

#[derive(Clone, Copy)]
pub struct Entity(u32);

impl Entity {
    pub fn index(&self) -> u32 {
        self.0
    }
}

impl From<u32> for Entity {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

pub type TypeIdMap<V> = HashMap<TypeId, V>;

pub trait ComponentVec: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn len(&self) -> usize;
    fn push_none(&mut self);
}

impl<C: Component> ComponentVec for RefCell<Vec<Option<C>>> {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn len(&self) -> usize { self.borrow().len() }
    fn push_none(&mut self) { self.borrow_mut().push(None); }
}

pub struct World {
    pub entity_index: u32,
    pub resources: TypeIdMap<BoxedResource>,
    pub entity_components: TypeIdMap<Box<dyn ComponentVec>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entity_index: 0,
            resources: TypeIdMap::new(),
            entity_components: TypeIdMap::new(),
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
        let ptr = Box::new(resource);
        let _ = self.resources.insert(type_id, ptr);
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
                let mut components = dynamic_components.as_any_mut()
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
                for _ in 0..self.entity_index { components.push(None); }

                components[index] = Some(component);
                let box_ptr = Box::new(RefCell::new(components));

                let _ = self.entity_components
                    .insert(type_id, box_ptr);
            }
        }
    }

    pub fn get_resource<T>(&self) -> Option<&T>
    where
        T: Resource,
    {
        self.resources.get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.as_ref().as_any().downcast_ref())
    }

    pub fn get_resource_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Resource,
    {
        self.resources.get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.as_mut().as_any_mut().downcast_mut())
    }

    pub fn borrow_components<'w, C>(&'w self) -> Option<RefMut<'w, Vec<Option<C>>>>
    where
        C: Component,
    {
        self.entity_components.get(&TypeId::of::<C>())
            .and_then(|box_ptr| box_ptr.as_any().downcast_ref::<RefCell<Vec<Option<C>>>>())
            .and_then(|cell| Some(cell.borrow_mut()))
    }
}

/// Caller must ensure, that the queryable components are registered
#[macro_export]
macro_rules! query {
    (&mut $world:ident, $c:ty) => {
        $world.get_components::<$c>().unwrap()
    };
    (&mut $world:ident, $($c:ty),+) => {
         (
             $($world.borrow_components::<$c>().unwrap()),+
         )
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_component_getters() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        struct Position(i32, i32);
        impl_component!(Position);

        let position = Position(5, 8);
        world.register_component(entity, position);

        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        struct Enemy;
        impl_component!(Enemy);

        let is_enemy = Enemy;
        world.register_component(entity, is_enemy);

        let positions = world.borrow_components::<Position>().unwrap();
        let position_opt = positions[entity.index() as usize];
        let enemies = world.borrow_components::<Enemy>().unwrap();
        let is_enemy_opt = enemies[entity.index() as usize];

        assert_eq!(Some(position), position_opt);
        assert_eq!(Some(is_enemy), is_enemy_opt);
    }

    #[test]
    fn test_resource_getters() {
        let mut world = World::new();

        #[derive(Clone, Copy, PartialEq, Debug)]
        struct WorldOrigin(f32, f32, f32);
        impl_resource!(WorldOrigin);

        let origin = WorldOrigin(1.0, 0.0, 3.0);
        world.register_resourse(origin);

        #[derive(Clone, Copy, PartialEq, Debug)]
        struct TimeDelta(f32);
        impl_resource!(TimeDelta);

        let mut dt = TimeDelta(0.016);
        world.register_resourse(dt);

        let origin_ref = world.get_resource::<WorldOrigin>();
        assert_eq!(Some(&origin), origin_ref);

        let dt_mut = world.get_resource_mut::<TimeDelta>();
        assert_eq!(Some(&mut dt), dt_mut);
    }

    #[test]
    fn test_query() {
        let mut world = World::new();
        let entity = world.spawn_entity();

        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        struct Position(i32, i32);
        impl_component!(Position);

        let position = Position(5, 8);
        world.register_component(entity, position);

        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        struct Enemy;
        impl_component!(Enemy);

        let is_enemy = Enemy;
        world.register_component(entity, is_enemy);

        let (positions, enemies) = query!(&mut world, Position, Enemy);

        assert_eq!(positions[entity.index() as usize], Some(position));
        assert_ne!(enemies[entity.index() as usize], None);
    }
}
