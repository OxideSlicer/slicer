use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct ModelComponent {
    pub id: usize,
    pub path: String,
}

#[derive(Component, Debug)]
pub struct NeedsCenteringComponent;

#[derive(Message, Clone, Debug)]
pub struct ImportModelEvent {
    pub path: String,
}

#[derive(Resource, Default, Debug)]
pub struct ModelList {
    pub next_id: usize,
}

#[derive(Resource, Default, Debug)]
pub struct SelectedModel {
    pub entity: Option<Entity>,
}