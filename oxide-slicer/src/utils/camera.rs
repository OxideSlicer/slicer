use bevy::{camera::Camera, prelude::*};

pub fn set_camera_active<C: Component, const ACTIVE: bool>(
    mut query: Query<&mut Camera, With<C>>,
) {
    if let Ok(mut cam) = query.single_mut() {
        cam.is_active = ACTIVE;
    }
}