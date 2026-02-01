use std::any::Any;

pub trait Component: Any + Send + Sync + 'static {}

impl<T: Any + Send + Sync + 'static> Component for T {}
