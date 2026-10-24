use crate::models::{
    component::{
        ImportModelEvent, ModelList, NeedsCenteringComponent, SelectedModel,
    },
    model,
};
use crate::states::AppState;
use bevy::prelude::*;

pub struct ModelsPlugin;

impl Plugin for ModelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_stl::StlPlugin)
            .init_resource::<ModelList>()
            .init_resource::<SelectedModel>()
            .add_message::<ImportModelEvent>()
            .add_systems(
                Update,
                (
                    model::import_model,
                    model::center_model_on_buildplate
                        .run_if(any_with_component::<NeedsCenteringComponent>),
                )
                    .run_if(in_state(AppState::View3D)),
            );
    }
}