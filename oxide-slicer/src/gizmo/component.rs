use bevy::prelude::*;

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct GizmoConfig;

#[derive(Component, Debug)]
pub struct TransformGizmoComponent;

#[derive(Resource, Default, PartialEq)]
pub enum GizmoMode {
    #[default]
    Translate,
    Rotate,
}

#[derive(Resource, Default)]
pub struct GizmoDragState {
    pub axis: Option<Vec3>,
    pub mode: GizmoMode,
}