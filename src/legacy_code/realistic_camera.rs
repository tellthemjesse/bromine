use nalgebra_glm as glm;
use glm::{Vec3, Mat4, vec3, vec2 ,length, cross};
use crate::collision::{AABB};
use std::f32::EPSILON;

const YAW: f32 = -90.0;
const PITCH: f32 = 0.0;
const VELOCITY: f32 = 1.4;
const JUMP_VELOCITY: f32 = 5.0;
const JUMP_ANGLE: f32 = 25.0;
const SPRINT_VELOCITY: f32 = 2.5;
const SENSITIVITY: f32 = 0.15;
const GRAVITY: f32 = 1.5;
const CAMERA_AABB_SCALE: f32 = 4.5;

pub enum Action {
    MoveForward,
    MoveBackwards,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Sprint,
    FlipGravity,
}

#[derive(Default)]
struct CameraState {
    move_forward: bool,
    move_backwards: bool,
    move_up: bool,
    move_down: bool,
    move_left: bool,
    move_right: bool,
    sprint: bool,
    flipped_gravity: bool,
}

pub struct Camera {
    // attributes
    pub position: Vec3,
    pub front: Vec3,
    pub up: Vec3,
    pub right: Vec3,
    pub world_up: Vec3,
    state: CameraState,
    // Gravity / Ground state
    vertical_velocity: f32,
    is_on_ground: bool,
    // options
    pub default_speed: f32,
    pub sprint_velocity: f32,
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
        // 1. Calculate horizontal speed
        let current_horizontal_speed = if self.state.sprint {
            self.sprint_velocity
        } else {
            self.default_speed // Use the new field
        };
        let horizontal_velocity_factor = current_horizontal_speed * delta_time;

        // 2. Calculate base horizontal displacement from W/A/S/D
        let mut wish_direction = vec3(0.0, 0.0, 0.0);
        if self.state.move_forward {
            wish_direction += &self.front;
        }
        if self.state.move_backwards {
            wish_direction -= &self.front;
        }
        if self.state.move_left {
            wish_direction -= &self.right;
        }
        if self.state.move_right {
            wish_direction += &self.right;
        }

        // Normalize horizontal wish direction
        let wish_direction_xz = vec2(wish_direction.x, wish_direction.z);
        let mut base_horizontal_displacement = vec3(0.0, 0.0, 0.0);
        let mut normalized_wish_xz = vec2(0.0, 0.0); // Keep track of direction for jump
        // .gt means greater than
        if length(&wish_direction_xz).gt(&EPSILON) { 
            normalized_wish_xz = glm::normalize(&wish_direction_xz);
            base_horizontal_displacement = 
                vec3(normalized_wish_xz.x, 0.0, normalized_wish_xz.y) * horizontal_velocity_factor;
        }

        // 3. Apply Gravity AND Handle Jump Input
        // Check for jump input
        let mut applied_jump_force = false;
        if self.state.move_up && self.is_on_ground {
            let jump_angle_rad = JUMP_ANGLE.to_radians();
            // Set initial vertical velocity
            self.vertical_velocity = JUMP_VELOCITY * jump_angle_rad.sin();
            
            // Calculate horizontal jump speed boost
            let horizontal_jump_speed = JUMP_VELOCITY * jump_angle_rad.cos();
            let horizontal_jump_displacement = horizontal_jump_speed * delta_time;

            // Add horizontal jump displacement in the direction of movement (or forward if standing still)
            if length(&normalized_wish_xz).gt(&EPSILON) {
                 base_horizontal_displacement += vec3(normalized_wish_xz.x, 0.0, normalized_wish_xz.y) * horizontal_jump_displacement;
            } else {
                 // If standing still, jump slightly forward based on camera view
                 base_horizontal_displacement += vec3(self.front.x, 0.0, self.front.z) * horizontal_jump_displacement;
            }

            self.is_on_ground = false;
            applied_jump_force = true;
        }

        // Apply gravity acceleration 
        if self.state.flipped_gravity {
            self.vertical_velocity += GRAVITY * delta_time;
        } else {
            self.vertical_velocity -= GRAVITY * delta_time;
        }
        let vertical_displacement = self.vertical_velocity * delta_time;

        // 4. Perform collision detection and response axis by axis
        let mut current_position = self.position;
        // If jump just happened, don't reset is_on_ground yet.
        if !applied_jump_force { 
             self.is_on_ground = false; 
        }

        // Define combined horizontal displacement for checks
        let final_horizontal_displacement = base_horizontal_displacement; // Renamed for clarity

        // Check X-axis movement
        let mut potential_pos_x = current_position;
        potential_pos_x.x += final_horizontal_displacement.x;
        let camera_aabb_x = AABB::new(&potential_pos_x, CAMERA_AABB_SCALE);
        if !check_collision(&camera_aabb_x, aabbs) {
            current_position.x = potential_pos_x.x;
        }

        // Check Z-axis movement (using potentially updated X)
        let mut potential_pos_z = current_position;
        potential_pos_z.z += final_horizontal_displacement.z;
        let camera_aabb_z = AABB::new(&potential_pos_z, CAMERA_AABB_SCALE);
        if !check_collision(&camera_aabb_z, aabbs) {
            current_position.z = potential_pos_z.z;
        }

        // Check Y-axis movement (using potentially updated X and Z)
        let mut potential_pos_y = current_position;
        potential_pos_y.y += vertical_displacement;
        let camera_aabb_y = AABB::new(&potential_pos_y, CAMERA_AABB_SCALE);
        if !check_collision(&camera_aabb_y, aabbs) {
            current_position.y = potential_pos_y.y;
        } else {
            // Collision occurred vertically
            if vertical_displacement < 0.0 { // Moving down?
                 self.is_on_ground = true;
                 self.vertical_velocity = 0.0; // Stop falling
                 // Maybe snap Y position here later
            } else { // Moving up into something?
                 self.vertical_velocity = 0.0; // Stop upward movement too
            }
            // Don't update current_position.y if collision occurred
        }

        // 5. Update final position
        self.position = current_position;
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
        self.right = glm::normalize(&cross(&self.front, &self.world_up));
        self.up = glm::normalize(&cross(&self.right, &self.front));
    }
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            position: Vec3::new(0.0, 5.0, 3.0),
            front: Vec3::new(0.0, 0.0, -1.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            right: Vec3::new(1.0, 0.0, 0.0),
            world_up: Vec3::new(0.0, 1.0, 0.0),
            state: Default::default(),
            yaw: YAW,
            pitch: PITCH,
            mouse_sensitivity: SENSITIVITY,
            default_speed: VELOCITY,
            sprint_velocity: SPRINT_VELOCITY,
            vertical_velocity: 0.0,
            is_on_ground: false,
        }
    }
}

// Helper function for collision checking (add if not present)
fn check_collision(camera_aabb: &AABB, aabbs: &Vec<AABB>) -> bool {
    for aabb in aabbs {
        // Use the intersect method you defined
        if AABB::intersect(*camera_aabb, *aabb) {
            return true; // Collision detected
        }
    }
    false // No collision
}