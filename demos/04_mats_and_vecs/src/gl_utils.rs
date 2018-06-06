use glfw;
use glfw::{Action, Context, Key};
use gl;
use gl::types::{GLubyte, GLfloat, GLuint, GLsizeiptr, GLchar, GLvoid, GLint, GLenum};
use chrono::prelude::Utc;

use std::string::String;
use std::ffi::CStr;
use std::mem;
use std::ptr;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::fmt::Write as FWrite;
use std::cell::Cell;
use std::sync::mpsc::Receiver;


const GL_LOG_FILE: &str = "gl.log";

pub static mut PREVIOUS_SECONDS: f64 = 0.;

// Keep track of window size for things like the viewport and the mouse cursor
const G_GL_WIDTH_DEFAULT: u32 = 640;
const G_GL_HEIGHT_DEFAULT: u32 = 480;

pub static mut G_GL_WIDTH: u32 = 640;
pub static mut G_GL_HEIGHT: u32 = 480;


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
fn glfw_error_callback(error: glfw::Error, description: String, error_count: &Cell<usize>) {
    gl_log_err(&format!("GLFW ERROR: code {} msg: {}", error, description));
    error_count.set(error_count.get() + 1);
}

/// Start a new log file with the time and date at the top.
pub fn restart_gl_log() -> bool {
    let file = File::create(GL_LOG_FILE);
    if file.is_err() {
        eprintln!(
            "ERROR: The GL_LOG_FILE log file {} could not be opened for writing.", GL_LOG_FILE
        );

        return false;
    }

    let mut file = file.unwrap();

    let date = Utc::now();
    write!(file, "GL_LOG_FILE log. local time {}", date).unwrap();
    // TODO: Use a build script in a build.rs file to generate this.
    write!(file, "build version: ??? ?? ???? ??:??:??\n\n").unwrap();

    return true;
}

/// Add a message to the log file.
pub fn gl_log(message: &str) -> bool {
    let file = OpenOptions::new().write(true).append(true).open(GL_LOG_FILE);
    if file.is_err() {
        eprintln!("ERROR: Could not open GL_LOG_FILE {} file for appending.", GL_LOG_FILE);
        return false;
    }

    let mut file = file.unwrap();
    writeln!(file, "{}", message).unwrap();

    return true;
}

/// Same as gl_log except also prints to stderr.
pub fn gl_log_err(message: &str) -> bool {
    let file = OpenOptions::new().write(true).append(true).open(GL_LOG_FILE);
    if file.is_err() {
        eprintln!("ERROR: Could not open GL_LOG_FILE {} file for appending.", GL_LOG_FILE);
        return false;
    }

    let mut file = file.unwrap();
    writeln!(file, "{}", message).unwrap();
    eprintln!("{}", message);

    return true;
}


// We can use a function like this to print some GL capabilities of our adapter
// to the log file. This is handy if we want to debug problems on other people's computers.
pub fn log_gl_params() {
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
    gl_log("GL Context Params:\n");
    unsafe {
        // integers - only works if the order is 0-10 integer return types
        for i in 0..10 {
            let mut v = 0;
            gl::GetIntegerv(params[i], &mut v);
            gl_log(&format!("{} {}", names[i], v));
        }
        // others
        let mut v: [GLint; 2] = [0; 2];
        gl::GetIntegerv(params[10], &mut v[0]);
        gl_log(&format!("{} {} {}\n", names[10], v[0], v[1]));
        let mut s = 0;
        gl::GetBooleanv(params[11], &mut s);
        gl_log(&format!("{} {}", names[11], s as usize));
        gl_log("-----------------------------");
    }
}

pub fn start_gl() -> Result<(glfw::Glfw, glfw::Window, Receiver<(f64, glfw::WindowEvent)>), String> {
    // Start a GL context and OS window using the GLFW helper library.
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    restart_gl_log();
    // Start GL context and O/S window using the GLFW helper library.
    gl_log(&format!("Starting GLFW\n{}\n", glfw::get_version_string()));
    // register the error call-back function that we wrote, above
    glfw.set_error_callback(Some(
        glfw::Callback { 
            f: glfw_error_callback,
            data: Cell::new(0),
        }
    ));

    // uncomment these lines if on Mac OS X.
    // glfwWindowHint (GLFW_CONTEXT_VERSION_MAJOR, 3);
    // glfwWindowHint (GLFW_CONTEXT_VERSION_MINOR, 2);
    // glfwWindowHint (GLFW_OPENGL_FORWARD_COMPAT, GL_TRUE);
    // glfwWindowHint (GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);

    // Set anti-aliasing factor to make diagonal edges appear less jagged.
    glfw.window_hint(glfw::WindowHint::Samples(Some(4)));

    let (mut window, events) = glfw.create_window(
        G_GL_WIDTH_DEFAULT, G_GL_HEIGHT_DEFAULT, "Extended Init.", glfw::WindowMode::Windowed
    )
    .expect("Failed to create GLFW window.");
    //glfw::ffi::glfwSetWindowSizeCallback(&mut window, Some(glfw_framebuffer_size_callback));

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
    gl_log(&format!("renderer: {}\nversion: {}\n", renderer, version));
    log_gl_params();

    Ok((glfw, window, events))
}

// We will use this function to update the window title with a frame rate.
pub fn _update_fps_counter(glfw: &glfw::Glfw, window: &mut glfw::Window) {
    let mut tmp: String = String::new();

    static mut FRAME_COUNT: usize = 0;

    let current_seconds = glfw.get_time();
    unsafe {
        let elapsed_seconds = current_seconds - PREVIOUS_SECONDS;
        if elapsed_seconds > 0.25 {
            PREVIOUS_SECONDS = current_seconds;

            let fps = FRAME_COUNT as f64 / elapsed_seconds;
            write!(&mut tmp, "OpenGL @ fps: {:.2}", fps).unwrap();
            window.set_title(&tmp);
            FRAME_COUNT = 0;
        }

        FRAME_COUNT += 1;
    }
}

