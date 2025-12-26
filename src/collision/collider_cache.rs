use std::collections::HashSet;
use crate::types::Entity;

pub type ColliderCache = HashSet<(Entity, Entity)>;
