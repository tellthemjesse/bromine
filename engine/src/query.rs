/// Caller must ensure, that the queryable [`Component`](crate::ecs::Component)'s are registered
#[macro_export]
macro_rules! query {
    ($world:ident, $c:ty) => {
        $world.borrow_components::<$c>().unwrap()
    };
    ($world:ident, $($c:ty),+) => {
         (
             $($world.borrow_components::<$c>().unwrap()),+
         )
    };
}

/// Prepend type with `mut` to get mutable access. Mutable queries are exclusive
/// 
/// # Panics 
/// 
/// Will panic if the queryable [`Resource`](crate::ecs::Resource) isn't registered
#[macro_export]
macro_rules! query_resource {
    ($world:ident, mut $c:ty) => {
        $world.borrow_resource::<$c>().unwrap()
    };
    ($world:ident, $(mut $c:ty),+) => {
         (
             $($world.borrow_resource::<$c>().unwrap()),+
         )
    };
    ($world:ident, $c:ty) => {
        $world.fetch_resource::<$c>().unwrap()
    };
    ($world:ident, $($c:ty),+) => {
         (
             $($world.fetch_resource::<$c>().unwrap()),+
         )
    };
}

#[cfg(test)]
mod test {
    use crate::ecs::*;

    #[test]
    fn test_query() {
        use crate::impl_component;

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

        let (positions, enemies) = query!(world, Position, Enemy);

        assert_eq!(positions[entity.index() as usize], Some(position));
        assert_ne!(enemies[entity.index() as usize], None);
    }
}
