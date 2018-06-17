extern crate gl;
extern crate glfw;
extern crate chrono;

#[macro_use] 
extern crate scan_fmt;

mod gl_utils;
mod graphics_math;
mod obj_parser;


use glfw::{Action, Context, Key};
use gl::types::{GLfloat, GLsizeiptr, GLvoid, GLsizei, GLuint};

use std::mem;
use std::ptr;

use gl_utils::*;

use graphics_math as math;
use math::Mat4;

const VERTEX_SHADER_FILE: &str = "src/test.vert.glsl";
const FRAGMENT_SHADER_FILE: &str = "src/test.frag.glsl";

static mut PREVIOUS_SECONDS: f64 = 0.0;


fn main() {
    restart_gl_log();
    // start GL context and O/S window using the GLFW helper library
    let (mut glfw, mut g_window, mut _g_events) = start_gl().unwrap();

    // tell GL to only draw onto a pixel if the shape is closer to the viewer
    unsafe {
        gl::Enable(gl::DEPTH_TEST); // enable depth-testing
        gl::DepthFunc(gl::LESS);    // depth-testing interprets a smaller value as "closer"
    }

    /* OTHER STUFF GOES HERE NEXT */
    let points: [GLfloat; 18] = [
        -0.5, -0.5, 0.0,  0.5, -0.5, 0.0,  0.5,  0.5, 0.0, 
         0.5,  0.5, 0.0, -0.5,  0.5, 0.0, -0.5, -0.5, 0.0
    ];

    // 2^16 == 65536
    let texcoords: [GLfloat; 12] = [
        0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0
    ];

    let mut points_vbo: GLuint = 0;
    gl::GenBuffers(1, &mut points_vbo);
    gl::BindBuffer(gl::ARRAY_BUFFER, points_vbo);
    gl::BufferData(
        gl::ARRAY_BUFFER, (points.len() * mem::size_of::<GLfloat>()) as GLsizeiptr, 
        points.as_ptr() as *const GLvoid, gl::STATIC_DRAW
    );

    let mut texcoords_vbo: GLuint = 0;
    gl::GenBuffers(1, &mut texcoords_vbo);
    gl::BindBuffer(gl::ARRAY_BUFFER, texcoords_vbo);
    gl::BufferData(
        gl::ARRAY_BUFFER, (texcoords.len() * mem::size_of::<GLfloat>()) as GLsizeiptr, 
        texcoords.as_ptr() as *const GLvoid, gl::STATIC_DRAW
    );

    let mut vao: GLuint = 0;
    gl::GenVertexArrays(1, &mut vao);
    gl::BindVertexArray(vao);
    gl::BindBuffer(gl::ARRAY_BUFFER, points_vbo);
    gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
    gl::BindBuffer(gl::ARRAY_BUFFER, texcoords_vbo);
    gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 0, ptr::null()); // normalize!
    gl::EnableVertexAttribArray(0);
    gl::EnableVertexAttribArray(1);

    let shader_programme = create_programme_from_files(VERTEX_SHADER_FILE, FRAGMENT_SHADER_FILE);

    // input variables
    let near = 0.1;                                  // clipping plane
    let far = 100.0;                                 // clipping plane
    let fov = 67.0;                                  // convert 67 degrees to radians
    let aspect = G_GL_WIDTH as f32 / G_GL_HEIGHT as f32; // aspect ratio
    let proj_mat = Mat4::perspective(fov, aspect, near, far);

    // matrix components
    let cam_speed = 1.0;             // 1 unit per second
    let cam_yaw_speed = 10.0;        // 10 degrees per second
    let cam_pos: [GLfloat; 3] = [0.0, 0.0, 2.0]; // don't start at zero, or we will be too close
    let cam_yaw = 0.0;               // y-rotation in degrees
    let mat_trans = Mat4::identity().translate(&math::vec3((-cam_pos[0], -cam_pos[1], -cam_pos[2])));
    let mat_rot = Mat4::identity().rotate_y_deg(-cam_yaw);
    let view_mat = mat_rot * mat_trans;

    let view_mat_location = gl::GetUniformLocation(shader_programme, "view".as_ptr() as *const i8);
    gl::UseProgram(shader_programme);
    gl::UniformMatrix4fv(view_mat_location, 1, gl::FALSE, view_mat.as_ptr());
    let proj_mat_location = gl::GetUniformLocation(shader_programme, "proj".as_ptr() as *const i8);
    gl::UseProgram(shader_programme);
    gl::UniformMatrix4fv(proj_mat_location, 1, gl::FALSE, proj_mat.as_ptr());

    // load texture
    GLuint tex;
    ( load_texture( "skulluvmap.png", &tex ) );

    gl::Enable(gl::CULL_FACE); // cull face
    gl::CullFace(gl::BACK);    // cull back face
    gl::FrontFace(gl::CCW);    // GL_CCW for counter clock-wise

    while !g_window.should_close() {
        let current_seconds = glfw.get_time();
        let elapsed_seconds = current_seconds - PREVIOUS_SECONDS;
        PREVIOUS_SECONDS = current_seconds;

        _update_fps_counter(&glfw, &mut g_window);
        // wipe the drawing surface clear
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        gl::Viewport(0, 0, G_GL_HEIGHT, G_GL_HEIGHT);

        gl::UseProgram(shader_programme);
        gl::BindVertexArray(vao);
        // draw points 0-3 from the currently bound VAO with current in-use shader
        gl::DrawArrays(gl::TRIANGLES, 0, 6);
        // update other events like input handling
        glfw.poll_events();

        // control keys
        let mut cam_moved = false;
        match g_window.get_key(Key::A) {
            Action::Press | Action::Repeat => {
                cam_pos[0] -= cam_speed * elapsed_seconds;
                cam_moved = true;
            }
            _ => {}
        }
        match g_window.get_key(Key::D) {
            Action::Press | Action::Repeat => {
                cam_pos[0] += cam_speed * elapsed_seconds;
                cam_moved = true;
            }
            _ => {}
        }
        match g_window.get_key(Key::Up) {
            Action::Press | Action::Repeat => {
                cam_pos[1] += cam_speed * elapsed_seconds;
                cam_moved = true;
            }
            _ => {}
        }
        match g_window.get_key(Key::Down) {
            Action::Press | Action::Repeat => {
                cam_pos[1] -= cam_speed * elapsed_seconds;
                cam_moved = true;
            }
            _ => {}
        }
        match g_window.get_key(Key::W) {
            Action::Press | Action::Repeat => {
                cam_pos[2] -= cam_speed * elapsed_seconds;
                cam_moved = true;
            }
            _ => {}
        }
        match g_window.get_key(Key::S) {
            Action::Press | Action::Repeat => {
                cam_pos[2] += cam_speed * elapsed_seconds;
                cam_moved = true;
            }
            _ => {}
        }
        match g_window.get_key(Key::Left) {
            Action::Press | Action::Repeat => {
                cam_yaw += cam_yaw_speed * elapsed_seconds;
                cam_moved = true;
            }
            _ => {}
        }
        match g_window.get_key(Key::Right) {
            Action::Press | Action::Repeat => {
                cam_yaw -= cam_yaw_speed * elapsed_seconds;
                cam_moved = true;
            }
            _ => {}
        }
        // update view matrix
        if cam_moved {
            mat_trans = Mat4::identity().translate(&math::vec3((-cam_pos[0], -cam_pos[1], -cam_pos[2])); // cam translation
            mat_rot = Mat4::identity().rotate_y_deg(-cam_yaw);                 //
            view_mat = mat_rot * mat_trans;
            gl::UniformMatrix4fv(view_mat_location, 1, gl::FALSE, view_mat.as_ptr());
        }

        match g_window.get_key(Key::Escape) {
            Action::Press | Action::Repeat => {
                g_window.set_should_close(true);
            }
            _ => {}
        }
        // Put the stuff we've been drawing onto the display.
        g_window.swap_buffers();
    }
}
