use std::{any::Any, cell::RefCell};

/// Resources are singleton data containers
pub trait Resource: 'static {}

pub trait ResourceSlot: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Resource> ResourceSlot for RefCell<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
