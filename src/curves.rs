use rand::Rng;

pub fn generate_random_bezier_points() -> Vec<(f32, f32)> {
    // Predefined points for a smooth, curving road
    // These points are chosen to represent a gentle S-curve or similar road shapes
    Vec::from([
        (0.0, 0.0),        // Starting point
        (1.0, 1.0),        // First control point, starts the curve
        (2.0, 3.0),      // Second control point, continues the curve
        (2.4, 3.0),      // Third control point, begins to straighten the curve
        (3.0, 3.0),      // Fourth control point, ends the curve
        (4.0, 5.0),      // Fifth control point, another curve begins
        (5.0, 5.0),      // Sixth control point, curve deepens
        (5.5, 6.0),      // Seventh control point, curve starts to straighten
        (7.0, 6.0),       // Eighth control point, curve ends
        (8.0, 5.5)        // Ending point of the road
    ])
}

pub fn points_on_sphere(points: Vec<(f32, f32)>) -> Vec<(f32, f32, f32)> {
    let max_theta = 80.0;  // Maximum extent of theta (longitude)
    let max_phi = 25.0;    // Maximum extent of phi (latitude)

    points.iter().map(|&(theta_deg, phi_deg)| {
        // Scale and convert degrees to radians
        let theta = (theta_deg / max_theta * 360.0).to_radians();
        let phi = (phi_deg / max_phi * 180.0 - 90.0).to_radians();

        // Convert spherical to Cartesian coordinates
        let x = theta.cos() * phi.sin();
        let y = theta.sin() * phi.sin();
        let z = phi.cos();

        (x * 2.03, y * 2.03, z * 2.03)
    }).collect()
}


pub fn bezier_curve_interpolation(points: Vec<(f32, f32)>) -> Vec<(f32, f32)> {
    let mut curve_points = Vec::new();
    let num_segments = 10;  // Uniform number of segments for each curve

    for window in points.windows(2) {
        if let [p0, p1] = window {
            let control_points = generate_control_points(*p0, *p1);
            for i in 0..=num_segments {
                let t = i as f32 / num_segments as f32;
                let x = bezier_point(t, p0.0, control_points.0.0, control_points.1.0, p1.0);
                let y = bezier_point(t, p0.1, control_points.0.1, control_points.1.1, p1.1);
                curve_points.push((x, y));
            }
        }
    }

    // Ensure the last point is included
    if let Some(&last) = points.last() {
        curve_points.push(last);
    }

    curve_points
}

fn generate_control_points(p0: (f32, f32), p1: (f32, f32)) -> ((f32, f32), (f32, f32)) {
    // Adjusted control points for smoother curve
    let mid_point = ((p0.0 + p1.0) / 2.0, (p0.1 + p1.1) / 2.0);
    let ctrl1 = (mid_point.0, p0.1);
    let ctrl2 = (mid_point.0, p1.1);
    (ctrl1, ctrl2)
}

fn bezier_point(t: f32, p0: f32, p1: f32, p2: f32, p3: f32) -> f32 {
    let one_minus_t = 1.0 - t;
    (one_minus_t.powi(3) * p0) +
        (3.0 * one_minus_t.powi(2) * t * p1) +
        (3.0 * one_minus_t * t.powi(2) * p2) +
        (t.powi(3) * p3)
}
