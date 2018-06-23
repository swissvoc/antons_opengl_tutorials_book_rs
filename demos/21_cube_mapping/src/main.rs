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
use gl::types::{GLenum, GLfloat, GLsizeiptr, GLvoid, GLuint};

use std::mem;
use std::ptr;
use std::process;

use stb_image::image;
use stb_image::image::LoadResult;

use gl_utils::*;

use graphics_math as math;
use math::{Vec3, Mat4, Versor};


const GL_LOG_FILE: &str = "gl.log";
const MESH_FILE: &str = "src/suzanne.obj";

/* choose pure reflection or pure refraction here. */
const MONKEY_VERT_FILE: &str = "src/reflect_vs.glsl";
const MONKEY_FRAG_FILE: &str = "src/reflect_fs.glsl";
//const MONKEY_VERT_FILE: &str = "refract_vs.glsl";
//const MONKEY_FRAG_FILE: &str = "refract_fs.glsl";

const CUBE_VERT_FILE: &str = "src/cube_vs.glsl";
const CUBE_FRAG_FILE: &str = "src/cube_fs.glsl";
const FRONT: &str = "src/negz.jpg";
const BACK: &str = "src/posz.jpg";
const TOP: &str = "src/posy.jpg";
const BOTTOM: &str = "src/negy.jpg";
const LEFT: &str = "src/negx.jpg";
const RIGHT: &str = "src/posx.jpg";

/* big cube. returns Vertex Array Object */
fn make_big_cube() -> GLuint {
    let points: [GLfloat; 108] = [
        -10.0,  10.0, -10.0, -10.0, -10.0, -10.0,  10.0, -10.0, -10.0,
         10.0, -10.0, -10.0,  10.0,  10.0, -10.0, -10.0,  10.0, -10.0,

        -10.0, -10.0,  10.0, -10.0, -10.0, -10.0, -10.0,  10.0, -10.0,
        -10.0,  10.0, -10.0, -10.0,  10.0,  10.0, -10.0, -10.0,  10.0,

         10.0, -10.0, -10.0,  10.0, -10.0,  10.0,  10.0,  10.0,  10.0,
         10.0,  10.0,  10.0,  10.0,  10.0, -10.0,  10.0, -10.0, -10.0,

        -10.0, -10.0,  10.0, -10.0,  10.0,  10.0,  10.0,  10.0,  10.0,
         10.0,  10.0,  10.0,  10.0, -10.0,  10.0, -10.0, -10.0,  10.0,

        -10.0,  10.0, -10.0,  10.0,  10.0, -10.0,  10.0,  10.0,  10.0,
         10.0,  10.0,  10.0, -10.0,  10.0,  10.0, -10.0,  10.0, -10.0,

        -10.0, -10.0, -10.0, -10.0, -10.0,  10.0,  10.0, -10.0, -10.0,
         10.0, -10.0, -10.0, -10.0, -10.0,  10.0,  10.0, -10.0,  10.0
    ];

    let mut vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, (3 * 36 * mem::size_of::<GLfloat>()) as GLsizeiptr,
            points.as_ptr() as *const GLvoid, gl::STATIC_DRAW
        );
    }

    let mut vao = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
        gl::EnableVertexAttribArray(0);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
    }

    vao
}

/* use stb_image to load an image file into memory, and then into one side of
a cube-map texture. */
fn load_cube_map_side(texture: GLuint, side_target: GLenum, file_name: &str) -> bool {
    unsafe {
        gl::BindTexture(gl::TEXTURE_CUBE_MAP, texture);
    }

    let force_channels = 4;
    let image_data = match image::load_with_depth(file_name, force_channels, false) {
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

    // copy image data into 'target' side of cube map
    unsafe {
        gl::TexImage2D(
            side_target, 0, gl::RGBA as i32, width as i32, height as i32, 0, 
            gl::RGBA, gl::UNSIGNED_BYTE,
            image_data.data.as_ptr() as *const GLvoid
        );
    }

    true
}

/* load all 6 sides of the cube-map from images, then apply formatting to the
final texture */
fn create_cube_map(
    front: &str, back: &str, top: &str,
    bottom: &str, left: &str, right: &str, tex_cube: &mut GLuint) {
    
    // generate a cube-map texture to hold all the sides
    unsafe {
        gl::ActiveTexture(gl::TEXTURE0);
        gl::GenTextures(1, tex_cube);
    }

    // load each image and copy into a side of the cube-map texture
    load_cube_map_side(*tex_cube, gl::TEXTURE_CUBE_MAP_NEGATIVE_Z, front);
    load_cube_map_side(*tex_cube, gl::TEXTURE_CUBE_MAP_POSITIVE_Z, back);
    load_cube_map_side(*tex_cube, gl::TEXTURE_CUBE_MAP_POSITIVE_Y, top);
    load_cube_map_side(*tex_cube, gl::TEXTURE_CUBE_MAP_NEGATIVE_Y, bottom);
    load_cube_map_side(*tex_cube, gl::TEXTURE_CUBE_MAP_NEGATIVE_X, left);
    load_cube_map_side(*tex_cube, gl::TEXTURE_CUBE_MAP_POSITIVE_X, right);
    
    // format cube map texture
    unsafe {
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
    }
}

#[allow(non_snake_case)]
fn main() {
    /*--------------------------------START OPENGL--------------------------------*/
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

    /*---------------------------------CUBE MAP-----------------------------------*/
    let cube_vao = make_big_cube();
    assert!(cube_vao > 0);

    let mut cube_map_texture = 0;
    create_cube_map(FRONT, BACK, TOP, BOTTOM, LEFT, RIGHT, &mut cube_map_texture);
    assert!(cube_map_texture > 0);

    /*------------------------------CREATE GEOMETRY------------------------------*/
    let mesh = match obj_parser::load_obj_file(MESH_FILE) {
        Ok(val) => val,
        Err(e) => {
            logger.log_err(&format!("ERROR: loading mesh file. Loader returned error\n{}", e));
            process::exit(1);
        }
    };

    let g_vp = mesh.points;
    let g_vn = mesh.normals;
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
            g_vp.as_ptr() as *const GLvoid, gl::STATIC_DRAW
        );
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::EnableVertexAttribArray(0);
    }
    assert!(points_vbo > 0);

    let mut normals_vbo = 0;
    unsafe {
        gl::GenBuffers( 1, &mut normals_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, normals_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, (3 * g_point_count * mem::size_of::<GLfloat>()) as GLsizeiptr, 
            g_vn.as_ptr() as *const GLvoid, gl::STATIC_DRAW
        );
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::EnableVertexAttribArray(1);
    }
    assert!(normals_vbo > 0);

    /*-------------------------------CREATE SHADERS-------------------------------*/
    let monkey_sp = create_programme_from_files(&logger, MONKEY_VERT_FILE, MONKEY_FRAG_FILE);
    assert!(monkey_sp > 0);

    let monkey_M_location = unsafe {
        gl::GetUniformLocation(monkey_sp, "M".as_ptr() as *const i8)
    };
    //assert!(monkey_M_location > -1);
    let monkey_V_location = unsafe { 
        gl::GetUniformLocation(monkey_sp, "V".as_ptr() as *const i8)
    };
    assert!(monkey_V_location > -1);
    let monkey_P_location = unsafe { 
        gl::GetUniformLocation(monkey_sp, "P".as_ptr() as *const i8)
    };
    assert!(monkey_P_location > -1);

    // cube-map shaders
    let cube_sp = create_programme_from_files(&logger, CUBE_VERT_FILE, CUBE_FRAG_FILE);
    assert!(cube_sp > 0);
    // note that this view matrix should NOT contain camera translation.
    let cube_V_location = unsafe {
        gl::GetUniformLocation(cube_sp, "V".as_ptr() as *const i8)
    };
    assert!(cube_V_location > -1);
    let cube_P_location = unsafe {
        gl::GetUniformLocation(cube_sp, "P".as_ptr() as *const i8)
    };
    assert!(cube_P_location > -1);


    /*-------------------------------CREATE CAMERA--------------------------------*/
    // input variables
    let near = 0.1;                                  // clipping plane
    let far = 100.0;                                 // clipping plane
    let fov = 67.0;                                  // convert 67 degrees to radians
    let aspect = context.width as f32 / context.height as f32; // aspect ratio
    let proj_mat = Mat4::perspective(fov, aspect, near, far);

    // matrix components
    let cam_speed: GLfloat = 3.0;             // 1 unit per second
    let cam_heading_speed: GLfloat = 50.0;        // 30 degrees per second
    let mut cam_pos: Vec3 = math::vec3((0.0, 0.0, 5.0)); // don't start at zero, or we will be too close
    let mut cam_heading: GLfloat = 0.0;               // y-rotation in degrees
    let mut mat_trans = Mat4::identity().translate(&math::vec3((-cam_pos.v[0], -cam_pos.v[1], -cam_pos.v[2])));
    let mut mat_rot = Mat4::identity().rotate_y_deg(-cam_heading);
    let mut q = Versor::from_axis_deg(-cam_heading, 0.0, 1.0, 0.0);
    let mut view_mat = mat_rot * mat_trans;

    let mut fwd = math::vec4((0.0, 0.0, -1.0, 0.0));
    let mut rgt = math::vec4((1.0, 0.0, 0.0, 0.0));
    let mut up = math::vec4((0.0, 1.0, 0.0, 0.0));

    /*---------------------------SET RENDERING DEFAULTS---------------------------*/
    unsafe {
        gl::UseProgram(monkey_sp);
        gl::UniformMatrix4fv(monkey_V_location, 1, gl::FALSE, view_mat.as_ptr());
        gl::UniformMatrix4fv(monkey_P_location, 1, gl::FALSE, proj_mat.as_ptr());
        gl::UseProgram(cube_sp);
        gl::UniformMatrix4fv(cube_V_location, 1, gl::FALSE, mat_rot.as_ptr());
        gl::UniformMatrix4fv(cube_P_location, 1, gl::FALSE, proj_mat.as_ptr());
    }

    // unique model matrix for each sphere
    let mut model_mat = Mat4::identity();

    unsafe {
        gl::Enable(gl::DEPTH_TEST); // enable depth-testing
        gl::DepthFunc(gl::LESS);      // depth-testing interprets a smaller value as "closer"
        gl::Enable(gl::CULL_FACE);   // cull face
        gl::CullFace(gl::BACK);       // cull back face
        gl::FrontFace(gl::CCW); // set counter-clock-wise vertex order to mean the front
        gl::ClearColor(0.2, 0.2, 0.2, 1.0); // grey background to help spot mistakes
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
            // Wipe the drawing surface clear.
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            
            // render a sky-box using the cube-map texture
            gl::DepthMask(gl::FALSE);
            gl::UseProgram(cube_sp);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, cube_map_texture);
            gl::BindVertexArray(cube_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
            gl::DepthMask(gl::TRUE);

            gl::UseProgram(monkey_sp);
            gl::BindVertexArray(vao);
            gl::UniformMatrix4fv(monkey_M_location, 1, gl::FALSE, model_mat.as_ptr());
            gl::DrawArrays(gl::TRIANGLES, 0, g_point_count as i32);
            // update other events like input handling
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
            cam_heading += cam_yaw;

            // re-calculate local axes so can move fwd in dir cam is pointing
            mat_rot = q.to_mat4();
            fwd = mat_rot * math::vec4((0.0, 0.0, -1.0, 0.0));
            rgt = mat_rot * math::vec4((1.0, 0.0,  0.0, 0.0));
            up  = mat_rot * math::vec4((0.0, 1.0,  0.0, 0.0));

            cam_pos = cam_pos + math::vec3(&fwd) * (-move_to.v[2]);
            cam_pos = cam_pos + math::vec3(&up) * (move_to.v[1]);
            cam_pos = cam_pos + math::vec3(&rgt) * (move_to.v[0]);
            mat_trans = Mat4::identity().translate(&math::vec3(cam_pos));

            view_mat = mat_rot.inverse() * mat_trans.inverse();
            unsafe {
                gl::UseProgram( monkey_sp );
                gl::UniformMatrix4fv( monkey_V_location, 1, gl::FALSE, view_mat.as_ptr());

                // cube-map view matrix has rotation, but not translation
                gl::UseProgram(cube_sp);
                gl::UniformMatrix4fv(cube_V_location, 1, gl::FALSE, mat_rot.inverse().as_ptr());
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
