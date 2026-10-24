use crate::states::states::AppState;
use bevy::prelude::{App, AppExtStates, Plugin};

pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>();
    }
}