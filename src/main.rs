use fltk::{app, prelude::*, window::GlWindow, image::IcoImage};
use glu_sys::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Instant};
use fltk::app::{event_button, event_dy, event_x, event_y, MouseButton, MouseWheel};
use fltk::enums::{Event, Key};

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

    let camera_zoom = Rc::new(RefCell::new(5.0 as f32)); // Initial zoom distance
    let camera_zoom_rc = camera_zoom.clone();
    let camera_zoom_target = Rc::new(RefCell::new(5.0 as f32));
    let camera_zoom_target_rc = camera_zoom_target.clone();

    let camera_coordinates = Rc::new(RefCell::new((0.0 as f32, 0.0 as f32)));  // horizontal, vertical angles
    let camera_coordinates_rc = camera_coordinates.clone();
    let camera_coordinates_rc_2 = camera_coordinates.clone();

    let mouse_position = Rc::new(RefCell::new((0, 0)));
    let mouse_position_rc = mouse_position.clone();

    let camera_rotation = Rc::new(RefCell::new((0.0 as f32, 0.0 as f32))); // New: for adjusting view direction
    let camera_rotation_rc = camera_rotation.clone();
    let camera_rotation_rc_2 = camera_rotation.clone();

    const ZOOM_SPEED: f32 = 0.2;
    const DRAG_SPEED: f32 = 0.1;

    wind.draw(move |_| {
        draw_sphere(&camera_coordinates_rc.borrow(), *camera_zoom_rc.borrow(), &camera_rotation_rc.borrow());
    });
    let key_states = Rc::new(RefCell::new(HashMap::new()));

    let key_states_rc = key_states.clone();

    wind.handle(move |_, ev| {
        match ev {
            Event::MouseWheel => {
                let mut zoom_target = camera_zoom_target_rc.borrow_mut();
                println!("{}", zoom_target);
                if event_dy() == MouseWheel::Up {
                    *zoom_target += f32::max(0.25, *zoom_target - 1.0) * ZOOM_SPEED; // Adjust zoom factor based on scroll direction
                }
                else if event_dy() == MouseWheel::Down {
                    *zoom_target -= f32::max(0.25, *zoom_target -1.0) * ZOOM_SPEED; // Adjust zoom factor based on scroll direction
                }
                *zoom_target = zoom_target.max(1.01).min(10.0); // Clamp the zoom level to a reasonable range
                true
            },
            Event::Push if event_button() == MouseButton::Left as i32 => {
                *mouse_position_rc.borrow_mut() = (event_x(), event_y());
                true
            },
            Event::Drag if event_button() == MouseButton::Left as i32 => {
                let (prev_x, prev_y) = *mouse_position_rc.borrow();
                let (new_x, new_y) = (event_x(), event_y());
                *mouse_position_rc.borrow_mut() = (new_x, new_y);

                let mut angles = camera_coordinates_rc_2.borrow_mut();
                angles.0 += (new_x - prev_x) as f32 * DRAG_SPEED; // Adjust sensitivity as needed
                angles.1 += (new_y - prev_y) as f32 * DRAG_SPEED; // Adjust sensitivity as needed
                true
            },
            Event::Push if event_button() == MouseButton::Middle as i32 => {
                *mouse_position_rc.borrow_mut() = (event_x(), event_y());
                true
            },
            Event::Drag if event_button() == MouseButton::Middle as i32 => {
                let (prev_x, prev_y) = *mouse_position_rc.borrow();
                let (new_x, new_y) = (event_x(), event_y());
                *mouse_position_rc.borrow_mut() = (new_x, new_y);
                
                camera_rotation_rc_2.borrow_mut().0 += (new_x - prev_x) as f32 * 0.25;
                camera_rotation_rc_2.borrow_mut().1 += (new_y - prev_y) as f32 * 0.25;
                true
            },
            Event::KeyDown => {
                let mut keys = key_states_rc.borrow_mut();
                keys.insert(app::event_key(), true);
                println!("{:?}", app::event_key());
                if app::event_key() == Key::Escape { app::quit() }
                true
            },
            Event::KeyUp => {
                let mut keys = key_states_rc.borrow_mut();
                keys.insert(app::event_key(), false);
                println!("{:?}", app::event_key());
                true
            },
            _ => false,
        }
    });
    let mut last_time = Instant::now();
    const CAMERA_SPEED: f32 = 50.0;
    while app.wait() {
        let current_time = Instant::now();
        let delta_ms = current_time.duration_since(last_time);
        let keys = key_states.borrow();
        let mut coordinates = camera_coordinates.borrow_mut();
        let mut zoom = camera_zoom.borrow_mut();

        // Check for specific keys being held down and perform actions
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
            *zoom += f32::max(zoom_diff * 0.05,f32::min(zoom_diff, 0.0001));
        }
        else if zoom_diff < 0.0 {
            *zoom += f32::min(zoom_diff * 0.05,f32::max(zoom_diff, -0.0001)); 
        }
        
        wind.redraw();
        last_time = current_time;
    }
}

fn draw_sphere(camera_coordinate: &(f32, f32), zoom: f32, camera_rotation: &(f32, f32)) {
    unsafe {
        glEnable(GL_DEPTH_TEST);
        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
        glEnable(GL_LIGHTING);
        glEnable(GL_LIGHT0);

        let light_position = [1.0, 1.0, 1.0, 0.0];
        glLightfv(GL_LIGHT0, GL_POSITION, light_position.as_ptr());

        glMatrixMode(GL_PROJECTION);
        glLoadIdentity();
        gluPerspective(45.0, W as f64 / H as f64, 0.01, 10.0);

        glMatrixMode(GL_MODELVIEW);
        glLoadIdentity();
        
        // calculate camera position based on angle
        let z = zoom * camera_coordinate.0.to_radians().cos() * camera_coordinate.1.to_radians().cos();
        let y = zoom * camera_coordinate.1.to_radians().sin();
        let x = zoom * -camera_coordinate.0.to_radians().sin() * camera_coordinate.1.to_radians().cos();

        // Vector from A to B
        let ab = (-x, -y, -z);

        // Convert degrees to radians
        let x_radians = camera_rotation.1.to_radians();
        let y_radians = camera_rotation.0.to_radians();


        // Rotation around the x-axis affects y and z
        let after_x_rotation = (
            ab.0,
            ab.1 * x_radians.cos() - ab.2 * x_radians.sin(),
            ab.1 * x_radians.sin() + ab.2 * x_radians.cos(),
        );

        // Rotation around the y-axis affects x and z
        let after_y_rotation = (
            after_x_rotation.0 * y_radians.cos() + after_x_rotation.2 * y_radians.sin(),
            after_x_rotation.1,  // y remains unchanged in y-rotation
            -after_x_rotation.0 * y_radians.sin() + after_x_rotation.2 * y_radians.cos(),
        );

        // Translating back to point A
        let c = (
            x + after_y_rotation.0,
            y + after_y_rotation.1,
            z + after_y_rotation.2,
        );
        
        println!("position: {}, {}, {}", x, y, z);
        println!("looking at: {}, {}, {}", c.0, c.1, c.2);

        gluLookAt(
            x as f64, y as f64, z as f64,
            c.0 as f64, c.1 as f64, c.2 as f64,
            0.0, 1.0, 0.0
        );

        let quadric = gluNewQuadric();
        gluQuadricNormals(quadric, GLU_SMOOTH);
        gluQuadricTexture(quadric, GL_TRUE as GLboolean);
        gluSphere(quadric, 1.0, 30, 30);
        gluDeleteQuadric(quadric);

        glDisable(GL_LIGHTING);
    }
}
