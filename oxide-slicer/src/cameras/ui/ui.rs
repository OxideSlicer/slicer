use crate::cameras::ui::component::UiCameraComponent;
use bevy::prelude::*;

/// UI Cam, used to render text, buttons, and other 2D elements.

pub fn spawn(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        UiCameraComponent,
        Camera {
            is_active: true,
            order: 1,
            viewport: None,
            computed: default(),
            output_mode: default(),
            msaa_writeback: MsaaWriteback::Off,
            clear_color: ClearColorConfig::Default,
            invert_culling: false,
            sub_camera_view: None,
        },
        Msaa::Off,
        Projection::Orthographic(OrthographicProjection::default_2d()),
    ));
}