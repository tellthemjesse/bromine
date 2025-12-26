use nalgebra_glm::Vec3;

#[derive(Clone, Debug)]
pub struct RigidBody {
    pub mass: f32,
    pub velocity: Vec3,
    pub acceleration: Vec3,
    /// bounciness factor (0.0 to 1.0)
    pub restitution: f32,
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            mass: 1.0,
            velocity: Vec3::zeros(),
            acceleration: Vec3::zeros(),
            restitution: 0.1,
        }
    }
}

impl RigidBody {
    pub fn new(mass: f32) -> Self {
        Self {
            mass,
            ..Self::default()
        }
    }

    pub fn with_velocity(mut self, velocity: Vec3) -> Self {
        self.velocity = velocity;
        self
    }

    pub fn with_acceleration(mut self, acceleration: Vec3) -> Self {
        self.acceleration = acceleration;
        self
    }

    pub fn with_restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution;
        self
    }

    pub fn clear_acceleration(&mut self) {
        self.acceleration = Vec3::zeros();
    }
}
