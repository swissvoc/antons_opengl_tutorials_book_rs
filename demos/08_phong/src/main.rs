extern crate gl;
extern crate glfw;
extern crate chrono;

#[macro_use] 
extern crate scan_fmt;

mod gl_utils;
mod graphics_math;
mod obj_parser;


use glfw::{Action, Context, Key, MouseButton};
use gl::types::{GLfloat, GLsizeiptr, GLvoid};

use std::mem;
use std::ptr;

use gl_utils::*;

use graphics_math as math;
use math::Mat4;

const MESH_FILE: &str = "src/sphere.obj";
const VERTEX_SHADER_FILE: &str = "src/test.vert.glsl";
const FRAGMENT_SHADER_FILE: &str = "src/test.frag.glsl";

static mut PREVIOUS_SECONDS: f64 = 0.0;


fn main() {
    restart_gl_log();
    // start GL context and O/S window using the GLFW helper library
    let (mut glfw, mut g_window, _g_events) = start_gl().unwrap();
    // tell GL to only draw onto a pixel if the shape is closer to the viewer
    unsafe {
        gl::Enable(gl::DEPTH_TEST); // enable depth-testing
        gl::DepthFunc(gl::LESS);      // depth-testing interprets a smaller value as "closer"
    }

    /* OTHER STUFF GOES HERE NEXT */
    let points = [
        0.0, 0.5, 0.0, 0.5, -0.5, 0.0, -0.5, -0.5, 0.0
    ];

    let normals = [
        0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0,
    ];

    let mut points_vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut points_vbo);
    }
    assert!(points_vbo != 0);
    unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, points_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, 9 * mem::size_of::<GLfloat>() as GLsizeiptr, 
            points.as_ptr() as *const GLvoid, gl::STATIC_DRAW
        );
    }

    let mut normals_vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut normals_vbo);
    }
    assert!(normals_vbo != 0);
    unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, normals_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, 9 * mem::size_of::<GLfloat>() as GLsizeiptr, 
            normals.as_ptr() as *const GLvoid, gl::STATIC_DRAW
        );
    }

    let mut vao = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
    }
    assert!(vao != 0);
    unsafe {
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, points_vbo);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::BindBuffer(gl::ARRAY_BUFFER, normals_vbo);
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::EnableVertexAttribArray(0);
        gl::EnableVertexAttribArray(1);
    }

    let shader_programme = create_programme_from_files(VERTEX_SHADER_FILE, FRAGMENT_SHADER_FILE);

    // input variables
    let near = 0.1;                                  // clipping plane
    let far = 100.0;                                 // clipping plane
    let fov = 67.0 * math::ONE_DEG_IN_RAD; // convert 67 degrees to radians
    let aspect = unsafe { G_GL_WIDTH as f32 / G_GL_HEIGHT as f32 }; // aspect ratio
    // matrix components
    let range = f32::tan(fov * 0.5) * near;
    let Sx = (2.0 * near) / (range * aspect + range * aspect);
    let Sy = near / range;
    let Sz = -(far + near) / (far - near);
    let Pz = -(2.0 * far * near) / (far - near);
    let proj_mat = math::mat4(
        Sx,   0.0,  0.0,  0.0, 
        0.0,  Sy,   0.0,  0.0,
        0.0,  0.0,  Sz,  -1.0, 
        0.0,  0.0,  Pz,   0.0
    );

    /* create VIEW MATRIX */
    let cam_pos = [0.0, 0.0, 2.0];   // don't start at zero, or we will be too close
    let cam_yaw = 0.0;               // y-rotation in degrees
    let mat_trans = Mat4::identity().translate(&math::vec3((-cam_pos[0], -cam_pos[1], -cam_pos[2])));
    let mat_rot = Mat4::identity().rotate_y_deg(-cam_yaw);
    let view_mat = mat_rot * mat_trans;

    /* matrix for moving the triangle */
    let mut model_mat = Mat4::identity();

    unsafe {
    gl::UseProgram(shader_programme);
    let view_mat_location = gl::GetUniformLocation(shader_programme, "view_mat".as_ptr() as *const i8);
    assert!(view_mat_location != -1);
    gl::UniformMatrix4fv(view_mat_location, 1, gl::FALSE, view_mat.as_ptr());
    let proj_mat_location = gl::GetUniformLocation(shader_programme, "projection_mat".as_ptr() as *const i8);
    assert!(proj_mat_location != -1);
    gl::UniformMatrix4fv(proj_mat_location, 1, gl::FALSE, proj_mat.as_ptr());
    let model_mat_location = gl::GetUniformLocation( shader_programme, "model_mat".as_ptr() as *const i8);
    assert!(model_mat_location != -1);
    gl::UniformMatrix4fv(model_mat_location, 1, gl::FALSE, model_mat.as_ptr());

    gl::Enable(gl::CULL_FACE); // cull face
    gl::CullFace(gl::BACK);    // cull back face
    gl::FrontFace(gl::CW);     // GL_CCW for counter clock-wise

    while !g_window.should_close() {
        _update_fps_counter(&mut glfw, &mut g_window);
        let current_seconds = glfw.get_time();

        // wipe the drawing surface clear
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        gl::Viewport(0, 0, G_GL_WIDTH as i32, G_GL_HEIGHT as i32);

        gl::UseProgram(shader_programme);

        model_mat.m[12] = f32::sin(current_seconds as f32);
        gl::UniformMatrix4fv(model_mat_location, 1, gl::FALSE, model_mat.as_ptr());

        gl::BindVertexArray(vao);
        // draw points 0-3 from the currently bound VAO with current in-use shader
        gl::DrawArrays(gl::TRIANGLES, 0, 3);
        // update other events like input handling
        glfw.poll_events();
        match g_window.get_key(Key::Escape) {
            Action::Press | Action::Repeat => {
                g_window.set_should_close(true);
            }
            _ => {}
        }

        // put the stuff we've been drawing onto the display
        g_window.swap_buffers();
    }
    }
}
