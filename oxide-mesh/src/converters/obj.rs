use std::path::Path;
use stl_io::{IndexedMesh, IndexedTriangle, Vertex};
use wavefront::Obj;

pub fn read_obj(path: &Path) -> Obj {
    match Obj::from_file(path) {
        Err(error) => panic!("Problem opening the file: {error:?}"),
        Ok(mesh) => mesh,
    }
}

pub fn obj_to_indexed_mesh(obj: &Obj) -> IndexedMesh {
    let vertices: Vec<Vertex> =
        obj.positions().iter().map(|&pos| stl_io::Vector(pos)).collect();

    let mut faces = Vec::new();

    for [v0, v1, v2] in obj.triangles() {
        let p0 = v0.position();
        let p1 = v1.position();
        let p2 = v2.position();

        let u = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let v = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];

        let mut nx = u[1] * v[2] - u[2] * v[1];
        let mut ny = u[2] * v[0] - u[0] * v[2];
        let mut nz = u[0] * v[1] - u[1] * v[0];

        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        if len > f32::EPSILON {
            nx /= len;
            ny /= len;
            nz /= len;
        }

        faces.push(IndexedTriangle {
            normal: stl_io::Vector([nx, ny, nz]),
            vertices: [
                v0.position_index(),
                v1.position_index(),
                v2.position_index(),
            ],
        });
    }

    IndexedMesh { vertices, faces }
}