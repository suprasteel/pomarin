use std::time::Duration;

use cgmath::{prelude::*, Matrix4, Point3, Vector3};
use winit::event::{ElementState, MouseScrollDelta, VirtualKeyCode};

use super::{Camera, Controller, GenericCamera};

pub struct OrbitalCamera {
    pub position: Point3<f32>,
    pub target: Point3<f32>,
}

impl OrbitalCamera {
    fn new(position: Point3<f32>, target: Point3<f32>) -> Self {
        Self { position, target }
    }
}

impl Camera for OrbitalCamera {
    fn calc_view_matrix(&self) -> Matrix4<f32> {
        cgmath::Matrix4::look_at_rh(self.position, self.target, Vector3::unit_y())
    }

    fn position(&self) -> Point3<f32> {
        self.position
    }
}

pub struct OrbitalController {
    speed: f32,
    is_upward_pressed: bool,
    is_downward_pressed: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl OrbitalController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_upward_pressed: false,
            is_downward_pressed: false,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }
}

impl Controller for OrbitalController {
    type Camera = GenericCamera;

    fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool {
        let is_pressed = state == ElementState::Pressed;
        match key {
            VirtualKeyCode::H => {
                self.is_upward_pressed = is_pressed;
                true
            }
            VirtualKeyCode::L => {
                self.is_downward_pressed = is_pressed;
                true
            }
            VirtualKeyCode::W | VirtualKeyCode::Up => {
                self.is_forward_pressed = is_pressed;
                true
            }
            VirtualKeyCode::A | VirtualKeyCode::Left => {
                self.is_left_pressed = is_pressed;
                true
            }
            VirtualKeyCode::S | VirtualKeyCode::Down => {
                self.is_backward_pressed = is_pressed;
                true
            }
            VirtualKeyCode::D | VirtualKeyCode::Right => {
                self.is_right_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }

    // yaw = turn l/r, pitch = up/down
    fn update_camera(&mut self, camera: &mut Self::Camera, _dt: Duration) {
        let forward = camera.target - camera.position;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // Prevents glitching when camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed && forward_mag > self.speed {
            camera.position += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.position -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(Vector3::unit_y());
        let up = forward_norm.cross(right);

        // Redo radius calc in case the fowrard/backward is pressed.
        let forward = camera.target - camera.position;
        let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            // Rescale the distance between the target and.position so
            // that it doesn't change. The.position therefore still
            // lies on the circle made by the target and.position.
            camera.position =
                camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            camera.position =
                camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }
        if self.is_upward_pressed {
            camera.position = camera.target - (forward - up * self.speed).normalize() * forward_mag;
        }
        if self.is_downward_pressed {
            camera.position = camera.target - (forward + up * self.speed).normalize() * forward_mag;
        }
    }

    fn process_mouse(&mut self, _mouse_dx: f64, _mouse_dy: f64) {}

    fn process_scroll(&mut self, _delta: &MouseScrollDelta) {}

    fn set_mouse_pressed(&mut self, _pressed: bool) {}

    fn mode(&self) -> super::ControlMode {
        super::ControlMode::ORBITAL
    }
}
se std::time::Duration;

use cgmath::{prelude::*, Matrix4, Point3, Vector3};
use winit::event::{ElementState, MouseScrollDelta, VirtualKeyCode};

use super::{Camera, Controller, GenericCamera};

pub struct OrbitalCamera {
    pub position: Point3<f32>,
    pub target: Point3<f32>,
}

impl OrbitalCamera {
    fn new(position: Point3<f32>, target: Point3<f32>) -> Self {
        Self { position, target }
    }
}

impl Camera for OrbitalCamera {
    fn calc_view_matrix(&self) -> Matrix4<f32> {
        cgmath::Matrix4::look_at_rh(self.position, self.target, Vector3::unit_y())
    }

    fn position(&self) -> Point3<f32> {
        self.position
    }
}

pub struct OrbitalController {
    speed: f32,
    is_upward_pressed: bool,
    is_downward_pressed: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl OrbitalController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_upward_pressed: false,
            is_downward_pressed: false,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }
}

impl Controller for OrbitalController {
    type Camera = GenericCamera;

    fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool {
        let is_pressed = state == ElementState::Pressed;
        match key {
            VirtualKeyCode::H => {
                self.is_upward_pressed = is_pressed;
                true
            }
            VirtualKeyCode::L => {
                self.is_downward_pressed = is_pressed;
                true
            }
            VirtualKeyCode::W | VirtualKeyCode::Up => {
                self.is_forward_pressed = is_pressed;
                true
            }
            VirtualKeyCode::A | VirtualKeyCode::Left => {
                self.is_left_pressed = is_pressed;
                true
            }
            VirtualKeyCode::S | VirtualKeyCode::Down => {
                self.is_backward_pressed = is_pressed;
                true
            }
            VirtualKeyCode::D | VirtualKeyCode::Right => {
                self.is_right_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }

    // yaw = turn l/r, pitch = up/down
    fn update_camera(&mut self, camera: &mut Self::Camera, _dt: Duration) {
        let forward = camera.target - camera.position;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // Prevents glitching when camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed && forward_mag > self.speed {
            camera.position += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.position -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(Vector3::unit_y());
        let up = forward_norm.cross(right);

        // Redo radius calc in case the fowrard/backward is pressed.
        let forward = camera.target - camera.position;
        let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            // Rescale the distance between the target and.position so
            // that it doesn't change. The.position therefore still
            // lies on the circle made by the target and.position.
            camera.position =
                camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            camera.position =
                camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }
        if self.is_upward_pressed {
            camera.position = camera.target - (forward - up * self.speed).normalize() * forward_mag;
        }
        if self.is_downward_pressed {
            camera.position = camera.target - (forward + up * self.speed).normalize() * forward_mag;
        }
    }

    fn process_mouse(&mut self, _mouse_dx: f64, _mouse_dy: f64) {}

    fn process_scroll(&mut self, _delta: &MouseScrollDelta) {}

    fn set_mouse_pressed(&mut self, _pressed: bool) {}

    fn mode(&self) -> super::ControlMode {
        super::ControlMode::ORBITAL
    }
}
