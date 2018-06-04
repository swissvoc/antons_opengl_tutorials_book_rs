extern crate gl;
extern crate glfw;

use glfw::{Glfw, Context};
use gl::types::{GLubyte};
use std::string::String;
use std::ffi::CStr;

fn glubyte_ptr_to_string_safe(cstr: *const GLubyte) -> String {
    unsafe {
        CStr::from_ptr(cstr as *const i8).to_string_lossy().into_owned()
    }
}

fn main() {
    // start GL context and O/S window using the GLFW helper library
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    // uncomment these lines if on Apple OS X.
    // glfwWindowHint (GLFW_CONTEXT_VERSION_MAJOR, 3);
    // glfwWindowHint (GLFW_CONTEXT_VERSION_MINOR, 2);
    // glfwWindowHint (GLFW_OPENGL_FORWARD_COMPAT, GL_TRUE);
    // glfwWindowHint (GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);

    let (mut window, events) = glfw.create_window(640, 480, "Hello Triangle", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.make_current();

    // Load the OpenGl function pointers.
    gl::load_with(|symbol| { window.get_proc_address(symbol) as *const _ });

    // get version info
    let renderer = glubyte_ptr_to_string_safe(
        unsafe { gl::GetString(gl::RENDERER) }
    );
    let version = glubyte_ptr_to_string_safe(
        unsafe { gl::GetString(gl::VERSION) }
    );

    println!("Renderer: {}", renderer);
    println!("OpenGL version supported {}", version);

    // tell GL to only draw onto a pixel if the shape is closer to the viewer
    unsafe {
        gl::Enable(gl::DEPTH_TEST); // enable depth-testing
        gl::DepthFunc(gl::LESS); // depth-testing interprets a smaller value as "closer"
    }
    /* OTHER STUFF GOES HERE NEXT */

    // close GL context and any other GLFW resources
    //glfw::ffi::GlfwTerminate();
}

