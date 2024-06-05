// #![windows_subsystem="windows"]

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::ffi::CString;
use std::ptr;
use std::rc::Rc;
use std::thread::sleep_ms;
use std::time::{Duration, Instant};

use cgmath::{Deg, Matrix, Matrix4, Point3, SquareMatrix, Vector3};
use fltk::{app, image::IcoImage, prelude::*, window::GlWindow};
use fltk::app::{event_button, event_dy, event_x, event_y, MouseButton, MouseWheel, sleep};
use fltk::enums::{Event, Key};
use gl::types::{GLchar, GLfloat, GLint, GLsizei, GLsizeiptr, GLuint, GLuint64};
use rand::Rng;

mod icosahedron;
mod shader_utils;
mod texture;
mod curves;

const W: i32 = 1200;
const H: i32 = 800;

fn main() {
    let app = app::App::default();
    let mut wind = GlWindow::new(100, 100, W, H, "Smooth Camera Control Example");
    let icon: IcoImage = IcoImage::load(std::path::Path::new("fltk.ico")).unwrap();
    wind.make_resizable(true);
    wind.set_icon(Some(icon));
    wind.end();
    wind.show();

    wind.make_current(); // This ensures the OpenGL context is current on this thread
    gl::load_with(|s| wind.get_proc_address(s) as *const _); // This is where you load the OpenGL functions

    // region: -- sphere
    let vertex_shader = include_str!("../shaders/vertex.glsl");
    let fragment_shader = include_str!("../shaders/fragment.glsl");

    let vertex_shader = shader_utils::compile_shader(vertex_shader, gl::VERTEX_SHADER);
    let fragment_shader = shader_utils::compile_shader(fragment_shader, gl::FRAGMENT_SHADER);
    let shader_program = shader_utils::link_program(vertex_shader, fragment_shader);

    // Setup vertex data and buffers and configure vertex attributes
    let (raw_vertices, indices) = icosahedron::get_vertices();
    let vertices: Vec<f32> = raw_vertices.into_iter().flatten().collect();
    let mut vbo = 0;
    let mut vao = 0;
    let mut ebo = 0;

    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        // VERTICES
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<GLfloat>()) as isize,
            vertices.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );
        // add to gl
        let stride = 9 * std::mem::size_of::<GLfloat>() as GLsizei; // 9 floats per vertex
        // add x, y, z at location 0
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(0);
        // add normals at location 1
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, stride, (3 * std::mem::size_of::<GLfloat>()) as *const _);
        gl::EnableVertexAttribArray(1);
        // add uv at location 2
        gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, stride, (6 * std::mem::size_of::<GLfloat>()) as *const _);
        gl::EnableVertexAttribArray(2);
        // add bary at location 3
        gl::VertexAttribPointer(3, 1, gl::FLOAT, gl::FALSE, stride, (8 * std::mem::size_of::<GLfloat>()) as *const _);
        gl::EnableVertexAttribArray(3);

        // INDICES
        gl::GenBuffers(1, &mut ebo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (indices.len() * std::mem::size_of::<u16>()) as isize,
            indices.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);

        gl::UseProgram(shader_program);

        // Add texture
        // this takes rather long
        let texture_id = texture::create_texture("C:/Users/dries/dev/fltk-opengl-test/target/debug/earth.png");
        gl::ActiveTexture(gl::TEXTURE0); // Activate the first texture unit
        gl::BindTexture(gl::TEXTURE_2D, texture_id);
        let texture_location = gl::GetUniformLocation(shader_program, CString::new("textureSampler").unwrap().as_ptr());
        gl::Uniform1i(texture_location, 0);

        // lighting
        let light_pos = gl::GetUniformLocation(shader_program, CString::new("light.position").unwrap().as_ptr());
        let light_ambient = gl::GetUniformLocation(shader_program, CString::new("light.ambient").unwrap().as_ptr());
        let light_diffuse = gl::GetUniformLocation(shader_program, CString::new("light.diffuse").unwrap().as_ptr());
        let light_specular = gl::GetUniformLocation(shader_program, CString::new("light.specular").unwrap().as_ptr());

        // Assuming you have set Vec3 struct to handle data
        gl::Uniform3fv(light_pos, 1, [-500.0, 500.0, -500.0].as_ptr()); // Example light position
        gl::Uniform3fv(light_ambient, 1, [0.5, 0.5, 0.5].as_ptr()); // Low intensity ambient light ~ 0.5 makes sense
        gl::Uniform3fv(light_diffuse, 1, [1.2, 1.2, 1.2].as_ptr()); // Medium intensity diffuse light ~1.25 makes sense
        gl::Uniform3fv(light_specular, 1, [0.95, 0.95, 0.95].as_ptr()); // Strong specular light ~0.75 makes sense

        // Set material properties
        let material_ambient = gl::GetUniformLocation(shader_program, CString::new("material.ambient").unwrap().as_ptr());
        let material_diffuse = gl::GetUniformLocation(shader_program, CString::new("material.diffuse").unwrap().as_ptr());
        let material_specular = gl::GetUniformLocation(shader_program, CString::new("material.specular").unwrap().as_ptr());
        let material_shininess = gl::GetUniformLocation(shader_program, CString::new("material.shininess").unwrap().as_ptr());

        gl::Uniform3fv(material_ambient, 1, [0.4, 0.4, 0.4].as_ptr());
        gl::Uniform3fv(material_diffuse, 1, [0.75, 0.75, 0.75].as_ptr());
        gl::Uniform3fv(material_specular, 1, [0.3, 0.3, 0.3].as_ptr());
        gl::Uniform1f(material_shininess, 32.0); // Shininess factor

        // cleanup
        gl::UseProgram(0);
    }
    // endregion: -- sphere

    // region: -- particles
    let num_particles = 256;
    let mut vao_particles = 1;
    let mut vbo_particles = 1;

    let vertex_shader_particles = include_str!("../shaders/particles_v.glsl");
    let fragment_shader_particles = include_str!("../shaders/particles_f.glsl");

    let vertex_shader = shader_utils::compile_shader(vertex_shader_particles, gl::VERTEX_SHADER);
    let fragment_shader = shader_utils::compile_shader(fragment_shader_particles, gl::FRAGMENT_SHADER);
    let particles_program = shader_utils::link_particles_program(vertex_shader, fragment_shader);
    unsafe {
        gl::GenVertexArrays(1, &mut vao_particles);
        gl::BindVertexArray(vao_particles);

        gl::GenBuffers(1, &mut vbo_particles);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo_particles);
        gl::BindBufferBase(gl::TRANSFORM_FEEDBACK_BUFFER, 0, vbo_particles);

        // Assuming each particle has a position and velocity (each 4 floats)
        let mut particles: Vec<f32> = Vec::with_capacity(num_particles * 8);
        for _ in 0..num_particles {
            particles.extend_from_slice(&[rand::thread_rng().gen_range(1.0..5.0), rand::thread_rng().gen_range(1.0..5.0), rand::thread_rng().gen_range(0.0..5.0), 1.0]);  // Position
            particles.extend_from_slice(&[rand::thread_rng().gen_range(0.001..0.01), rand::thread_rng().gen_range(0.001..0.01), 0.0, 0.0]);  // Velocity
        }
        gl::BufferData(gl::ARRAY_BUFFER,
                       (particles.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr,
                       particles.as_ptr() as *const _,
                       gl::STATIC_DRAW);

        gl::VertexAttribPointer(0, 4, gl::FLOAT, gl::FALSE, 8 * std::mem::size_of::<GLfloat>() as GLsizei, std::ptr::null());
        gl::EnableVertexAttribArray(0);

        gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, 8 * std::mem::size_of::<GLfloat>() as GLsizei, (4 * std::mem::size_of::<GLfloat>()) as *const _);
        gl::EnableVertexAttribArray(1);
    }
    // endregion: -- particles

    // region: -- lines
    let mut vao_lines = 2;
    let mut vbo_lines = 2;

    let vertex_shader_lines = include_str!("../shaders/lines_v.glsl");
    let fragment_shader_lines = include_str!("../shaders/lines_f.glsl");

    let vertex_shader = shader_utils::compile_shader(vertex_shader_lines, gl::VERTEX_SHADER);
    let fragment_shader = shader_utils::compile_shader(fragment_shader_lines, gl::FRAGMENT_SHADER);
    let lines_program = shader_utils::link_program(vertex_shader, fragment_shader);
    unsafe {
        gl::GenVertexArrays(1, &mut vao_lines);
        gl::BindVertexArray(vao_lines);

        gl::GenBuffers(1, &mut vbo_lines);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo_lines);

        // Assuming each particle has a position and velocity (each 4 floats)
        let line_points_curve = curves::bezier_curve_interpolation(curves::generate_random_bezier_points());
        let points_on_sphere = curves::points_on_sphere(line_points_curve);
        let mut float_vec = Vec::with_capacity(points_on_sphere.len() * 4); // Pre-allocate memory for efficiency
        for (x, y, z) in points_on_sphere {
            // For each (x, y) tuple, append x, y, 1.0, 1.0 to the new vector
            float_vec.extend_from_slice(&[x, y, z, 1.0]);
        }
        gl::BufferData(gl::ARRAY_BUFFER,
                       (float_vec.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr,
                       float_vec.as_ptr() as *const _,
                       gl::DYNAMIC_DRAW);

        gl::VertexAttribPointer(0, 4, gl::FLOAT, gl::FALSE, 4 * std::mem::size_of::<GLfloat>() as GLsizei, std::ptr::null());
        gl::EnableVertexAttribArray(0);

    }
    // endregion: -- lines

    // Clean up shaders (they're linked into the program now, so no longer needed separately)
    unsafe {
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);
    }

    // Other initializations like setting the background color, enabling depth test etc.
    unsafe {
        gl::ClearColor(0.1, 0.1, 0.1, 1.0); // Clear color
        gl::Enable(gl::DEPTH_TEST); // Enable depth test
        // gl::Disable(gl::CULL_FACE);
    }

    check_gl_error("During initialization");


    // CAMERA
    let camera_zoom = Rc::new(RefCell::new(5.0 as f32)); // Initial zoom distance
    let camera_zoom_rc = camera_zoom.clone();
    let camera_zoom_target = Rc::new(RefCell::new(5.0 as f32));
    let camera_zoom_target_rc = camera_zoom_target.clone();

    let camera_coordinates = Rc::new(RefCell::new((0.0 as f32, 0.0 as f32))); // horizontal, vertical angles
    let camera_coordinates_rc = camera_coordinates.clone();
    let camera_coordinates_rc_2 = camera_coordinates.clone();

    let mouse_position = Rc::new(RefCell::new((0, 0)));
    let mouse_position_rc = mouse_position.clone();

    let camera_rotation = Rc::new(RefCell::new((0.0 as f32, 0.0 as f32))); // New: for adjusting view direction
    let camera_rotation_rc = camera_rotation.clone();
    let camera_rotation_rc_2 = camera_rotation.clone();

    const ZOOM_SPEED: f32 = 0.2;
    const DRAG_SPEED: f32 = 0.2;


    // TIMING SHADERS
    let mut shader_timings = Vec::new();
    let mut timing_history: VecDeque<Vec<f32>> = VecDeque::with_capacity(10);

    let mut update_timing_history = move |new_timings: Vec<f32>| -> Vec<f32> {
        // Add new timing data to the deque
        if timing_history.len() == 10 {
            timing_history.pop_front(); // Remove the oldest entry if at capacity
        }
        timing_history.push_back(new_timings.clone());

        // Compute the average timings for each shader
        let num_entries = timing_history.len();
        let num_shaders = new_timings.len();
        let mut average_timings = vec![0.0; num_shaders];

        for frame_timings in &timing_history {
            for (i, timing) in frame_timings.iter().enumerate() {
                average_timings[i] += timing;
            }
        }

        for avg in &mut average_timings {
            *avg /= num_entries as f32;
        }

        average_timings
    };

    // window draw call
    wind.draw(move |_| {
        shader_timings = draw(&shader_program, &particles_program, &lines_program, vao, vao_particles, vao_lines, &vertices, &indices, &camera_coordinates_rc.borrow(), *camera_zoom_rc.borrow());

        // Update the timing history and calculate the average of the last ten frames
        let average_shader_timings = update_timing_history(shader_timings.clone());
        println!("Average shader timings over the last ten frames: {:?}", average_shader_timings);
    });

    // region: -- windowing
    let key_states = Rc::new(RefCell::new(HashMap::new()));
    let key_states_rc = key_states.clone();

    wind.handle(move |_, ev| {
        match ev {
            Event::MouseWheel => {
                let mut zoom_target = camera_zoom_target_rc.borrow_mut();
                if event_dy() == MouseWheel::Up {
                    *zoom_target += f32::max(0.25, *zoom_target - 1.0) * ZOOM_SPEED;
                } else if event_dy() == MouseWheel::Down {
                    *zoom_target -= f32::max(0.25, *zoom_target - 1.0) * ZOOM_SPEED;
                }
                *zoom_target = zoom_target.max(1.01).min(10.0);
                true
            }
            Event::Push if event_button() == MouseButton::Left as i32 => {
                *mouse_position_rc.borrow_mut() = (event_x(), event_y());
                true
            }
            Event::Drag if event_button() == MouseButton::Left as i32 => {
                let (prev_x, prev_y) = *mouse_position_rc.borrow();
                let (new_x, new_y) = (event_x(), event_y());
                *mouse_position_rc.borrow_mut() = (new_x, new_y);

                let mut angles = camera_coordinates_rc_2.borrow_mut();
                angles.0 += (new_x - prev_x) as f32 * DRAG_SPEED;
                angles.1 += (new_y - prev_y) as f32 * DRAG_SPEED;
                true
            }
            Event::Push if event_button() == MouseButton::Middle as i32 => {
                *mouse_position_rc.borrow_mut() = (event_x(), event_y());
                true
            }
            Event::Drag if event_button() == MouseButton::Middle as i32 => {
                let (prev_x, prev_y) = *mouse_position_rc.borrow();
                let (new_x, new_y) = (event_x(), event_y());
                *mouse_position_rc.borrow_mut() = (new_x, new_y);

                camera_rotation_rc_2.borrow_mut().0 += (new_x - prev_x) as f32 * 0.25;
                camera_rotation_rc_2.borrow_mut().1 += (new_y - prev_y) as f32 * 0.25;
                true
            }
            Event::KeyDown => {
                let mut keys = key_states_rc.borrow_mut();
                keys.insert(app::event_key(), true);
                if app::event_key() == Key::Escape {
                    app::quit()
                }
                true
            }
            Event::KeyUp => {
                let mut keys = key_states_rc.borrow_mut();
                keys.insert(app::event_key(), false);
                true
            }
            _ => false,
        }
    });
    let mut last_time = Instant::now();
    const CAMERA_SPEED: f32 = 100.0;
    while app.wait() {
        let current_time = Instant::now();
        let delta_ms = current_time.duration_since(last_time);
            let keys = key_states.borrow();
            let mut coordinates = camera_coordinates.borrow_mut();
            let mut zoom = camera_zoom.borrow_mut();

            if *keys.get(&Key::from_char('w')).unwrap_or(&false) {
                coordinates.1 += delta_ms.as_secs_f32() * CAMERA_SPEED;
            }
            if *keys.get(&Key::from_char('a')).unwrap_or(&false) {
                coordinates.0 += delta_ms.as_secs_f32() * CAMERA_SPEED;
            }
            if *keys.get(&Key::from_char('r')).unwrap_or(&false) {
                coordinates.1 -= delta_ms.as_secs_f32() * CAMERA_SPEED;
            }
            if *keys.get(&Key::from_char('s')).unwrap_or(&false) {
                coordinates.0 -= delta_ms.as_secs_f32() * CAMERA_SPEED;
            }
            let zoom_target = *camera_zoom_target.borrow();
            let zoom_diff = zoom_target - *zoom;
            if zoom_diff > 0.0 {
                *zoom += f32::max(zoom_diff * 0.05, f32::min(zoom_diff, 0.0001));
            } else if zoom_diff < 0.0 {
                *zoom += f32::min(zoom_diff * 0.05, f32::max(zoom_diff, -0.0001));
            }

            wind.redraw();
            sleep(0.016); // sleep 16ms for 60fps
            last_time = current_time;
    }
    // endregion: -- windowing

    // Cleanup resources after the loop ends
    unsafe {
        gl::DeleteVertexArrays(1, &mut vao);
        gl::DeleteBuffers(1, &mut vbo);
        gl::DeleteProgram(shader_program);
    }
}

fn draw(
    shader_program: &GLuint,
    particles_program: &GLuint,
    lines_program: &gl::types::GLuint,
    vao: GLuint,
    vao_particles: GLuint,
    vao_lines: GLuint,
    vertices: &Vec<f32>,
    indices: &Vec<u16>,
    sphere_rotation: &(f32, f32),
    zoom: f32
) -> Vec<f32> {
    unsafe {
        // Clear the screen and depth buffer
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        // Timer objects
        // TODO: time using program, setting uniforms and executing draw each
        // TODO: create method for executing the shader program + measuring it with a macro
        let mut queries = vec![0; 4]; // Assuming 2 shaders, hence 4 queries (2 per shader)
        gl::GenQueries(4, queries.as_mut_ptr());

        gl::BeginQuery(gl::TIME_ELAPSED, queries[0]); // time the execution

        // Bind the shader program and VAO
        gl::UseProgram(*shader_program);
        gl::BindVertexArray(vao);

        // Draw the sphere
        // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
        gl::DrawElements(
            gl::TRIANGLES,
            indices.len() as i32,
            gl::UNSIGNED_SHORT,
            ptr::null(),
        );

        gl::EndQuery(gl::TIME_ELAPSED); // end the timer

        gl::BeginQuery(gl::TIME_ELAPSED, queries[1]); // time the execution

        let camera_x = zoom * sphere_rotation.0.to_radians().cos() * sphere_rotation.1.to_radians().cos();
        let camera_y = zoom * sphere_rotation.1.to_radians().sin();
        let camera_z = zoom * sphere_rotation.0.to_radians().sin() * sphere_rotation.1.to_radians().cos();

        let eye = Point3::new(camera_x, camera_y, camera_z); // Camera's position
        let target = Point3::new(0.0, 0.0, 0.0); // Where the camera is looking
        let up = Vector3::new(0.0, 1.0, 0.0); // 'Up' direction in world space

        let view = Matrix4::look_at(eye, target, up);
        let projection = cgmath::perspective(Deg(45.0), W as f32 / H as f32, 0.1, 100.0);
        let model = Matrix4::<f32>::identity(); // Model matrix, for example

        let view_loc = gl::GetUniformLocation(*shader_program, CString::new("view").unwrap().as_ptr());
        let proj_loc = gl::GetUniformLocation(*shader_program, CString::new("projection").unwrap().as_ptr());
        let model_loc = gl::GetUniformLocation(*shader_program, CString::new("model").unwrap().as_ptr());
        let view_pos_location = gl::GetUniformLocation(*shader_program, CString::new("viewPos").unwrap().as_ptr());
        gl::UniformMatrix4fv(view_loc, 1, gl::FALSE, view.as_ptr());
        gl::UniformMatrix4fv(proj_loc, 1, gl::FALSE, projection.as_ptr());
        gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, model.as_ptr());
        gl::Uniform3fv(view_pos_location, 1, [camera_x, camera_y, camera_z].as_ptr());

        gl::EndQuery(gl::TIME_ELAPSED); // end the timer

        // Unbind the VAO and the shader program
        // gl::BindVertexArray(0);
        // gl::UseProgram(0);

        gl::BeginQuery(gl::TIME_ELAPSED, queries[2]); // time the execution
        // PARTICLES
        // Bind the shader program and VAO
        gl::UseProgram(*particles_program);
        gl::BindVertexArray(vao_particles);


        // Enable Transform Feedback and disable rasterization to update particles
        gl::Enable(gl::RASTERIZER_DISCARD); // Disable rasterization during feedback
        gl::BeginTransformFeedback(gl::POINTS);
        gl::DrawArrays(gl::POINTS, 0, 256); // Draw particles to update their positions
        gl::EndTransformFeedback();
        gl::Disable(gl::RASTERIZER_DISCARD); // Re-enable rasterization

        // Optionally: Draw the updated particles
        // To visualize the particles, you would usually draw them again here
        // For debugging, you can draw them without disabling rasterization to see the update immediately
        gl::PointSize(100.0);
        gl::DrawArrays(gl::POINTS, 0, 256); // Draw updated particles

        let view_loc = gl::GetUniformLocation(*particles_program, CString::new("view").unwrap().as_ptr());
        let proj_loc = gl::GetUniformLocation(*particles_program, CString::new("projection").unwrap().as_ptr());
        let model_loc = gl::GetUniformLocation(*particles_program, CString::new("model").unwrap().as_ptr());
        // let view_pos_location = gl::GetUniformLocation(*particles_program, CString::new("viewPos").unwrap().as_ptr());
        gl::UniformMatrix4fv(view_loc, 1, gl::FALSE, view.as_ptr());
        gl::UniformMatrix4fv(proj_loc, 1, gl::FALSE, projection.as_ptr());
        gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, model.as_ptr());
        // gl::Uniform3fv(view_pos_location, 1, [camera_x, camera_y, camera_z].as_ptr());
        // END PARTICLES
        gl::EndQuery(gl::TIME_ELAPSED); // end the timer

        gl::BeginQuery(gl::TIME_ELAPSED, queries[3]); // time the execution
        // LINES
        // Bind the shader program and VAO
        gl::UseProgram(*lines_program);
        gl::BindVertexArray(vao_lines);

        gl::LineWidth(10.0);
        gl::DrawArrays(gl::LINE_STRIP, 0, 100); // Draw updated particles

        let view_loc = gl::GetUniformLocation(*lines_program, CString::new("view").unwrap().as_ptr());
        let proj_loc = gl::GetUniformLocation(*lines_program, CString::new("projection").unwrap().as_ptr());
        let model_loc = gl::GetUniformLocation(*lines_program, CString::new("model").unwrap().as_ptr());
        // let view_pos_location = gl::GetUniformLocation(*particles_program, CString::new("viewPos").unwrap().as_ptr());
        gl::UniformMatrix4fv(view_loc, 1, gl::FALSE, view.as_ptr());
        gl::UniformMatrix4fv(proj_loc, 1, gl::FALSE, projection.as_ptr());
        gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, model.as_ptr());
        // gl::Uniform3fv(view_pos_location, 1, [camera_x, camera_y, camera_z].as_ptr());
        // END LINES
        gl::EndQuery(gl::TIME_ELAPSED); // end the timer

        // Unbind the VAO and the shader program
        gl::BindVertexArray(0);
        gl::UseProgram(0);

        // Retrieving and summing up the times
        let mut timings = Vec::with_capacity(queries.len());
        for (i, query) in queries.iter().enumerate() {
            let mut time: GLint = 0;
            let query_id = *query - 4; // -4 to get those of previous frame
            gl::GetQueryObjectiv(query_id, gl::QUERY_RESULT, &mut time);
            timings.push(time as f32 / 1_000_000.0);
        }


        check_gl_error("During draw call");

        timings

        // println!("rotation: {}, {}", camera_rotation.0, camera_rotation.1);
        // println!("position: {}, {}, {}", camera_x, camera_y, camera_z);
        // println!("looking at: {}, {}, {}", look_at_x, look_at_y, look_at_z);
    }
}

fn check_gl_error(operation: &str) {
    let error = unsafe { gl::GetError() };
    if error != gl::NO_ERROR {
        eprintln!("OpenGL error 0x{:X} after {}", error, operation);
    }
}
