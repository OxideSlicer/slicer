use crate::{
    models::component::{
        ImportModelEvent, ModelComponent, ModelList, NeedsCenteringComponent,
        SelectedModel,
    },
    volume::VolumeConfig,
};
use bevy::{
    asset::AssetPath, mesh::VertexAttributeValues, prelude::*,
    render::render_resource::Face,
};

pub fn import_model(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut messages: MessageReader<ImportModelEvent>,
    mut model_list: ResMut<ModelList>,
    mut selected: ResMut<SelectedModel>,
) {
    for event in messages.read() {
        let handle: Handle<Mesh> = asset_server
            .load(AssetPath::from_path(std::path::Path::new(&event.path)));

        let id = model_list.next_id;
        model_list.next_id += 1;

        let entity = commands
            .spawn((
                ModelComponent { id, path: event.path.clone() },
                NeedsCenteringComponent,
                Mesh3d(handle),
                MeshMaterial3d(materials.add(StandardMaterial {
                    cull_mode: Some(Face::Back),
                    base_color: Color::srgb(0.8, 0.8, 0.8),
                    perceptual_roughness: 0.7,
                    metallic: 0.0,
                    ..default()
                })),
                Transform::from_rotation(Quat::from_rotation_x(
                    -std::f32::consts::FRAC_PI_2,
                )),
                Visibility::default(),
            ))
            .id();

        if selected.entity.is_none() {
            selected.entity = Some(entity);
        }
    }
}

pub fn center_model_on_buildplate(
    meshes: Res<Assets<Mesh>>,
    build_volume: Res<VolumeConfig>,
    mut query: Query<
        (Entity, &Mesh3d, &mut Transform),
        (With<NeedsCenteringComponent>, With<ModelComponent>),
    >,
    mut commands: Commands,
) {
    for (entity, mesh_handle, mut transform) in &mut query {
        let Some(mesh) = meshes.get(&mesh_handle.0) else { continue };

        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            continue;
        };

        let (local_min, local_max) = positions.iter().fold(
            (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)),
            |(min, max), &p| {
                let v = Vec3::from(p);
                (min.min(v), max.max(v))
            },
        );

        let local_center = transform.rotation * (local_min + local_max) * 0.5;

        let min_y_rotated = positions
            .iter()
            .map(|&p| (transform.rotation * Vec3::from(p)).y)
            .fold(f32::MAX, f32::min);

        transform.translation = Vec3::new(
            build_volume.width / 2.0 - local_center.x,
            -min_y_rotated,
            build_volume.depth / 2.0 - local_center.z,
        );

        commands.entity(entity).remove::<NeedsCenteringComponent>();
    }
}