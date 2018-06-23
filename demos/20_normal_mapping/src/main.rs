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
use gl::types::{GLfloat, GLsizeiptr, GLvoid, GLuint};

use std::mem;
use std::ptr;
use std::process;

use stb_image::image;
use stb_image::image::LoadResult;

use gl_utils::*;

use graphics_math as math;
use math::Mat4;

use assimp::import as ai;


const GL_LOG_FILE: &str = "gl.log";
const VERTEX_SHADER_FILE: &str = "src/test.vert.glsl";
const FRAGMENT_SHADER_FILE: &str = "src/test.frag.glsl";
const MESH_FILE: &str = "src/suzanne.obj";
const NMAP_IMG_FILE: &str = "src/brickwork_normal-map.png";

const GL_TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FE;
const GL_MAX_TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FF;


fn calc_tangent_space() -> ai::structs::CalcTangentSpace {
    ai::structs::CalcTangentSpace {
        enable: true,
        max_smoothing_angle: 45.0,
        texture_channel: 0,
    }
}

struct AiMesh {
    vp: Vec<f32>,
    vn: Vec<f32>,
    vt: Vec<f32>,
    vtans: Vec<f32>,
    point_count: u32,
}

fn load_mesh(file_name: &str) -> Result<AiMesh, String> {
    let mut importer = ai::Importer::new();
    importer.calc_tangent_space(|calc| {});
    let scene = match importer.read_file(file_name) {
        Ok(val) => val,
        Err(_) => {
            eprintln!("ERROR: reading mesh {}", file_name);
            return Err(format!("ERROR: reading mesh {}", file_name));
        }
    };


    println!("  {} animations", scene.num_animations());
    println!("  {} cameras", scene.num_cameras());
    println!("  {} lights", scene.num_lights());
    println!("  {} materials", scene.num_materials());
    println!("  {} meshes", scene.num_meshes());
    println!("  {} textures", scene.num_textures());


    // get first mesh only
    let mesh = match scene.mesh(0) {
        Some(val) => val,
        None => {
            eprintln!("ERROR: scene \"{}\" has not meshes.", file_name);
            return Err(format!("ERROR: scene \"{}\" has not meshes.", file_name));
        }
    };
    println!("    {} vertices in mesh[0]", mesh.num_vertices());
    
    let mut g_vp: Vec<GLfloat> = vec![];
    let mut g_vn: Vec<GLfloat> = vec![];
    let mut g_vt: Vec<GLfloat> = vec![];
    let mut g_vtans: Vec<GLfloat> = vec![];
    let g_point_count = mesh.num_vertices();

    // allocate memory for vertex points
    if mesh.has_positions() {
        println!("mesh has positions");
        g_vp = vec![0.0; 3 * (g_point_count as usize) * mem::size_of::<GLfloat>()];
    }
    if mesh.has_normals() {
        println!("mesh has normals");
        g_vn = vec![0.0; 3 * (g_point_count as usize) * mem::size_of::<GLfloat>()];
    }
    if mesh.has_texture_coords(0) {
        println!("mesh has texture coords");
        g_vt = vec![0.0; 2 * (g_point_count as usize) * mem::size_of::<GLfloat>()];
    }
    if mesh.has_tangents_and_bitangents() {
        println!("mesh has tangents");
        g_vtans = vec![0.0; 4 * (g_point_count as usize) * mem::size_of::<GLfloat>()];
    }

    for v_i in 0..mesh.num_vertices() as usize {
        if mesh.has_positions() {
            let vp = mesh.get_vertex(v_i as u32).unwrap();
            g_vp[3 * v_i] = vp.x;
            g_vp[3 * v_i + 1] = vp.y;
            g_vp[3 * v_i + 2] = vp.z;
        }
        if mesh.has_normals() {
            let vn = mesh.get_normal(v_i as u32).unwrap();
            g_vn[3 * v_i] = vn.x;
            g_vn[3 * v_i + 1] = vn.y;
            g_vn[3 * v_i + 2] = vn.z;
        }
        if mesh.has_texture_coords(0) {
            let vt = mesh.get_texture_coord(0, v_i as u32).unwrap();
            g_vt[2 * v_i] = vt.x;
            g_vt[2 * v_i + 1] = vt.y;
        }
        if mesh.has_tangents_and_bitangents() {
            let tangent = mesh.get_tangent(v_i as u32).unwrap();
            let bitangent = mesh.get_bitangent(v_i as u32).unwrap();
            let normal = mesh.get_normal(v_i as u32).unwrap();

            // put the three vectors into my vec3 struct format for doing maths
            let t = math::vec3((tangent.x, tangent.y, tangent.z));
            let n = math::vec3((normal.x, normal.y, normal.z));
            let b = math::vec3((bitangent.x, bitangent.y, bitangent.z));
            // orthogonalise and normalise the tangent so we can use it in something
            // approximating a T,N,B inverse matrix
            let t_i = (t - n * n.dot(&t)).normalize();

            // get determinant of T,B,N 3x3 matrix by dot*cross method
            let mut det = (n.cross(&t)).dot(&b);
            if det < 0.0 {
                det = -1.0;
            } else {
                det = 1.0;
            }

            // push back 4d vector for inverse tangent with determinant
            g_vtans[4 * v_i] = t_i.v[0];
            g_vtans[4 * v_i + 1] = t_i.v[1];
            g_vtans[4 * v_i + 2] = t_i.v[2];
            g_vtans[4 * v_i + 3] = det;
        }
    }

    println!("mesh loaded");

    return Ok(AiMesh {
        vp: g_vp,
        vn: g_vn,
        vt: g_vt,
        vtans: g_vtans,
        point_count: g_point_count,
    });
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
    }

    /*------------------------------CREATE GEOMETRY------------------------------*/
    let mesh = match load_mesh(MESH_FILE) {
        Ok(val) => val,
        Err(e) => {
            logger.log_err(&format!("ERROR: loading mesh file. Loader returned error\n{}", e));
            process::exit(1);
        }
    };

    let g_vp = mesh.vp;
    let g_vn = mesh.vn;
    let g_vt = mesh.vt;
    let g_vtans = mesh.vtans;
    let g_point_count = mesh.point_count as usize;

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

    let mut texcoords_vbo = 0;
    unsafe {
        gl::GenBuffers( 1, &mut texcoords_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, texcoords_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, (2 * g_point_count * mem::size_of::<GLfloat>()) as GLsizeiptr, 
            g_vt.as_ptr() as *const GLvoid, gl::STATIC_DRAW
        );
        gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::EnableVertexAttribArray(2);
    }
    assert!(texcoords_vbo > 0);

    let mut tangents_vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut tangents_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, tangents_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, (4 * g_point_count * mem::size_of::<GLfloat>()) as GLsizeiptr, 
            g_vtans.as_ptr() as *const GLvoid, gl::STATIC_DRAW
        );
        gl::VertexAttribPointer(3, 4, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::EnableVertexAttribArray(3);
    }
    assert!(tangents_vbo > 0);

    let shader_programme = create_programme_from_files(&logger, VERTEX_SHADER_FILE, FRAGMENT_SHADER_FILE);

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

    let model_mat_location = unsafe {
        gl::GetUniformLocation(shader_programme, "model".as_ptr() as *const i8)
    };
    assert!(model_mat_location > -1);

    let view_mat_location = unsafe {
        gl::GetUniformLocation(shader_programme, "view".as_ptr() as *const i8)
    };
    assert!(view_mat_location > -1);

    let proj_mat_location = unsafe { 
        gl::GetUniformLocation(shader_programme, "proj".as_ptr() as *const i8)
    };
    assert!(proj_mat_location > -1);

    let model_mat = Mat4::identity();
    unsafe {
        gl::UseProgram(shader_programme);
        gl::UniformMatrix4fv(model_mat_location, 1, gl::FALSE, model_mat.as_ptr());
        gl::UniformMatrix4fv(view_mat_location, 1, gl::FALSE, view_mat.as_ptr());
        gl::UniformMatrix4fv(proj_mat_location, 1, gl::FALSE, proj_mat.as_ptr());
    }

    // load normal map image into texture
    let mut nmap_tex = 0;
    load_texture(NMAP_IMG_FILE, &mut nmap_tex);

    unsafe {
        // Cull face.
        gl::Enable(gl::CULL_FACE);
        // Cull back face.
        gl::CullFace(gl::BACK);
        // GL_CW for clockwise.    
        gl::FrontFace(gl::CCW);
    }

    while !context.window.should_close() {
        let current_seconds = context.glfw.get_time();
        let elapsed_seconds = current_seconds - context.elapsed_time_seconds;
        context.elapsed_time_seconds = current_seconds;

        update_fps_counter(&mut context);
        unsafe {
            // Wipe the drawing surface clear.
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::Viewport(0, 0, context.width as i32, context.height as i32);

            gl::UseProgram(shader_programme);
            gl::BindVertexArray(vao);

            // Update other events like input handling
            gl::DrawArrays(gl::TRIANGLES, 0, g_point_count as i32);
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
