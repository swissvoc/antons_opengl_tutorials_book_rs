extern crate gl;
extern crate glfw;
extern crate chrono;

mod gl_utils;


use glfw::{Action, Context, Key};
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


const GL_LOG_FILE: &str = "gl.log";

static mut PREVIOUS_SECONDS: f64 = 0.;

// Keep track of window size for things like the viewport and the mouse cursor
const G_GL_WIDTH_DEFAULT: u32 = 640;
const G_GL_HEIGHT_DEFAULT: u32 = 480;

static mut G_GL_WIDTH: u32 = 640;
static mut G_GL_HEIGHT: u32 = 480;


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
    gl_utils::gl_log_err(&format!("GLFW ERROR: code {} msg: {}", error, description));
    error_count.set(error_count.get() + 1);
}

fn main() {
    let points: [GLfloat; 9] = [
        0.0,  0.5, 0.0, 0.5, -0.5, 0.0, -0.5, -0.5, 0.0
    ];

    // Start a GL context and OS window using the GLFW helper library.
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    gl_utils::restart_gl_log();
    // Start GL context and O/S window using the GLFW helper library.
    gl_utils::gl_log(&format!("Starting GLFW\n{}\n", glfw::get_version_string()));
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
    let renderer = gl_utils::glubyte_ptr_to_string(unsafe { gl::GetString(gl::RENDERER) });
    let version = gl_utils::glubyte_ptr_to_string(unsafe { gl::GetString(gl::VERSION) });
    println!("Renderer: {}", renderer);
    println!("OpenGL version supported: {}", version);
    gl_utils::gl_log(&format!("renderer: {}\nversion: {}\n", renderer, version));
    gl_utils::log_gl_params();

    unsafe {
        // Tell GL to only draw onto a pixel if the shape is closer to the viewer.
        // Enable depth-testing.
        gl::Enable(gl::DEPTH_TEST);
        // Depth-testing interprets a smaller value as "closer".
        gl::DepthFunc(gl::LESS);

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

        let vs: GLuint = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(vs, 1, &(vertex_shader.as_ptr() as *const GLchar), ptr::null());
        gl::CompileShader(vs);

        let fs: GLuint = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(fs, 1, &(fragment_shader.as_ptr() as *const GLchar), ptr::null());
        gl::CompileShader(fs);

        let shader_programme: GLuint = gl::CreateProgram();
        gl::AttachShader(shader_programme, vs);
        gl::AttachShader(shader_programme, fs);
        gl::LinkProgram(shader_programme);

        let mut programme_info_log_len = 0;
        let mut programme_info_log = vec![0; 1024];
        gl::GetProgramInfoLog(
            shader_programme, 
            programme_info_log.capacity() as i32,
            &mut programme_info_log_len,
            programme_info_log.as_mut_ptr()
        );
        println!("SHADER PROGRAM LOG:");
        for i in 0..programme_info_log_len as usize {
            print!("{}", programme_info_log[i] as u8 as char);
        }
        println!("END SHADER PROGRAM LOG.");

        PREVIOUS_SECONDS = glfw.get_time();
        while !window.should_close() {
            gl_utils::_update_fps_counter(&mut glfw, &mut window);
            // Wipe the drawing surface clear.
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Viewport(0, 0, G_GL_WIDTH as GLint, G_GL_HEIGHT as GLint);

            gl::UseProgram(shader_programme);
            gl::BindVertexArray(vao);
            // Draw points 0-3 from the currently bound VAO with current in-use shader.
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            // Update other events like input handling.
            glfw.poll_events();
            for (time, event) in glfw::flush_messages(&events) {
                match event {
                    glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                        window.set_should_close(true);
                    }
                    _ => {

                    }
                }
            }

            // Put the stuff we've been drawing onto the display.
            window.swap_buffers();
        }
    }
}

