extern crate gl;
extern crate glfw;
extern crate chrono;

mod gl_utils;
mod gl_math;


use glfw::{Action, Context, Key};
use gl::types::{GLfloat, GLuint, GLsizeiptr, GLchar, GLvoid, GLint, GLenum};

use std::string::String;
use std::mem;
use std::ptr;
use std::fs::{File};
use std::io::{Read};
use std::process;
use gl_utils::*;
use gl_math::*;


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

fn parse_file_into_str(file_name: &str, shader_str: &mut [u8], max_len: usize) -> bool {
    let file = File::open(file_name);
    if file.is_err() {
        gl_log_err(&format!("ERROR: opening file for reading: {}\n", file_name));
        return false;
    }

    let mut file = file.unwrap();

    let bytes_read = file.read(shader_str);
    if bytes_read.is_err() {
        gl_log_err(&format!("ERROR: reading shader file {}\n", file_name));
        return false;
    }

    let bytes_read = bytes_read.unwrap();
    if bytes_read >= (max_len - 1) {
        gl_log_err(&format!("WARNING: file {} too big - truncated.\n", file_name));
    }

    // append \0 to end of file string.
    shader_str[bytes_read] = 0;

    return true;
}

fn default_camera_pos() -> [f32; 3] {
    [0.0, 0.0, 2.0]
}

fn main() {
    let points: [GLfloat; 9] = [
        0.0,  0.5, 0.0, 0.5, -0.5, 0.0, -0.5, -0.5, 0.0
    ];

    let colours: [GLfloat; 9] = [
        1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0
    ];

    restart_gl_log();
    let (mut glfw, mut window, events) = start_gl().unwrap();
    unsafe {
        let mut points_vbo: GLuint = 0;
        gl::GenBuffers(1, &mut points_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, points_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, (mem::size_of::<GLfloat>() * points.len()) as GLsizeiptr, 
            points.as_ptr() as *const GLvoid, gl::STATIC_DRAW
        );

        let mut colours_vbo: GLuint = 0;
        gl::GenBuffers(1, &mut colours_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, colours_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, (mem::size_of::<GLfloat>() * colours.len()) as GLsizeiptr, 
            colours.as_ptr() as *const GLvoid, gl::STATIC_DRAW
        );

        let mut vao: GLuint = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, points_vbo);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::BindBuffer(gl::ARRAY_BUFFER, colours_vbo);
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::EnableVertexAttribArray(0);
        gl::EnableVertexAttribArray(1);

        /* load shaders from files here */
        let mut vertex_shader = vec![0; 1024 * 256];
        parse_file_into_str("src/test_vs.glsl", &mut vertex_shader, 1024 * 256);

        let mut fragment_shader = vec![0; 1024 * 256];
        parse_file_into_str("src/test_fs.glsl", &mut fragment_shader, 1024 * 256);

        let vs: GLuint = gl::CreateShader(gl::VERTEX_SHADER);
        let p = vertex_shader.as_ptr() as *const GLchar;
        gl::ShaderSource(vs, 1, &p, ptr::null());
        gl::CompileShader(vs);

        let mut params = -1;
        gl::GetShaderiv(vs, gl::COMPILE_STATUS, &mut params);
        if params != gl::TRUE as i32 {
            eprintln!("ERROR: GL shader index {} did not compile", vs);
            _print_shader_info_log(vs);
            process::exit(1);
        }

        let fs: GLuint = gl::CreateShader(gl::FRAGMENT_SHADER);
        let p = fragment_shader.as_ptr() as *const GLchar;
        gl::ShaderSource(fs, 1, &p, ptr::null());
        gl::CompileShader(fs);

        /* check for compile errors */
        params = -1;
        gl::GetShaderiv(fs, gl::COMPILE_STATUS, &mut params);
        if params != gl::TRUE as i32 {
            eprintln!("ERROR: GL shader index {} did not compile", fs);
            _print_shader_info_log(fs);
            process::exit(1);
        }

        let shader_programme: GLuint = gl::CreateProgram();
        gl::AttachShader(shader_programme, vs);
        gl::AttachShader(shader_programme, fs);
        gl::LinkProgram(shader_programme);

        /* check for shader linking errors - very important! */
        params = -1;
        gl::GetProgramiv(shader_programme, gl::LINK_STATUS, &mut params);
        if params != gl::TRUE as i32 {
            eprintln!("ERROR: could not link shader programme GL index {}", shader_programme);
            _print_programme_info_log(shader_programme);
            process::exit(1);
        }
        print_all(shader_programme);
        let result = is_valid(shader_programme);
        assert!(result);

        let mut speed = 1.0 * 1e4;
        let mut last_position = 0.0;
        // Camera parameters.
        let mut cam_speed = 1.0 * 2e4;       // 1 unit per second.
        let mut cam_yaw_speed = 10.0 * 2e4;  // 10 degrees per second.
        let mut cam_pos = default_camera_pos();
        let mut cam_yaw = 0.0;
        // Camera translation and rotation.
        let T = Mat4::identity().translate(&vec3((-cam_pos[0], -cam_pos[1], -cam_pos[2])));
        let R = Mat4::identity().rotate_y_deg(-cam_yaw);
        let view_mat = &R * &T;

        // Set up project matrix. We will put this into a math function later.
        let near = 0.1;
        let far = 100.0;
        let fov = 67.0 * ONE_DEG_IN_RAD; // Convert 67 degrees to radians.
        let aspect = G_GL_WIDTH as f32 / G_GL_HEIGHT as f32;
        let range = f32::tan(fov * 0.5) * near;
        let Sx = (2.0 * near) / (range * aspect + range * aspect);
        let Sy = near / range;
        let Sz = -(far + near) / (far - near);
        let Pz = -(2.0 * far * near) / (far - near);

        let proj_mat = mat4(
            Sx, 0.0, 0.0,  0.0, 
            0.0, Sy, 0.0,  0.0, 
            0.0, 0.0, Sz, -1.0,
            0.0, 0.0, Pz,  0.0
        );

        let view_mat_location = gl::GetUniformLocation(shader_programme, "view".as_ptr() as *const i8);
        assert!(view_mat_location != -1);
        let proj_mat_location = gl::GetUniformLocation(shader_programme, "proj".as_ptr() as *const i8);
        assert!(proj_mat_location != -1);
        gl::UseProgram(shader_programme);
        gl::UniformMatrix4fv(view_mat_location, 1, gl::FALSE, view_mat.as_ptr());
        gl::UniformMatrix4fv(proj_mat_location, 1, gl::FALSE, proj_mat.as_ptr());

        // Tell GL to only draw onto a pixel if the shape is closer to the viewer.
        // Enable depth-testing.
        gl::Enable(gl::DEPTH_TEST);
        // Depth-testing interprets a smaller value as "closer".
        gl::DepthFunc(gl::LESS);

        // Cull face.
        gl::Enable(gl::CULL_FACE);
        // Cull back face.
        gl::CullFace(gl::BACK);
         // GL_CCW for counter clock-wise.
        gl::FrontFace(gl::CW);

        while !window.should_close() {
            // Add timer for doing animation.
            PREVIOUS_SECONDS = glfw.get_time();
            let current_seconds = glfw.get_time();
            let elapsed_seconds = (current_seconds - PREVIOUS_SECONDS) as f32;
            PREVIOUS_SECONDS = current_seconds;

            // Wipe the drawing surface clear.
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Viewport(0, 0, G_GL_WIDTH as GLint, G_GL_HEIGHT as GLint);

            gl::UseProgram(shader_programme);
            gl::BindVertexArray(vao);
            // Draw points 0-3 from the currently bound VAO with current in-use shader.
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            // Update other events like input handling.
            glfw.poll_events();
            
            /*-----------------------------move camera
         
            * here-------------------------------*/
            // Control keys
            let mut cam_moved = false;

            match window.get_key(Key::A) {
                Action::Press | Action::Repeat => {
                    cam_pos[0] -= cam_speed * elapsed_seconds;
                    cam_moved = true;
                }
                _ => {}
            }
            match window.get_key(Key::D) {
                Action::Press | Action::Repeat => {
                    cam_pos[0] += cam_speed * elapsed_seconds;
                    cam_moved = true;
                }
                _ => {}
            }
            match window.get_key(Key::PageUp) {
                Action::Press | Action::Repeat => {
                    cam_pos[1] += cam_speed * elapsed_seconds;
                    cam_moved = true;
                }
                _ => {}
            }
            match window.get_key(Key::PageDown) {
                Action::Press | Action::Repeat => {
                    cam_pos[1] -= cam_speed * elapsed_seconds;
                    cam_moved = true;
                }
                _ => {}
            }
            match window.get_key(Key::W) {
                Action::Press | Action::Repeat => {
                    cam_pos[2] -= cam_speed * elapsed_seconds;
                    cam_moved = true;
                }
                _ => {}
            }
            match window.get_key(Key::S) {
                Action::Press | Action::Repeat => {
                    cam_pos[2] += cam_speed * elapsed_seconds;
                    cam_moved = true;
                }
                _ => {}                
            }
            match window.get_key(Key::Left) {
                Action::Press | Action::Repeat => {
                    cam_yaw += cam_yaw_speed * elapsed_seconds;
                    cam_moved = true;
                }
                _ => {}
            }
            match window.get_key(Key::Right) {
                Action::Press | Action::Repeat => {
                    cam_yaw -= cam_yaw_speed * elapsed_seconds;
                    cam_moved = true;
                }
                _ => {}
            }
            match window.get_key(Key::R) {
                Action::Press | Action::Repeat => {
                    cam_pos = default_camera_pos();
                    cam_moved = true;
                }
                _ => {}
            }

            /* update view matrix */
            if cam_moved {
                let T = Mat4::identity().translate(&vec3((-cam_pos[0], -cam_pos[1], -cam_pos[2]))); // cam translation
                let R = Mat4::identity().rotate_y_deg(-cam_yaw);
                let view_mat = &R * &T;
                gl::UniformMatrix4fv(view_mat_location, 1, gl::FALSE, view_mat.as_ptr());
            }

            match window.get_key(Key::Escape) {
                Action::Press | Action::Repeat => {
                    window.set_should_close(true);
                }
                _ => {}
            }
            
            // Put the stuff we've been drawing onto the display.
            window.swap_buffers();
        }
    }
}

