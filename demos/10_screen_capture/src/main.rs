extern crate gl;
extern crate glfw;
extern crate chrono;
extern crate stb_image;
extern crate png;

#[macro_use] 
extern crate scan_fmt;

mod gl_utils;
mod graphics_math;
mod obj_parser;
mod screen;
mod logger;


use glfw::{Action, Context, Key};
use gl::types::{GLfloat, GLsizeiptr, GLvoid, GLuint};

use stb_image::image;
use stb_image::image::LoadResult;

use gl_utils::*;

use std::mem;
use std::ptr;

use graphics_math as math;
use math::Mat4;


const GL_LOG_FILE: &str = "gl.log";
const VERTEX_SHADER_FILE: &str = "src/test.vert.glsl";
const FRAGMENT_SHADER_FILE: &str = "src/test.frag.glsl";
const TEXTURE_FILE: &str = "src/skulluvmap.png";

const GL_TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FE;
const GL_MAX_TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FF;


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

    // Check that the image size is a power of two.
    if (width & (width - 1)) != 0 || (height & (height - 1)) != 0 {
        eprintln!("WARNING: texture {} is not power-of-2 dimensions", file_name);
    }

    let width_in_bytes = 4 *width;
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

fn gl_capture_frame_buffer(context: &GLContext, buffer: &mut [u8]) -> bool {
    unsafe {
        gl::ReadPixels(
            0, 0, context.width as i32, context.height as i32, 
            gl::RGB, gl::UNSIGNED_BYTE, 
            buffer.as_mut_ptr() as *mut GLvoid
        );
    }

    true
}

fn main() {
    let logger = restart_gl_log(GL_LOG_FILE);
    let mut context = start_gl(&logger).unwrap();

    // Instruct GL to only draw onto a pixel if the shape is closer to the viewer.
    unsafe {
        // Enable depth testing.
        gl::Enable(gl::DEPTH_TEST);
        // Depth testing interprets a smaller value as closer to the camera.
        gl::DepthFunc(gl::LESS);
    }

    /* OTHER STUFF GOES HERE NEXT */
    let points: [GLfloat; 18] = [
        -0.5, -0.5, 0.0,  0.5, -0.5, 0.0,  0.5,  0.5, 0.0, 
         0.5,  0.5, 0.0, -0.5,  0.5, 0.0, -0.5, -0.5, 0.0
    ];

    let texcoords: [GLfloat; 12] = [
        0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0
    ];

    let mut points_vbo: GLuint = 0;
    unsafe {
        gl::GenBuffers(1, &mut points_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, points_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, (points.len() * mem::size_of::<GLfloat>()) as GLsizeiptr, 
            points.as_ptr() as *const GLvoid, gl::STATIC_DRAW
        );
    }
    assert!(points_vbo != 0);

    let mut texcoords_vbo: GLuint = 0;
    unsafe {
        gl::GenBuffers(1, &mut texcoords_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, texcoords_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, (texcoords.len() * mem::size_of::<GLfloat>()) as GLsizeiptr, 
            texcoords.as_ptr() as *const GLvoid, gl::STATIC_DRAW
        );
    }
    assert!(texcoords_vbo != 0);

    let mut vao: GLuint = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, points_vbo);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::BindBuffer(gl::ARRAY_BUFFER, texcoords_vbo);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 0, ptr::null()); // normalize!
        gl::EnableVertexAttribArray(0);
        gl::EnableVertexAttribArray(1);
    }
    assert!(vao != 0);

    let shader_programme = create_programme_from_files(&logger, VERTEX_SHADER_FILE, FRAGMENT_SHADER_FILE);

    // Camera model input variables.
    let near = 0.1;                                  // clipping plane
    let far = 100.0;                                 // clipping plane
    let fov = 67.0;                                  // convert 67 degrees to radians
    let aspect = context.width as f32 / context.height as f32; // aspect ratio
    let proj_mat = Mat4::perspective(fov, aspect, near, far);

    // View matrix components.
    let cam_speed: GLfloat = 1.0;             // 1 unit per second
    let cam_yaw_speed: GLfloat = 10.0;        // 10 degrees per second
    let mut cam_pos: [GLfloat; 3] = [0.0, 0.0, 2.0]; // don't start at zero, or we will be too close
    let mut cam_yaw: GLfloat = 0.0;               // y-rotation in degrees
    let mut mat_trans = Mat4::identity().translate(&math::vec3((-cam_pos[0], -cam_pos[1], -cam_pos[2])));
    let mut mat_rot = Mat4::identity().rotate_y_deg(-cam_yaw);
    let mut view_mat = mat_rot * mat_trans;

    let view_mat_location = unsafe {
        gl::GetUniformLocation(shader_programme, "view".as_ptr() as *const i8)
    };
    assert!(view_mat_location != -1);
    unsafe {
        gl::UseProgram(shader_programme);
        gl::UniformMatrix4fv(view_mat_location, 1, gl::FALSE, view_mat.as_ptr());
    }

    let proj_mat_location = unsafe { 
        gl::GetUniformLocation(shader_programme, "proj".as_ptr() as *const i8)
    };
    unsafe {
        gl::UseProgram(shader_programme);
        gl::UniformMatrix4fv(proj_mat_location, 1, gl::FALSE, proj_mat.as_ptr());
    }
    assert!(proj_mat_location != -1);

    // Load texture.
    let mut tex: GLuint = 0;
    load_texture(TEXTURE_FILE, &mut tex);
    assert!(tex != 0);

    unsafe {
        gl::Enable(gl::CULL_FACE);
        gl::CullFace(gl::BACK);
        gl::FrontFace(gl::CCW);
    }

    while !context.window.should_close() {
        let current_seconds = context.glfw.get_time();
        let delta_seconds = current_seconds - context.elapsed_time_seconds;
        context.elapsed_time_seconds = current_seconds;

        update_fps_counter(&mut context);
        unsafe {
            // Clear the drawing canvas.
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::Viewport(0, 0, context.width as i32, context.height as i32);

            gl::UseProgram(shader_programme);
            gl::BindVertexArray(vao);
            // Draw points 0-3 from the currently bound VAO with current in-use shader.
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }

        context.glfw.poll_events();

        match context.window.get_key(Key::PrintScreen) {
            Action::Press | Action::Repeat => {
                println!("Screen captured.");
                screen::capture(
                    context.height as usize, context.width as usize, context.channel_depth as usize, 
                    &|buf| { gl_capture_frame_buffer(&context, buf) }
                ).unwrap();
            }
            _ => {}
        }

        // Process I/O events.
        // Camera control keys.
        let mut cam_moved = false;
        match context.window.get_key(Key::A) {
            Action::Press | Action::Repeat => {
                cam_pos[0] -= cam_speed * (delta_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::D) {
            Action::Press | Action::Repeat => {
                cam_pos[0] += cam_speed * (delta_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::Up) {
            Action::Press | Action::Repeat => {
                cam_pos[1] += cam_speed * (delta_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::Down) {
            Action::Press | Action::Repeat => {
                cam_pos[1] -= cam_speed * (delta_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::W) {
            Action::Press | Action::Repeat => {
                cam_pos[2] -= cam_speed * (delta_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::S) {
            Action::Press | Action::Repeat => {
                cam_pos[2] += cam_speed * (delta_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::Left) {
            Action::Press | Action::Repeat => {
                cam_yaw += cam_yaw_speed * (delta_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::Right) {
            Action::Press | Action::Repeat => {
                cam_yaw -= cam_yaw_speed * (delta_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }

        // Update view matrix.
        if cam_moved {
            // Camera translation.
            mat_trans = Mat4::identity().translate(&math::vec3((-cam_pos[0], -cam_pos[1], -cam_pos[2])));
            mat_rot = Mat4::identity().rotate_y_deg(-cam_yaw);
            view_mat = mat_rot * mat_trans;
            unsafe {
                gl::UniformMatrix4fv(view_mat_location, 1, gl::FALSE, view_mat.as_ptr());
            }
        }

        // Check whether the user signaled GLFW to close the window.
        match context.window.get_key(Key::Escape) {
            Action::Press | Action::Repeat => {
                context.window.set_should_close(true);
            }
            _ => {}
        }

        // Display the next frame.
        context.window.swap_buffers();
    }
}
