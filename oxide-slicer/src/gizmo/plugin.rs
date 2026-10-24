use crate::{
    gizmo::component, gizmo::gizmo, models::SelectedModel, states::AppState,
    utils::despawn,
};
use bevy::prelude::*;

pub struct GizmoPlugin;

impl Plugin for GizmoPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<component::GizmoConfig>()
            .init_resource::<component::GizmoDragState>()
            .add_systems(PostStartup, gizmo::config)
            .add_systems(OnEnter(AppState::View3D), gizmo::spawn)
            .add_systems(
                OnExit(AppState::View3D),
                despawn::<component::TransformGizmoComponent>,
            )
            .add_systems(
                Update,
                (
                    gizmo::update_position,
                    gizmo::draw,
                    gizmo::select_axis,
                    gizmo::drag,
                )
                    .chain()
                    .run_if(in_state(AppState::View3D))
                    .run_if(resource_exists::<SelectedModel>),
            );
    }
}