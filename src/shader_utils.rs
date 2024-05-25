extern crate gl;
use gl::types::*;
use std::ffi::CString;
use std::ptr;
use std::str;

// Function to compile a shader
pub(crate) fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    let shader = unsafe { gl::CreateShader(ty) };
    unsafe {
        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // Check for compilation error
        let mut status = GLint::from(gl::FALSE);
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
        if status != GLint::from(gl::TRUE) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            let log = str::from_utf8(&buf).ok().expect("ShaderInfoLog not valid utf8");
            panic!("Failed to compile shader: {}", log);
        }
    }
    shader
}

// Function to link shaders into a program
pub(crate) fn link_program(vertex_shader_id: GLuint, fragment_shader_id: GLuint) -> GLuint {
    let program = unsafe { gl::CreateProgram() };
    unsafe {
        gl::AttachShader(program, vertex_shader_id);
        gl::AttachShader(program, fragment_shader_id);
        gl::LinkProgram(program);

        // Check for linking error
        let mut status = GLint::from(gl::FALSE);
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
        if status != GLint::from(gl::TRUE) {
            let mut len: GLint = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            let log = std::str::from_utf8(&buf).expect("ShaderInfoLog not valid utf8");
            println!("Shader compilation error: {}", log);
            panic!();
        }
    }
    program
}

pub(crate) fn link_particles_program(vertex_shader_id: GLuint, fragment_shader_id: GLuint) -> GLuint {
    let program = unsafe { gl::CreateProgram() };
    unsafe {
        gl::AttachShader(program, vertex_shader_id);
        gl::AttachShader(program, fragment_shader_id);

        // Specify the output of the vertex shader we want to capture in transform feedback
        let varyings = ["outPosition", "outVelocity"];
        let c_varyings: Vec<CString> = varyings.iter().map(|&v| CString::new(v).unwrap()).collect();
        let c_varyings_ptrs: Vec<*const GLchar> = c_varyings.iter().map(|c_str| c_str.as_ptr()).collect();

        gl::TransformFeedbackVaryings(program, 2, c_varyings_ptrs.as_ptr(), gl::INTERLEAVED_ATTRIBS);
        gl::LinkProgram(program);

        // Check for linking error
        let mut status = GLint::from(gl::FALSE);
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
        if status != GLint::from(gl::TRUE) {
            let mut len: GLint = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            let log = std::str::from_utf8(&buf).expect("ShaderInfoLog not valid utf8");
            println!("Shader compilation error: {}", log);
            panic!();
        }
    }
    program
}
