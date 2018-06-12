extern crate gl;
extern crate glfw;
extern crate chrono;

#[macro_use] 
extern crate scan_fmt;


mod gl_utils;
mod graphics_math;
mod obj_parser;


use glfw::{Action, Context, Key};
use gl::types::{GLfloat, GLsizeiptr, GLvoid};

use std::mem;
use std::ptr;

use gl_utils::*;

use graphics_math as math;
use math::{Mat4, Versor};


const MESH_FILE: &str = "src/sphere.obj";
const VERTEX_SHADER_FILE: &str = "src/test.vert.glsl";
const FRAGMENT_SHADER_FILE: &str = "src/test.frag.glsl";
const NUM_SPHERES: usize = 4;

static mut PREVIOUS_SECONDS: f64 = 0.0;

// Create a unit quaternion q from an angle in degrees a, and an axis x,y,z.
fn create_versor(q: &mut [f32; 4], degrees: f32, x: f32, y: f32, z: f32) {
    let rad = math::ONE_DEG_IN_RAD * degrees;
    q[0] = f32::cos(rad / 2.0);
    q[1] = f32::sin(rad / 2.0) * x;
    q[2] = f32::sin(rad / 2.0) * y;
    q[3] = f32::sin(rad / 2.0) * z;
}

// Convert a unit quaternion q to a 4x4 matrix m.
fn quat_to_mat4(m: &mut [f32; 16], q: &[f32; 4]) {
    let w = q[0];
    let x = q[1];
    let y = q[2];
    let z = q[3];
    m[0] = 1.0 - 2.0 * y * y - 2.0 * z * z;
    m[1] = 2.0 * x * y + 2.0 * w * z;
    m[2] = 2.0 * x * z - 2.0 * w * y;
    m[3] = 0.0;
    m[4] = 2.0 * x * y - 2.0 * w * z;
    m[5] = 1.0 - 2.0 * x * x - 2.0 * z * z;
    m[6] = 2.0 * y * z + 2.0 * w * x;
    m[7] = 0.0;
    m[8] = 2.0 * x * z + 2.0 * w * y;
    m[9] = 2.0 * y * z - 2.0 * w * x;
    m[10] = 1.0 - 2.0 * x * x - 2.0 * y * y;
    m[11] = 0.0;
    m[12] = 0.0;
    m[13] = 0.0;
    m[14] = 0.0;
    m[15] = 1.0;
}

// Normalize a quaternion in case it got a bit mangled.
fn normalize_quat(q: &mut [f32; 4]) {
    // norm(q) = q / magnitude (q)
    // magnitude (q) = sqrt (w*w + x*x...)
    // only compute sqrt if interior sum != 1.0
    let sum = q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3];
    // NB: floats have min 6 digits of precision
    let threshold = 0.0001;
    if f32::abs(1.0 - sum) < threshold {
        return;
    }
    let norm = f32::sqrt(sum);

    q[0] = q[0] / norm;
    q[1] = q[1] / norm;
    q[2] = q[2] / norm;
    q[3] = q[3] / norm;
}

// Multiply quaternions to get another one. result= R*S.
fn mult_quat_quat(result: &mut [f32; 4], r: &[f32; 4], s: &[f32; 4]) {
    let w = s[0] * r[0] - s[1] * r[1] - s[2] * r[2] - s[3] * r[3];
    let x = s[0] * r[1] + s[1] * r[0] - s[2] * r[3] + s[3] * r[2];
    let y = s[0] * r[2] + s[1] * r[3] + s[2] * r[0] - s[3] * r[1];
    let z = s[0] * r[3] - s[1] * r[2] + s[2] * r[1] + s[3] * r[0];
    result[0] = w;
    result[1] = x;
    result[2] = y;
    result[3] = z;

    // Renormalize in case of mangling.
    normalize_quat(result);
}

fn main() {
    // Start OpenGL.
    restart_gl_log();
    // Start GL context and O/S window using the GLFW helper library
    let (mut glfw, mut g_window, _g_events) = start_gl().unwrap();

    /*------------------------------CREATE
     * GEOMETRY-------------------------------*/
    let mesh = obj_parser::load_obj_file(MESH_FILE).unwrap();
    let vp = mesh.points;     // array of vertex points
    let vt = mesh.tex_coords; // array of vertex normals
    let vn = mesh.normals;    // array of texture coordinates
    let point_count = mesh.point_count;

    let mut vao = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        let mut points_vbo = 0;
        if !vp.is_empty() {
            gl::GenBuffers(1, &mut points_vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, points_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER, (3 * point_count * mem::size_of::<GLfloat>()) as GLsizeiptr, 
                vp.as_ptr() as *const GLvoid, gl::STATIC_DRAW
            );
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
            gl::EnableVertexAttribArray(0);
        }
    }
    /*-------------------------------CREATE
     * SHADERS-------------------------------*/
    let shader_programme = create_programme_from_files(VERTEX_SHADER_FILE, FRAGMENT_SHADER_FILE);
    let model_mat_location = unsafe { gl::GetUniformLocation( shader_programme, "model".as_ptr() as *const i8) };
    assert!(model_mat_location != -1);
    let view_mat_location  = unsafe { gl::GetUniformLocation( shader_programme, "view".as_ptr() as *const i8) };
    assert!(view_mat_location != -1);
    let proj_mat_location  = unsafe { gl::GetUniformLocation( shader_programme, "proj".as_ptr() as *const i8) };
    assert!(proj_mat_location != -1);
    /*-------------------------------CREATE CAMERA--------------------------------*/
    // input variables
    let near = 0.1;                                                 // Near clipping plane
    let far = 100.0;                                                // Far clipping plane
    let fovy = 67.0;                                                // 67 degrees
    let aspect = unsafe { G_GL_WIDTH as f32 / G_GL_HEIGHT as f32 }; // aspect ratio
    let proj_mat = Mat4::perspective(fovy, aspect, near, far);

    let cam_speed = 5.0;           // 1 unit per second
    let cam_heading_speed = 100.0; // 30 degrees per second
    let mut cam_pos = math::vec3((0.0, 0.0, 5.0));
    let cam_heading = 0.0;     // y-rotation in degrees
    let mut mat_trans = Mat4::translate(&Mat4::identity(), &math::vec3((-cam_pos.v[0], -cam_pos.v[1], -cam_pos.v[2])));
    // Rotation matrix from my maths library. just holds 16 floats
    let mut mat_rot = Mat4::zero();
    // make a quaternion representing negated initial camera orientation
    //let mut quaternion = [0.0; 4];
    //create_versor(&mut quaternion, -cam_heading, 0.0, 1.0, 0.0);
    let mut quaternion = Versor::from_axis_deg(-cam_heading, 0.0, 1.0, 0.0);
    // convert the quaternion to a rotation matrix (just an array of 16 floats)
    //quat_to_mat4(mat_rot.as_mut(), &quaternion);
    quaternion.to_mut_mat4(&mut mat_rot);

    // combine the inverse rotation and transformation to make a view matrix
    let mut view_mat = mat_rot * mat_trans;
    // keep track of some useful vectors that can be used for keyboard movement
    let mut fwd = math::vec4((0.0, 0.0, -1.0, 0.0));
    let mut rgt = math::vec4((1.0, 0.0, 0.0, 0.0));
    let mut up  = math::vec4((0.0, 1.0, 0.0, 0.0));

    /*---------------------------SET RENDERING
     * DEFAULTS---------------------------*/
    unsafe {
        gl::UseProgram(shader_programme);
        gl::UniformMatrix4fv(view_mat_location, 1, gl::FALSE, view_mat.as_ptr());
        gl::UniformMatrix4fv(proj_mat_location, 1, gl::FALSE, proj_mat.as_ptr());
    }

    // A world position for each sphere in the scene.
    let sphere_pos_wor = [
        math::vec3((-2.0, 0.0,  0.0)), math::vec3((2.0, 0.0,  0.0)),
        math::vec3((-2.0, 0.0, -2.0)), math::vec3((1.5, 1.0, -1.0))
    ];    
    // Unique model matrix for each sphere.
    let mut model_mats = vec![];
    for i in 0..NUM_SPHERES {
        model_mats.push(Mat4::translate(&Mat4::identity(), &sphere_pos_wor[i]));
    }

    unsafe {
        gl::Enable(gl::DEPTH_TEST);   // enable depth-testing
        gl::DepthFunc(gl::LESS);      // depth-testing interprets a smaller value as "closer"
        gl::Enable(gl::CULL_FACE);    // cull face
        gl::CullFace(gl::BACK);       // cull back face
        gl::FrontFace(gl::CCW);       // set counter-clock-wise vertex order to mean the front
        gl::ClearColor(0.2, 0.2, 0.2, 1.0); // grey background to help spot mistakes
        gl::Viewport(0, 0, G_GL_WIDTH as i32, G_GL_HEIGHT as i32);

        // Rendering loop.
        while !g_window.should_close() {
            // Update timers.
            let current_seconds = glfw.get_time();
            let elapsed_seconds = current_seconds - PREVIOUS_SECONDS;
            PREVIOUS_SECONDS = current_seconds;
            _update_fps_counter(&glfw, &mut g_window);

            // Wipe the drawing surface clear.
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::UseProgram(shader_programme);
            for i in 0..NUM_SPHERES {
                gl::UniformMatrix4fv(model_mat_location, 1, gl::FALSE, model_mats[i].as_ptr());
                gl::DrawArrays(gl::TRIANGLES, 0, point_count as i32);
            }

            // Update other events like input handling.
            glfw.poll_events();

            // Control keys.
            let mut cam_moved = false;
            let mut move_to = math::vec3((0.0, 0.0, 0.0));
            let mut cam_yaw: f32 = 0.0; // y-rotation in degrees
            let mut cam_pitch: f32 = 0.0;
            let mut cam_roll: f32 = 0.0;
            match g_window.get_key(Key::A) {
                Action::Press | Action::Repeat => {
                    move_to.v[0] -= (cam_speed as f32) * (elapsed_seconds as f32);
                    cam_moved = true;
                }
                _ => {}
            }
            match g_window.get_key(Key::D) {
                Action::Press | Action::Repeat => {
                    move_to.v[0] += (cam_speed as f32) * (elapsed_seconds as f32);
                    cam_moved = true;
                }
                _ => {}
            }
            match g_window.get_key(Key::Q) {
                Action::Press | Action::Repeat => {
                    move_to.v[1] += (cam_speed as f32) * (elapsed_seconds as f32);
                    cam_moved = true;
                }
                _ => {}
            }
            match g_window.get_key(Key::E) {
                Action::Press | Action::Repeat => {
                    move_to.v[1] -= (cam_speed as f32) * (elapsed_seconds as f32);
                    cam_moved = true;
                }
                _ => {}
            }
            match g_window.get_key(Key::W) {
                Action::Press | Action::Repeat => {
                    move_to.v[2] -= (cam_speed as f32) * (elapsed_seconds as f32);
                    cam_moved = true;
                }
                _ => {}
            }
            match g_window.get_key(Key::S) {
                Action::Press | Action::Repeat => {
                    move_to.v[2] += (cam_speed as f32) * (elapsed_seconds as f32);
                    cam_moved = true;
                }
                _ => {}
            }
            match g_window.get_key(Key::Left) {
                Action::Press | Action::Repeat => {
                    cam_yaw += (cam_heading_speed as f32) * (elapsed_seconds as f32);
                    cam_moved = true;

                    // create a quaternion representing change in heading (the yaw)
                    //let mut q_yaw = [0.0; 4];
                    //create_versor(&mut q_yaw, cam_yaw, up.v[0], up.v[1], up.v[2]);
                    let mut q_yaw = Versor::from_axis_deg(cam_yaw, up.v[0], up.v[1], up.v[2]);
                    // add yaw rotation to the camera's current orientation
                    //let quaternion_copy = quaternion.clone();
                    //mult_quat_quat(&mut quaternion, &q_yaw, &quaternion_copy);
                    quaternion = q_yaw * &quaternion;

                    // recalc axes to suit new orientation
                    //quat_to_mat4(mat_rot.as_mut(), &quaternion);
                    quaternion.to_mut_mat4(&mut mat_rot);

                    fwd = mat_rot * math::vec4((0.0, 0.0, -1.0, 0.0));
                    rgt = mat_rot * math::vec4((1.0, 0.0, 0.0, 0.0));
                    up  = mat_rot * math::vec4((0.0, 1.0, 0.0, 0.0));
                }
                _ => {}
            }
            match g_window.get_key(Key::Right) {
                Action::Press | Action::Repeat => {
                    cam_yaw -= (cam_heading_speed as f32) * (elapsed_seconds as f32);
                    cam_moved = true;
                    //let mut q_yaw = [0.0; 4];
                    //create_versor(&mut q_yaw, cam_yaw, up.v[0], up.v[1], up.v[2]);
                    //let quaternion_copy = quaternion.clone();
                    //mult_quat_quat(&mut quaternion, &q_yaw, &quaternion_copy);
                    let mut q_yaw = Versor::from_axis_deg(cam_yaw, up.v[0], up.v[1], up.v[2]);
                    quaternion = q_yaw * &quaternion;

                    // Recalculate axes to suit new orientation.
                    // quat_to_mat4(mat_rot.as_mut(), &quaternion);
                    quaternion.to_mut_mat4(&mut mat_rot);

                    fwd = mat_rot * math::vec4((0.0, 0.0, -1.0, 0.0));
                    rgt = mat_rot * math::vec4((1.0, 0.0, 0.0, 0.0));
                    up  = mat_rot * math::vec4((0.0, 1.0, 0.0, 0.0));
                }
                _ => {}
            }
            match g_window.get_key(Key::Up) {
                Action::Press | Action::Repeat => {
                    cam_pitch += (cam_heading_speed as f32) * (elapsed_seconds as f32);
                    cam_moved = true;
                    //let mut q_pitch = [0.0; 4];
                    //create_versor(&mut q_pitch, cam_pitch, rgt.v[0], rgt.v[1], rgt.v[2]);
                    let mut q_pitch = Versor::from_axis_deg(cam_pitch, rgt.v[0], rgt.v[1], rgt.v[2]);
                    //let quaternion_copy = quaternion.clone();
                    //mult_quat_quat(&mut quaternion, &q_pitch, &quaternion_copy);
                    quaternion = q_pitch * &quaternion;

                    // Recalculate axes to suit new orientation.
                    //quat_to_mat4(mat_rot.as_mut(), &quaternion);
                    quaternion.to_mut_mat4(&mut mat_rot);

                    fwd = mat_rot * math::vec4((0.0, 0.0, -1.0, 0.0));
                    rgt = mat_rot * math::vec4((1.0, 0.0, 0.0, 0.0));
                    up  = mat_rot * math::vec4((0.0, 1.0, 0.0, 0.0));
                }
                _ => {}
            }
            match g_window.get_key(Key::Down) {
                Action::Press | Action::Repeat => {
                    cam_pitch -= (cam_heading_speed as f32) * (elapsed_seconds as f32);
                    cam_moved = true;
                    //let mut q_pitch = [0.0; 4];
                    //create_versor(&mut q_pitch, cam_pitch, rgt.v[0], rgt.v[1], rgt.v[2]);
                    let mut q_pitch = Versor::from_axis_deg(cam_pitch, rgt.v[0], rgt.v[1], rgt.v[2]);
                    //let quaternion_copy = quaternion.clone();
                    //mult_quat_quat(&mut quaternion, &q_pitch, &quaternion_copy);
                    quaternion = q_pitch * &quaternion;

                    // recalc axes to suit new orientation
                    //quat_to_mat4(mat_rot.as_mut(), &quaternion);
                    quaternion.to_mut_mat4(&mut mat_rot);
                    fwd = mat_rot * math::vec4((0.0, 0.0, -1.0, 0.0));
                    rgt = mat_rot * math::vec4((1.0, 0.0, 0.0, 0.0));
                    up  = mat_rot * math::vec4((0.0, 1.0, 0.0, 0.0));
                }
                _ => {}
            }
            match g_window.get_key(Key::Z) {
                Action::Press | Action::Repeat => {
                    cam_roll -= (cam_heading_speed as f32) * (elapsed_seconds as f32);
                    cam_moved = true;
                    //let mut q_roll = [0.0; 4];
                    //create_versor(&mut q_roll, cam_roll, fwd.v[0], fwd.v[1], fwd.v[2]);
                    let mut q_roll = Versor::from_axis_deg(cam_roll, fwd.v[0], fwd.v[1], fwd.v[2]);
                    //let quaternion_copy = quaternion.clone();
                    //mult_quat_quat(&mut quaternion, &q_roll, &quaternion_copy);
                    quaternion = q_roll * &quaternion;

                    // Recalculate axes to suit new orientation.
                    //quat_to_mat4(mat_rot.as_mut(), &quaternion);
                    quaternion.to_mut_mat4(&mut mat_rot);
                    fwd = mat_rot * math::vec4((0.0, 0.0, -1.0, 0.0));
                    rgt = mat_rot * math::vec4((1.0, 0.0, 0.0, 0.0));
                    up  = mat_rot * math::vec4((0.0, 1.0, 0.0, 0.0));
                }
                _ => {}
            }
            match g_window.get_key(Key::C) {
                Action::Press | Action::Repeat => {
                    cam_roll += (cam_heading_speed as f32) * (elapsed_seconds as f32);
                    cam_moved = true;
                    //let mut q_roll = [0.0; 4];
                    //create_versor(&mut q_roll, cam_roll, fwd.v[0], fwd.v[1], fwd.v[2]);
                    let mut q_roll = Versor::from_axis_deg(cam_roll, fwd.v[0], fwd.v[1], fwd.v[2]);
                    //let quaternion_copy = quaternion.clone();
                    //mult_quat_quat(&mut quaternion, &q_roll, &quaternion_copy);
                    quaternion = q_roll * &quaternion;

                    // recalc axes to suit new orientation
                    quaternion.to_mut_mat4(&mut mat_rot);
                    fwd = mat_rot * math::vec4((0.0, 0.0, -1.0, 0.0));
                    rgt = mat_rot * math::vec4((1.0, 0.0, 0.0, 0.0));
                    up  = mat_rot * math::vec4((0.0, 1.0, 0.0, 0.0));
                }
                _ => {}
            }

            // Update view matrix
            if cam_moved {
                //quat_to_mat4(mat_rot.as_mut(), &quaternion);
                quaternion.to_mut_mat4(&mut mat_rot);

                // checking for fp errors
                //  printf ("dot fwd . up %f\n", dot (fwd, up));
                //  printf ("dot rgt . up %f\n", dot (rgt, up));
                //  printf ("dot fwd . rgt\n %f", dot (fwd, rgt));

                cam_pos = cam_pos + math::vec3(fwd) * -move_to.v[2];
                cam_pos = cam_pos + math::vec3(up) * move_to.v[1];
                cam_pos = cam_pos + math::vec3(rgt) * move_to.v[0];
                mat_trans = Mat4::translate(&Mat4::identity(), &math::vec3(cam_pos));

                view_mat = mat_rot.inverse() * mat_trans.inverse();
                gl::UniformMatrix4fv(view_mat_location, 1, gl::FALSE, view_mat.as_ptr());
            }
        
            match g_window.get_key(Key::Escape) {
                Action::Press | Action::Repeat => {
                    g_window.set_should_close(true);
                }
                _ => {}
            }

            g_window.swap_buffers();
        }
    }
}
