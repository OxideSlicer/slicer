use image::{ImageBuffer, Rgb};
use std::vec;
use stl_io::IndexedMesh;

#[derive(Debug, Clone, Copy)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    pub a: Vec3,
    pub b: Vec3,
    pub c: Vec3,
}

impl Triangle {
    pub fn new(a: Vec3, b: Vec3, c: Vec3) -> Self {
        Self { a, b, c }
    }

    pub fn z_bounds(&self) -> (f64, f64) {
        let z_min = self.a.z.min(self.b.z).min(self.c.z);
        let z_max = self.a.z.max(self.b.z).max(self.c.z);
        (z_min, z_max)
    }

    pub fn display(&self) {
        println!("[-----------------]");
        let (x, y, z) = (self.a.x, self.a.y, self.a.z);
        println!("[ {x} ; {y} ; {z} ]");
        let (x, y, z) = (self.b.x, self.b.y, self.b.z);
        println!("[ {x} ; {y} ; {z} ]");
        let (x, y, z) = (self.c.x, self.c.y, self.c.z);
        println!("[ {x} ; {y} ; {z} ]");
        println!("[-----------------]");
    }
}

fn create_layer(
    layer: &[Vec<bool>],
    width: u32,
    height: u32,
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let mut img = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(width, height);

    for y in 0..height {
        for x in 0..width {
            img.put_pixel(
                x,
                y,
                match layer[x as usize][y as usize] {
                    true => Rgb([255, 255, 255]),
                    false => Rgb([0, 0, 0]),
                },
            );
        }
    }
    img
}

fn get_edge_plane_intersection(
    p1: Vec3,
    p2: Vec3,
    z: f64,
) -> Option<(f64, f64)> {
    // height delta between both points of the edge
    let dz = p2.z - p1.z;
    // if delta is too small, don't intersect edge and plane
    if dz.abs() < f64::EPSILON {
        return None;
    }
    // t is the position of the plane on the edge
    let t = (z - p1.z) / dz;
    // if 0.0 < t < 1.0 then the edge and the plane intersects
    // adding epsilon to the scale because we're using float (see "floating point precision problem")
    if t < -f64::EPSILON || t > 1.0 + f64::EPSILON {
        return None;
    }
    let t = t.clamp(0.0, 1.0);
    // intersection coords
    Some((p1.x + t * (p2.x - p1.x), p1.y + t * (p2.y - p1.y)))
}

fn get_triangle_plane_intersection(
    triangle: &Triangle,
    z_val: f64,
) -> Option<[(f64, f64); 2]> {
    // 3 edges of the triangle
    let edges = [
        (triangle.a, triangle.b),
        (triangle.b, triangle.c),
        (triangle.c, triangle.a),
    ];
    // only x, y because z is the plane (z_val)
    let mut pts: Vec<(f64, f64)> = Vec::new();

    for (p1, p2) in &edges {
        // exract intersection between edge and plane if it exists
        if let Some(pt) = get_edge_plane_intersection(*p1, *p2, z_val) {
            // check if the point is already in the point list. example : if the plane intersect with a summit, the point will be in 2 of the edges and we need to avoid counting it twice
            let dup = pts.iter().any(|q| {
                (q.0 - pt.0).abs() < 1e-9 && (q.1 - pt.1).abs() < 1e-9
            });
            if !dup {
                pts.push(pt);
            }
        }
    }

    if pts.len() < 2 {
        return None;
    }
    Some([pts[0], pts[1]])
}

fn fill_segment(
    grid: &mut [Vec<bool>],
    x_size: usize,
    y_size: usize,
    p0: (f64, f64),
    p1: (f64, f64),
) {
    // define the number of steps between the two points to avoid gaps
    let steps =
        ((p1.0 - p0.0).abs().max((p1.1 - p0.1).abs()) as usize).max(1) * 2;

    // fill the segment 1 voxel by 1 voxel
    for i in 0..=steps {
        let t = i as f64 / steps as f64;
        let x = (p0.0 + t * (p1.0 - p0.0)).round() as isize;
        let y = (p0.1 + t * (p1.1 - p0.1)).round() as isize;
        // check that the voxel is in the grid
        if x >= 0 && x < x_size as isize && y >= 0 && y < y_size as isize {
            grid[x as usize][y as usize] = true;
        }
    }
}

fn is_triangle_in_plane(triangle: &Triangle, plane: f64) -> bool {
    let Triangle { a, b, c } = triangle;
    (a.z - plane).abs() < 1e-9
        && (b.z - plane).abs() < 1e-9
        && (c.z - plane).abs() < 1e-9
}

fn fill_half_triangle(
    grid: &mut [Vec<bool>],
    x_size: usize,
    y_size: usize,
    p_start: &Vec3,
    p_mid: &Vec3,
    p_end: &Vec3,
) {
    // interpolate y on segment [p1, p2] at position x
    let eval = |p1: &Vec3, p2: &Vec3, x: f64| -> f64 {
        let dx = p2.x - p1.x;
        let t = if dx.abs() < 1e-9 { 0.0 } else { (x - p1.x) / dx };
        p1.y + t.clamp(0.0, 1.0) * (p2.y - p1.y)
    };

    let x_start = p_start.x.round() as isize;
    let x_end = p_mid.x.round() as isize;

    let (x_start, x_end) =
        if x_start <= x_end { (x_start, x_end) } else { (x_end, x_start) };

    for xi in x_start..=x_end {
        if xi < 0 || xi >= x_size as isize {
            continue;
        }
        let x = xi as f64;
        // y on the long edge and y on the short edge
        let y1 = eval(p_start, p_end, x);
        let y2 = eval(p_start, p_mid, x);

        let (y_low, y_high) = if y1 <= y2 {
            (y1.round() as isize, y2.round() as isize)
        } else {
            (y2.round() as isize, y1.round() as isize)
        };

        for y in y_low..=y_high {
            if y >= 0 && y < y_size as isize {
                grid[xi as usize][y as usize] = true;
            }
        }
    }
}

fn fill_triangle(
    grid: &mut [Vec<bool>],
    x_size: usize,
    y_size: usize,
    triangle: &Triangle,
) {
    let Triangle { a, b, c } = triangle;
    let mut pts = [a, b, c];
    // sort points by x
    pts.sort_by(|p, q| p.x.total_cmp(&q.x));
    let [a, b, c] = pts;

    // degenerate case : triangle is a vertical line
    if (c.x - a.x).abs() < 1e-9 {
        fill_segment(grid, x_size, y_size, (a.x, a.y), (c.x, c.y));
        return;
    }

    // split into two half-triangles, eval handles vertical edges via clamp
    if (b.x - a.x).abs() > 1e-9 {
        fill_half_triangle(grid, x_size, y_size, a, b, c); // short edge AB
    }
    if (c.x - b.x).abs() > 1e-9 {
        fill_half_triangle(grid, x_size, y_size, c, b, a); // short edge CB
    }
}

fn propagate_cells(
    grid: &mut [Vec<bool>],
    layer: &mut [Vec<u8>],
    x_size: usize,
    y_size: usize,
    is_filling: bool,
    coords_x: usize,
    coords_y: usize,
) {
    let mut points: Vec<(usize, usize)> = Vec::new();
    points.push((coords_x, coords_y));

    while let Some((x, y)) = points.pop() {
        if layer[x][y] == 1 {
            continue;
        }
        if layer[x][y] == 0 {
            layer[x][y] = if is_filling { 2 } else { 3 };
            let neighbors: [(isize, isize); 4] =
                [(-1, 0), (1, 0), (0, -1), (0, 1)];
            for (dx, dy) in neighbors {
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx < 0
                    || nx >= x_size as isize
                    || ny < 0
                    || ny >= y_size as isize
                {
                    continue;
                }
                let (nx, ny) = (nx as usize, ny as usize);
                if layer[nx][ny] == 0 {
                    points.push((nx, ny));
                }
            }
        }
    }
}

fn fill_model_slice(grid: &mut [Vec<bool>], x_size: usize, y_size: usize) {
    let mut layer: Vec<Vec<u8>> = vec![vec![0; y_size]; x_size];

    // init layer
    for y in 0..y_size {
        for x in 0..x_size {
            if grid[x][y] == true {
                layer[x][y] = 1;
            }
        }
    }

    // fill the layer with temp values
    let mut is_filling = false;
    for y in 0..y_size {
        for x in 0..x_size {
            if layer[x][y] == 0 {
                propagate_cells(
                    grid, &mut layer, x_size, y_size, is_filling, x, y,
                );
                is_filling = !is_filling;
            }
        }
    }

    // fill the grid with layer's data
    for y in 0..y_size {
        for x in 0..x_size {
            grid[x][y] = match layer[x][y] {
                1 => true,
                2 => true,
                _ => false,
            }
        }
    }
}

pub struct VoxelGrid {
    pub x_size: usize,
    pub y_size: usize,
    pub z_size: usize,
    pub slices: Vec<Vec<Vec<bool>>>,
}

impl VoxelGrid {
    pub fn new(x_size: usize, y_size: usize, z_size: usize) -> Self {
        Self {
            x_size,
            y_size,
            z_size,
            slices: vec![vec![vec![false; y_size]; x_size]; z_size],
        }
    }

    pub fn slice_mesh(&mut self, mesh: IndexedMesh, scale: f64) {
        let scale = scale.clamp(0.0, 1.0);

        let triangles: Vec<Triangle> = mesh
            .faces
            .iter()
            .map(|face| {
                let a = mesh.vertices[face.vertices[0]];
                let b = mesh.vertices[face.vertices[1]];
                let c = mesh.vertices[face.vertices[2]];
                Triangle::new(
                    Vec3::new(a[0] as f64, a[1] as f64, a[2] as f64),
                    Vec3::new(b[0] as f64, b[1] as f64, b[2] as f64),
                    Vec3::new(c[0] as f64, c[1] as f64, c[2] as f64),
                )
            })
            .collect();

        for triangle in &triangles {
            triangle.display();
        }

        // bounds of the model
        let x_min = triangles
            .iter()
            .flat_map(|t| [t.a.x, t.b.x, t.c.x])
            .fold(f64::INFINITY, f64::min);
        let x_max = triangles
            .iter()
            .flat_map(|t| [t.a.x, t.b.x, t.c.x])
            .fold(f64::NEG_INFINITY, f64::max);
        let y_min = triangles
            .iter()
            .flat_map(|t| [t.a.y, t.b.y, t.c.y])
            .fold(f64::INFINITY, f64::min);
        let y_max = triangles
            .iter()
            .flat_map(|t| [t.a.y, t.b.y, t.c.y])
            .fold(f64::NEG_INFINITY, f64::max);
        let z_min = triangles
            .iter()
            .flat_map(|t| [t.a.z, t.b.z, t.c.z])
            .fold(f64::INFINITY, f64::min);
        let z_max = triangles
            .iter()
            .flat_map(|t| [t.a.z, t.b.z, t.c.z])
            .fold(f64::NEG_INFINITY, f64::max);

        let min_bound = self.x_size.min(self.y_size.min(self.z_size)); // need to change that later because it clamp the model size in the lower dimension of the grid

        let scale_x =
            (min_bound - 1) as f64 / (x_max - x_min).max(f64::EPSILON) * scale;
        let scale_y =
            (min_bound - 1) as f64 / (y_max - y_min).max(f64::EPSILON) * scale;
        let scale_z =
            (min_bound - 1) as f64 / (z_max - z_min).max(f64::EPSILON) * scale;

        let offset_x = (min_bound as f64 - (x_max - x_min) * scale_x) / 2.0;
        let offset_y = (min_bound as f64 - (y_max - y_min) * scale_y) / 2.0;
        let offset_z = 0.0;

        let norm = |t: &Triangle| Triangle {
            a: Vec3::new(
                (t.a.x - x_min) * scale_x + offset_x,
                (t.a.y - y_min) * scale_y + offset_y,
                (t.a.z - z_min) * scale_z + offset_z,
            ),
            b: Vec3::new(
                (t.b.x - x_min) * scale_x + offset_x,
                (t.b.y - y_min) * scale_y + offset_y,
                (t.b.z - z_min) * scale_z + offset_z,
            ),
            c: Vec3::new(
                (t.c.x - x_min) * scale_x + offset_x,
                (t.c.y - y_min) * scale_y + offset_y,
                (t.c.z - z_min) * scale_z + offset_z,
            ),
        };

        let triangles: Vec<Triangle> = triangles.iter().map(norm).collect();

        // iter on planes
        for z_idx in 0..self.z_size {
            let z = z_idx as f64;
            // iter on triangles
            for triangle in &triangles {
                let (z_low, z_high) = triangle.z_bounds();
                if z < z_low - f64::EPSILON || z > z_high + f64::EPSILON {
                    continue;
                }
                if is_triangle_in_plane(triangle, z) {
                    fill_triangle(
                        &mut self.slices[z_idx],
                        self.x_size,
                        self.y_size,
                        triangle,
                    );
                } else if let Some([p0, p1]) =
                    get_triangle_plane_intersection(triangle, z)
                {
                    fill_segment(
                        &mut self.slices[z_idx],
                        self.x_size,
                        self.y_size,
                        p0,
                        p1,
                    );
                }
            }
            fill_model_slice(&mut self.slices[z_idx], self.x_size, self.y_size);
        }
    }

    pub fn get_slice(&self, z_idx: usize) -> &Vec<Vec<bool>> {
        &self.slices[z_idx]
    }

    pub fn print_slice(&self, z_idx: usize) {
        let slice = &self.slices[z_idx];
        println!("=== Slice z={} ===", z_idx);
        for y in (0..self.y_size).rev() {
            for x in 0..self.x_size {
                print!("{}", if slice[x][y] { "█" } else { "·" });
            }
            println!();
        }
    }

    pub fn create_layers(&self, path: &std::path::Path) {
        for z in 0..self.slices.len() {
            let layer: ImageBuffer<Rgb<u8>, Vec<u8>> = create_layer(
                &self.slices[z],
                self.x_size as u32,
                self.y_size as u32,
            );
            layer.save(path.join(format!("{}.png", z + 1))).unwrap();
        }
    }

    pub fn clean_model(&mut self) {
        // remove top located empty layers
        let mut wipe: isize = -1;
        for z in 0..self.slices.len() {
            wipe = z as isize;
            for y in 0..self.y_size {
                for x in 0..self.x_size {
                    if self.slices[z][x][y] == true {
                        wipe = -1;
                    }
                }
            }
            if wipe != -1 {
                break;
            }
        }
        if wipe != -1 {
            self.slices.truncate(wipe as usize);
        }
    }
}