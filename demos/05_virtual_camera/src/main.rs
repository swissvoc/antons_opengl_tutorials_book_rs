extern crate gl;
extern crate glfw;
extern crate chrono;

mod gl_utils;
mod graphics_math;


use glfw::{Action, Context, Key};
use gl::types::{GLfloat, GLuint, GLsizeiptr, GLchar, GLvoid, GLint, GLenum};

use std::string::String;
use std::mem;
use std::ptr;

use std::process;
use gl_utils::*;

use graphics_math as math;
use math::{Mat4};


const GL_LOG_FILE: &str = "gl.log";

static mut PREVIOUS_SECONDS: f64 = 0.;



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
    let (mut glfw, mut window, mut events) = start_gl().unwrap();
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

        let mut last_position = 0.0;

        // Camera movement parameters.
        let cam_speed = 1.0 * 2e4;       // 1 unit per second.
        let cam_yaw_speed = 10.0 * 2e4;  // 10 degrees per second.
        let mut cam_pos = default_camera_pos();
        let mut cam_yaw = 0.0;

        // Camera translation and rotation.
        let t_mat = Mat4::identity().translate(&math::vec3((-cam_pos[0], -cam_pos[1], -cam_pos[2])));
        let r_mat = Mat4::identity().rotate_y_deg(-cam_yaw);
        let view_mat = &r_mat * &t_mat;

        // Set up project matrix. We will put this into a math function later.
        let near = 0.1;
        let far = 100.0;
        let fov = 67.0; // Convert 67 degrees to radians.
        let aspect = G_GL_WIDTH as f32 / G_GL_HEIGHT as f32;
        let proj_mat = Mat4::perspective(fov, aspect, near, far);

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

            _update_fps_counter(&glfw, &mut window);

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
                let t_mat = Mat4::identity().translate(&math::vec3((-cam_pos[0], -cam_pos[1], -cam_pos[2]))); // cam translation
                let r_mat = Mat4::identity().rotate_y_deg(-cam_yaw);
                let view_mat = &r_mat * &t_mat;
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

