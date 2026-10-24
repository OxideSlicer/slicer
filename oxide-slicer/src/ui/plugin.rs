use crate::ui::hud::HUDUIPlugin;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((EguiPlugin::default(), HUDUIPlugin));
    }
}