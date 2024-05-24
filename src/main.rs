#![windows_subsystem="windows"]

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CString;
use std::ptr;
use std::rc::Rc;
use std::time::Instant;

use cgmath::{Deg, Matrix, Matrix4, Point3, SquareMatrix, Vector3};
use fltk::{app, image::IcoImage, prelude::*, window::GlWindow};
use fltk::app::{event_button, event_dy, event_x, event_y, MouseButton, MouseWheel};
use fltk::enums::{Event, Key};
use gl::types::{GLfloat, GLsizei, GLuint};

mod icosahedron;
mod shader_utils;
mod texture;

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

    // region: -- init shader
    let vs_src = r#"
    #version 330 core
layout (location = 0) in vec3 position;
layout (location = 2) in vec2 texCoords;
layout (location = 1) in vec3 normal;  // Input for normals

out vec3 FragPos;
out vec3 Normal;
out vec2 TexCoords;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main()
{
    FragPos = vec3(model * vec4(position, 1.0)); // Position in world space
    Normal = mat3(transpose(inverse(model))) * normal; // Transform normals
    TexCoords = texCoords;
    gl_Position = projection * view * model * vec4(position, 1.0);
}
    "#;

    let fs_src = r#"
#version 330 core
out vec4 FragColor;

in vec3 Normal;
in vec3 FragPos;
in vec2 TexCoords;

struct Material {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    float shininess;
};

struct Light {
    vec3 position;
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

uniform Material material;
uniform Light light;
uniform vec3 viewPos;  // Camera position
uniform sampler2D textureSampler;

void main()
{
    // Ambient
    vec3 ambient = light.ambient * material.ambient;
    
    // Diffuse 
    vec3 norm = normalize(Normal);
    vec3 lightDir = normalize(light.position - FragPos);
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = light.diffuse * (diff * material.diffuse);
    
    // Specular
    vec3 viewDir = normalize(viewPos - FragPos);
    vec3 reflectDir = reflect(-lightDir, norm);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
    vec3 specular = light.specular * (spec * material.specular);
    
    vec3 light = ambient + diffuse + specular;
    vec3 result = texture(textureSampler, TexCoords).rgb * light;
    FragColor = vec4(result, 1.0);
}
    "#;

    let vertex_shader = shader_utils::compile_shader(vs_src, gl::VERTEX_SHADER);
    let fragment_shader = shader_utils::compile_shader(fs_src, gl::FRAGMENT_SHADER);
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
        // INDICES
        gl::GenBuffers(1, &mut ebo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (indices.len() * std::mem::size_of::<u16>()) as isize,
            indices.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        // add to gl
        let stride = 8 * std::mem::size_of::<GLfloat>() as GLsizei; // 5 floats per vertex
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(0); // Position attribute

        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (3 * std::mem::size_of::<GLfloat>()) as *const _,
        );
        gl::EnableVertexAttribArray(1); // Normal attribute

        gl::VertexAttribPointer(
            2,
            2,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (6 * std::mem::size_of::<GLfloat>()) as *const _,
        );
        gl::EnableVertexAttribArray(2); // Texture coordinates attribute

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);

        gl::UseProgram(shader_program);

        // lighting
        let light_pos = gl::GetUniformLocation(shader_program, CString::new("light.position").unwrap().as_ptr());
        let light_ambient = gl::GetUniformLocation(shader_program, CString::new("light.ambient").unwrap().as_ptr());
        let light_diffuse = gl::GetUniformLocation(shader_program, CString::new("light.diffuse").unwrap().as_ptr());
        let light_specular = gl::GetUniformLocation(shader_program, CString::new("light.specular").unwrap().as_ptr());

        // Assuming you have set Vec3 struct to handle data
        gl::Uniform3fv(light_pos, 1, [500.0, 500.0, 500.0].as_ptr()); // Example light position
        gl::Uniform3fv(light_ambient, 1, [0.4, 0.4, 0.4].as_ptr()); // Low intensity ambient light
        gl::Uniform3fv(light_diffuse, 1, [0.75, 0.75, 0.75].as_ptr()); // Medium intensity diffuse light
        gl::Uniform3fv(light_specular, 1, [0.3, 0.3, 0.3].as_ptr()); // Strong specular light

        // Set material properties
        let material_ambient = gl::GetUniformLocation(shader_program, CString::new("material.ambient").unwrap().as_ptr());
        let material_diffuse = gl::GetUniformLocation(shader_program, CString::new("material.diffuse").unwrap().as_ptr());
        let material_specular = gl::GetUniformLocation(shader_program, CString::new("material.specular").unwrap().as_ptr());
        let material_shininess = gl::GetUniformLocation(shader_program, CString::new("material.shininess").unwrap().as_ptr());

        gl::Uniform3fv(material_ambient, 1, [0.4, 0.4, 0.4].as_ptr());
        gl::Uniform3fv(material_diffuse, 1, [0.75, 0.75, 0.75].as_ptr());
        gl::Uniform3fv(material_specular, 1, [0.3, 0.3, 0.3].as_ptr());
        gl::Uniform1f(material_shininess, 32.0); // Shininess factor

        // Add texture
        // this takes rather long
        let texture_id = texture::create_texture("C:/Users/dries/dev/fltk-opengl-test/target/debug/earth.png");
        gl::ActiveTexture(gl::TEXTURE0); // Activate the first texture unit
        gl::BindTexture(gl::TEXTURE_2D, texture_id);
        let texture_location = gl::GetUniformLocation(shader_program, CString::new("textureSampler").unwrap().as_ptr());
        gl::Uniform1i(texture_location, 0);

        gl::UseProgram(0);
    }

    // Clean up shaders (they're linked into the program now, so no longer needed separately)
    unsafe {
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);
    }

    // Other initializations like setting the background color, enabling depth test etc.
    unsafe {
        gl::ClearColor(0.1, 0.1, 0.1, 1.0); // Clear color
        gl::Enable(gl::DEPTH_TEST); // Enable depth test
        gl::Disable(gl::CULL_FACE);
    }

    check_gl_error("During initialization");

    // endregion: -- init shader

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

    wind.draw(move |_| {
        draw_sphere(&shader_program, vao, &vertices, &indices, &camera_coordinates_rc.borrow(), *camera_zoom_rc.borrow(), &camera_rotation_rc.borrow());
    });
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
        last_time = current_time;
    }

    // Cleanup resources after the loop ends
    unsafe {
        gl::DeleteVertexArrays(1, &mut vao);
        gl::DeleteBuffers(1, &mut vbo);
        gl::DeleteProgram(shader_program);
    }
}

fn draw_sphere(shader_program: &gl::types::GLuint, vao: GLuint, vertices: &Vec<f32>, indices: &Vec<u16>, camera_coordinate: &(f32, f32), zoom: f32, camera_rotation: &(f32, f32)) {
    unsafe {
        // Clear the screen and depth buffer
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        // Bind the shader program and VAO
        gl::UseProgram(*shader_program);
        gl::BindVertexArray(vao);

        // Draw the sphere
        gl::DrawElements(
            gl::TRIANGLES,
            indices.len() as i32,
            gl::UNSIGNED_SHORT,
            ptr::null(),
        );

        let camera_x = zoom * camera_coordinate.0.to_radians().cos() * camera_coordinate.1.to_radians().cos();
        let camera_y = zoom * camera_coordinate.1.to_radians().sin();
        let camera_z = zoom * camera_coordinate.0.to_radians().sin() * camera_coordinate.1.to_radians().cos();
        // let rotation_matrix = Matrix4::from_angle_y(Deg(camera_rotation.0)) * Matrix4::from_angle_x(Deg(camera_rotation.1));
        // let translation_matrix = Matrix4::from_translation(vec3(-camera_x, -camera_y, -camera_z));
        // let view_matrix = rotation_matrix * translation_matrix;
        // glLoadMatrixf(view_matrix.as_ptr());

        let projection = cgmath::perspective(Deg(45.0), W as f32 / H as f32, 0.1, 100.0);
        let model = Matrix4::<f32>::identity(); // Model matrix, for example
        let eye = Point3::new(camera_x, camera_y, camera_z); // Camera's position
        let target = Point3::new(0.0, 0.0, 0.0); // Where the camera is looking
        let up = Vector3::new(0.0, 1.0, 0.0); // 'Up' direction in world space
        let view = Matrix4::look_at(eye, target, up);

        let view_loc = gl::GetUniformLocation(*shader_program, CString::new("view").unwrap().as_ptr());
        let proj_loc = gl::GetUniformLocation(*shader_program, CString::new("projection").unwrap().as_ptr());
        let model_loc = gl::GetUniformLocation(*shader_program, CString::new("model").unwrap().as_ptr());
        gl::UniformMatrix4fv(view_loc, 1, gl::FALSE, view.as_ptr());
        gl::UniformMatrix4fv(proj_loc, 1, gl::FALSE, projection.as_ptr());
        gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, model.as_ptr());

        let view_pos_location =
            gl::GetUniformLocation(*shader_program, CString::new("viewPos").unwrap().as_ptr());
        gl::Uniform3fv(
            view_pos_location,
            1,
            [camera_x, camera_y, camera_z].as_ptr(),
        ); // Update with actual camera position

        check_gl_error("During draw call");

        // Unbind the VAO and the shader program
        gl::BindVertexArray(0);
        gl::UseProgram(0);

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
