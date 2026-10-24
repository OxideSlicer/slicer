use bevy::prelude::{Commands, Component, Entity, Query, With};

pub fn despawn<C: Component>(
    mut commands: Commands,
    query: Query<Entity, With<C>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}