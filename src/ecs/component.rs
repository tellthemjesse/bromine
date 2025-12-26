use std::any::Any;

// Base trait for all components
// Needs Any + Send + Sync + 'static for multi-threading safety (eventually)
// Start simple for now
pub trait Component: Any + Send + Sync + 'static {}

// Blanket implementation for any type that meets the bounds
impl<T: Any + Send + Sync + 'static> Component for T {}