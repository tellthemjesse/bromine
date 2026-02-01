use std::collections::HashSet;
use std::ops::{Add, AddAssign};

use engine::ecs::Resource;
use engine::impl_resource;
use glam::Mat4;
use winit::keyboard::KeyCode;

macro_rules! impl_newtype {
    ($type:ty, $target:ty) => {
        impl std::ops::Deref for $type {
            type Target = $target;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for $type {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl From<$target> for $type {
            fn from(value: $target) -> Self {
                Self(value)
            }
        }
    };
}

#[derive(Debug, Clone, Copy)]
/// World projection matrix
pub struct Projection(Mat4);

impl_resource!(Projection);
impl_newtype!(Projection, Mat4);

#[derive(Debug, Clone, Copy)]
/// Camera view matrix
pub struct View(Mat4);

impl_resource!(View);
impl_newtype!(View, Mat4);

#[derive(Debug, Clone)]
/// Collection of pressed keys since since last update
pub struct PressedKeys(HashSet<KeyCode>);

impl_resource!(PressedKeys);
impl_newtype!(PressedKeys, HashSet<KeyCode>);

#[derive(Debug, Clone, Copy)]
/// Mouse displacement since last update
pub struct MouseDelta(f64, f64);

impl MouseDelta {
    pub fn new(dx: f64, dy: f64) -> Self {
        Self(dx, dy)
    }
    /// Displacement along X-axis
    pub fn dx(&self) -> f64 {
        self.0
    }
    /// Displacement along Y-axis 
    pub fn dy(&self) -> f64 {
        self.1
    }
    /// Sets (dx, dy) to (0.0, 0.0)
    pub fn clear(&mut self) {
        self.0 = 0.0;
        self.1 = 0.0;
    }
}

impl AddAssign<(f64, f64)> for MouseDelta {
    fn add_assign(&mut self, rhs: (f64, f64)) {
        *self = Self(self.0 + rhs.0, self.1 + rhs.1);
    }
}

impl_resource!(MouseDelta);

pub struct TimeDelta(f64);