extern crate gl;
extern crate glfw;
extern crate chrono;


use glfw::Context;
use gl::types::{GLubyte, GLfloat, GLuint, GLsizei, GLsizeiptr, GLchar, GLvoid, GLint, GLenum};
use chrono::prelude::Utc;

use std::string::String;
use std::ffi::CStr;
use std::mem;
use std::ptr;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::cell::Cell;


const GL_LOG_FILE: &str = "gl.log";


static previous_seconds: f64 = 0.;


#[inline]
fn glubyte_ptr_to_string(cstr: *const GLubyte) -> String {
    unsafe {
        CStr::from_ptr(cstr as *const i8).to_string_lossy().into_owned()
    }
}

/// Start a new log file with the time and date at the top.
fn restart_gl_log() -> bool {
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
fn gl_log(message: &str) -> bool {
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
fn gl_log_err(message: &str) -> bool {
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

/* we will tell GLFW to run this function whenever it finds an error */
fn glfw_error_callback(error: glfw::Error, description: String, error_count: &Cell<usize>) {
    gl_log_err(&format!("GLFW ERROR: code {} msg: {}", error, description));
    error_count.set(error_count.get() + 1);
}

// Keep track of window size for things like the viewport and the mouse cursor
static mut g_gl_width: usize = 640;
static mut g_gl_height: usize = 480;

// We will tell GLFW to run this function whenever the framebuffer size is changed.
fn glfw_framebuffer_size_callback(window: &glfw::Window, width: usize, height: usize) {
    unsafe {
        g_gl_width = width;
        g_gl_height = height;
    }
    println!("width {} height {}", width, height);
    /* Update any perspective matrices used here */
}

// We can use a function like this to print some GL capabilities of our adapter
// to the log file. This is handy if we want to debug problems on other people's computers.
fn log_gl_params() {
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

fn main() {
    // Start a GL context and OS window using the GLFW helper library.
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    // uncomment these lines if on Apple OS X.
    // glfwWindowHint (GLFW_CONTEXT_VERSION_MAJOR, 3);
    // glfwWindowHint (GLFW_CONTEXT_VERSION_MINOR, 2);
    // glfwWindowHint (GLFW_OPENGL_FORWARD_COMPAT, GL_TRUE);
    // glfwWindowHint (GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);

    let (mut window, _) = glfw.create_window(640, 480, "Hello Triangle", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.make_current();

    // Load the OpenGl function pointers.
    gl::load_with(|symbol| { window.get_proc_address(symbol) as *const _ });

    // Get renderer and version info.
    let renderer = glubyte_ptr_to_string(
        unsafe { gl::GetString(gl::RENDERER) }
    );
    let version = glubyte_ptr_to_string(
        unsafe { gl::GetString(gl::VERSION) }
    );

    println!("Renderer: {}", renderer);
    println!("OpenGL version supported: {}", version);

    // Tell GL to only draw onto a pixel if the shape is closer to the viewer.
    unsafe {
        // Enable depth-testing.
        gl::Enable(gl::DEPTH_TEST);
        // Depth-testing interprets a smaller value as "closer".
        gl::DepthFunc(gl::LESS);

        /* OTHER STUFF GOES HERE NEXT */
        let points: [GLfloat; 9] = [
             0.0,  0.5, 0.0, 0.5, -0.5, 0.0, -0.5, -0.5, 0.0
        ];

        let mut vbo: GLuint = 0;
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, (mem::size_of::<GLfloat>() * points.len()) as GLsizeiptr, 
            points.as_ptr() as *const GLvoid, gl::STATIC_DRAW
        );

        let mut vao: GLuint = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
        gl::EnableVertexAttribArray(0);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());

        let vertex_shader: &str = "
            #version 460

            in vec3 vp;

            void main () {
                gl_Position = vec4 (vp, 1.0);
            }
        ";

        let fragment_shader: &str = "
            #version 460

            out vec4 frag_colour;

            void main() {
                frag_colour = vec4 (0.5, 0.0, 0.5, 1.0);
            }
        ";

        let mut shader_info_log_len = 0;
        let mut shader_info_log = vec![0; 1024];

        let vs: GLuint = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(vs, 1, &(vertex_shader.as_ptr() as *const GLchar), ptr::null());
        gl::CompileShader(vs);

        gl::GetShaderInfoLog(
            vs, 
            shader_info_log.capacity() as GLsizei, 
            &mut shader_info_log_len, 
            shader_info_log.as_mut_ptr()
        );
        println!("VERTEX SHADER LOG:");
        println!("BUFFER LENGTH: {}", shader_info_log_len);
        for i in 0..shader_info_log_len as usize {
            print!("{}", shader_info_log[i] as u8 as char);
        }
        println!("END VERTEX SHADER LOG.");

        let fs: GLuint = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(fs, 1, &(fragment_shader.as_ptr() as *const GLchar), ptr::null());
        gl::CompileShader(fs);

        gl::GetShaderInfoLog(
            fs, 
            shader_info_log.capacity() as GLsizei, 
            &mut shader_info_log_len, 
            shader_info_log.as_mut_ptr()
        );
        println!("FRAGMENT SHADER LOG:");
        println!("BUFFER LENGTH: {}", shader_info_log_len);
        for i in 0..shader_info_log_len as usize {
            print!("{}", shader_info_log[i] as u8 as char);
        }
        println!("END FRAGMENT SHADER LOG.");

        let shader_programme: GLuint = gl::CreateProgram();
        gl::AttachShader(shader_programme, vs);
        gl::AttachShader(shader_programme, fs);
        gl::LinkProgram(shader_programme);

        let mut programme_info_log_len = 0;
        let mut programme_info_log = vec![0; 1024];
        gl::GetProgramInfoLog(
            shader_programme, 
            programme_info_log.capacity() as GLsizei,
            &mut programme_info_log_len,
            programme_info_log.as_mut_ptr()
        );
        println!("SHADER PROGRAM LOG:");
        for i in 0..programme_info_log_len as usize {
            print!("{}", programme_info_log[i] as u8 as char);
        }
        println!("END SHADER PROGRAM LOG.");

        while !window.should_close() {
            // wipe the drawing surface clear
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::UseProgram(shader_programme);
            gl::BindVertexArray(vao);
            // draw points 0-3 from the currently bound VAO with current in-use shader
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            // update other events like input handling 
            glfw.poll_events();
            // put the stuff we've been drawing onto the display
            window.swap_buffers();
        }
    }
}

