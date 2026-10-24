use bevy::prelude::*;

pub const MAX_PITCH: f32 = std::f32::consts::FRAC_PI_2 - 0.01;

#[derive(Resource, Debug)]
pub struct CameraSettings {
    pub speed: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub mouse_sensitivity: f32,
    pub scroll_sensitivity: f32,
    pub keyboard_sensitivity: f32,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub aspect_ratio: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            speed: 50.0,
            yaw: 0.0,
            pitch: 0.0,
            mouse_sensitivity: 0.005,
            scroll_sensitivity: 0.5,
            keyboard_sensitivity: 2.0,
            fov: std::f32::consts::PI / 4.0,
            near: 0.1,
            far: 1000.0,
            aspect_ratio: 1.0,
        }
    }
}