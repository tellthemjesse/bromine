use crate::ecs::component::Component;
use crate::ecs::world::EcsWorld;

pub type Entity = usize;

pub struct EntityConstructor {
    entity: Entity,
    operations: Vec<Box<dyn FnOnce(&mut EcsWorld)>>,
}

impl EntityConstructor {
    pub fn new(entity: Entity) -> Self {
        println!("[Debug] Preparing constructor for Entity#{}", entity);
        Self { 
            entity, 
            operations: Vec::new(),
        }
    }

    pub fn with<T: Component + Send + Sync>(mut self, component: T) -> Self {  
        let entity = self.entity;
        self.operations.push(Box::new(move |world: &mut EcsWorld| {
            world.add_component(entity, component); 
        }));
        self
    }

    pub fn apply(self, world: &mut EcsWorld) {
        println!("[Debug] Applying constructor for Entity#{}", self.entity);
        for op in self.operations {
            op(world);
        }
    }
}
