use crate::{
    cameras::{FreeCameraComponent, free::free},
    states::AppState,
    utils,
};
use bevy::prelude::*;

pub struct FreeCameraPlugin;

impl Plugin for FreeCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, free::spawn)
            .add_systems(
                OnEnter(AppState::View3D),
                utils::set_camera_active::<FreeCameraComponent, true>,
            )
            .add_systems(
                OnExit(AppState::View3D),
                utils::set_camera_active::<FreeCameraComponent, false>,
            )
            .add_systems(
                Update,
                (
                    free::r#move,
                    free::keyboard_look,
                    free::mouse_look,
                    free::mouse_scroll,
                    free::apply_rotation,
                )
                    .run_if(in_state(AppState::View3D)),
            );
    }
}