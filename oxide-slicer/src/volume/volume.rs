use crate::volume::component::{VolumeComponent, VolumeConfig};
use bevy::{
    asset::RenderAssetUsages,
    light::{NotShadowCaster, NotShadowReceiver},
    mesh::PrimitiveTopology,
    prelude::*,
    render::render_resource::Face,
};

const COLOR_STRONG: [f32; 4] = [0.47, 0.55, 1.0, 1.0];
const COLOR_MED: [f32; 4] = [0.47, 0.55, 1.0, 0.6];
const COLOR_THIN: [f32; 4] = [0.47, 0.55, 1.0, 0.25];

pub fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    build_volume: ResMut<VolumeConfig>,
) {
    commands.spawn((
        VolumeComponent,
        Mesh3d(meshes.add(generate_grid(
            build_volume.width,
            build_volume.height,
            build_volume.depth,
        ))),
        MeshMaterial3d(materials.add(StandardMaterial {
            cull_mode: Some(Face::Back),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::default(),
        NotShadowCaster,
        NotShadowReceiver,
    ));
}

#[derive(Clone, Copy)]
struct GridFace {
    fixed_axis: usize,
    fixed_val: f32,
    axis_a: (usize, f32, f32), // (axe, min, max)
    axis_b: (usize, f32, f32),
}

fn generate_grid(width: f32, height: f32, depth: f32) -> Mesh {
    let faces = [
        GridFace {
            fixed_axis: 1,
            fixed_val: 0.0,
            axis_a: (0, 0.0, width),
            axis_b: (2, 0.0, depth),
        },
        GridFace {
            fixed_axis: 1,
            fixed_val: height,
            axis_a: (0, 0.0, width),
            axis_b: (2, 0.0, depth),
        },
        GridFace {
            fixed_axis: 0,
            fixed_val: 0.0,
            axis_a: (2, 0.0, depth),
            axis_b: (1, 0.0, height),
        },
        GridFace {
            fixed_axis: 0,
            fixed_val: width,
            axis_a: (2, 0.0, depth),
            axis_b: (1, 0.0, height),
        },
        GridFace {
            fixed_axis: 2,
            fixed_val: 0.0,
            axis_a: (0, 0.0, width),
            axis_b: (1, 0.0, height),
        },
        GridFace {
            fixed_axis: 2,
            fixed_val: depth,
            axis_a: (0, 0.0, width),
            axis_b: (1, 0.0, height),
        },
    ];

    let (w, h, d) = (width as usize, height as usize, depth as usize);
    let capacity = 2
        * (2 * (w + 1 + d + 1) + 2 * (d + 1 + h + 1) + 2 * (w + 1 + h + 1) + 4);
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(capacity);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(capacity);

    for face in &faces {
        push_face_lines(&mut positions, &mut colors, *face);
    }
    push_vertical_edges(&mut positions, &mut colors, width, depth, height);

    let mut mesh =
        Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh
}

#[inline]
fn make_point(
    fixed: (usize, f32),
    a: (usize, f32),
    b: (usize, f32),
) -> [f32; 3] {
    let mut v = [0.0_f32; 3];
    v[fixed.0] = fixed.1;
    v[a.0] = a.1;
    v[b.0] = b.1;
    v
}

#[inline]
fn grid_color(i: i32) -> [f32; 4] {
    if i % 10 == 0 {
        COLOR_STRONG
    } else if i % 5 == 0 {
        COLOR_MED
    } else {
        COLOR_THIN
    }
}

fn push_face_lines(
    positions: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    face: GridFace,
) {
    let (aa, a_min, a_max) = face.axis_a;
    let (ab, b_min, b_max) = face.axis_b;
    let fixed = (face.fixed_axis, face.fixed_val);

    for i in (a_min as i32)..=(a_max as i32) {
        let c = grid_color(i);
        positions.extend_from_slice(&[
            make_point(fixed, (aa, i as f32), (ab, b_min)),
            make_point(fixed, (aa, i as f32), (ab, b_max)),
        ]);
        colors.extend_from_slice(&[c, c]);
    }

    for j in (b_min as i32)..=(b_max as i32) {
        let c = grid_color(j);
        positions.extend_from_slice(&[
            make_point(fixed, (aa, a_min), (ab, j as f32)),
            make_point(fixed, (aa, a_max), (ab, j as f32)),
        ]);
        colors.extend_from_slice(&[c, c]);
    }
}

fn push_vertical_edges(
    positions: &mut Vec<[f32; 3]>,
    colors: &mut Vec<[f32; 4]>,
    width: f32,
    depth: f32,
    height: f32,
) {
    for [x, z] in [[0.0, 0.0], [width, 0.0], [width, depth], [0.0, depth]] {
        positions.extend_from_slice(&[[x, 0.0, z], [x, height, z]]);
        colors.extend_from_slice(&[COLOR_STRONG, COLOR_STRONG]);
    }
}