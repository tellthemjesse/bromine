use nalgebra_glm as glm;
use glm::{Vec3, Mat4};
use crate::collision::{AABB};

const YAW: f32 = -90.0f32;
const PITCH: f32 = 0.0f32;
const SPEED: f32 = 1.4f32;
const SPRINT_SPEED: f32 = 2.5f32;
const SENSITIVITY: f32 = 0.15f32;

pub enum Action {
    MoveForward,
    MoveBackwards,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    FlipGravity,
    Sprint
}

#[derive(Default)]
struct CameraState {
    move_forward: bool,
    move_backwards: bool,
    move_up: bool,
    move_down: bool,
    move_left: bool,
    move_right: bool,
    flipped_gravity: bool,
    sprint: bool,
}

pub struct Camera {
    // attributes
    pub position: Vec3,
    pub potential_position: Vec3,
    pub front: Vec3,
    pub up: Vec3,
    pub right: Vec3,
    pub world_up: Vec3,
    state: CameraState,
    // options
    current_speed: f32,
    pub default_speed: f32,
    pub sprint_speed: f32,
    pub mouse_sensitivity: f32,
    // angles
    pub yaw: f32,
    pub pitch: f32,
}

impl Camera {
    pub fn new() -> Camera {
        Default::default()
    }

    pub fn get_view(&self) -> Mat4 {
        glm::look_at(&self.position, &(&self.position + &self.front), &self.up)
    }

    pub fn dispatch_state(&mut self, action: Action, state: bool) {
        match action {
            Action::MoveForward => self.state.move_forward = state,
            Action::MoveBackwards => self.state.move_backwards = state,
            Action::MoveUp => self.state.move_up = state,
            Action::MoveDown => self.state.move_down = state,
            Action::MoveRight => self.state.move_right = state,
            Action::MoveLeft => self.state.move_left = state,
            Action::Sprint => self.state.sprint = state,
            Action::FlipGravity => self.state.flipped_gravity = state
        }
    }

    pub fn process_movement(&mut self, delta_time: f32, aabbs: &Vec<AABB>) {
        if self.state.sprint {
            self.current_speed = self.sprint_speed;
        } else {
            self.current_speed = self.default_speed;
        }

        let velocity = self.current_speed * delta_time;

        if self.state.move_forward {
            self.potential_position += self.front * velocity;
        }
        if self.state.move_backwards {
            self.potential_position -= self.front * velocity;
        }
        if self.state.move_up {
            self.potential_position += self.world_up * velocity;
        }
        if self.state.move_down {
            self.potential_position -= self.world_up * velocity;
        }
        if self.state.move_left {
            self.potential_position -= self.right * velocity;
        }
        if self.state.move_right {
            self.potential_position += self.right * velocity;
        }

        use crate::collision::{AABB};

        let mut intersection = false;

        let camera_aabb = AABB::new(&self.potential_position, 1.6f32);

        aabbs.iter().for_each(|aabb| {
            if AABB::intersect(camera_aabb, *aabb) {
                intersection = true;
            }
        });

        if intersection {
            self.potential_position = self.position;
            println!("Collision detected at {:?}", self.potential_position);
        } else {
            self.position = self.potential_position;
        }
    }

    pub fn process_mouse_movement(&mut self, mut x_offset: f32, mut y_offset: f32, constraint_pitch: bool) {
        x_offset *= self.mouse_sensitivity;
        y_offset *= self.mouse_sensitivity;

        self.yaw += x_offset;
        self.pitch += y_offset;

        if constraint_pitch {
            if self.pitch > 89.0f32 {
                self.pitch = 89.0f32
            }
            if self.pitch < -89.0f32 {
                self.pitch = -89.0f32
            }
        }

        self.update();
    }

    fn update(&mut self) {
        let yaw_radians = self.yaw.to_radians();
        let pitch_radians = self.pitch.to_radians();

        let mut front = Vec3::new(0.0, 0.0, 0.0);
        front.x = yaw_radians.cos() * pitch_radians.cos();
        front.y = pitch_radians.sin();
        front.z = yaw_radians.sin() * pitch_radians.cos();

        self.front = glm::normalize(&front);
        self.right = glm::normalize(&glm::cross(&self.front, &self.world_up));
        self.up = glm::normalize(&glm::cross(&self.right, &self.front));
    }
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            position: Vec3::new(0.0, 5.0, 3.0),
            potential_position: Vec3::new(0.0, 5.0, 3.0),
            front: Vec3::new(0.0, 0.0, -1.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            right: Vec3::new(1.0, 0.0, 0.0),
            world_up: Vec3::new(0.0, 1.0, 0.0),
            state: Default::default(),
            yaw: YAW,
            pitch: PITCH,
            mouse_sensitivity: SENSITIVITY,
            current_speed: SPEED,
            default_speed: SPEED,
            sprint_speed: SPRINT_SPEED
        }
    }
}