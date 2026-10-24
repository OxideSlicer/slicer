use bevy::prelude::{Component, Resource};

#[derive(Component, Debug)]
pub struct VolumeComponent;

#[derive(Resource, Debug)]
pub struct VolumeConfig {
    pub width: f32,
    pub height: f32,
    pub depth: f32,
}

impl Default for VolumeConfig {
    fn default() -> Self {
        Self { width: 140.0, height: 180.0, depth: 80.0 }
    }
}