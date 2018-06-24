extern crate gl;
extern crate glfw;
extern crate chrono;
extern crate stb_image;
extern crate png;
extern crate assimp;

#[macro_use] 
extern crate scan_fmt;

mod gl_utils;
mod graphics_math;
mod obj_parser;
mod logger;


use glfw::{Action, Context, Key};
use gl::types::{GLchar, GLfloat, GLint, GLsizeiptr, GLvoid, GLuint};

use std::mem;
use std::ptr;
use std::process;

use stb_image::image;
use stb_image::image::LoadResult;

use gl_utils::*;

use graphics_math as math;
use math::{Mat4, Versor};
use logger::Logger;


const GL_LOG_FILE: &str = "gl.log";
const GP_VS_FILE: &str = "src/gp_vs.glsl";
const GP_FS_FILE: &str = "src/gp_fs.glsl";
const GUI_VS_FILE: &str = "src/gui_vs.glsl";
const GUI_FS_FILE: &str = "src/gui_fs.glsl";

const GL_TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FE;
const GL_MAX_TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FF;


struct AppState {
    g_viewport_width: u32,
    g_viewport_height: u32,

    // virtual camera view matrix
    view_mat: Mat4,
    // virtual camera projection matrix
    proj_mat: Mat4,

    gp_sp: GLuint,           // ground plane shader programme
    gp_view_mat_loc: GLint,  // view matrix location in gp_sp
    gp_proj_mat_loc: GLint,  // projection matrix location in gp_sp
    gui_sp: GLuint,          // 2d GUI panel shader programme
    gui_scale_loc: GLint,    // scale factors for gui shader   
}

fn init_app_state() -> AppState {
    AppState {
        g_viewport_width: 640,
        g_viewport_height: 480,
        view_mat: Mat4::identity(),
        proj_mat: Mat4::identity(),
        gp_sp: 0,
        gp_view_mat_loc: -1,
        gp_proj_mat_loc: -1,
        gui_sp: 0,
        gui_scale_loc: -1,
    }
}

fn create_ground_plane_shaders(logger: &Logger, app: &mut AppState) {
    // Here I used negative y from the buffer as the z value so that it was on
    // the floor but also that the 'front' was on the top side. also note how I
    // work out the texture coordinates, st, from the vertex point position.
    let mut gp_vs_str = vec![0; 1024];
    let mut gp_fs_str = vec![0; 1024];
    if !parse_file_into_str(logger, GP_VS_FILE, &mut gp_vs_str, 1024) {
        panic!("Failed to parse ground plane vertex shader file.");
    }

    if !parse_file_into_str(logger, GP_FS_FILE, &mut gp_fs_str, 1024) {
        panic!("Failed to parse ground plane fragment shader file.");
    }
    
    unsafe {
        let gp_vs = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(gp_vs, 1, &(gp_vs_str.as_ptr() as *const GLchar), ptr::null());
        gl::CompileShader(gp_vs);
        assert!(gp_vs > 0);

        let gp_fs = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(gp_fs, 1, &(gp_fs_str.as_ptr() as *const GLchar), ptr::null());
        gl::CompileShader(gp_fs);
        assert!(gp_fs > 0);

        let gp_sp = gl::CreateProgram();
        gl::AttachShader(gp_sp, gp_vs);
        gl::AttachShader(gp_sp, gp_fs);
        gl::LinkProgram(gp_sp);
        assert!(gp_sp > 0);

        // Get uniform locations of camera view and projection matrices.
        let gp_view_mat_loc = gl::GetUniformLocation(gp_sp, "view".as_ptr() as *const i8);
        assert!(gp_view_mat_loc > -1);

        let gp_proj_mat_loc = gl::GetUniformLocation(gp_sp, "proj".as_ptr() as *const i8);
        assert!(gp_proj_mat_loc > -1);

        // Set defaults for matrices
        gl::UseProgram(gp_sp);
        gl::UniformMatrix4fv(gp_view_mat_loc, 1, gl::FALSE, app.view_mat.as_ptr());
        gl::UniformMatrix4fv(gp_proj_mat_loc, 1, gl::FALSE, app.proj_mat.as_ptr());

        app.gp_sp = gp_sp;
        app.gp_view_mat_loc = gp_view_mat_loc;
        app.gp_proj_mat_loc = gp_proj_mat_loc;
    }
}

fn create_gui_shaders(logger: &Logger, app: &mut AppState) {
    // Note that I scaled down the size to 0.5 * the viewport size here.
    let mut gui_vs_str = vec![0; 1024];
    let mut gui_fs_str = vec![0; 1024];
    if parse_file_into_str(logger, GUI_VS_FILE, &mut gui_vs_str, 1024) {
        panic!("Failed to parse gui vertex shader file.");
    }

    if parse_file_into_str(logger, GUI_FS_FILE, &mut gui_fs_str, 1024) {
        panic!("Failed to parse gui fragment shader file.");
    }   

    unsafe {
        let gui_vs = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(gui_vs, 1, &(gui_vs_str.as_ptr() as *const GLchar), ptr::null());
        gl::CompileShader(gui_vs);
        assert!(gui_vs > 0);

        let gui_fs = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(gui_fs, 1, &(gui_fs_str.as_ptr() as *const GLchar), ptr::null());
        gl::CompileShader(gui_fs);
        assert!(gui_fs > 0);

        let gui_sp = gl::CreateProgram();
        gl::AttachShader(gui_sp, gui_vs);
        gl::AttachShader(gui_sp, gui_fs);
        gl::LinkProgram(gui_sp);
        assert!(gui_sp > 0);
        let gui_scale_loc = gl::GetUniformLocation(gui_sp, "gui_scale".as_ptr() as *const i8);
        assert!(gui_scale_loc > -1);

        app.gui_sp = gui_sp;
        app.gui_scale_loc = gui_scale_loc;
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

/* we will tell GLFW to run this function whenever the window is resized */
fn glfw_framebuffer_size_callback(context: &mut GLContext, app: &mut AppState, width: u32, height: u32) {
    context.width = width;
    context.height = height;
    /* update any perspective matrices used here */
    app.proj_mat = Mat4::perspective(67.0, context.width as f32 / context.height as f32, 0.1, 100.0);
    unsafe {
        gl::Viewport(0, 0, context.width as i32, context.height as i32);
    }
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

    let mut app = init_app_state();

    // create a 2d panel. from 2 triangles = 6 xy coords.
    let points: [f32; 12] = [
        -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0
    ];

    // for the ground plane we can just re-use panel points but y is now z
    let mut vp_vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut vp_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vp_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, (points.len() * mem::size_of::<GLfloat>()) as GLsizeiptr, 
            points.as_ptr() as *const GLvoid, gl::STATIC_DRAW
        );
    }
    assert!(vp_vbo > 0);

    let mut vao = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
        // NOTE: vertex buffer is already bound
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::EnableVertexAttribArray(0);
    }
    assert!(vao > 0);

    // create a 3d camera to move in 3d so that we can tell that the panel is 2d
    // keep track of some useful vectors that can be used for keyboard movement
    let mut fwd = math::vec4((0.0, 0.0, -1.0, 0.0));
    let mut rgt = math::vec4((1.0, 0.0,  0.0, 0.0));
    let mut up  = math::vec4((0.0, 1.0,  0.0, 0.0));
    let mut cam_pos = math::vec3((0.0, 1.0, 5.0));
    let mut mat_trans_inv = Mat4::identity().translate(&cam_pos);

    // point slightly downwards to see the plane
    let mut q = Versor::from_axis_deg(0.0, 1.0, 0.0, 0.0);
    let mut mat_rot_inv = q.to_mat4();
    // combine the inverse rotation and transformation to make a view matrix
    let mut view = mat_rot_inv.inverse() * mat_trans_inv.inverse();
    // projection matrix
    let mut proj = Mat4::perspective(67.0, context.width as f32 / context.height as f32, 0.1, 100.0);
    let cam_speed = 3.0;          // 1 unit per second
    let cam_heading_speed = 50.0; // 30 degrees per second

    create_ground_plane_shaders(&logger, &mut app);
    create_gui_shaders(&logger, &mut app);

    // textures for ground plane and gui
    let mut gp_tex = 0;
    load_texture("src/tile2-diamonds256x256.png", &mut gp_tex);
    assert!(gp_tex > 0);

    let mut gui_tex = 0;
    load_texture("src/skulluvmap.png", &mut gui_tex);
    assert!(gui_tex > 0);

    unsafe {
        // rendering defaults
        gl::DepthFunc(gl::LESS);   // set depth function but don't enable yet
        gl::Enable(gl::CULL_FACE); // cull face
        gl::CullFace(gl::BACK);    // cull back face
        gl::FrontFace(gl::CCW);    // GL_CCW for counter clock-wise
    }

    // absolute panel dimensions in pixels
    let panel_width = 256.0;
    let panel_height = 256.0;
    unsafe {
        gl::Viewport(0, 0, context.width as i32, context.height as i32);
    }

    /*-------------------------------RENDERING LOOP-------------------------------*/
    while !context.window.should_close() {
        // Update timers.
        let current_seconds = context.glfw.get_time();
        let elapsed_seconds = current_seconds - context.elapsed_time_seconds;
        context.elapsed_time_seconds = current_seconds;
        update_fps_counter(&mut context);

        unsafe {
            // wipe the drawing surface clear
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            // draw ground plane. note: depth test is enabled here
            gl::Enable(gl::DEPTH_TEST);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, gp_tex);
            gl::UseProgram(app.gp_sp);
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);

            // draw GUI panel. note: depth test is disabled here and drawn AFTER scene
            gl::Disable(gl::DEPTH_TEST);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, gui_tex);
            gl::UseProgram(app.gui_sp);
            // resize panel to size in pixels
            let x_scale = panel_width / (context.width as f32);
            let y_scale = panel_height / (context.height as f32);
            gl::Uniform2f(app.gui_scale_loc, x_scale, y_scale);
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }

        context.glfw.poll_events();

        // control keys
        let mut cam_moved = false;
        let mut move_to = math::vec3((0.0, 0.0, 0.0));
        let mut cam_yaw = 0.0; // y-rotation in degrees
        let mut cam_pitch = 0.0;
        let mut cam_roll = 0.0;
        match context.window.get_key(Key::A) {
            Action::Press | Action::Repeat => {
                move_to.v[0] -= cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::D) {
            Action::Press | Action::Repeat => {
                move_to.v[0] += cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::Q) {
            Action::Press | Action::Repeat => {
                move_to.v[1] += cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::E) {
            Action::Press | Action::Repeat => {
                move_to.v[1] -= cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::W) {
            Action::Press | Action::Repeat => {
                move_to.v[2] -= cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::S) {
            Action::Press | Action::Repeat => {
                move_to.v[2] += cam_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
            }
            _ => {}
        }
        match context.window.get_key(Key::Left) {
            Action::Press | Action::Repeat => {
                cam_yaw += cam_heading_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
                let q_yaw = Versor::from_axis_deg(cam_yaw, up.v[0], up.v[1], up.v[2]);
                q = q_yaw * &q;
            }
            _ => {}
        }
        match context.window.get_key(Key::Right) {
            Action::Press | Action::Repeat => {
                cam_yaw -= cam_heading_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
                let q_yaw = Versor::from_axis_deg(cam_yaw, up.v[0], up.v[1], up.v[2]);
                q = q_yaw * &q;
            }
            _ => {}
        }
        match context.window.get_key(Key::Up) {
            Action::Press | Action::Repeat => {
                cam_pitch += cam_heading_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
                let q_pitch = Versor::from_axis_deg(cam_pitch, rgt.v[0], rgt.v[1], rgt.v[2]);
                q = q_pitch * &q;
            }
            _ => {}
        }
        match context.window.get_key(Key::Down) {
            Action::Press | Action::Repeat => {
                cam_pitch -= cam_heading_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
                let q_pitch = Versor::from_axis_deg(cam_pitch, rgt.v[0], rgt.v[1], rgt.v[2]);
                q = q_pitch * &q;
            }
            _ => {}
        }
        match context.window.get_key(Key::Z) {
            Action::Press | Action::Repeat => {
                cam_roll -= cam_heading_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
                let q_roll = Versor::from_axis_deg(cam_roll, fwd.v[0], fwd.v[1], fwd.v[2]);
                q = q_roll * &q;
            }
            _ => {}
        }
        match context.window.get_key(Key::C) {
            Action::Press | Action::Repeat => {
                cam_roll += cam_heading_speed * (elapsed_seconds as GLfloat);
                cam_moved = true;
                let q_roll = Versor::from_axis_deg(cam_roll, fwd.v[0], fwd.v[1], fwd.v[2]);
                q = q_roll * &q;        
            }
            _ => {}
        }

        // update view matrix
        if cam_moved {
            // re-calculate local axes so can move fwd in dir cam is pointing
            mat_rot_inv = q.to_mat4();
            fwd = mat_rot_inv * math::vec4((0.0, 0.0, -1.0, 0.0));
            rgt = mat_rot_inv * math::vec4((1.0, 0.0,  0.0, 0.0));
            up  = mat_rot_inv * math::vec4((0.0, 1.0,  0.0, 0.0));

            cam_pos = cam_pos + math::vec3(fwd) * -move_to.v[2];
            cam_pos = cam_pos + math::vec3(up)  *  move_to.v[1];
            cam_pos = cam_pos + math::vec3(rgt) *  move_to.v[0];
            mat_trans_inv = Mat4::identity().translate(&cam_pos);

            view = mat_rot_inv.inverse() * mat_trans_inv.inverse();
            unsafe {
                gl::UseProgram(app.gp_sp);
                gl::UniformMatrix4fv(app.gp_view_mat_loc, 1, gl::FALSE, view.as_ptr());
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
