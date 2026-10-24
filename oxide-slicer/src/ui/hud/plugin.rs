use crate::ui::hud::hud;
use bevy::{
    diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::EguiPrimaryContextPass;

pub struct HUDUIPlugin;

impl Plugin for HUDUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            EntityCountDiagnosticsPlugin::default(),
        ))
            .add_systems(EguiPrimaryContextPass, (hud::top_panel, hud::side_panel));
    }
}