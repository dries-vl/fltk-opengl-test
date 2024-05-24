use std::collections::HashMap;

const PHI: f32 = (1.0 + 2.23606) / 2.0; // 2.236 is sqrt(5)

fn calculate_uv(x: f32, y: f32, z: f32) -> [f32; 5] {
    let length = (x * x + y * y + z * z).sqrt();
    let u = 0.5 + (z.atan2(x) / (2.0 * std::f32::consts::PI));
    let v = 0.5 - (y / length).asin() / std::f32::consts::PI;
    [x, y, z, u, v]
}

fn compute_normals(vertices: &Vec<[f32; 5]>, indices: &Vec<u16>) -> Vec<[f32; 8]> {
    let mut temp_normals: Vec<Vec<f32>> = vec![vec![0.0; 3]; vertices.len()];
    let mut final_vertices = vec![];

    for chunk in indices.chunks(3) {
        let idx0 = chunk[0] as usize;
        let idx1 = chunk[1] as usize;
        let idx2 = chunk[2] as usize;

        let v0 = &vertices[idx0];
        let v1 = &vertices[idx1];
        let v2 = &vertices[idx2];

        let u = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let v = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

        let normal = [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ];

        let length = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();

        for &i in &[idx0, idx1, idx2] {
            temp_normals[i][0] += normal[0] / length;
            temp_normals[i][1] += normal[1] / length;
            temp_normals[i][2] += normal[2] / length;
        }
    }

    for (i, vertex) in vertices.iter().enumerate() {
        let len = (temp_normals[i][0] * temp_normals[i][0]
            + temp_normals[i][1] * temp_normals[i][1]
            + temp_normals[i][2] * temp_normals[i][2])
            .sqrt();
        final_vertices.push([
            vertex[0],
            vertex[1],
            vertex[2], // Position
            temp_normals[i][0] / len,
            temp_normals[i][1] / len,
            temp_normals[i][2] / len, // Normal
            vertex[3],
            vertex[4], // UV Coordinates
        ]);
    }

    final_vertices
}

pub fn get_vertices() -> (Vec<[f32; 8]>, Vec<u16>) {
    let vertices = vec![
        calculate_uv(-1.0, PHI, 0.0),
        calculate_uv(1.0, PHI, 0.0),
        calculate_uv(-1.0, -PHI, 0.0),
        calculate_uv(1.0, -PHI, 0.0),
        calculate_uv(0.0, -1.0, PHI),
        calculate_uv(0.0, 1.0, PHI),
        calculate_uv(0.0, -1.0, -PHI),
        calculate_uv(0.0, 1.0, -PHI),
        calculate_uv(PHI, 0.0, -1.0),
        calculate_uv(PHI, 0.0, 1.0),
        calculate_uv(-PHI, 0.0, -1.0),
        calculate_uv(-PHI, 0.0, 1.0),
    ];
    let indices = get_indices();
    let (vertices, indices) = subdivide_icosahedron(&vertices, &indices);
    let (vertices, indices) = subdivide_icosahedron(&vertices, &indices);
    let (vertices, indices) = subdivide_icosahedron(&vertices, &indices);
    (compute_normals(&vertices, &indices), indices)
}

fn get_indices() -> Vec<u16> {
    vec![
        11, 5, 0, 5, 1, 0, 1, 7, 0, 7, 10, 0, 10, 11, 0, 5, 9, 1, 11, 4, 5, 10, 2, 11, 7, 6, 10, 1,
        8, 7, 9, 4, 3, 4, 2, 3, 2, 6, 3, 6, 8, 3, 8, 9, 3, 9, 5, 4, 4, 11, 2, 2, 10, 6, 6, 7, 8, 1,
        9, 8,
    ]
}

fn subdivide_icosahedron(vertices: &Vec<[f32; 5]>, indices: &Vec<u16>) -> (Vec<[f32; 5]>, Vec<u16>) {
    let mut new_vertices = vertices.clone();
    let mut new_indices = Vec::new();
    let mut midpoint_index_cache = HashMap::new();

    // Iterate over each triangle and create 4 new ones
    for chunk in indices.chunks(3) {
        let v1 = chunk[0] as usize;
        let v2 = chunk[1] as usize;
        let v3 = chunk[2] as usize;

        let a = vertex_for_edge(v1, v2, &vertices, &mut new_vertices, &mut midpoint_index_cache);
        let b = vertex_for_edge(v2, v3, &vertices, &mut new_vertices, &mut midpoint_index_cache);
        let c = vertex_for_edge(v3, v1, &vertices, &mut new_vertices, &mut midpoint_index_cache);

        new_indices.extend_from_slice(&[chunk[0], a, c]);
        new_indices.extend_from_slice(&[a, chunk[1], b]);
        new_indices.extend_from_slice(&[b, chunk[2], c]);
        new_indices.extend_from_slice(&[a, b, c]);
    }

    (new_vertices, new_indices)
}

// Function to find or create a vertex
fn vertex_for_edge(
    v1: usize,
    v2: usize,
    vertices: &Vec<[f32; 5]>,
    new_vertices: &mut Vec<[f32; 5]>,
    cache: &mut HashMap<(usize, usize), u16>
) -> u16 {
    let key = if v1 < v2 { (v1, v2) } else { (v2, v1) };
    if let Some(&index) = cache.get(&key) {
        return index;
    }
    let p1 = &vertices[v1];
    let p2 = &vertices[v2];
    let mut midpoint = [
        (p1[0] + p2[0]) / 2.0,
        (p1[1] + p2[1]) / 2.0,
        (p1[2] + p2[2]) / 2.0,
        0.0,  // u, v will be calculated later
        0.0,
    ];

    // Normalize to place on unit sphere
    let length = (midpoint[0] * midpoint[0] + midpoint[1] * midpoint[1] + midpoint[2] * midpoint[2]).sqrt() * 0.5;
    midpoint[0] /= length;
    midpoint[1] /= length;
    midpoint[2] /= length;
    // Calculate UV coordinates for the new vertex
    midpoint[3] = 0.5 + (midpoint[2].atan2(midpoint[0]) / (2.0 * std::f32::consts::PI));
    midpoint[4] = 0.5 - (midpoint[1] / length).asin() / std::f32::consts::PI;
    midpoint = calculate_uv(midpoint[0], midpoint[1], midpoint[2]);

    let new_index = new_vertices.len() as u16;
    new_vertices.push(midpoint);
    cache.insert(key, new_index);
    new_index
}
