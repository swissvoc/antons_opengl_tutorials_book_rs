extern crate gl;
extern crate glfw;
extern crate chrono;

mod gl_utils;


use glfw::{Action, Context, Key};
use gl::types::{GLubyte, GLfloat, GLuint, GLsizeiptr, GLchar, GLvoid, GLint, GLenum};
use chrono::prelude::Utc;

use std::string::String;
use std::ffi::CStr;
use std::mem;
use std::ptr;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Read, Write};
use std::fmt::Write as FWrite;
use std::cell::Cell;


const GL_LOG_FILE: &str = "gl.log";


fn GL_type_to_string(gl_type: GLenum) -> &'static str {
    match gl_type {
        gl::BOOL => "bool",
        gl::INT => "int",
        gl::FLOAT => "float",
        gl::FLOAT_VEC2 => "vec2",
        gl::FLOAT_VEC3 => "vec3",
        gl::FLOAT_VEC4 => "vec4",
        gl::FLOAT_MAT2 => "mat2",
        gl::FLOAT_MAT3 => "mat3",
        gl::FLOAT_MAT4 => "mat4",
        gl::SAMPLER_2D => "sampler2D",
        gl::SAMPLER_3D => "sampler3D",
        gl::SAMPLER_CUBE => "samplerCube",
        gl::SAMPLER_2D_SHADOW => "sampler2DShadow",
        _ => "other"
    }
}

/* print errors in shader compilation */
fn _print_shader_info_log(shader_index: GLuint) {
    let max_length = 2048;
    let mut actual_length = 0;
    let mut log = [0; 2048];
    
    unsafe {
        gl::GetShaderInfoLog(shader_index, max_length, &mut actual_length, &mut log[0]);
    }
    
    println!("Shader info log for GL index {}:", shader_index);
    for i in 0..actual_length as usize {
        print!("{}", log[i] as u8 as char);
    }
    println!();
}

/* print errors in shader linking */
fn _print_programme_info_log(sp: GLuint) {
    let max_length = 2048;
    let mut actual_length = 0;
    let mut log = [0 as i8; 2048];
    
    unsafe {
        gl::GetProgramInfoLog(sp, max_length, &mut actual_length, &mut log[0]);
    }
    
    println!("Program info log for GL index {}:", sp);
    for i in 0..actual_length as usize {
        print!("{}", log[i] as u8 as char);
    }
    println!();
}

/* validate shader */
fn is_valid(sp: GLuint) -> bool {
    let mut params = -1;
    unsafe {
        gl::ValidateProgram(sp);
        gl::GetProgramiv(sp, gl::VALIDATE_STATUS, &mut params);
    }

    println!("Program {} GL_VALIDATE_STATUS = {}\n", sp, params);
    if gl::TRUE as i32 != params {
        _print_programme_info_log(sp);
        return false;
    }
    return true;
}

/* print absolutely everything about a shader - only useful if you get really
stuck wondering why a shader isn't working properly */
fn print_all(sp: GLuint) {
    let mut params = -1;

    unsafe {
        println!("--------------------\nshader programme {} info:", sp);
        gl::GetProgramiv(sp, gl::LINK_STATUS, &mut params);
        println!("GL_LINK_STATUS = {}", params);

        gl::GetProgramiv(sp, gl::ATTACHED_SHADERS, &mut params);
        println!("GL_ATTACHED_SHADERS = {}", params);

        gl::GetProgramiv(sp, gl::ACTIVE_ATTRIBUTES, &mut params);
        println!("GL_ACTIVE_ATTRIBUTES = {}", params);
    }

    for i in 0..params {
        let mut name = [0; 64];
        let max_length = 64;
        let mut actual_length = 0;
        let mut size = 0;
        let mut gl_type: GLenum = 0;
        unsafe {
            gl::GetActiveAttrib(sp, i as GLuint, max_length, &mut actual_length, &mut size, &mut gl_type, &mut name[0]);
        }
        if size > 1 {
            for j in 0..size {
                let mut long_name = vec![];
                //write!(long_name, "{}[{}]", name, j);
                let location = unsafe { gl::GetAttribLocation(sp, long_name.as_ptr() as *const i8) };
                println!(
                    "  {}) type:{} name:{} location:{}", 
                    i, GL_type_to_string(gl_type), long_name.iter().map(|ch| *ch as u8 as char).collect::<String>(), location
                );
            }
        } else {
            let location = unsafe { gl::GetAttribLocation(sp, &mut name[0]) };
            println!(
                "  {}) type:{} name:{} location:{}",
                i, GL_type_to_string(gl_type), name.iter().map(|ch| *ch as u8 as char).collect::<String>(), location
            );
        }
    }
    
    unsafe {
        gl::GetProgramiv(sp, gl::ACTIVE_UNIFORMS, &mut params);
    }
    println!("GL_ACTIVE_UNIFORMS = {}", params);
    for i in 0..params {
        let mut name = [0; 64];
        let max_length = 64;
        let mut actual_length = 0;
        let mut size = 0;
        let mut gl_type: GLenum = 0;
        unsafe {
            gl::GetActiveUniform(sp, i as u32, max_length, &mut actual_length, &mut size, &mut gl_type, &mut name[0]);
        }
        if size > 1 {
            for j in 0..size {
                let long_name = [0; 64];

                //write!(long_name, "{}[{}]", name, j);
                let location = unsafe { gl::GetUniformLocation(sp, long_name.as_ptr()) };
                println!(
                    "  {}) type:{} name:{} location:{}",
                    i, GL_type_to_string(gl_type), long_name.iter().map(|ch| *ch as u8 as char).collect::<String>(), location
                );
            }
        } else {
            let location = unsafe { gl::GetUniformLocation(sp, &name[0]) };
            println!(
                "  {}) type:{} name:{} location:{}", 
                i, GL_type_to_string(gl_type), name.iter().map(|ch| *ch as u8 as char).collect::<String>(), location
            );
        }
    }

    _print_programme_info_log(sp);
}

fn parse_file_into_str(file_name: &str, shader_str: &mut Vec<u8>, max_len: usize) -> bool {
    let file = File::open(file_name);
    if file.is_err() {
        gl_utils::gl_log_err(&format!("ERROR: opening file for reading: {}\n", file_name));
        return false;
    }

    let mut file = file.unwrap();

    let bytes_read = file.read_to_end(shader_str);
    if bytes_read.is_err() {
        gl_utils::gl_log_err(&format!("ERROR: reading shader file {}\n", file_name));
        return false;
    }

    if bytes_read.unwrap() >= (max_len - 1) {
        gl_utils::gl_log_err(&format!("WARNING: file {} too big - truncated.\n", file_name));
    }

    // append \0 to end of file string.
    shader_str.push(0);
    return true;
}

fn main() {
    let points: [GLfloat; 9] = [
        0.0,  0.5, 0.0, 0.5, -0.5, 0.0, -0.5, -0.5, 0.0
    ];

    let (mut glfw, mut window, events) = gl_utils::start_gl().unwrap();
    unsafe {
        // Tell GL to only draw onto a pixel if the shape is closer to the viewer.
        // Enable depth-testing.
        gl::Enable(gl::DEPTH_TEST);
        // Depth-testing interprets a smaller value as "closer".
        gl::DepthFunc(gl::LESS);

        let mut vbo: GLuint = 0;
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, (mem::size_of::<GLfloat>() * points.len()) as GLsizeiptr, 
            points.as_ptr() as *const GLvoid, gl::STATIC_DRAW
        );

        let mut vao: GLuint = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
        gl::EnableVertexAttribArray(0);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());

        let vertex_shader: &str = "
            #version 460

            in vec3 vp;

            void main () {
                gl_Position = vec4 (vp, 1.0);
            }
        ";

        let fragment_shader: &str = "
            #version 460

            out vec4 frag_colour;

            void main() {
                frag_colour = vec4 (0.5, 0.0, 0.5, 1.0);
            }
        ";

        let vs: GLuint = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(vs, 1, &(vertex_shader.as_ptr() as *const GLchar), ptr::null());
        gl::CompileShader(vs);

        let fs: GLuint = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(fs, 1, &(fragment_shader.as_ptr() as *const GLchar), ptr::null());
        gl::CompileShader(fs);

        let shader_programme: GLuint = gl::CreateProgram();
        gl::AttachShader(shader_programme, vs);
        gl::AttachShader(shader_programme, fs);
        gl::LinkProgram(shader_programme);

        let mut programme_info_log_len = 0;
        let mut programme_info_log = vec![0; 1024];
        gl::GetProgramInfoLog(
            shader_programme, 
            programme_info_log.capacity() as i32,
            &mut programme_info_log_len,
            programme_info_log.as_mut_ptr()
        );
        println!("SHADER PROGRAM LOG:");
        for i in 0..programme_info_log_len as usize {
            print!("{}", programme_info_log[i] as u8 as char);
        }
        println!("END SHADER PROGRAM LOG.");

        gl_utils::PREVIOUS_SECONDS = glfw.get_time();
        while !window.should_close() {
            gl_utils::_update_fps_counter(&mut glfw, &mut window);
            // Wipe the drawing surface clear.
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Viewport(0, 0, gl_utils::G_GL_WIDTH as GLint, gl_utils::G_GL_HEIGHT as GLint);

            gl::UseProgram(shader_programme);
            gl::BindVertexArray(vao);
            // Draw points 0-3 from the currently bound VAO with current in-use shader.
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            // Update other events like input handling.
            glfw.poll_events();
            for (time, event) in glfw::flush_messages(&events) {
                match event {
                    glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                        window.set_should_close(true);
                    }
                    _ => {

                    }
                }
            }

            // Put the stuff we've been drawing onto the display.
            window.swap_buffers();
        }
    }
}

