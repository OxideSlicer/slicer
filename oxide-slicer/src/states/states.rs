use bevy::prelude::States;

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    View3D,
    ViewSlicing,
    ProjectSetup,
}