use crate::cameras::{
    component::{CameraSettings, MAX_PITCH},
    free::component::{FreeCameraComponent, FreeCameraLightComponent},
};
use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
    render::render_resource::TextureUsages,
};

pub fn spawn(mut commands: Commands, camera_settings: ResMut<CameraSettings>) {
    commands.spawn((
        Camera3d {
            depth_load_op: default(),
            depth_texture_usages: TextureUsages::RENDER_ATTACHMENT.into(),
            screen_space_specular_transmission_steps: 1,
            screen_space_specular_transmission_quality: default(),
        },
        FreeCameraComponent,
        Camera {
            is_active: true,
            order: 0,
            viewport: None,
            computed: default(),
            output_mode: default(),
            msaa_writeback: MsaaWriteback::Off,
            clear_color: ClearColorConfig::Default,
            invert_culling: false,
            sub_camera_view: None,
        },
        FreeCameraLightComponent,
        DirectionalLight {
            color: default(),
            illuminance: 2000.0,
            shadows_enabled: false,
            ..default()
        },
        Msaa::Off,
        Transform { translation: Vec3::new(0.0, 0.0, 10.0), ..default() }
            .looking_at(Vec3::ZERO, Vec3::Y),
        Projection::Perspective(PerspectiveProjection {
            fov: camera_settings.fov,
            near: camera_settings.near,
            far: camera_settings.far,
            aspect_ratio: camera_settings.aspect_ratio,
            near_clip_plane: vec4(0.0, 0.0, -1.0, -0.1),
        }),
    ));
}

pub fn r#move(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    camera_settings: Res<CameraSettings>,
    mut query: Query<&mut Transform, With<FreeCameraComponent>>,
) {
    let dt = time.delta_secs();

    for mut transform in &mut query {
        let mut direction = Vec3::ZERO;

        if keyboard.pressed(KeyCode::KeyW) {
            direction += *transform.forward();
        }
        if keyboard.pressed(KeyCode::KeyS) {
            direction -= *transform.forward();
        }
        if keyboard.pressed(KeyCode::KeyA) {
            direction -= *transform.right();
        }
        if keyboard.pressed(KeyCode::KeyD) {
            direction += *transform.right();
        }
        if keyboard.pressed(KeyCode::Space) {
            direction += Vec3::Y;
        }
        if keyboard.pressed(KeyCode::ShiftLeft) {
            direction -= Vec3::Y;
        }

        if direction.length_squared() > 0.0 {
            transform.translation +=
                direction.normalize() * camera_settings.speed * dt;
        }
    }
}

pub fn keyboard_look(
    time: Res<Time>,
    mut camera_settings: ResMut<CameraSettings>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let dt = time.delta_secs();

    let mut yaw = 0.0;
    let mut pitch = 0.0;

    if keyboard.pressed(KeyCode::ArrowLeft) {
        yaw += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        yaw -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowUp) {
        pitch += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        pitch -= 1.0;
    }

    camera_settings.yaw += yaw * camera_settings.keyboard_sensitivity * dt;
    camera_settings.pitch += pitch * camera_settings.keyboard_sensitivity * dt;
    camera_settings.pitch = camera_settings.pitch.clamp(-MAX_PITCH, MAX_PITCH);
}

pub fn mouse_look(
    button_input: Res<ButtonInput<MouseButton>>,
    mut camera_settings: ResMut<CameraSettings>,
    accumulated_motion: Res<AccumulatedMouseMotion>,
) {
    if !button_input.pressed(MouseButton::Right) {
        return;
    }

    let delta = accumulated_motion.delta;

    if delta == Vec2::ZERO {
        return;
    }

    camera_settings.yaw -= delta.x * camera_settings.mouse_sensitivity;
    camera_settings.pitch -= delta.y * camera_settings.mouse_sensitivity;
    camera_settings.pitch = camera_settings.pitch.clamp(-MAX_PITCH, MAX_PITCH);
}

pub fn mouse_scroll(
    mut query: Query<&mut Transform, With<FreeCameraComponent>>,
    scroll: Res<AccumulatedMouseScroll>,
    camera_settings: Res<CameraSettings>,
) {
    let delta = scroll.delta.y;

    if delta == 0.0 {
        return;
    }

    for mut transform in &mut query {
        let transform_forward = transform.forward();
        transform.translation +=
            transform_forward * delta * camera_settings.scroll_sensitivity;
    }
}
pub fn apply_rotation(
    camera_settings: Res<CameraSettings>,
    mut query: Query<&mut Transform, With<FreeCameraComponent>>,
) {
    for mut transform in &mut query {
        let pitch = Quat::from_rotation_x(camera_settings.pitch);
        let yaw = Quat::from_rotation_y(camera_settings.yaw);

        transform.rotation = yaw * pitch;
    }
}