use image::io::Reader as ImageReader;
use image::DynamicImage;

fn load_image(file_path: &str) -> DynamicImage {
    ImageReader::open(file_path)
        .expect("Failed to open texture file")
        .decode()
        .expect("Failed to decode image")
}

pub fn create_texture(file_path: &str) -> u32 {

    // 1000ms for large texture
    let image = load_image(file_path);
    // 1000ms for large texture
    let img = image.fliph().into_rgba8();
    let (width, height) = img.dimensions();

    let mut texture_id = 0;
    unsafe {
        gl::GenTextures(1, &mut texture_id);
        gl::BindTexture(gl::TEXTURE_2D, texture_id);

        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            width as i32,
            height as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            img.as_ptr() as *const _,
        );

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

        gl::GenerateMipmap(gl::TEXTURE_2D);
    }

    texture_id
}
