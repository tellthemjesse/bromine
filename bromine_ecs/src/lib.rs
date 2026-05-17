mod component;
mod entity;
mod hash;
mod resource;
mod world;

pub use component::*;
pub use entity::*;
pub use resource::*;
pub use world::*;

#[macro_export]
macro_rules! implement_component {
    ($type:ty) => {
        impl Component for $type {}
    };
}

#[macro_export]
macro_rules! implement_resource {
    ($type:ty) => {
        impl Resource for $type {}
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
        implement_component!(Position);

        let position = Position(5, 8);
        world.register_component(entity, position);

        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        struct Enemy;
        implement_component!(Enemy);

        let is_enemy = Enemy;
        world.register_component(entity, is_enemy);

        let positions = world.borrow_components::<Position>().unwrap();
        let position_opt = positions[entity.index() as usize];
        let enemies = world.fetch_components::<Enemy>().unwrap();
        let is_enemy_opt = enemies[entity.index() as usize];

        assert_eq!(Some(position), position_opt);
        assert_eq!(Some(is_enemy), is_enemy_opt);
    }

    #[test]
    fn test_resource_getters() {
        let mut world = World::new();

        #[derive(Clone, Copy, PartialEq, Debug)]
        struct WorldOrigin(f32, f32, f32);
        implement_resource!(WorldOrigin);

        let origin = WorldOrigin(1.0, 0.0, 3.0);
        world.register_resourse(origin);

        #[derive(Clone, Copy, PartialEq, Debug)]
        struct TimeDelta(f32);
        implement_resource!(TimeDelta);

        let dt = TimeDelta(0.016);
        world.register_resourse(dt);

        let origin_ref = world.fetch_resource::<WorldOrigin>().unwrap();
        let dt_ref = world.borrow_resource::<TimeDelta>().unwrap();

        assert_eq!(origin, *origin_ref);
        assert_eq!(dt, *dt_ref);
    }
}
