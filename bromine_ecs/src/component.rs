use std::{any::Any, cell::RefCell};

/// Components represent entity related data
pub trait Component: 'static {}

pub trait ComponentVec: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn push_none(&mut self);
    /// Checks if the value is present at given index
    fn peek(&self, index: usize) -> Option<()>;
}

impl<T: Component> ComponentVec for RefCell<Vec<Option<T>>> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn len(&self) -> usize {
        self.borrow().len()
    }
    fn is_empty(&self) -> bool {
        self.borrow().is_empty()
    }
    fn push_none(&mut self) {
        self.borrow_mut().push(None);
    }
    fn peek(&self, index: usize) -> Option<()> {
        self.borrow()
            .get(index)
            .and_then(|opt| opt.as_ref().map(|_| ()))
    }
}
