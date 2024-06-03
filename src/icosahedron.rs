use std::collections::HashMap;
use cgmath::{InnerSpace, Vector3};

const PHI: f32 = (1.0 + 2.23606) / 2.0; // 2.236 is sqrt(5)

fn get_indices() -> Vec<u16> {
    vec![
        11, 5, 0, 5, 1, 0, 1, 7, 0, 7, 10, 0, 10, 11, 0, 5, 9, 1, 11, 4, 5, 10, 2, 11, 7, 6, 10, 1,
        8, 7, 9, 4, 3, 4, 2, 3, 2, 6, 3, 6, 8, 3, 8, 9, 3, 9, 5, 4, 4, 11, 2, 2, 10, 6, 6, 7, 8, 1,
        9, 8,
    ]
}

fn calculate_uv(x: f32, y: f32, z: f32) -> [f32; 5] {
    let length = (x * x + y * y + z * z).sqrt();
    let mut u = 0.5 + (z.atan2(x) / (2.0 * std::f32::consts::PI));
    let mut v = 0.5 - (y / length).asin() / std::f32::consts::PI;

    [x, y, z, u, v]
}

pub fn get_vertices() -> (Vec<[f32; 9]>, Vec<u16>) {
    let mut vertices = vec![
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
    let mut indices = get_indices();
    let (mut vertices, mut indices) = transform_to_unique_vertices(&vertices, &indices);
    let (vertices, indices) = subdivide_icosahedron(&vertices, &indices);
    let (vertices, indices) = subdivide_icosahedron(&vertices, &indices);
    let (mut vertices, mut indices) = subdivide_icosahedron(&vertices, &indices);
    let duplicated_vertices = repair_texture_wrap_seam(&mut vertices, &mut indices);
    let vertices = compute_normals(&vertices, &indices, &duplicated_vertices);
    (vertices, indices)
}

fn transform_to_unique_vertices(original_vertices: &Vec<[f32; 5]>, indices: &Vec<u16>) -> (Vec<[f32; 6]>, Vec<u16>) {
    // Create a new vertices array where each triangle has unique vertices
    let mut new_vertices = Vec::new();
    let mut new_indices = Vec::new();
    let mut index_count = 0;

    // create duplicated vertices and add barys
    for i in (0..indices.len()).step_by(3) {
        {
            let [x, y, z, u, v] = original_vertices[indices[i] as usize];
            let bary = index_count % 3;
            new_vertices.push([x, y, z, u, v, bary as f32]);
        }
        {
            let [x, y, z, u, v] = original_vertices[indices[i + 1] as usize];
            let bary = (index_count + 1) % 3;
            new_vertices.push([x, y, z, u, v, bary as f32]);
        }
        {
            let [x, y, z, u, v] = original_vertices[indices[i + 2] as usize];
            let bary = (index_count + 2) % 3;
            new_vertices.push([x, y, z, u, v, bary as f32]);
        }

        // Create new sequential indices
        new_indices.push(index_count);
        new_indices.push(index_count + 1);
        new_indices.push(index_count + 2);
        index_count += 3;
    }
    (new_vertices, new_indices)
}

fn subdivide_icosahedron(vertices: &Vec<[f32; 6]>, indices: &Vec<u16>) -> (Vec<[f32; 6]>, Vec<u16>) {
    let mut new_vertices = vertices.clone();
    let mut new_indices = Vec::new();
    let mut midpoint_index_cache = HashMap::new();

    // Iterate over each triangle and create 4 new ones
    for chunk in indices.chunks(3) {
        let v0 = chunk[0] as usize;
        let v1 = chunk[1] as usize;
        let v2 = chunk[2] as usize;

        let v12 = vertex_for_edge(v1, v2, &vertices, &mut new_vertices, &mut midpoint_index_cache); // -> 0
        let v20 = vertex_for_edge(v2, v0, &vertices, &mut new_vertices, &mut midpoint_index_cache); // -> 1
        let v01 = vertex_for_edge(v0, v1, &vertices, &mut new_vertices, &mut midpoint_index_cache); // -> 2

        new_indices.extend_from_slice(&[v0 as u16, v01, v20]);
        new_indices.extend_from_slice(&[v01, v1 as u16, v12]);
        new_indices.extend_from_slice(&[v20, v12, v2 as u16]);
        new_indices.extend_from_slice(&[v01, v12, v20]);
    }

    (new_vertices, new_indices)
}

fn vertex_for_edge(v1: usize, v2: usize, vertices: &Vec<[f32; 6]>, new_vertices: &mut Vec<[f32; 6]>, cache: &mut HashMap<(usize, usize), u16>) -> u16 {
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
        3.0 - p1[5] - p2[5] // calculate bary; 3 minus the sum of the other two gives the correct bary
    ];

    // Normalize to same length as radius of sphere
    let length_p1 = (p1[0] * p1[0] + p1[1] * p1[1] + p1[2] * p1[2]).sqrt();
    let length = (midpoint[0] * midpoint[0] + midpoint[1] * midpoint[1] + midpoint[2] * midpoint[2]).sqrt();
    midpoint[0] *= length_p1 / length;
    midpoint[1] *= length_p1 / length;
    midpoint[2] *= length_p1 / length;
    // Calculate UV coordinates for the new vertex
    // midpoint[3] = 0.5 + (midpoint[2].atan2(midpoint[0]) / (2.0 * std::f32::consts::PI));
    // midpoint[4] = 0.5 - (midpoint[1] / length).asin() / std::f32::consts::PI;
    let [x, y, z, u, v] = calculate_uv(midpoint[0], midpoint[1], midpoint[2]);
    midpoint = [x, y, z, u, v, midpoint[5]];

    let new_index = new_vertices.len() as u16;
    new_vertices.push(midpoint);
    cache.insert(key, new_index);
    new_index
}

fn compute_normals(vertices: &Vec<[f32; 6]>, indices: &Vec<u16>, duplicated_vertices: &HashMap<usize, usize>) -> Vec<[f32; 9]> {
    let mut temp_normals: Vec<Vec<f32>> = vec![vec![0.0; 3]; vertices.len()];
    let mut final_vertices = vec![];

    // calculate normal for each triangle -> add it up to each vertex belonging to that triangle

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
    // make sure vertices on the same location have the same normal by combining both
    for (&old_index, &new_index) in duplicated_vertices.iter() {
        let old_normal = Vector3::new(
            temp_normals[old_index][0],
            temp_normals[old_index][1],
            temp_normals[old_index][2],
        );
        let new_normal = Vector3::new(
            temp_normals[new_index][0],
            temp_normals[new_index][1],
            temp_normals[new_index][2],
        );

        let actual_normal = old_normal + new_normal;

        // Update the normals for both the old and new vertex
        for &index in &[old_index, new_index] {
            temp_normals[index][0] = actual_normal.x;
            temp_normals[index][1] = actual_normal.y;
            temp_normals[index][2] = actual_normal.z;
        }
    }

    // for each vertex, normalize the summed normals ~average across all triangles this vertex was part of
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
            vertex[5] // bary
        ]);
    }

    final_vertices
}

fn repair_texture_wrap_seam(vertices: &mut Vec<[f32; 6]>, indices: &mut Vec<u16>) -> HashMap<usize, usize> {
    let mut new_indices: Vec<u16> = Vec::new();
    let mut corrections = 0;
    // list of vertex indices and their corrected counterpart
    let mut correction_list = std::collections::HashMap::new();

    let mut i = indices.len() as isize - 3;
    while i >= 0 {
        let idx0 = indices[i as usize] as usize;
        let idx1 = indices[i as usize + 1] as usize;
        let idx2 = indices[i as usize + 2] as usize;

        let v0 = Vector3::new(vertices[idx0][3], vertices[idx0][4], 0.0);
        let v1 = Vector3::new(vertices[idx1][3], vertices[idx1][4], 0.0);
        let v2 = Vector3::new(vertices[idx2][3], vertices[idx2][4], 0.0);

        let cross = (v1 - v0).cross(v2 - v1);

        // TODO: not fixed close to poles yet

        // if "direction" of the uvs is unnatural, it is a bad triangle
        if cross.z <= 0.0 {
            // loop over the three indices of this triangle
            for j in i..i+3 {
                let index = indices[j as usize] as usize;
                // get the vertex
                let mut vertex = vertices[index].clone();

                // if the vertex uv.x is very high, create a new one with -1
                if vertex[3] <= 0.3 {
                    // don't duplicate a vertex that was already added this way
                    if let Some(&corrected_index) = correction_list.get(&index) {
                        new_indices.push(corrected_index as u16);
                    } else {
                        vertex[3] += 1.0;
                        corrections += 1;
                        vertices.push(vertex);
                        let corrected_vertex_index = (vertices.len() - 1) as u16;
                        correction_list.insert(index, corrected_vertex_index as usize);
                        new_indices.push(corrected_vertex_index);
                    }
                } else {
                    new_indices.push(index as u16);
                }
            }
        } else {
            new_indices.extend_from_slice(&indices[(i as usize)..(i as usize + 3)]);
        }

        i -= 3;
    }

    indices.clear();
    indices.extend_from_slice(&new_indices);
    correction_list
}
