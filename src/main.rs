use fltk::{app, enums, prelude::*, window::GlWindow, image::IcoImage};
use glu_sys::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};
use fltk::app::{event_dy, MouseWheel};
use fltk::enums::{Event, Key};

const W: i32 = 1200;
const H: i32 = 800;

fn main() {
    let app = app::App::default();
    let mut wind = GlWindow::new(100, 100, W, H, "Smooth Camera Control Example");
    let icon: IcoImage = IcoImage::load(&std::path::Path::new("fltk.ico")).unwrap();
    wind.make_resizable(true);
    wind.set_icon(Some(icon));
    wind.end();
    wind.show();

    let camera_zoom = Rc::new(RefCell::new(5.0 as f32)); // Initial zoom distance
    let camera_zoom_rc = camera_zoom.clone();
    let camera_zoom_target = Rc::new(RefCell::new(5.0 as f32));
    let camera_zoom_target_rc = camera_zoom_target.clone();

    let camera_angles = Rc::new(RefCell::new((0.0 as f32, 0.0 as f32)));  // horizontal, vertical angles
    let camera_angles_rc = camera_angles.clone();

    wind.draw(move |_| {
        draw_sphere(&camera_angles_rc.borrow(), *camera_zoom_rc.borrow());
    });

    let mut key_states = Rc::new(RefCell::new(HashMap::new()));
    let key_states_rc = key_states.clone();

    wind.handle(move |_, ev| {
        match ev {
            Event::MouseWheel => {
                let mut zoom_target = camera_zoom_target_rc.borrow_mut();
                println!("{}", zoom_target);
                if event_dy() == MouseWheel::Up {
                    *zoom_target += f32::max(0.25, *zoom_target - 1.0) * 0.2; // Adjust zoom factor based on scroll direction
                }
                else if event_dy() == MouseWheel::Down {
                    *zoom_target -= f32::max(0.25, *zoom_target -1.0) * 0.2; // Adjust zoom factor based on scroll direction
                }
                *zoom_target = zoom_target.max(1.01).min(10.0); // Clamp the zoom level to a reasonable range
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
    const ZOOM_SPEED: f32 = 0.01;
    while app.wait() {
        let current_time = Instant::now();
        let delta_ms = current_time.duration_since(last_time);
        let keys = key_states.borrow();
        let mut angles = camera_angles.borrow_mut();
        let mut zoom = camera_zoom.borrow_mut();

        // Check for specific keys being held down and perform actions
        if *keys.get(&Key::from_char('w')).unwrap_or(&false) {
            angles.1 += delta_ms.as_secs_f32() * CAMERA_SPEED;
        }
        if *keys.get(&Key::from_char('a')).unwrap_or(&false) {
            angles.0 += delta_ms.as_secs_f32() * CAMERA_SPEED;
        }
        if *keys.get(&Key::from_char('r')).unwrap_or(&false) {
            angles.1 -= delta_ms.as_secs_f32() * CAMERA_SPEED;
        }
        if *keys.get(&Key::from_char('s')).unwrap_or(&false) {
            angles.0 -= delta_ms.as_secs_f32() * CAMERA_SPEED;
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

fn draw_sphere(camera_angles: &(f32, f32), zoom: f32) {
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
        gluLookAt(
            (zoom * camera_angles.0.to_radians().cos() * camera_angles.1.to_radians().cos()) as f64,
            (zoom * camera_angles.1.to_radians().sin()) as f64,
            (zoom * camera_angles.0.to_radians().sin() * camera_angles.1.to_radians().cos()) as f64,
            0.0, 0.0, 0.0,  // Look at the center of the sphere
            0.0, 1.0, 0.0,  // Up vector
        );

        let quadric = gluNewQuadric();
        gluQuadricNormals(quadric, GLU_SMOOTH);
        gluQuadricTexture(quadric, GL_TRUE as GLboolean);
        gluSphere(quadric, 1.0, 30, 30);
        gluDeleteQuadric(quadric);

        glDisable(GL_LIGHTING);
    }
}