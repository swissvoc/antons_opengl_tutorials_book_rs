use glfw;
use glfw::Context;
use gl;
use gl::types::{GLubyte, GLuint, GLchar, GLint, GLenum};

use logger::Logger;

use std::string::String;
use std::ffi::CStr;
use std::ptr;
use std::fs::File;
use std::io::{Read, Write, BufReader};
use std::fmt::Write as FWrite;
use std::cell::Cell;
use std::sync::mpsc::Receiver;


const MAX_SHADER_LENGTH: usize = 262144;

// Keep track of window size for things like the viewport and the mouse cursor
const G_GL_WIDTH_DEFAULT: u32 = 640;
const G_GL_HEIGHT_DEFAULT: u32 = 480;

pub static mut G_GL_WIDTH: u32 = 640;
pub static mut G_GL_HEIGHT: u32 = 480;
pub static mut G_GL_CHANNEL_DEPTH: u32 = 3;

static mut PREVIOUS_SECONDS: f64 = 0.0;
static mut FRAME_COUNT: usize = 0;

#[inline]
pub fn glubyte_ptr_to_string(cstr: *const GLubyte) -> String {
    unsafe {
        CStr::from_ptr(cstr as *const i8).to_string_lossy().into_owned()
    }
}

// We will tell GLFW to run this function whenever the framebuffer size is changed.
fn glfw_framebuffer_size_callback(window: &mut glfw::Window, width: u32, height: u32) {
    unsafe {
        G_GL_WIDTH = width;
        G_GL_HEIGHT = height;
    }
    println!("width {} height {}", width, height);
    /* Update any perspective matrices used here */
}

/* we will tell GLFW to run this function whenever it finds an error */
fn glfw_error_callback(logger: &Logger, error: glfw::Error, description: String, error_count: &Cell<usize>) {
    logger.log_err(&format!("GLFW ERROR: code {} msg: {}", error, description));
    error_count.set(error_count.get() + 1);
}


pub fn restart_gl_log(log_file: &str) -> Logger {
    Logger::from_log_file(log_file)
}


// We can use a function like this to print some GL capabilities of our adapter
// to the log file. This is handy if we want to debug problems on other people's computers.
pub fn log_gl_params(logger: &Logger) {
    let params: [GLenum; 12] = [
        gl::MAX_COMBINED_TEXTURE_IMAGE_UNITS,
        gl::MAX_CUBE_MAP_TEXTURE_SIZE,
        gl::MAX_DRAW_BUFFERS,
        gl::MAX_FRAGMENT_UNIFORM_COMPONENTS,
        gl::MAX_TEXTURE_IMAGE_UNITS,
        gl::MAX_TEXTURE_SIZE,
        gl::MAX_VARYING_FLOATS,
        gl::MAX_VERTEX_ATTRIBS,
        gl::MAX_VERTEX_TEXTURE_IMAGE_UNITS,
        gl::MAX_VERTEX_UNIFORM_COMPONENTS,
        gl::MAX_VIEWPORT_DIMS,
        gl::STEREO,
    ];
    let names: [&str; 12] = [
        "GL_MAX_COMBINED_TEXTURE_IMAGE_UNITS",
        "GL_MAX_CUBE_MAP_TEXTURE_SIZE",
        "GL_MAX_DRAW_BUFFERS",
        "GL_MAX_FRAGMENT_UNIFORM_COMPONENTS",
        "GL_MAX_TEXTURE_IMAGE_UNITS",
        "GL_MAX_TEXTURE_SIZE",
        "GL_MAX_VARYING_FLOATS",
        "GL_MAX_VERTEX_ATTRIBS",
        "GL_MAX_VERTEX_TEXTURE_IMAGE_UNITS",
        "GL_MAX_VERTEX_UNIFORM_COMPONENTS",
        "GL_MAX_VIEWPORT_DIMS",
        "GL_STEREO",
    ];
    logger.log("GL Context Params:\n");
    unsafe {
        // integers - only works if the order is 0-10 integer return types
        for i in 0..10 {
            let mut v = 0;
            gl::GetIntegerv(params[i], &mut v);
            logger.log(&format!("{} {}", names[i], v));
        }
        // others
        let mut v: [GLint; 2] = [0; 2];
        gl::GetIntegerv(params[10], &mut v[0]);
        logger.log(&format!("{} {} {}\n", names[10], v[0], v[1]));
        let mut s = 0;
        gl::GetBooleanv(params[11], &mut s);
        logger.log(&format!("{} {}", names[11], s as usize));
        logger.log("-----------------------------");
    }
}

pub fn start_gl(logger: &Logger) -> Result<(glfw::Glfw, glfw::Window, Receiver<(f64, glfw::WindowEvent)>), String> {
    // Start a GL context and OS window using the GLFW helper library.
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    logger.restart();
    // Start GL context and O/S window using the GLFW helper library.
    logger.log(&format!("Starting GLFW\n{}\n", glfw::get_version_string()));

    // uncomment these lines if on Mac OS X.
    // glfwWindowHint (GLFW_CONTEXT_VERSION_MAJOR, 3);
    // glfwWindowHint (GLFW_CONTEXT_VERSION_MINOR, 2);
    // glfwWindowHint (GLFW_OPENGL_FORWARD_COMPAT, GL_TRUE);
    // glfwWindowHint (GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);

    // Set anti-aliasing factor to make diagonal edges appear less jagged.
    glfw.window_hint(glfw::WindowHint::Samples(Some(4)));

    let (mut window, events) = glfw.create_window(
        G_GL_WIDTH_DEFAULT, G_GL_HEIGHT_DEFAULT, "Vectors And Matrices", glfw::WindowMode::Windowed
    )
    .expect("Failed to create GLFW window.");

    window.make_current();
    window.set_key_polling(true);
    window.set_size_polling(true);
    window.set_refresh_polling(true);
    window.set_size_polling(true);

    // Load the OpenGl function pointers.
    gl::load_with(|symbol| { window.get_proc_address(symbol) as *const _ });

    // Get renderer and version info.
    let renderer = glubyte_ptr_to_string(unsafe { gl::GetString(gl::RENDERER) });
    let version = glubyte_ptr_to_string(unsafe { gl::GetString(gl::VERSION) });
    println!("Renderer: {}", renderer);
    println!("OpenGL version supported: {}", version);
    logger.log(&format!("renderer: {}\nversion: {}\n", renderer, version));
    log_gl_params(logger);

    Ok((glfw, window, events))
}

// We will use this function to update the window title with a frame rate.
pub fn _update_fps_counter(glfw: &glfw::Glfw, window: &mut glfw::Window) {
    unsafe {        
        let current_seconds = glfw.get_time();
        let elapsed_seconds = current_seconds - PREVIOUS_SECONDS;
        if elapsed_seconds > 0.25 {
            PREVIOUS_SECONDS = current_seconds;
            let fps = FRAME_COUNT as f64 / elapsed_seconds;
            let mut title: String = String::new();
            write!(&mut title, "OpenGL @ FPS: {:.2}", fps).unwrap();
            window.set_title(&title);
            FRAME_COUNT = 0;
        }

        FRAME_COUNT += 1;
    }
}

pub fn gl_type_to_string(gl_type: GLenum) -> &'static str {
    match gl_type {
        gl::BOOL => "bool",
        gl::INT => "int",
        gl::FLOAT => "float",
        gl::FLOAT_VEC2 => "vec2",
        gl::FLOAT_VEC3 => "vec3",
        gl::FLOAT_VEC4 => "vec4",
        gl::FLOAT_MAT2 => "mat2",
        gl::FLOAT_MAT3 => "mat3",
        gl::FLOAT_MAT4 => "mat4",
        gl::SAMPLER_2D => "sampler2D",
        gl::SAMPLER_3D => "sampler3D",
        gl::SAMPLER_CUBE => "samplerCube",
        gl::SAMPLER_2D_SHADOW => "sampler2DShadow",
        _ => "other"
    }
}

pub fn parse_file_into_str(logger: &Logger, file_name: &str, shader_str: &mut [u8], max_len: usize) -> bool {
    shader_str[0] = 0;
    let file = File::open(file_name);
    if file.is_err() {
        logger.log_err(&format!("ERROR: opening file for reading: {}\n", file_name));
        return false;
    }

    let file = file.unwrap();
    let mut reader = BufReader::new(file);

    let bytes_read = reader.read(shader_str);
    if bytes_read.is_err() {
        logger.log_err(&format!("ERROR: reading shader file {}\n", file_name));
        return false;
    }

    let bytes_read = bytes_read.unwrap();
    if bytes_read >= (max_len - 1) {
        logger.log_err(&format!("WARNING: file {} too big - truncated.\n", file_name));
    }

    // append \0 to end of file string.
    shader_str[bytes_read] = 0;

    return true;
}

fn create_shader(logger: &Logger, file_name: &str, shader: &mut GLuint, gl_type: GLenum) -> bool {
    logger.log(&format!("Creating shader from {}...\n", file_name));

    let mut shader_string = vec![0; MAX_SHADER_LENGTH];
    parse_file_into_str(logger, file_name, &mut shader_string, MAX_SHADER_LENGTH);

    *shader = unsafe { gl::CreateShader(gl_type) };
    let p = shader_string.as_ptr() as *const GLchar;
    
    unsafe {
        gl::ShaderSource(*shader, 1, &p, ptr::null());
        gl::CompileShader(*shader);
    }
    // Check for compile errors.
    let mut params = -1;
    unsafe {
        gl::GetShaderiv(*shader, gl::COMPILE_STATUS, &mut params);
    }

    if params != gl::TRUE as i32 {
        logger.log_err(&format!("ERROR: GL shader index {} did not compile\n", *shader));
        print_shader_info_log(*shader);
        
        return false;
    }
    logger.log(&format!("Shader compiled with index {}\n", *shader));
    
    return true;
}

/* print errors in shader compilation */
pub fn print_shader_info_log(shader_index: GLuint) {
    let max_length = 2048;
    let mut actual_length = 0;
    let mut log = [0; 2048];
    
    unsafe {
        gl::GetShaderInfoLog(shader_index, max_length, &mut actual_length, &mut log[0]);
    }
    
    println!("Shader info log for GL index {}:", shader_index);
    for i in 0..actual_length as usize {
        print!("{}", log[i] as u8 as char);
    }
    println!();
}

/* print errors in shader linking */
pub fn print_programme_info_log(sp: GLuint) {
    let max_length = 2048;
    let mut actual_length = 0;
    let mut log = [0 as i8; 2048];
    
    unsafe {
        gl::GetProgramInfoLog(sp, max_length, &mut actual_length, &mut log[0]);
    }
    
    println!("Program info log for GL index {}:", sp);
    for i in 0..actual_length as usize {
        print!("{}", log[i] as u8 as char);
    }
    println!();
}

/* validate shader */
pub fn is_programme_valid(logger: &Logger, sp: GLuint) -> bool {
    let mut params = -1;
    unsafe {
        gl::ValidateProgram(sp);
        gl::GetProgramiv(sp, gl::VALIDATE_STATUS, &mut params);
    }

    if gl::TRUE as i32 != params {
        logger.log_err(&format!("Program {} GL_VALIDATE_STATUS = GL_FALSE\n", sp));
        print_programme_info_log(sp);
        return false;
    }

    logger.log(&format!("Program {} GL_VALIDATE_STATUS = {}\n", sp, params));
    
    return true;
}

pub fn create_programme(logger: &Logger, vertex_shader: GLuint, fragment_shader: GLuint, programme: &mut GLuint) -> bool {
    unsafe {
        *programme = gl::CreateProgram();
        logger.log(&format!(
            "Created programme {}. attaching shaders {} and {}...\n", 
            programme, vertex_shader, fragment_shader)
        );
        gl::AttachShader(*programme, vertex_shader);
        gl::AttachShader(*programme, fragment_shader);

        // Link the shader programme. If binding input attributes do that before linking.
        gl::LinkProgram( *programme );
        let mut params = -1;
        gl::GetProgramiv(*programme, gl::LINK_STATUS, &mut params);
        if params != gl::TRUE as i32 {
            logger.log_err(&format!(
                "ERROR: could not link shader programme GL index {}\n", *programme)
            );
            print_programme_info_log(*programme);
        
            return false;
        }
        is_programme_valid(logger, *programme);
        // Delete shaders here to free memory
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);
        return true;
    }
}

pub fn create_programme_from_files(logger: &Logger, vert_file_name: &str, frag_file_name: &str) -> GLuint {
    let mut vertex_shader: GLuint = 0;
    let mut fragment_shader: GLuint = 0;
    let mut programme: GLuint = 0;
    
    create_shader(logger, vert_file_name, &mut vertex_shader, gl::VERTEX_SHADER);
    create_shader(logger, frag_file_name, &mut fragment_shader, gl::FRAGMENT_SHADER);
    create_programme(logger, vertex_shader, fragment_shader, &mut programme);
    
    programme
}


/* print absolutely everything about a shader - only useful if you get really
stuck wondering why a shader isn't working properly */
pub fn print_all(sp: GLuint) {
    let mut params = -1;

    unsafe {
        println!("--------------------\nshader programme {} info:", sp);
        gl::GetProgramiv(sp, gl::LINK_STATUS, &mut params);
        println!("GL_LINK_STATUS = {}", params);

        gl::GetProgramiv(sp, gl::ATTACHED_SHADERS, &mut params);
        println!("GL_ATTACHED_SHADERS = {}", params);

        gl::GetProgramiv(sp, gl::ACTIVE_ATTRIBUTES, &mut params);
        println!("GL_ACTIVE_ATTRIBUTES = {}", params);
    }

    for i in 0..params {
        let mut name = [0; 64];
        let max_length = 64;
        let mut actual_length = 0;
        let mut size = 0;
        let mut gl_type: GLenum = 0;
        unsafe {
            gl::GetActiveAttrib(sp, i as GLuint, max_length, &mut actual_length, &mut size, &mut gl_type, &mut name[0]);
        }
        if size > 1 {
            for j in 0..size {
                let mut long_name = vec![];
                //write!(long_name, "{}[{}]", name, j);
                let location = unsafe { gl::GetAttribLocation(sp, long_name.as_ptr() as *const i8) };
                println!(
                    "  {}) type:{} name:{} location:{}", 
                    i, gl_type_to_string(gl_type), long_name.iter().map(|ch| *ch as u8 as char).collect::<String>(), location
                );
            }
        } else {
            let location = unsafe { gl::GetAttribLocation(sp, &mut name[0]) };
            println!(
                "  {}) type:{} name:{} location:{}",
                i, gl_type_to_string(gl_type), name.iter().map(|ch| *ch as u8 as char).collect::<String>(), location
            );
        }
    }
    
    unsafe {
        gl::GetProgramiv(sp, gl::ACTIVE_UNIFORMS, &mut params);
    }
    println!("GL_ACTIVE_UNIFORMS = {}", params);
    for i in 0..params {
        let mut name = [0; 64];
        let max_length = 64;
        let mut actual_length = 0;
        let mut size = 0;
        let mut gl_type: GLenum = 0;
        unsafe {
            gl::GetActiveUniform(sp, i as u32, max_length, &mut actual_length, &mut size, &mut gl_type, &mut name[0]);
        }
        if size > 1 {
            for j in 0..size {
                let long_name = [0; 64];

                //write!(long_name, "{}[{}]", name, j);
                let location = unsafe { gl::GetUniformLocation(sp, long_name.as_ptr()) };
                println!(
                    "  {}) type:{} name:{} location:{}",
                    i, gl_type_to_string(gl_type), long_name.iter().map(|ch| *ch as u8 as char).collect::<String>(), location
                );
            }
        } else {
            let location = unsafe { gl::GetUniformLocation(sp, &name[0]) };
            println!(
                "  {}) type:{} name:{} location:{}", 
                i, gl_type_to_string(gl_type), name.iter().map(|ch| *ch as u8 as char).collect::<String>(), location
            );
        }
    }

    print_programme_info_log(sp);
}

