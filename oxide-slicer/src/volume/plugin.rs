use crate::{
    states::AppState,
    utils::despawn,
    volume::{VolumeConfig, component::VolumeComponent, volume},
};
use bevy::prelude::*;

pub struct VolumePlugin;

impl Plugin for VolumePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VolumeConfig>()
            .add_systems(OnEnter(AppState::View3D), volume::spawn)
            .add_systems(OnExit(AppState::View3D), despawn::<VolumeComponent>);
    }
}