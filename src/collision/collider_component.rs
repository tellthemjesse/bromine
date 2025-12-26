use nalgebra_glm::{vec3, Vec3};

#[derive(Clone, Debug, Copy)]
pub struct Collider3D {
    pub center: Vec3,
    pub radius: Vec3,
}

impl Collider3D {
    pub fn new(center: Vec3, radius: Vec3) -> Self {
        Self {
            center,
            radius,
        }
    }

    pub fn would_collide(&self, rhs: &Collider3D) -> bool {
        if (self.center.x - rhs.center.x).abs() > (self.radius.x + rhs.radius.x) { return false; }
        if (self.center.y - rhs.center.y).abs() > (self.radius.y + rhs.radius.y) { return false; }
        if (self.center.z - rhs.center.z).abs() > (self.radius.z + rhs.radius.z) { return false; }

        true
    }

    // the caller must ensure that collision happened between self and rhs
    pub fn get_collision_info(&self, rhs: &Collider3D) -> Option<(Vec3, f32)> {
        if self.would_collide(rhs) {
            let difference = vec3(
                self.center.x - rhs.center.x,
                self.center.y - rhs.center.y,
                self.center.z - rhs.center.z
            );

            let radius = vec3(
                self.radius.x + rhs.radius.x,
                self.radius.y + rhs.radius.y,
                self.radius.z + rhs.radius.z
            );

            let overlap = radius - difference.abs();

            let (axis, depth) = overlap.argmin();

            let normal: Vec3 = match axis {
                0 => -sgn(difference.x) * Vec3::x(),
                1 => -sgn(difference.y) * Vec3::y(),
                2 => -sgn(difference.z) * Vec3::z(),
                _ => Vec3::zeros()
            };

            return Some((normal, depth));
        }

        None
    }

    pub fn update_position(&mut self, new_position: Vec3) {
        self.center = new_position;
    }

    #[allow(unused)]
    pub fn contains_point(&self, point: &Vec3) -> bool {
        (point.x >= self.center.x - self.radius.x) && (point.x <= self.center.x + self.radius.x) &&
        (point.y >= self.center.y - self.radius.y) && (point.y <= self.center.y + self.radius.y) &&
        (point.z >= self.center.z - self.radius.z) && (point.z <= self.center.z + self.radius.z)
    }
}

fn sgn(x: f32) -> f32 {
    if x > 0.0 { 1.0 }
    else if x < 0.0 { -1.0 }
    else { 0.0 }
}
