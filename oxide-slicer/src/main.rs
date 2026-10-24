use crate::app::app;
use bevy::app::AppExit;

mod app;
mod cameras;
mod debug;
mod gizmo;
mod models;
mod states;
mod ui;
mod utils;
mod volume;

fn main() -> AppExit {
    app().run()
}