use std::{fs, path::Path};
use stl_io::{IndexedMesh, IndexedTriangle, Vertex};
use truck_meshalgo::prelude::*;
use truck_stepio::r#in::*;

fn get_unit_conversion_to_mm(step_content: &str) -> f64 {
    let mut conversion_factor = 1000.0;

    for line in step_content.lines() {
        let line_upper = line.to_uppercase();

        if line_upper.contains("SI_UNIT") && line_upper.contains(".METRE.") {
            conversion_factor = 1000.0;
            break;
        } else if line_upper.contains("CONVERSION_BASED_UNIT")
            || line_upper.contains("LENGTH_UNIT")
        {
            if line_upper.contains("'MILLIMETRE'")
                || line_upper.contains("'MM'")
            {
                conversion_factor = 1.0;
                break;
            } else if line_upper.contains("'CENTIMETRE'")
                || line_upper.contains("'CM'")
            {
                conversion_factor = 10.0;
                break;
            } else if line_upper.contains("'METRE'")
                || line_upper.contains("'M'")
            {
                conversion_factor = 1000.0;
                break;
            } else if line_upper.contains("'INCH'")
                || line_upper.contains("'IN'")
            {
                conversion_factor = 25.4;
                break;
            } else if line_upper.contains("'FOOT'")
                || line_upper.contains("'FT'")
            {
                conversion_factor = 304.8;
                break;
            } else if line_upper.contains("'MICROMETRE'")
                || line_upper.contains("'MICROMETER'")
            {
                conversion_factor = 0.001;
                break;
            }
        }
    }

    if let Some(pos) = step_content.find("LENGTH_MEASURE_WITH_UNIT") {
        let snippet =
            &step_content[pos..pos.saturating_add(200).min(step_content.len())];
        if let Some(measure_start) = snippet.find("LENGTH_MEASURE(") {
            if let Some(measure_end) = snippet[measure_start..].find(')') {
                let measure_str =
                    &snippet[measure_start + 15..measure_start + measure_end];
                if let Ok(scale) = measure_str.trim().parse::<f64>() {
                    if scale != 1.0 && scale > 0.0 {
                        conversion_factor *= scale;
                    }
                }
            }
        }
    }

    conversion_factor
}

pub fn step_to_indexed_mesh_truck(path: &Path) -> IndexedMesh {
    let step_file = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => panic!("Could not read STEP file: {}", e),
    };

    let unit_scale = get_unit_conversion_to_mm(&step_file);

    let exchange = match ruststep::parser::parse(&step_file) {
        Ok(e) => e,
        Err(e) => panic!("Could not parse STEP file: {:?}", e),
    };

    let table = Table::from_data_section(&exchange.data[0]);
    let mut all_vertices: Vec<Vertex> = Vec::new();
    let mut all_faces: Vec<IndexedTriangle> = Vec::new();

    for (_, shell) in table.shell.iter() {
        let compressed_shell = match table.to_compressed_shell(shell) {
            Ok(cs) => cs,
            Err(_) => continue,
        };

        let pre = compressed_shell.robust_triangulation(0.01).to_polygon();
        let bdd = pre.bounding_box();
        let tolerance = bdd.diameter() * 0.0001;
        let polygon =
            compressed_shell.robust_triangulation(tolerance).to_polygon();
        let positions = polygon.positions();

        let base_vertex_count = all_vertices.len();
        for pos in positions {
            all_vertices.push(stl_io::Vector([
                (pos.x * unit_scale) as f32,
                (pos.y * unit_scale) as f32,
                (pos.z * unit_scale) as f32,
            ]));
        }

        for [v0_idx, v1_idx, v2_idx] in polygon.faces().tri_faces() {
            let v0_local_idx = v0_idx.pos;
            let v1_local_idx = v1_idx.pos;
            let v2_local_idx = v2_idx.pos;

            let v0_global_idx = base_vertex_count + v0_local_idx;
            let v1_global_idx = base_vertex_count + v1_local_idx;
            let v2_global_idx = base_vertex_count + v2_local_idx;

            let v0 = positions[v0_local_idx];
            let v1 = positions[v1_local_idx];
            let v2 = positions[v2_local_idx];

            let u = [
                v1.x as f32 - v0.x as f32,
                v1.y as f32 - v0.y as f32,
                v1.z as f32 - v0.z as f32,
            ];
            let v = [
                v2.x as f32 - v0.x as f32,
                v2.y as f32 - v0.y as f32,
                v2.z as f32 - v0.z as f32,
            ];

            let mut nx = u[1] * v[2] - u[2] * v[1];
            let mut ny = u[2] * v[0] - u[0] * v[2];
            let mut nz = u[0] * v[1] - u[1] * v[0];

            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            if len > f32::EPSILON {
                nx /= len;
                ny /= len;
                nz /= len;
            }

            all_faces.push(IndexedTriangle {
                normal: stl_io::Vector([nx, ny, nz]),
                vertices: [v0_global_idx, v1_global_idx, v2_global_idx],
            });
        }
    }

    if all_vertices.is_empty() {
        all_vertices.push(stl_io::Vector([0.0, 0.0, 0.0]));
        all_vertices.push(stl_io::Vector([1.0, 0.0, 0.0]));
        all_vertices.push(stl_io::Vector([0.0, 1.0, 0.0]));

        let normal = stl_io::Vector([0.0, 0.0, 1.0]);
        all_faces.push(IndexedTriangle { normal, vertices: [0, 1, 2] });
    }

    IndexedMesh { vertices: all_vertices, faces: all_faces }
}

pub fn exchange_to_indexed_mesh(
    exchange: &ruststep::ast::Exchange,
) -> IndexedMesh {
    let mut all_vertices: Vec<Vertex> = Vec::new();
    let mut all_faces: Vec<IndexedTriangle> = Vec::new();

    let full_content = format!("{:?}", exchange.data);
    let parts: Vec<&str> = full_content.split("CARTESIAN_POINT").collect();
    for part in parts.iter().skip(1) {
        if let Some(list_pos) = part.find("List([") {
            let after_list = &part[list_pos..];
            if let Some(close_pos) = after_list.find("])") {
                let coords_section = &after_list[6..close_pos];

                let mut coords = Vec::new();
                for token in coords_section.split(',') {
                    let trimmed = token.trim();
                    if let Some(real_pos) = trimmed.find("Real(") {
                        if let Some(close_real) = trimmed[real_pos..].find(')')
                        {
                            let num_str =
                                &trimmed[real_pos + 5..real_pos + close_real];
                            if let Ok(num) = num_str.parse::<f32>() {
                                coords.push(num);
                            }
                        }
                    } else if let Ok(num) = trimmed.parse::<f32>() {
                        coords.push(num);
                    }
                }

                if coords.len() >= 3 {
                    all_vertices.push(stl_io::Vector([
                        coords[0], coords[1], coords[2],
                    ]));
                }
            }
        }
    }

    if all_vertices.len() >= 3 {
        for i in (0..all_vertices.len()).step_by(3) {
            if i + 2 < all_vertices.len() {
                let normal = stl_io::Vector([0.0, 0.0, 1.0]);
                all_faces.push(IndexedTriangle {
                    normal,
                    vertices: [i, i + 1, i + 2],
                });
            }
        }
    }

    if all_vertices.is_empty() {
        all_vertices.push(stl_io::Vector([0.0, 0.0, 0.0]));
        all_vertices.push(stl_io::Vector([1.0, 0.0, 0.0]));
        all_vertices.push(stl_io::Vector([0.0, 1.0, 0.0]));

        let normal = stl_io::Vector([0.0, 0.0, 1.0]);
        all_faces.push(IndexedTriangle { normal, vertices: [0, 1, 2] });
    }

    IndexedMesh { vertices: all_vertices, faces: all_faces }
}

pub fn read_step(path: &Path) -> ruststep::ast::Exchange {
    let file: String =
        fs::read_to_string(path).expect("Problem reading the file");
    match ruststep::parser::parse(&file) {
        Err(error) => panic!("Problem parsing the file: {error:?}"),
        Ok(mesh) => mesh,
    }
}