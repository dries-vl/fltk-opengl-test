use fltk::{app, enums, prelude::*, window::GlWindow, image::IcoImage};
use glu_sys::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::ffi::CString;
use fltk::enums::Key;

const W: i32 = 600;
const H: i32 = 400;

fn main() {
    let app = app::App::default();
    let mut wind = GlWindow::new(100, 100, W, H, "3D Sphere Example with Keyboard-Controlled Camera");
    let icon: IcoImage = IcoImage::load(&std::path::Path::new("fltk.ico")).unwrap();
    wind.make_resizable(true);
    wind.set_icon(Some(icon));
    wind.end();
    wind.show();

    let camera_angles = Rc::new(RefCell::new((0.0, 0.0)));  // horizontal, vertical angles
    let camera_angles_rc = camera_angles.clone();

    wind.draw(move |_| {
        draw_sphere(&camera_angles_rc.borrow());
    });

    wind.handle(move |_, ev| match ev {
        enums::Event::KeyDown => {
            let mut angles = camera_angles.borrow_mut();
            const W:Key = enums::Key::from_char('w');
            const A:Key = enums::Key::from_char('a');
            const R:Key = enums::Key::from_char('r');
            const S:Key = enums::Key::from_char('s');
            match app::event_key() {
                A => angles.0 += 5.0,
                S => angles.0 -= 5.0,
                R => {
                    angles.1 -= 5.0;
                    angles.1 = angles.1.max(-89.0); // Limit vertical angle to avoid flipping
                },
                W => {
                    angles.1 += 5.0;
                    angles.1 = angles.1.min(89.0); // Limit vertical angle to avoid flipping
                },
                _ => {}
            }
            true
        },
        _ => false,
    });

    while app.wait() {
        wind.redraw();
    }
}

fn draw_sphere(camera_angles: &(f32, f32)) {
    unsafe {
        glEnable(GL_DEPTH_TEST);
        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
        glEnable(GL_LIGHTING);
        glEnable(GL_LIGHT0);

        let light_position = [1.0, 1.0, 1.0, 0.0];
        glLightfv(GL_LIGHT0, GL_POSITION, light_position.as_ptr());

        glMatrixMode(GL_PROJECTION);
        glLoadIdentity();
        gluPerspective(45.0, (W as f32 / H as f32) as GLdouble, 1.0, 10.0);

        glMatrixMode(GL_MODELVIEW);
        glLoadIdentity();
        gluLookAt(
            (5.0 * camera_angles.0.to_radians().cos() * camera_angles.1.to_radians().cos()) as GLdouble,
            (5.0 * camera_angles.1.to_radians().sin()) as GLdouble,
            (5.0 * camera_angles.0.to_radians().sin() * camera_angles.1.to_radians().cos()) as GLdouble,
            0.0, 0.0, 0.0,  // Look at the center of the sphere
            0.0, 1.0, 0.0,  // Up vector
        );

        let quadric = gluNewQuadric();
        gluQuadricNormals(quadric, GLU_SMOOTH);
        gluQuadricTexture(quadric, GL_TRUE as GLboolean);
        gluSphere(quadric, 1.0, 20, 20);
        gluDeleteQuadric(quadric);

        glDisable(GL_LIGHTING);
    }
}
