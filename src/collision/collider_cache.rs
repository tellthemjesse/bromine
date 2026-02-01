use crate::types::Entity;
use std::collections::HashSet;

pub type ColliderCache = HashSet<(Entity, Entity)>;
