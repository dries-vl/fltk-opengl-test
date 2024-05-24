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

pub fn get_vertices() -> Vec<f32> {
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
    let vertices_with_normals = compute_normals(&vertices, &indices);
    vertices_with_normals.into_iter().flatten().collect()
}

pub fn get_indices() -> Vec<u16> {
    vec![
        11, 5, 0, 5, 1, 0, 1, 7, 0, 7, 10, 0, 10, 11, 0, 5, 9, 1, 11, 4, 5, 10, 2, 11, 7, 6, 10, 1,
        8, 7, 9, 4, 3, 4, 2, 3, 2, 6, 3, 6, 8, 3, 8, 9, 3, 9, 5, 4, 4, 11, 2, 2, 10, 6, 6, 7, 8, 1,
        9, 8,
    ]
}

pub fn convert_triangles_to_lines(triangle_indices: &[u16]) -> Vec<u16> {
    let mut line_indices = std::collections::HashSet::new();

    for i in (0..triangle_indices.len()).step_by(3) {
        let a = triangle_indices[i];
        let b = triangle_indices[i + 1];
        let c = triangle_indices[i + 2];

        line_indices.insert((a.min(b), a.max(b)));
        line_indices.insert((b.min(c), b.max(c)));
        line_indices.insert((c.min(a), c.max(a)));
    }

    let mut line_array: Vec<u16> = line_indices
        .into_iter()
        .flat_map(|(min, max)| vec![min, max])
        .collect();
    line_array.sort();
    line_array
}

pub fn cube_indices_lines() -> Vec<u16> {
    convert_triangles_to_lines(&get_indices())
}
