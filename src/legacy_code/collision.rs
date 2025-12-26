use nalgebra_glm as glm;
use glm::{Vec3, vec3};

#[derive(Copy, Clone, Debug)]
// Axis-aligned bounding box
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

// https://developer.mozilla.org/en-US/docs/Games/Techniques/3D_collision_detection
impl AABB {
    pub fn new(position: &Vec3, scale: f32) -> Self {
        let (min, max) = AABB::define_box_bounds(position, scale);
        Self { min, max }
    }

    fn define_box_bounds(center: &Vec3, scale: f32) -> (Vec3, Vec3) {
        let half_size = vec3(0.5, 0.5, 0.5f32) * scale;
        let min = center - half_size;
        let max = center + half_size;
        return (min, max);
    }

    // If true, two boxes intersect with each other
    pub fn intersect(aabb1: AABB, aabb2: AABB) -> bool {
        return (aabb1.min.x <= aabb2.max.x && aabb1.max.x >= aabb2.min.x) &&
            (aabb1.min.y <= aabb2.max.y && aabb1.max.y >= aabb2.min.y) &&
            (aabb1.min.z <= aabb2.max.z && aabb1.max.z >= aabb2.min.z);
    }

    pub fn intersect_point(point: Vec3, aabb: AABB) -> bool {
        return (point.x >= aabb.min.x && point.x <= aabb.max.x) &&
            (point.y >= aabb.min.y && point.y <= aabb.max.y) &&
            (point.z >= aabb.min.z && point.z <= aabb.max.z);
    }
}
