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
use std::process;

use gl_utils::*;

use graphics_math as math;
use math::{Vec3, Mat4, Versor};

const MESH_FILE: &str = "src/sphere.obj";
const VERTEX_SHADER_FILE: &str = "src/test.vert.glsl";
const FRAGMENT_SHADER_FILE: &str = "src/test.frag.glsl";
const NUM_SPHERES: usize = 4;
const SPHERE_RADIUS: f32 = 1.0;

static mut PREVIOUS_SECONDS: f64 = 0.0;
static mut G_SELECTED_SPHERE: isize = -1;


///
/// Take the mouse position on screen and return ray cast into the scene in
/// world space coordinates.
///
fn get_ray_from_mouse(proj_mat: &Mat4, view_mat: &Mat4, mouse_x: f32, mouse_y: f32) -> Vec3 {
    // Screen space (Viewport coordinates).
    let x = (2.0 * mouse_x) / (G_GL_WIDTH as f32) - 1.0;
    let y = 1.0 - (2.0 * mouse_y) / (G_GL_HEIGHT as f32);
    let z = 1.0;
    // Normalised device coordinates.
    let ray_nds = math::vec3((x, y, z));
    // Clip space.
    let ray_clip = math::vec4((ray_nds.v[0], ray_nds.v[1], -1.0, 1.0));
    // Eye space.
    let ray_eye = proj_mat.inverse() * ray_clip;
    let ray_eye = math::vec4((ray_eye.v[0], ray_eye.v[1], -1.0, 0.0));
    // World space.
    let ray_wor = math::vec3(view_mat.inverse() * ray_eye);
    // Don't forget to normalize the vector at some point.
    ray_wor = ray_wor.normalize();
    
    ray_wor
}

/* check if a ray and a sphere intersect. if not hit, returns false. it rejects
intersections behind the ray caster's origin, and sets intersection_distance to
the closest intersection */
fn ray_sphere(
    ray_origin_wor: Vec3, ray_direction_wor: Vec3,
    sphere_centre_wor: Vec3, sphere_radius: f32, intersection_distance: &f32) -> bool {
    
    // Work out components of quadratic.
    let dist_to_sphere = ray_origin_wor - sphere_centre_wor;
    let b = ray_direction_wor.dot(&dist_to_sphere);
    let c = dist_to_sphere.dot(&dist_to_sphere) - sphere_radius * sphere_radius;
    let b_squared_minus_c = b * b - c;
    // Check for "imaginary" answer. == ray completely misses sphere
    if b_squared_minus_c < 0.0 {
        return false;
    }
    // Check whether the ray hits the sphere twice (into and out of the sphere).
    if b_squared_minus_c > 0.0 {
        // Get the 2 intersection distances along the ray.
        let t_a = -b + f32::sqrt(b_squared_minus_c);
        let t_b = -b - f32::sqrt(b_squared_minus_c);
        *intersection_distance = t_b;
        // if the object is behind the viewer, throw one or both away.
        if t_a < 0.0 {
            if t_b < 0.0 {
                return false;
            }
        } else if t_b < 0.0 {
            *intersection_distance = t_a;
        }

        return true;
    }
    // Check whether the ray skims the surface (i.e. it hits at one point).
    if b_squared_minus_c == 0.0 {
        // If the ray hits behind the viewer, throw away the ray.
        let t = -b + f32::sqrt(b_squared_minus_c);
        if t < 0.0 {
            return false;
        }
        *intersection_distance = t;
        return true;
    }
    // NOTE: we could also check if the ray origin is inside the sphere radius.
    return false;
}

/* this function is called when the mouse buttons are clicked or un-clicked */
fn glfw_mouse_click_callback(GLFWwindow *window, int button, int action, int mods) {
    // Note: could query if window has lost focus here
    if ( GLFW_PRESS == action ) {
        double xpos, ypos;
        glfwGetCursorPos( g_window, &xpos, &ypos );
        // work out ray
        vec3 ray_wor = get_ray_from_mouse( (float)xpos, (float)ypos );
        // check ray against all spheres in scene
        int closest_sphere_clicked = -1;
        float closest_intersection = 0.0f;
        for ( int i = 0; i < NUM_SPHERES; i++ ) {
            float t_dist = 0.0f;
            if ( ray_sphere( cam_pos, ray_wor, sphere_pos_wor[i], sphere_radius,
                                             &t_dist ) ) {
                // if more than one sphere is in path of ray, only use the closest one
                if ( -1 == closest_sphere_clicked || t_dist < closest_intersection ) {
                    closest_sphere_clicked = i;
                    closest_intersection = t_dist;
                }
            }
        } // endfor
        g_selected_sphere = closest_sphere_clicked;
        printf( "sphere %i was clicked\n", closest_sphere_clicked );
    }
}

fn main() {
    /*--------------------------------START OPENGL--------------------------------*/
    restart_gl_log();
    // Start GL context and OS window using the GLFW helper library.
    let (mut glfw, mut g_window, _g_events) = start_gl().unwrap();
    // set a function to be called when the mouse is clicked
    //glfw::ffi::glfwSetMouseButtonCallback( g_window, glfw_mouse_click_callback );
    
    /*------------------------------CREATE GEOMETRY-------------------------------*/
    let mesh = match obj_parser::load_obj_file(MESH_FILE) {
        Ok(val) => val,
        Err(e) => {
            gl_log_err(&format!("ERROR: loading mesh file. Loader returned error\n{}", e));
            process::exit(1);
        }
    };

    let vp = mesh.points;     
    let vt = mesh.tex_coords;
    let vn = mesh.normals;
    let g_point_count = mesh.point_count;

    let mut vao = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
    }

    let mut points_vbo = 0;
    if !vp.is_empty() {
        unsafe {
            gl::GenBuffers(1, &mut points_vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, points_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER, (3 * g_point_count * mem::size_of::<GLfloat>()) as GLsizeiptr, 
                vp.as_ptr() as *const GLvoid, gl::STATIC_DRAW
            );
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
            gl::EnableVertexAttribArray(0);
        }
    }

    /*-------------------------------CREATE SHADERS-------------------------------*/
    // FIXME: Why don't the gl::GetUniformLocation calls fetch the resources when the functions are called?
    let shader_programme = create_programme_from_files(VERTEX_SHADER_FILE, FRAGMENT_SHADER_FILE);
    let model_mat_location = unsafe { gl::GetUniformLocation(shader_programme, "model".as_ptr() as *const i8) };
    assert!(model_mat_location != -1);
    let view_mat_location  = unsafe { gl::GetUniformLocation(shader_programme, "view".as_ptr() as *const i8) };
    assert!(view_mat_location != -1);
    let proj_mat_location  = unsafe { gl::GetUniformLocation(shader_programme, "proj".as_ptr() as *const i8) };
    assert!(proj_mat_location != -1);
    let blue_location = unsafe { gl::GetUniformLocation(shader_programme, "blue".as_ptr() as *const i8 ) };
    assert!(blue_location != -1);

    /*-------------------------------CREATE CAMERA--------------------------------*/
    const ONE_DEG_IN_RAD: f32 = math::ONE_DEG_IN_RAD; // 0.017444444
    // Input variables for camera model.
    let near = 0.1;                                                 // Near clipping plane
    let far = 100.0;                                                // Far clipping plane
    let fovy = 67.0;                                                // 67 Degree field of view.
    let aspect = unsafe { G_GL_WIDTH as f32 / G_GL_HEIGHT as f32 }; // Aspect ratio
    let proj_mat = Mat4::perspective(fovy, aspect, near, far);

    let cam_speed = 3.0;          // 1 unit per second
    let cam_heading_speed = 50.0; // 30 degrees per second
    let cam_heading = 0.0;        // y-rotation in degrees
    let mut cam_pos = math::vec3((0.0, 0.0, 5.0));
    let mut mat_trans = Mat4::translate(&Mat4::identity(), &math::vec3((-cam_pos.v[0], -cam_pos.v[1], -cam_pos.v[2])));
    let mut mat_rot = Mat4::identity().rotate_y_deg(-cam_heading);
    let mut q = Versor::from_axis_deg(-cam_heading, 0.0, 1.0, 0.0);

    let mut view_mat = mat_rot * mat_trans;
    // Keep track of some useful vectors that can be used for keyboard movement.
    let mut fwd = math::vec4((0.0, 0.0, -1.0, 0.0));
    let mut rgt = math::vec4((1.0, 0.0, 0.0, 0.0));
    let mut up  = math::vec4((0.0, 1.0, 0.0, 0.0));

    /*---------------------------SET RENDERING DEFAULTS---------------------------*/
    unsafe {
        gl::UseProgram(shader_programme);
        gl::UniformMatrix4fv(view_mat_location, 1, gl::FALSE, view_mat.as_ptr());
        gl::UniformMatrix4fv(proj_mat_location, 1, gl::FALSE, proj_mat.as_ptr());
    }

    let sphere_pos_wor = [
        math::vec3((-2.0, 0.0, 0.0)),  math::vec3((2.0, 0.0, 0.0)),
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
    }

    /*-------------------------------RENDERING LOOP-------------------------------*/
    while !g_window.should_close() {
        // Update timers.
        let current_seconds = glfw.get_time();
        let elapsed_seconds = unsafe { current_seconds - PREVIOUS_SECONDS };
        unsafe {
            PREVIOUS_SECONDS = current_seconds;
        }
        // Update FPS.
        _update_fps_counter(&glfw, &mut g_window);

        unsafe {
            // Wipe the drawing surface clear.
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::UseProgram(shader_programme);
            gl::BindVertexArray(vao);
            for i in 0..NUM_SPHERES {
                if i as isize == G_SELECTED_SPHERE {
                    gl::Uniform1f(blue_location, 1.0);
                } else {
                    gl::Uniform1f(blue_location, 0.0);
                }
                gl::UniformMatrix4fv(model_mat_location, 1, gl::FALSE, model_mats[i].as_ptr());
                gl::DrawArrays(gl::TRIANGLES, 0, g_point_count as i32);
            }
        }

        // Update other events like input handling.
        glfw.poll_events();

        // control keys
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
                let mut q_yaw = Versor::from_axis_deg(cam_yaw, up.v[0], up.v[1], up.v[2]);
                q = q_yaw * &q;
            }
            _ => {}
        }
        match g_window.get_key(Key::Right) {
            Action::Press | Action::Repeat => {
                cam_yaw -= (cam_heading_speed as f32) * (elapsed_seconds as f32);
                cam_moved = true;
                let mut q_yaw = Versor::from_axis_deg(cam_yaw, up.v[0], up.v[1], up.v[2]);
                q = q_yaw * &q;
            }
            _ => {}
        }
        match g_window.get_key(Key::Up) {
            Action::Press | Action::Repeat => {
                cam_pitch += (cam_heading_speed as f32) * (elapsed_seconds as f32);
                cam_moved = true;
                let mut q_pitch = Versor::from_axis_deg(cam_pitch, rgt.v[0], rgt.v[1], rgt.v[2]);
                q = q_pitch * &q;
            }
            _ => {}
        }
        match g_window.get_key(Key::Down) {
            Action::Press | Action::Repeat => {
                cam_pitch -= (cam_heading_speed as f32) * (elapsed_seconds as f32);
                cam_moved = true;
                let mut q_pitch = Versor::from_axis_deg(cam_pitch, rgt.v[0], rgt.v[1], rgt.v[2]);
                q = q_pitch * &q;
            }
            _ => {}
        }
        match g_window.get_key(Key::Z) {
            Action::Press | Action::Repeat => {
                cam_roll -= (cam_heading_speed as f32) * (elapsed_seconds as f32);
                cam_moved = true;
                let mut q_roll = Versor::from_axis_deg(cam_roll, fwd.v[0], fwd.v[1], fwd.v[2]);
                q = q_roll * &q;
            }
            _ => {}
        }
        match g_window.get_key(Key::C) {
            Action::Press | Action::Repeat => {
                cam_roll += (cam_heading_speed as f32) * (elapsed_seconds as f32);
                cam_moved = true;
                let mut q_roll = Versor::from_axis_deg(cam_roll, fwd.v[0], fwd.v[1], fwd.v[2]);
                q = q_roll * &q;
            }
            _ => {}
        }

        // Update view matrix.
        if cam_moved {
            // Recalculate local axes so we can move forward in the direction the 
            // camera is pointing.
            // R = quat_to_mat4( q );
            q.to_mut_mat4(&mut mat_rot);

            fwd = mat_rot * math::vec4((0.0, 0.0, -1.0, 0.0));
            rgt = mat_rot * math::vec4((1.0, 0.0, 0.0, 0.0));
            up  = mat_rot * math::vec4((0.0, 1.0, 0.0, 0.0));

            cam_pos = cam_pos + math::vec3(fwd) * -move_to.v[2];
            cam_pos = cam_pos + math::vec3(up)  *  move_to.v[1];
            cam_pos = cam_pos + math::vec3(rgt) *  move_to.v[0];
            mat_trans = Mat4::translate(&Mat4::identity(), &math::vec3(cam_pos));

            view_mat = mat_rot.inverse() * mat_trans.inverse();
            unsafe {
                gl::UniformMatrix4fv(view_mat_location, 1, gl::FALSE, view_mat.as_ptr());
            }
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
