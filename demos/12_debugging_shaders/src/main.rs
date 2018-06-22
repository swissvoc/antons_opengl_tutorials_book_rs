extern crate gl;
extern crate glfw;
extern crate chrono;
extern crate stb_image;

#[macro_use] 
extern crate scan_fmt;

mod gl_utils;
mod graphics_math;
mod obj_parser;
mod logger;


use glfw::{Action, Context, Key};
use gl::types::{GLfloat, GLsizeiptr, GLvoid, GLuint};
use stb_image::image;
use stb_image::image::{LoadResult, Image};

use std::mem;
use std::ptr;
use std::process;

use gl_utils::*;

use graphics_math as math;
use math::Mat4;

const GL_LOG_FILE: &str = "gl.log";
const VERTEX_SHADER_FILE: &str = "src/test.vert.glsl";
const FRAGMENT_SHADER_FILE: &str = "src/test.frag.glsl";
const MESH_FILE: &str = "src/suzanne.obj";

const GL_TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FE;
const GL_MAX_TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FF;

static mut PREVIOUS_SECONDS: f64 = 0.0;


fn load_texture(file_name: &str, tex: &mut GLuint) -> bool {
    let force_channels = 4;
    let mut image_data = match image::load_with_depth(file_name, force_channels, false) {
        LoadResult::ImageU8(image_data) => image_data,
        LoadResult::Error(_) => {
            eprintln!("ERROR: could not load {}", file_name);
            return false;
        }
        LoadResult::ImageF32(_) => {
            eprintln!("ERROR: Tried to load an image as byte vectors, got f32: {}", file_name);
            return false;
        }
    };

    let width = image_data.width;
    let height = image_data.height;
    let depth = image_data.depth;

    // Check that the image size is a power of two.
    if (width & (width - 1)) != 0 || (height & (height - 1)) != 0 {
        eprintln!("WARNING: texture {} is not power-of-2 dimensions", file_name);
    }

    let width_in_bytes = width * 4;
    let half_height = height / 2;
    for row in 0..half_height {
        for col in 0..width_in_bytes {
            let temp = image_data.data[row * width_in_bytes + col];
            image_data.data[row * width_in_bytes + col] = image_data.data[((height - row - 1) * width_in_bytes) + col];
            image_data.data[((height - row - 1) * width_in_bytes) + col] = temp;
        }
    }

    unsafe {
        gl::GenTextures(1, tex);
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, *tex);
        gl::TexImage2D(
            gl::TEXTURE_2D, 0, gl::RGBA as i32, width as i32, height as i32, 0, 
            gl::RGBA, gl::UNSIGNED_BYTE, 
            image_data.data.as_ptr() as *const GLvoid
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
    }

    let mut max_aniso = 0.0;
    // TODO: Check this against my dependencies.
    unsafe {
        gl::GetFloatv(GL_MAX_TEXTURE_MAX_ANISOTROPY_EXT, &mut max_aniso);
        // Set the maximum!
        gl::TexParameterf(gl::TEXTURE_2D, GL_TEXTURE_MAX_ANISOTROPY_EXT, max_aniso);
    }

    return true;
}

fn main() {
    let logger = restart_gl_log(GL_LOG_FILE);
    // Start a GL context and O/S window using the GLFW helper library.
    let mut context = match start_gl(&logger) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Failed to Initialize OpenGL context. Got error:");
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    // Tell GL to only draw onto a pixel if the shape is closer to the viewer.
    unsafe {
        // Enable depth testing.
        gl::Enable(gl::DEPTH_TEST);
        // Depth testing interprets a smaller value as closer to the eye.
        gl::DepthFunc(gl::LESS);
        // Cull face.
        gl::Enable(gl::CULL_FACE);
        // Cull back face.
        gl::CullFace(gl::BACK);
        // GL_CW for clockwise.    
        gl::FrontFace(gl::CCW);
        // grey background to help spot mistakes
        gl::ClearColor(0.2, 0.2, 0.2, 1.0);
        gl::Viewport(0, 0, context.width as i32, context.height as i32);
    }

    /*------------------------------CREATE GEOMETRY------------------------------*/
    let mesh = match obj_parser::load_obj_file(MESH_FILE) {
        Ok(val) => val,
        Err(e) => {
            logger.log_err(&format!("ERROR: loading mesh file. Loader returned error\n{}", e));
            process::exit(1);
        }
    };

    let vp = mesh.points;
    let vn = mesh.normals;
    let vt = mesh.tex_coords;
    let g_point_count = mesh.point_count;

    let mut vao = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
    }
    assert!(vao > 0);

    let mut points_vbo = 0;
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
    assert!(points_vbo > 0);

    let mut normals_vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut normals_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, normals_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, (3 * g_point_count * mem::size_of::<GLfloat>()) as GLsizeiptr, 
            vn.as_ptr() as *const GLvoid, gl::STATIC_DRAW
        );
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::EnableVertexAttribArray(1);
    }
    assert!(normals_vbo > 0);

    let mut texcoords_vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut texcoords_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, texcoords_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, (2 * g_point_count * mem::size_of::<GLfloat>()) as GLsizeiptr, 
            vp.as_ptr() as *const GLvoid, gl::STATIC_DRAW
        );
        gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::EnableVertexAttribArray(2);
    }
    assert!(texcoords_vbo > 0);

    /*-------------------------------CREATE SHADERS------------------------------*/
    let shader_programme = create_programme_from_files(&logger, VERTEX_SHADER_FILE, FRAGMENT_SHADER_FILE);
    let view_mat_location = unsafe {
        gl::GetUniformLocation(shader_programme, "view".as_ptr() as *const i8)
    };
    assert!(view_mat_location > -1);
    let proj_mat_location = unsafe { 
        gl::GetUniformLocation(shader_programme, "proj".as_ptr() as *const i8)
    };
    assert!(proj_mat_location > -1);

    /* if converting to GLSL 410 do this to replace GLSL texture bindings:
    GLint diffuse_map_loc, specular_map_loc, ambient_map_loc, emission_map_loc;
    diffuse_map_loc = glGetUniformLocation (shader_programme, "diffuse_map");
    specular_map_loc = glGetUniformLocation (shader_programme, "specular_map");
    ambient_map_loc = glGetUniformLocation (shader_programme, "ambient_map");
    emission_map_loc = glGetUniformLocation (shader_programme, "emission_map");
    assert (diffuse_map_loc > -1);
    assert (specular_map_loc > -1);
    assert (ambient_map_loc > -1);
    assert (emission_map_loc > -1);
    glUseProgram (shader_programme);
    glUniform1i (diffuse_map_loc, 0);
    glUniform1i (specular_map_loc, 1);
    glUniform1i (ambient_map_loc, 2);
    glUniform1i (emission_map_loc, 3);
    */

    // load texture
    let mut tex_diff = 0; 
    let mut tex_spec = 0; 
    let mut tex_amb = 0; 
    let mut tex_emiss = 0;
    unsafe {
        gl::ActiveTexture(gl::TEXTURE0);
        load_texture("boulder_diff.png", &mut tex_diff);
        gl::ActiveTexture(gl::TEXTURE1);
        load_texture("boulder_spec.png", &mut tex_spec);
        gl::ActiveTexture(gl::TEXTURE2);
        load_texture("ao.png", &mut tex_amb);
        gl::ActiveTexture(gl::TEXTURE3);
        load_texture("tileable9b_emiss.png", &mut tex_emiss);
    }

    // input variables
    let near = 0.1;                                  // clipping plane
    let far = 100.0;                                 // clipping plane
    let fov = 67.0;                                  // convert 67 degrees to radians
    let aspect = context.width as f32 / context.height as f32; // aspect ratio
    let proj_mat = Mat4::perspective(fov, aspect, near, far);

    // matrix components
    let cam_speed: GLfloat = 1.0;             // 1 unit per second
    let cam_yaw_speed: GLfloat = 10.0;        // 10 degrees per second
    let mut cam_pos: [GLfloat; 3] = [0.0, 0.0, 5.0]; // don't start at zero, or we will be too close
    let mut cam_yaw: GLfloat = 0.0;               // y-rotation in degrees
    let mut mat_trans = Mat4::identity().translate(&math::vec3((-cam_pos[0], -cam_pos[1], -cam_pos[2])));
    let mut mat_rot = Mat4::identity().rotate_y_deg(-cam_yaw);
    let mut view_mat = mat_rot * mat_trans;

    unsafe {
        gl::UseProgram(shader_programme);
        gl::UniformMatrix4fv(view_mat_location, 1, gl::FALSE, view_mat.as_ptr());
        gl::UniformMatrix4fv(proj_mat_location, 1, gl::FALSE, proj_mat.as_ptr());
    }

    while !context.window.should_close() {
        let current_seconds = context.glfw.get_time();
        let elapsed_seconds = unsafe { current_seconds - PREVIOUS_SECONDS };
        unsafe {
            PREVIOUS_SECONDS = current_seconds;
        }

        update_fps_counter(&mut context);
        unsafe {
            // wipe the drawing surface clear
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::Viewport(0, 0, context.width as i32, context.height as i32);

            gl::UseProgram(shader_programme);
            gl::BindVertexArray(vao);
            // draw points 0-3 from the currently bound VAO with current in-use shader
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            // update other events like input handling
        }

        context.glfw.poll_events();

        // control keys
        let mut cam_moved = false;
        match context.window.get_key(Key::A) {
            Action::Press | Action::Repeat => {
                cam_pos[0] -= cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::D) {
            Action::Press | Action::Repeat => {
                cam_pos[0] += cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::Up) {
            Action::Press | Action::Repeat => {
                cam_pos[1] += cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::Down) {
            Action::Press | Action::Repeat => {
                cam_pos[1] -= cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::W) {
            Action::Press | Action::Repeat => {
                cam_pos[2] -= cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::S) {
            Action::Press | Action::Repeat => {
                cam_pos[2] += cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::Left) {
            Action::Press | Action::Repeat => {
                cam_yaw += cam_yaw_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::Right) {
            Action::Press | Action::Repeat => {
                cam_yaw -= cam_yaw_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        // update view matrix
        if cam_moved {
            mat_trans = Mat4::identity().translate(&math::vec3((-cam_pos[0], -cam_pos[1], -cam_pos[2]))); // cam translation
            mat_rot = Mat4::identity().rotate_y_deg(-cam_yaw);
            view_mat = mat_rot * mat_trans;
            unsafe {
                gl::UniformMatrix4fv(view_mat_location, 1, gl::FALSE, view_mat.as_ptr());
            }
        }

        match context.window.get_key(Key::Escape) {
            Action::Press | Action::Repeat => {
                context.window.set_should_close(true);
            }
            _ => {}
        }
        // Put the stuff we've been drawing onto the display.
        context.window.swap_buffers();
    }
}
