use crate::cameras::ui::ui;
use bevy::prelude::*;

pub struct UICameraPlugin;

impl Plugin for UICameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, ui::spawn);
    }
}