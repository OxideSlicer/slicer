use crate::{
    cameras::FreeCameraComponent,
    gizmo::component::{
        GizmoConfig, GizmoDragState, GizmoMode, TransformGizmoComponent,
    },
    models::{ModelComponent, SelectedModel},
};
use bevy::{
    camera::{Camera, visibility::RenderLayers},
    color::Color,
    input::{ButtonInput, mouse::AccumulatedMouseMotion},
    math::{Isometry3d, Quat, Vec2, Vec3},
    prelude::*,
};

const GIZMO_HIGHLIGHT: Color = Color::WHITE;
const GIZMO_LENGTH: f32 = 20.0;

const AXIS_COLORS: [(Vec3, Color); 3] = [
    (Vec3::X, Color::srgb(0.9, 0.2, 0.2)),
    (Vec3::Y, Color::srgb(0.2, 0.8, 0.2)),
    (Vec3::Z, Color::srgb(0.2, 0.4, 0.9)),
];

pub fn spawn(mut commands: Commands) {
    commands.spawn((
        TransformGizmoComponent,
        Transform::default(),
        Visibility::default(),
    ));
}

pub fn update_position(
    selected: Res<SelectedModel>,
    model: Query<&Transform, With<ModelComponent>>,
    mut gizmo: Query<
        &mut Transform,
        (With<TransformGizmoComponent>, Without<ModelComponent>),
    >,
) {
    let Some(entity) = selected.entity else { return };
    let Ok(model_transform) = model.get(entity) else { return };
    let Ok(mut gizmo_transform) = gizmo.single_mut() else { return };

    gizmo_transform.translation = model_transform.translation;
}

pub fn draw(
    mut gizmos: Gizmos<GizmoConfig>,
    gizmo_entity: Query<&Transform, With<TransformGizmoComponent>>,
    selected: Res<SelectedModel>,
    state: Res<GizmoDragState>,
) {
    if selected.entity.is_none() {
        return;
    }

    let Ok(transform) = gizmo_entity.single() else { return };
    let origin = transform.translation;
    let length = GIZMO_LENGTH;
    let head_size = length * 0.08;
    let ring_r = length * 0.55;

    let ring_rotations = [
        Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),
        Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
        Quat::IDENTITY,
    ];

    for ((axis, color), ring_rot) in
        AXIS_COLORS.iter().zip(ring_rotations.iter())
    {
        let axis = *axis;
        let color = *color;

        let t_col =
            if state.axis == Some(axis) && state.mode == GizmoMode::Translate {
                GIZMO_HIGHLIGHT
            } else {
                color
            };
        let r_col =
            if state.axis == Some(axis) && state.mode == GizmoMode::Rotate {
                GIZMO_HIGHLIGHT
            } else {
                color
            };

        let tip = origin + axis * length;
        gizmos.line(origin, tip, t_col);

        let perp1 = axis.cross(Vec3::Y).normalize_or(Vec3::X) * head_size;
        let perp2 = axis.cross(perp1).normalize() * head_size;
        let cone_base = tip - axis * (head_size * 2.5);
        for p in [perp1, -perp1, perp2, -perp2] {
            gizmos.line(tip, cone_base + p, t_col);
        }
        gizmos.line(cone_base + perp1, cone_base + perp2, t_col);
        gizmos.line(cone_base + perp2, cone_base - perp1, t_col);
        gizmos.line(cone_base - perp1, cone_base - perp2, t_col);
        gizmos.line(cone_base - perp2, cone_base + perp1, t_col);

        gizmos.circle(Isometry3d::new(origin, *ring_rot), ring_r, r_col);
    }
}

pub fn select_axis(
    mut state: ResMut<GizmoDragState>,
    mouse: Res<ButtonInput<MouseButton>>,
    gizmo_entity: Query<&Transform, With<TransformGizmoComponent>>,
    selected: Res<SelectedModel>,
    camera: Query<(&Camera, &GlobalTransform), With<FreeCameraComponent>>,
    windows: Query<&Window>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    if selected.entity.is_none() {
        return;
    }

    let Ok(gizmo_transform) = gizmo_entity.single() else { return };
    let Ok((camera, cam_transform)) = camera.single() else { return };
    let Ok(window) = windows.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    let Ok(ray) = camera.viewport_to_world(cam_transform, cursor) else {
        return;
    };

    let origin = gizmo_transform.translation;
    let length = GIZMO_LENGTH;
    let ring_r = length * 0.55;
    let cam_dist = (cam_transform.translation() - origin).length();
    let threshold = (cam_dist * 0.025).max(0.5);

    let mut best_t_axis = None;
    let mut best_t_dist = f32::MAX;

    for (axis, _) in AXIS_COLORS {
        let dist = distance_ray_segment(
            ray.origin,
            ray.direction.as_vec3(),
            origin,
            origin + axis * length,
        );
        if dist < threshold && dist < best_t_dist {
            best_t_axis = Some(axis);
            best_t_dist = dist;
        }
    }

    let r_threshold = (cam_dist * 0.04).max(1.5);
    let mut best_r_axis = None;
    let mut best_r_dist = r_threshold;

    for (axis, _) in AXIS_COLORS {
        let dist = distance_ray_torus(
            ray.origin,
            ray.direction.as_vec3(),
            origin,
            axis,
            ring_r,
        );
        if dist < best_r_dist {
            best_r_axis = Some(axis);
            best_r_dist = dist;
        }
    }

    let (mode, axis) = match (best_t_axis, best_r_axis) {
        (Some(t), None) => (GizmoMode::Translate, Some(t)),
        (None, Some(r)) => (GizmoMode::Rotate, Some(r)),
        (Some(t), Some(r)) => {
            if best_t_dist / threshold <= best_r_dist / r_threshold {
                (GizmoMode::Translate, Some(t))
            } else {
                (GizmoMode::Rotate, Some(r))
            }
        }
        (None, None) => {
            state.axis = None;
            return;
        }
    };

    state.axis = axis;
    state.mode = mode;
}

pub fn drag(
    mut state: ResMut<GizmoDragState>,
    mouse: Res<ButtonInput<MouseButton>>,
    accumulated_motion: Res<AccumulatedMouseMotion>,
    selected: Res<SelectedModel>,
    mut model: Query<&mut Transform, With<ModelComponent>>,
    camera: Query<(&Camera, &GlobalTransform), With<FreeCameraComponent>>,
) {
    if mouse.just_released(MouseButton::Left) {
        state.axis = None;
        return;
    }

    let Some(axis) = state.axis else { return };
    let Some(entity) = selected.entity else { return };
    let Ok(mut transform) = model.get_mut(entity) else { return };
    let Ok((_, cam_transform)) = camera.single() else { return };

    let delta = accumulated_motion.delta;
    if delta == Vec2::ZERO {
        return;
    }

    match state.mode {
        GizmoMode::Translate => {
            let magnitude = delta.x * cam_transform.right().dot(axis)
                - delta.y * cam_transform.up().dot(axis);
            transform.translation += axis * magnitude * 0.02;
        }
        GizmoMode::Rotate => {
            let cam_axis = cam_transform.right().dot(axis);
            let angle = (delta.x + delta.y) * 0.02 * cam_axis.signum();
            transform.rotation = (Quat::from_axis_angle(axis, angle)
                * transform.rotation)
                .normalize();
        }
    }
}

fn distance_ray_segment(
    ray_origin: Vec3,
    ray_dir: Vec3,
    a: Vec3,
    b: Vec3,
) -> f32 {
    let ab = b - a;
    let cross = ray_dir.cross(ab);
    let denom = cross.length();
    if denom < 0.0001 {
        return f32::MAX;
    }
    (ray_origin - a).dot(cross).abs() / denom
}

fn distance_ray_torus(
    ray_origin: Vec3,
    ray_dir: Vec3,
    center: Vec3,
    axis: Vec3,
    major_r: f32,
) -> f32 {
    let tangent = if axis.abs().dot(Vec3::Y) < 0.99 {
        axis.cross(Vec3::Y).normalize()
    } else {
        axis.cross(Vec3::X).normalize()
    };
    let binormal = axis.cross(tangent).normalize();

    (0..32)
        .map(|i| {
            let angle = i as f32 * std::f32::consts::TAU / 32.0;
            let point = center
                + (tangent * angle.cos() + binormal * angle.sin()) * major_r;
            let t = ray_dir.dot(point - ray_origin).max(0.0);
            let closest = ray_origin + ray_dir * t;
            (closest - point).length()
        })
        .fold(f32::MAX, f32::min)
}

pub fn config(mut store: ResMut<GizmoConfigStore>) {
    let (config, _) = store.config_mut::<DefaultGizmoConfigGroup>();
    config.render_layers = RenderLayers::layer(0);
}