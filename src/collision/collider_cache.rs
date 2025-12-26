use std::collections::{HashSet};
use crate::ecs::entity::Entity;

pub type ColliderCache = HashSet<(Entity, Entity)>;