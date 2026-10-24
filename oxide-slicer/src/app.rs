use crate::{
    cameras::CamerasPlugin, debug::DebugPlugin, gizmo::GizmoPlugin,
    models::ModelsPlugin, states::StatesPlugin, ui::UIPlugin,
    volume::VolumePlugin,
};
use bevy::{
    asset::{AssetMetaCheck, UnapprovedPathMode},
    log::LogPlugin,
    prelude::*,
    window::{
        CursorGrabMode, CursorOptions, EnabledButtons, ExitCondition,
        PresentMode, WindowMode, WindowResolution, WindowTheme,
    },
};

pub const DEFAULT_FILTER: &str =
    concat!("wgpu=error,", "naga=warn,", "calloop::loop_logic=error,",);

fn configure_app(app: &mut App) -> &mut App {
    app.add_plugins(
        DefaultPlugins
            .set(
                #[cfg(debug_assertions)]
                {
                    LogPlugin {
                        filter: DEFAULT_FILTER.to_string(),
                        level: bevy::log::Level::INFO,
                        custom_layer: |_| None,
                        fmt_layer: |_| None,
                    }
                },
                #[cfg(not(debug_assertions))]
                {
                    LogPlugin {
                        filter: DEFAULT_FILTER.to_string(),
                        level: bevy::log::Level::WARN,
                        custom_layer: |_| None,
                        fmt_layer: |_| None,
                    }
                },
            )
            .set(WindowPlugin {
                primary_window: Window {
                    decorations: true,
                    title: "Oxide Slicer - v".to_string()
                        + env!("CARGO_PKG_VERSION"),
                    position: WindowPosition::Centered(
                        MonitorSelection::Primary,
                    ),
                    resolution: WindowResolution::new(1920, 1080),
                    mode: WindowMode::Windowed,
                    present_mode: PresentMode::AutoNoVsync,
                    name: Option::from("main".to_string()),
                    resizable: false,
                    window_theme: Some(WindowTheme::Dark),
                    visible: true,
                    enabled_buttons: EnabledButtons {
                        // Didn't work on Linux (tested on X11)
                        close: true,
                        maximize: true,
                        minimize: true,
                    },
                    ..default()
                }
                    .into(),
                primary_cursor_options: CursorOptions {
                    visible: true,
                    grab_mode: CursorGrabMode::None,
                    hit_test: true,
                }
                    .into(),
                exit_condition: ExitCondition::OnAllClosed,
                close_when_requested: true,
            })
            .set(AssetPlugin {
                mode: AssetMode::Unprocessed,
                file_path: "../../assets".to_string(),
                processed_file_path: "../../assets".to_string(),
                watch_for_changes_override: None,
                use_asset_processor_override: None,
                meta_check: AssetMetaCheck::default(),
                unapproved_path_mode: UnapprovedPathMode::Allow,
            }),
    )
}

pub fn app() -> App {
    let mut app = App::new();

    configure_app(&mut app)
        .add_plugins((
            StatesPlugin,
            UIPlugin,
            ModelsPlugin,
            VolumePlugin,
            CamerasPlugin,
            GizmoPlugin,
            #[cfg(debug_assertions)]
            DebugPlugin,
        ))
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 100.0,
            affects_lightmapped_meshes: false,
        })
        .insert_resource(ClearColor(Color::srgba_u8(100, 100, 100, 1)));
    app
}