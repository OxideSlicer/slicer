use crate::cameras::{
    component::CameraSettings, free::FreeCameraPlugin, ui::UICameraPlugin,
};
use bevy::prelude::*;

pub struct CamerasPlugin;

impl Plugin for CamerasPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraSettings>().add_plugins((
            // OrbitCameraPlugin,
            FreeCameraPlugin,
            UICameraPlugin,
        ));
    }
}