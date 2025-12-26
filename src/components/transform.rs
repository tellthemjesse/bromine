use nalgebra_glm::{Vec3, Mat4, Quat, translation, quat_to_mat4, quat_identity, scaling, vec3};

#[derive(Clone, Debug)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat, // Use Quaternion for rotation
    pub scale: Vec3,
}

impl Transform {
    pub fn identity() -> Self {
        Transform {
            position: Vec3::zeros(),
            rotation: quat_identity(),
            scale: vec3(1.0, 1.0, 1.0),
        }
    }

    pub fn to_matrix(&self) -> Mat4 {
        let translation_matrix = translation(&self.position);
        let rotation_matrix = quat_to_mat4(&self.rotation);
        let scale_matrix = scaling(&self.scale);
        
        translation_matrix * rotation_matrix * scale_matrix
    }

    /// Returned tuple items correspond to multiplication order: translation * rotation * scale
    pub fn to_components(&self) -> (Mat4, Mat4, Mat4) {
        // TODO: Let shaders compute final model matrix
        (translation(&self.position), quat_to_mat4(&self.rotation), scaling(&self.scale))
    }

    pub fn with_position(mut self, p: Vec3) -> Self {
        self.position = p;
        self
    }

    pub fn with_scale(mut self, s: Vec3) -> Self {
        self.scale = s;
        self
    }

    pub fn with_rotation(mut self, r: Quat) -> Self {
        self.rotation = r;
        self
    }
} 