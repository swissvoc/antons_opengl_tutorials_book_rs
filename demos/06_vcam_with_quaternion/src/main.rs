extern crate gl;
extern crate glfw;
extern crate chrono;

mod gl_utils;
mod graphics_math;


use glfw::{Action, Context, Key};
use gl::types::{GLfloat, GLuint, GLsizeiptr, GLchar, GLvoid, GLint, GLenum};

use std::string::String;
use std::mem;
use std::ptr;

use std::process;
use gl_utils::*;

use graphics_math as math;
use math::{Mat4};


const GL_LOG_FILE: &str = "gl.log";

static mut PREVIOUS_SECONDS: f64 = 0.;


fn main() {

}
