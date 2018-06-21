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


use glfw::{Action, Context, Key};
use gl::types::{GLfloat, GLsizeiptr, GLvoid, GLuint};

use stb_image::image;
use stb_image::image::LoadResult;

use png::HasParameters;

use chrono::prelude::Utc;

use std::mem;
use std::ptr;
use std::path::Path;
use std::fs::File;
use std::io::BufWriter;

use gl_utils::*;

use graphics_math as math;
use math::Mat4;

const VERTEX_SHADER_FILE: &str = "src/test.vert.glsl";
const FRAGMENT_SHADER_FILE: &str = "src/test.frag.glsl";
const TEXTURE_FILE: &str = "src/skulluvmap.png";

const GL_TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FE;
const GL_MAX_TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FF;

const G_VIDEO_SECONDS_TOTAL: usize = 10;
const G_VIDEO_FPS: usize = 25;

static mut PREVIOUS_SECONDS: f64 = 0.0;


struct FrameBufferDumper {
    width: usize,
    height: usize,
    channels: usize,
    index: Vec<(usize, usize)>,
    data: Vec<u8>,
}

impl FrameBufferDumper {
    fn new(
        video_fps: usize, video_seconds_total: usize, 
        width: usize, height: usize, channels: usize) -> FrameBufferDumper {
        
        FrameBufferDumper {
            width: width,
            height: height,
            channels: channels,
            index: vec![],
            data: vec![0; video_fps * video_seconds_total * width * height * channels],
        }
    }

    fn frame_count(&self) -> usize {
        self.index.len()
    }

    fn size_bytes(&self) -> usize {
        self.data.len()
    }

    fn make_new_frame(&mut self) -> &mut [u8] {
        // There are currently not frames in the dumper's buffer.
        let start = match self.index.is_empty() {
            true => 0,
            false => self.index[self.index.len() - 1].0,
        };

        let end = start + self.width * self.height * self.channels;
        self.index.push((start, end));

        &mut self.data[start..end]
    }

    fn dump_video_frame(&self, frame_number: usize) {
        let file_name = format!("video_frame_{:03}.png", frame_number); 
        let (start, end) = self.index[frame_number];

        let path = Path::new(&file_name);
        let file = File::create(path).unwrap();
        let buf_writer = BufWriter::new(file);
        let mut encoder = png::Encoder::new(buf_writer, self.width as u32, self.height as u32);
        encoder.set(png::ColorType::RGB).set(png::BitDepth::Eight);
        let mut png_writer = encoder.write_header().unwrap();

        let result = png_writer.write_image_data(&self.data[start..end]);
        if result.is_err() {
            eprintln!("ERROR: could not write video frame file {}", file_name);
        }
    }

    fn dump_video_frames(&self) {
        for frame_number in 0..self.index.len() {
            self.dump_video_frame(frame_number);
        }
    }
}

fn grab_video_frame(dumper: &mut FrameBufferDumper) {
    // Copy the frame buffer contents into into a 24-bit RGB image.
    unsafe {
        gl::ReadPixels(
            0, 0, G_GL_WIDTH as i32, G_GL_HEIGHT as i32, gl::RGB, gl::UNSIGNED_BYTE,
            dumper.make_new_frame().as_mut_ptr() as *mut GLvoid
        );
    }
}


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

    let shader_programme = create_programme_from_files(VERTEX_SHADER_FILE, FRAGMENT_SHADER_FILE);

    // input variables
    let near = 0.1;                                  // clipping plane
    let far = 100.0;                                 // clipping plane
    let fov = 67.0;                                  // convert 67 degrees to radians
    let aspect = unsafe { G_GL_WIDTH as f32 / G_GL_HEIGHT as f32 }; // aspect ratio
    let proj_mat = Mat4::perspective(fov, aspect, near, far);

    // matrix components
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
    assert!(proj_mat_location != -1);
    unsafe {
        gl::UseProgram(shader_programme);
        gl::UniformMatrix4fv(proj_mat_location, 1, gl::FALSE, proj_mat.as_ptr());
    }

    // load texture
    let mut tex: GLuint = 0;
    load_texture(TEXTURE_FILE, &mut tex);
    assert!(tex != 0);

    unsafe {
        gl::Enable(gl::CULL_FACE); // cull face
        gl::CullFace(gl::BACK);    // cull back face
        gl::FrontFace(gl::CCW);    // GL_CCW for counter clock-wise
    }

    // Initialize timers for video dumping.
    let mut dump_video = false;
    let mut video_timer = 0.0;      // time video has been recording
    let mut video_dump_timer = 0.0; // timer for next frame grab
    let frame_time = 0.04;          // 1/25 seconds of time
    let mut dumper = unsafe {
        FrameBufferDumper::new(
            G_VIDEO_SECONDS_TOTAL, G_VIDEO_FPS,
            G_GL_WIDTH as usize, G_GL_HEIGHT as usize, 3
        )
    };

    while !g_window.should_close() {
        let current_seconds = glfw.get_time();
        let elapsed_seconds = unsafe { current_seconds - PREVIOUS_SECONDS };
        unsafe {
            PREVIOUS_SECONDS = current_seconds;
        }

        if dump_video {
            // elapsed_seconds is seconds since last loop iteration
            video_timer += elapsed_seconds;
            video_dump_timer += elapsed_seconds;
            // only record 10s of video, then quit
            if video_timer > 10.0 {
                break;
            }
        }

        _update_fps_counter(&glfw, &mut g_window);
        unsafe {
            // wipe the drawing surface clear
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::Viewport(0, 0, G_GL_HEIGHT as i32, G_GL_HEIGHT as i32);

            gl::UseProgram(shader_programme);
            gl::BindVertexArray(vao);
            // draw points 0-3 from the currently bound VAO with current in-use shader
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            // update other events like input handling
        }

        glfw.poll_events();

        match g_window.get_key(Key::Space) {
            Action::Press | Action::Repeat => {
                dump_video = true;
                println!("dump_video set to true.");
            }
            _ => {}
        }

        // control keys
        let mut cam_moved = false;
        match g_window.get_key(Key::A) {
            Action::Press | Action::Repeat => {
                cam_pos[0] -= cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match g_window.get_key(Key::D) {
            Action::Press | Action::Repeat => {
                cam_pos[0] += cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match g_window.get_key(Key::Up) {
            Action::Press | Action::Repeat => {
                cam_pos[1] += cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match g_window.get_key(Key::Down) {
            Action::Press | Action::Repeat => {
                cam_pos[1] -= cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match g_window.get_key(Key::W) {
            Action::Press | Action::Repeat => {
                cam_pos[2] -= cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match g_window.get_key(Key::S) {
            Action::Press | Action::Repeat => {
                cam_pos[2] += cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match g_window.get_key(Key::Left) {
            Action::Press | Action::Repeat => {
                cam_yaw += cam_yaw_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match g_window.get_key(Key::Right) {
            Action::Press | Action::Repeat => {
                cam_yaw -= cam_yaw_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        // update view matrix
        if cam_moved {
            mat_trans = Mat4::identity().translate(&math::vec3((-cam_pos[0], -cam_pos[1], -cam_pos[2]))); // cam translation
            mat_rot = Mat4::identity().rotate_y_deg(-cam_yaw);                 //
            view_mat = mat_rot * mat_trans;
            unsafe {
                gl::UniformMatrix4fv(view_mat_location, 1, gl::FALSE, view_mat.as_ptr());
            }
        }

        if dump_video { // check if recording mode is enabled
            while video_dump_timer > frame_time {
                grab_video_frame(&mut dumper); // 25 Hz so grab a frame
                video_dump_timer -= frame_time;
            }
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

    if dump_video {
        dumper.dump_video_frames();
    }
}
