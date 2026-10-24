use crate::{
    models::{ImportModelEvent, ModelComponent, SelectedModel},
    states::AppState,
};
use bevy::{
    diagnostic::{
        DiagnosticsStore, EntityCountDiagnosticsPlugin,
        FrameTimeDiagnosticsPlugin,
    },
    prelude::*,
};
use bevy_egui::{EguiContexts, egui};
use oxide_mesh::{obj, step, stl};

pub fn top_panel(
    mut contexts: EguiContexts,
    mut writer: MessageWriter<ImportModelEvent>,
    current_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    egui::TopBottomPanel::top("top_bar")
        .show(ctx, |ui| {
            let total_width = ui.available_width();
            let col_width = total_width / 3.0;
            let row_height = 28.0;

            ui.horizontal(|ui| {
                ui.allocate_ui(egui::vec2(col_width, row_height), |ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {

                        ui.menu_button("Project", |ui| {
                            if ui.button("Import").clicked() { ui.close(); }
                            if ui.button("Export").clicked() { ui.close(); }
                        });

                        ui.menu_button("Models", |ui| {
                            if ui.button("Import").clicked() {
                                ui.close();
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("3D files", &["stl", "obj", "step", "stp"])
                                    .pick_file()
                                {
                                    let path_str = path.to_string_lossy().to_string();
                                    let final_path = match path.extension().and_then(|e| e.to_str()) {
                                        Some("stl") => Some(path_str),
                                        Some("obj") => {
                                            let obj_path = std::path::Path::new(&path_str);
                                            let file_stem = match obj_path.file_stem().and_then(|s| s.to_str()) {
                                                Some(s) => s.to_string(),
                                                None => { eprintln!("Invalid file name"); return; }
                                            };
                                            let tmp_dir = std::env::temp_dir().join("oxideslicer_converter");
                                            if std::fs::create_dir_all(&tmp_dir).is_err() {
                                                eprintln!("Cannot create tmp dir"); return;
                                            }
                                            let output_path = tmp_dir.join(format!("{}.stl", file_stem));
                                            let obj = obj::read_obj(obj_path);
                                            let mesh = obj::obj_to_indexed_mesh(&obj);
                                            if let Err(e) = stl::write_stl(&output_path, &mesh) {
                                                eprintln!("Failed to write STL: {}", e); return;
                                            }
                                            Some(output_path.to_string_lossy().to_string())
                                        }
                                        Some("step") | Some("stp") => {
                                            let step_path = std::path::Path::new(&path_str);
                                            let file_stem = match step_path.file_stem().and_then(|s| s.to_str()) {
                                                Some(s) => s.to_string(),
                                                None => { eprintln!("Invalid file name"); return; }
                                            };
                                            let tmp_dir = std::env::temp_dir().join("oxideslicer_converter");
                                            if std::fs::create_dir_all(&tmp_dir).is_err() {
                                                eprintln!("Cannot create tmp dir"); return;
                                            }
                                            let output_path = tmp_dir.join(format!("{}.stl", file_stem));
                                            let mesh = step::step_to_indexed_mesh_truck(step_path);
                                            if let Err(e) = stl::write_stl(&output_path, &mesh) {
                                                eprintln!("Failed to write STL: {}", e); return;
                                            }
                                            Some(output_path.to_string_lossy().to_string())
                                        }
                                        _ => { eprintln!("Not supported format"); None }
                                    };
                                    if let Some(p) = final_path {
                                        writer.write(ImportModelEvent { path: p });
                                    }
                                }
                            }
                        });

                        ui.menu_button("Settings", |ui| { let _ = ui.button("Settings"); });
                        ui.menu_button("Help", |ui| { let _ = ui.button("Help"); });
                    });
                });

                ui.allocate_ui(egui::vec2(col_width, row_height), |ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        let (label, target, color) = match current_state.get() {
                            AppState::View3D => (
                                "✂  Slice View",
                                AppState::ViewSlicing,
                                egui::Color32::from_rgb(80, 120, 200),
                            ),
                            AppState::ViewSlicing => (
                                "🎲  3D View",
                                AppState::View3D,
                                egui::Color32::from_rgb(80, 180, 120),
                            ),
                            _ => ("Switch View", AppState::View3D, egui::Color32::from_rgb(80, 120, 200)),
                        };

                        let button = egui::Button::new(
                            egui::RichText::new(label).color(egui::Color32::WHITE).size(13.0)
                        )
                            .fill(color)
                            .corner_radius(6.0)
                            .min_size(egui::vec2(140.0, 22.0));

                        if ui.add(button).clicked() {
                            next_state.set(target);
                        }
                    });
                });

            });
        });
    Ok(())
}

pub fn side_panel(
    mut contexts: EguiContexts,
    diagnostics: Res<DiagnosticsStore>,
    mut model_query: Query<(Entity, &ModelComponent, &mut Transform)>,
    mut selected: ResMut<SelectedModel>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.average())
        .unwrap_or(0.0);

    let entities = diagnostics
        .get(&EntityCountDiagnosticsPlugin::ENTITY_COUNT)
        .and_then(|d| d.value())
        .unwrap_or(0.0);

    egui::SidePanel::right("right_panel").default_width(250.0).show(
        ctx,
        |ui| {
            egui::CollapsingHeader::new("Render infos")
                .default_open(true)
                .show(ui, |ui| {
                    ui.label(format!("FPS: {:.1}", fps));
                    ui.label(format!("Entities: {}", entities));
                });

            ui.separator();

            volume_section(ui);

            ui.separator();

            camera_section(ui);

            ui.separator();

            model_section(ui, &mut model_query, &mut selected);

            ui.separator();

            lights_section(ui);
        },
    );

    Ok(())
}

fn drag_value_row(ui: &mut egui::Ui, label: &str, value: &mut f32) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::DragValue::new(value).speed(0.01));
    });
}

fn volume_section(ui: &mut egui::Ui) {
    egui::CollapsingHeader::new("Volume").default_open(true).show(ui, |ui| {
        let mut volume = [0.0_f32; 3];
        egui::Grid::new("volume_grid")
            .num_columns(2)
            .spacing([10.0, 4.0])
            .show(ui, |ui| {
                drag_value_row(ui, "Width", &mut volume[0]);
                ui.end_row();
                drag_value_row(ui, "Height", &mut volume[1]);
                ui.end_row();
                drag_value_row(ui, "Depth", &mut volume[2]);
                ui.end_row();
            });
    });
}

fn xyz_drag_grid(
    grid_id: impl std::hash::Hash,
    ui: &mut egui::Ui,
    x: &mut f32,
    y: &mut f32,
    z: &mut f32,
) {
    egui::Grid::new(grid_id).num_columns(2).spacing([10.0, 4.0]).show(
        ui,
        |ui| {
            drag_value_row(ui, "X", x);
            ui.end_row();
            drag_value_row(ui, "Y", y);
            ui.end_row();
            drag_value_row(ui, "Z", z);
            ui.end_row();
        },
    );
}

fn camera_section(ui: &mut egui::Ui) {
    egui::CollapsingHeader::new("Camera").default_open(true).show(ui, |ui| {
        let mut translation: Vec3 = Vec3::new(0.0, 0.0, 0.0);
        let mut rotation: Quat = Quat::default();
        let mut euler = rotation.to_euler(EulerRot::XYZ);

        ui.vertical(|ui| {
            ui.add_space(4.0);
            ui.heading("Translation");
            xyz_drag_grid(
                "camera_translation",
                ui,
                &mut translation.x,
                &mut translation.y,
                &mut translation.z,
            );
            ui.add_space(4.0);
            ui.heading("Rotation");
            xyz_drag_grid(
                "camera_rotation",
                ui,
                &mut euler.0,
                &mut euler.1,
                &mut euler.2,
            );
            rotation =
                Quat::from_euler(EulerRot::XYZ, euler.0, euler.1, euler.2)
        });
    });
}
fn model_section(
    ui: &mut egui::Ui,
    model_query: &mut Query<(Entity, &ModelComponent, &mut Transform)>,
    selected: &mut ResMut<SelectedModel>,
) {
    egui::CollapsingHeader::new("Models").default_open(true).show(ui, |ui| {
        for (entity, model, mut transform) in model_query.iter_mut() {
            let is_selected = selected.entity == Some(entity);
            let label = if is_selected {
                format!("● Model {}", model.id)
            } else {
                format!("  Model {}", model.id)
            };

            ui.collapsing(label, |ui| {
                if !is_selected && ui.button("Select").clicked() {
                    selected.entity = Some(entity);
                }

                ui.heading("Translation");
                let mut translation = transform.translation;
                xyz_drag_grid(
                    "model_translation",
                    ui,
                    &mut translation.x,
                    &mut translation.y,
                    &mut translation.z,
                );
                transform.translation = translation;

                ui.add_space(4.0);
                ui.heading("Rotation");
                let mut euler = transform.rotation.to_euler(EulerRot::XYZ);
                xyz_drag_grid(
                    "camera_rotation",
                    ui,
                    &mut euler.0,
                    &mut euler.1,
                    &mut euler.2,
                );
                transform.rotation =
                    Quat::from_euler(EulerRot::XYZ, euler.0, euler.1, euler.2);

                ui.add_space(4.0);
                ui.heading("Scale");
                let mut scale = transform.scale;
                xyz_drag_grid(
                    format!("model_scale_{}", model.id),
                    ui,
                    &mut scale.x,
                    &mut scale.y,
                    &mut scale.z,
                );
                transform.scale = scale;
            });
        }
    });
}
fn lights_section(ui: &mut egui::Ui) {
    egui::CollapsingHeader::new("Lights").default_open(true).show(ui, |ui| {
        let mut intensity_cam = 0.0;
        let mut intensity_glob = 0.0;
        let mut state_cam = true;
        let mut state_glob = true;

        ui.vertical(|ui| {
            ui.add_space(4.0);
            ui.heading("Camera light");
            egui::Grid::new("camera_light")
                .num_columns(2)
                .spacing([10.0, 4.0])
                .show(ui, |ui| {
                    ui.checkbox(&mut state_glob, "state");
                    drag_value_row(ui, "intensity", &mut intensity_cam);
                    ui.end_row();
                });
            ui.add_space(4.0);
            ui.heading("Global light");
            egui::Grid::new("global_light")
                .num_columns(2)
                .spacing([10.0, 4.0])
                .show(ui, |ui| {
                    ui.checkbox(&mut state_cam, "state");
                    drag_value_row(ui, "intensity", &mut intensity_glob);
                    ui.end_row();
                });
        });
    });
}