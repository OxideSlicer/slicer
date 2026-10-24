use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use stl_io::IndexedMesh;

fn main() { divan::main() }

fn model_path() -> PathBuf {
    std::env::var_os("STL_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("calibration_cube.stl")
        })
}

fn load_mesh() -> IndexedMesh {
    let path = model_path();
    let file = File::open(&path)
        .unwrap_or_else(|err| panic!("failed to open STL at {}: {err}", path.display()));
    stl_io::read_stl(&mut BufReader::new(file))
        .unwrap_or_else(|err| panic!("failed to read STL mesh from {}: {err}", path.display()))
}

#[divan::bench]
fn slice(bencher: divan::Bencher) {
    // Parsed once, outside the timing loop.
    let mesh = load_mesh();

    bencher
        .with_inputs(|| {
            (
                mesh.clone(), // fresh mesh per iteration: slice_mesh takes it by value
                oxide_engine::engine::VoxelGrid::new(1920, 1080, 1000),
            )
        })
        .bench_local_values(|(mesh, mut grid)| grid.slice_mesh(mesh, 0.9));
}
