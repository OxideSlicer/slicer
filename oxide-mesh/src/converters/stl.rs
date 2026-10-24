use std::fs::OpenOptions;
use std::path::Path;
use stl_io::IndexedMesh;

pub fn read_stl(path: &Path) -> IndexedMesh {
    let mut file = match OpenOptions::new().read(true).open(path) {
        Err(error) => panic!("Problem opening the file: {error:?}"),
        Ok(file) => file,
    };
    let stl: IndexedMesh = match stl_io::read_stl(&mut file) {
        Err(error) => panic!("Could not read the file as stl: {error:?}"),
        Ok(stl) => stl,
    };

    stl
}

pub fn write_stl(
    path: &Path,
    mesh: &IndexedMesh,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    let triangle_iterator = mesh.faces.iter().map(|face| stl_io::Triangle {
        normal: face.normal,
        vertices: [
            mesh.vertices[face.vertices[0]],
            mesh.vertices[face.vertices[1]],
            mesh.vertices[face.vertices[2]],
        ],
    });

    stl_io::write_stl(&mut file, triangle_iterator)?;
    Ok(())
}