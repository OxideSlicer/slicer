use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use stl_io::IndexedMesh;

fn main() {
    let model_path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("calibration_cube.stl"));

    let file = File::open(&model_path)
        .unwrap_or_else(|err| panic!("failed to open STL at {}: {err}", model_path.display()));
    let mut reader = BufReader::new(file);
    let mesh: IndexedMesh = stl_io::read_stl(&mut reader)
        .unwrap_or_else(|err| panic!("failed to read STL mesh from {}: {err}", model_path.display()));

    let mut grid = oxide_engine::engine::VoxelGrid::new(1920, 1080, 1000);
    let output_dir = std::env::current_dir().unwrap();
    std::fs::create_dir_all(&output_dir).unwrap();
    grid.slice_mesh(mesh, 0.9);
    grid.clean_model();
    grid.create_layers(&output_dir);
}
