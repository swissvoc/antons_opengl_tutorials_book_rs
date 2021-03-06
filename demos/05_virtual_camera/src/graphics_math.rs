use std::fmt;
use std::ops;
use std::convert::From;


// Constants used to convert degrees into radians.
pub const M_PI: f32 = 3.14159265358979323846264338327950288;
pub const TAU: f32 = 2.0 * M_PI;
pub const ONE_DEG_IN_RAD: f32 = (2.0 * M_PI) / 360.0; // == 0.017444444
pub const ONE_RAD_IN_DEG: f32 = 360.0 / (2.0 * M_PI); // == 57.2957795


#[derive(Copy, Clone, Debug)]
pub struct Vec2 {
    v: [f32; 2],
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Vec2 {
        Vec2 { v: [x, y] }
    }

    pub fn zero() -> Vec2 { 
        Vec2 { v: [0.0, 0.0] }
    }
}

#[inline]
pub fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

impl fmt::Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:.2}, {:.2}]", self.v[0], self.v[1])
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Vec3 {
    v: [f32; 3],
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 { v: [x, y, z] }
    }

    pub fn zero() -> Vec3 {
        Vec3 { v: [0.0, 0.0, 0.0] }
    }
    
    pub fn norm(&self) -> f32 {
        f32::sqrt(self.v[0] * self.v[0] + self.v[1] * self.v[1] + self.v[2] * self.v[2])
    }

    // Squared length.
    pub fn norm2(&self) -> f32 {
        self.v[0] * self.v[0] + self.v[1] * self.v[1] + self.v[2] * self.v[2]
    }

    pub fn normalize(&self) -> Vec3 {
        let norm_v = self.norm();
        if norm_v == 0.0 {
            return Vec3::zero();
        }

        Vec3::new(self.v[0] / norm_v, self.v[1] / norm_v, self.v[2] / norm_v)
    }

    pub fn dot(&self, other: &Vec3) -> f32 {
        self.v[0] * other.v[0] + self.v[1] * other.v[1] + self.v[2] * other.v[2]
    }

    pub fn cross(&self, other: &Vec3) -> Vec3 {
        let x = self.v[1] * other.v[2] - self.v[2] * other.v[1];
        let y = self.v[2] * other.v[0] - self.v[0] * other.v[2];
        let z = self.v[0] * other.v[1] - self.v[1] * other.v[0];
    
        Vec3::new(x, y, z)
    }

    pub fn get_squared_dist(&self, to: &Vec3) -> f32 {
        let x = (to.v[0] - self.v[0]) * (to.v[0] - self.v[0]);
        let y = (to.v[1] - self.v[1]) * (to.v[1] - self.v[1]);
        let z = (to.v[2] - self.v[2]) * (to.v[2] - self.v[2]);
    
        x + y + z
    }
}

#[inline]
pub fn vec3<T: Into<Vec3>>(v: T) -> Vec3 {
    v.into()
}

impl From<(f32, f32, f32)> for Vec3 {
    #[inline]
    fn from((x, y, z): (f32, f32, f32)) -> Vec3 {
        Vec3::new(x, y, z)
    }
}

impl From<(Vec2, f32)> for Vec3 {
    #[inline]
    fn from((v, z): (Vec2, f32)) -> Vec3 {
        Vec3::new(v.v[0], v.v[1], z)
    }
}

impl<'a> From<(&'a Vec2, f32)> for Vec3 {
    #[inline]
    fn from((v, z): (&'a Vec2, f32)) -> Vec3 {
        Vec3::new(v.v[0], v.v[1], z)
    }
}

impl fmt::Display for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:.2}, {:.2}, {:.2}]", self.v[0], self.v[1], self.v[2])
    }
}

impl<'a> ops::Add<Vec3> for &'a Vec3 {
    type Output = Vec3;

    fn add(self, other: Vec3) -> Self::Output {
        Vec3 {
            v: [
                self.v[0] + other.v[0],
                self.v[1] + other.v[1],
                self.v[2] + other.v[2],
            ]
        }
    }
}

impl ops::Add<Vec3> for Vec3 {
    type Output = Vec3;

    fn add(self, other: Vec3) -> Self::Output {
        Vec3 {
            v: [
                self.v[0] + other.v[0],
                self.v[1] + other.v[1],
                self.v[2] + other.v[2],
            ]
        }
    }
}

impl<'a> ops::Add<&'a Vec3> for Vec3 {
    type Output = Vec3;

    fn add(self, other: &'a Vec3) -> Self::Output {
        Vec3 {
            v: [
                self.v[0] + other.v[0],
                self.v[1] + other.v[1],
                self.v[2] + other.v[2],               
            ]
        }
    }
}

impl<'a, 'b> ops::Add<&'b Vec3> for &'a Vec3 {
    type Output = Vec3;

    fn add(self, other: &'b Vec3) -> Self::Output {
        Vec3 {
            v: [
                self.v[0] + other.v[0],
                self.v[1] + other.v[1],
                self.v[2] + other.v[2],
            ]
        }
    }
}

impl ops::Add<f32> for Vec3 {
    type Output = Vec3;

    fn add(self, other: f32) -> Self::Output {
        Vec3 {
            v: [
                self.v[0] + other,
                self.v[1] + other,
                self.v[2] + other,
            ]
        }
    }
}

impl<'a> ops::Sub<Vec3> for &'a Vec3 {
    type Output = Vec3;

    fn sub(self, other: Vec3) -> Self::Output {
        Vec3 {
            v: [
                self.v[0] - other.v[0],
                self.v[1] - other.v[1],
                self.v[2] - other.v[2],
            ]
        }
    }
}

impl ops::Sub<Vec3> for Vec3 {
    type Output = Vec3;

    fn sub(self, other: Vec3) -> Self::Output {
        Vec3 {
            v: [
                self.v[0] - other.v[0],
                self.v[1] - other.v[1],
                self.v[2] - other.v[2],
            ]
        }
    }
}

impl<'a> ops::Sub<&'a Vec3> for Vec3 {
    type Output = Vec3;

    fn sub(self, other: &'a Vec3) -> Self::Output {
        Vec3 {
            v: [
                self.v[0] - other.v[0],
                self.v[1] - other.v[1],
                self.v[2] - other.v[2],               
            ]
        }
    }
}

impl<'a, 'b> ops::Sub<&'b Vec3> for &'a Vec3 {
    type Output = Vec3;

    fn sub(self, other: &'b Vec3) -> Self::Output {
        Vec3 {
            v: [
                self.v[0] - other.v[0],
                self.v[1] - other.v[1],
                self.v[2] - other.v[2],
            ]
        }
    }
}

impl ops::Sub<f32> for Vec3 {
    type Output = Vec3;

    fn sub(self, other: f32) -> Self::Output {
        Vec3 {
            v: [
                self.v[0] - other,
                self.v[1] - other,
                self.v[2] - other,
            ]
        }
    }
}

impl ops::AddAssign<Vec3> for Vec3 {
    fn add_assign(&mut self, other: Vec3) {
        *self = Vec3 {
            v: [
                self.v[0] + other.v[0],
                self.v[1] + other.v[1],
                self.v[2] + other.v[2],
            ]
        }
    }
}

impl<'a> ops::AddAssign<&'a Vec3> for Vec3 {
    fn add_assign(&mut self, other: &'a Vec3) {
        *self = Vec3 {
            v: [
                self.v[0] + other.v[0],
                self.v[1] + other.v[1],
                self.v[2] + other.v[2],
            ]
        }
    }
}

impl<'a> ops::AddAssign<Vec3> for &'a mut Vec3 {
    fn add_assign(&mut self, other: Vec3) {
        **self = Vec3 {
            v: [
                self.v[0] + other.v[0],
                self.v[1] + other.v[1],
                self.v[2] + other.v[2],
            ]
        }
    }
}

impl<'a, 'b> ops::AddAssign<&'a Vec3> for &'b mut Vec3 {
    fn add_assign(&mut self, other: &'a Vec3) {
        **self = Vec3 {
            v: [
                self.v[0] + other.v[0],
                self.v[1] + other.v[1],
                self.v[2] + other.v[2],
            ]
        }
    }
}

impl ops::AddAssign<f32> for Vec3 {
    fn add_assign(&mut self, other: f32) {
        *self = Vec3 {
            v: [
                self.v[0] + other,
                self.v[1] + other,
                self.v[2] + other,
            ]
        }
    }
}

impl ops::SubAssign<Vec3> for Vec3 {
    fn sub_assign(&mut self, other: Vec3) {
        *self = Vec3 {
            v: [
                self.v[0] - other.v[0],
                self.v[1] - other.v[1],
                self.v[2] - other.v[2],
            ]
        }
    }
}

impl<'a> ops::SubAssign<&'a Vec3> for Vec3 {
    fn sub_assign(&mut self, other: &'a Vec3) {
        *self = Vec3 {
            v: [
                self.v[0] - other.v[0],
                self.v[1] - other.v[1],
                self.v[2] - other.v[2],
            ]
        }
    }
}

impl<'a> ops::SubAssign<Vec3> for &'a mut Vec3 {
    fn sub_assign(&mut self, other: Vec3) {
        **self = Vec3 {
            v: [
                self.v[0] - other.v[0],
                self.v[1] - other.v[1],
                self.v[2] - other.v[2],
            ]
        }
    }
}

impl<'a, 'b> ops::SubAssign<&'a Vec3> for &'b mut Vec3 {
    fn sub_assign(&mut self, other: &'a Vec3) {
        **self = Vec3 {
            v: [
                self.v[0] - other.v[0],
                self.v[1] - other.v[1],
                self.v[2] - other.v[2],
            ]
        }
    }
}

impl ops::SubAssign<f32> for Vec3 {
    fn sub_assign(&mut self, other: f32) {
        *self = Vec3 {
            v: [
                self.v[0] - other,
                self.v[1] - other,
                self.v[2] - other,
            ]
        }
    }
}

impl ops::Mul<f32> for Vec3 {
    type Output = Vec3;

    fn mul(self, other: f32) -> Vec3 {
        Vec3 {
            v: [
                self.v[0] * other,
                self.v[1] * other,
                self.v[2] * other,
            ]
        }
    }
}

impl<'a> ops::Mul<f32> for &'a Vec3 {
    type Output = Vec3;

    fn mul(self, other: f32) -> Vec3 {
        Vec3 {
            v: [
                self.v[0] * other,
                self.v[1] * other,
                self.v[2] * other,
            ]
        }
    }
}

impl ops::Div<f32> for Vec3 {
    type Output = Vec3;

    fn div(self, other: f32) -> Vec3 {
        Vec3 {
            v: [
                self.v[0] / other,
                self.v[1] / other,
                self.v[2] / other,
            ]
        }
    }
}

impl<'a> ops::Div<f32> for &'a Vec3 {
    type Output = Vec3;

    fn div(self, other: f32) -> Vec3 {
        Vec3 {
            v: [
                self.v[0] / other,
                self.v[1] / other,
                self.v[2] / other,
            ]
        }
    }
}

impl ops::DivAssign<f32> for Vec3 {
    fn div_assign(&mut self, other: f32) {
        *self = Vec3 {
            v: [
                self.v[0] / other,
                self.v[1] / other,
                self.v[2] / other,
            ]
        }
    }
}

impl<'a> ops::DivAssign<f32> for &'a mut Vec3 {
    fn div_assign(&mut self, other: f32) {
        **self = Vec3 {
            v: [
                self.v[0] / other,
                self.v[1] / other,
                self.v[2] / other,
            ]
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Vec4 {
    v: [f32; 4],
}

impl Vec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Vec4 {
        Vec4 { v: [x, y, z, w] }
    }

    pub fn zero() -> Vec4 {
        Vec4 { v: [0.0, 0.0, 0.0, 0.0] }
    }
}

#[inline]
pub fn vec4<T: Into<Vec4>>(v: T) -> Vec4 {
    v.into()
}

impl From<(f32, f32, f32, f32)> for Vec4 {
    #[inline]
    fn from((x, y, z, w): (f32, f32, f32, f32)) -> Vec4 {
        Vec4::new(x, y, z, w)
    }
}

impl From<(Vec2, f32, f32)> for Vec4 {
    #[inline]
    fn from((v, z, w): (Vec2, f32, f32)) -> Vec4 {
        Vec4::new(v.v[0], v.v[1], z, w)
    }
}

impl<'a> From<(&'a Vec2, f32, f32)> for Vec4 {
    #[inline]
    fn from((v, z, w): (&'a Vec2, f32, f32)) -> Vec4 {
        Vec4::new(v.v[0], v.v[1], z, w)
    }
}

impl From<(Vec3, f32)> for Vec4 {
    #[inline]
    fn from((v, w): (Vec3, f32)) -> Vec4 {
        Vec4::new(v.v[0], v.v[1], v.v[2], w)
    }
}

impl<'a> From<(&'a Vec3, f32)> for Vec4 {
    #[inline]
    fn from((v, w): (&'a Vec3, f32)) -> Vec4 {
        Vec4::new(v.v[0], v.v[1], v.v[2], w)
    }
}

impl fmt::Display for Vec4 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:.2}, {:.2}, {:.2}, {:.2}]", self.v[0], self.v[1], self.v[2], self.v[3])
    }
}

///
/// The `Mat3` type represents 3x3 matrices in column-major order.
///
#[derive(Copy, Clone, Debug)]
pub struct Mat3 {
    m: [f32; 9],
}

impl Mat3 {
    pub fn new(
        m11: f32, m12: f32, m13: f32, 
        m21: f32, m22: f32, m23: f32, 
        m31: f32, m32: f32, m33: f32) -> Mat3 {

        Mat3 {
            m: [
                m11, m12, m13, // Column 1
                m21, m22, m23, // Column 2
                m31, m32, m33  // Column 3
            ]
        }
    }

    pub fn zero() -> Mat3 {
        Mat3::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
    }

    pub fn identity() -> Mat3 {
        Mat3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0)
    }

    pub fn as_ptr(&self) -> *const f32 {
        self.m.as_ptr()
    }
}

impl fmt::Display for Mat3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, 
            "\n[{:.2}][{:.2}][{:.2}]\n[{:.2}][{:.2}][{:.2}]\n[{:.2}][{:.2}][{:.2}]", 
            self.m[0], self.m[3], self.m[6],
            self.m[1], self.m[4], self.m[7],
            self.m[2], self.m[5], self.m[8],
        )
    }
}

#[inline]
fn mat3(m11: f32, m12: f32, m13: f32, 
        m21: f32, m22: f32, m23: f32, 
        m31: f32, m32: f32, m33: f32) -> Mat3 {

    Mat3::new(m11, m12, m13, m21, m22, m23, m31, m32, m33)
}

///
/// The `Mat4` type represents 4x4 matrices in column-major order.
///
#[derive(Copy, Clone, Debug)]
pub struct Mat4 {
    pub m: [f32; 16],
}

impl Mat4 {
    pub fn new(
        m11: f32, m12: f32, m13: f32, m14: f32,
        m21: f32, m22: f32, m23: f32, m24: f32,
        m31: f32, m32: f32, m33: f32, m34: f32,
        m41: f32, m42: f32, m43: f32, m44: f32) -> Mat4 {

        Mat4 {
            m: [
                m11, m12, m13, m14, // Column 1
                m21, m22, m23, m24, // Column 2
                m31, m32, m33, m34, // Column 3
                m41, m42, m43, m44  // Column 4
            ]
        }
    }

    pub fn zero() -> Mat4 {
        Mat4::new(
            0.0, 0.0, 0.0, 0.0, 
            0.0, 0.0, 0.0, 0.0, 
            0.0, 0.0, 0.0, 0.0, 
            0.0, 0.0, 0.0, 0.0
        )
    }

    pub fn identity() -> Mat4 {
        Mat4::new(
            1.0, 0.0, 0.0, 0.0, 
            0.0, 1.0, 0.0, 0.0, 
            0.0, 0.0, 1.0, 0.0, 
            0.0, 0.0, 0.0, 1.0
        )
    }

    pub fn transpose(&self) -> Mat4 {
        Mat4::new(
            self.m[0], self.m[4], self.m[8],  self.m[12],
            self.m[1], self.m[5], self.m[9],  self.m[13], 
            self.m[2], self.m[6], self.m[10], self.m[14], 
            self.m[3], self.m[7], self.m[11], self.m[15]
        )
    }

    pub fn translate(&self, v: &Vec3) -> Mat4 {
        let mut m_t = Mat4::identity();
        m_t.m[12] = v.v[0];
        m_t.m[13] = v.v[1];
        m_t.m[14] = v.v[2];

        m_t * self
    }

    // Rotate around x axis by an angle in degrees.
    pub fn rotate_x_deg(&self, deg: f32) -> Mat4 {
        // Convert to radians.
        let rad = deg * ONE_DEG_IN_RAD;
        let mut m_r = Mat4::identity();
        m_r.m[5]  =  f32::cos(rad);
        m_r.m[9]  = -f32::sin(rad);
        m_r.m[6]  =  f32::sin(rad);
        m_r.m[10] =  f32::cos(rad);
    
        m_r * self
    }

    // Rotate around y axis by an angle in degrees.
    pub fn rotate_y_deg(&self, deg: f32) -> Mat4 {
        // Convert to radians.
        let rad = deg * ONE_DEG_IN_RAD;
        let mut m_r = Mat4::identity();
        m_r.m[0]  =  f32::cos(rad);
        m_r.m[8]  =  f32::sin(rad);
        m_r.m[2]  = -f32::sin(rad);
        m_r.m[10] =  f32::cos(rad);
    
        m_r * self
    }

    // Rotate around z axis by an angle in degrees.
    pub fn rotate_z_deg(&self, deg: f32) -> Mat4 {
        // Convert to radians.
        let rad = deg * ONE_DEG_IN_RAD;
        let mut m_r = Mat4::identity();
        m_r.m[0] =  f32::cos(rad);
        m_r.m[4] = -f32::sin(rad);
        m_r.m[1] =  f32::sin(rad);
        m_r.m[5] =  f32::cos(rad);
    
        m_r * self
    }

    // scale a matrix by [x, y, z]
    pub fn scale(&self, v: &Vec3) -> Mat4 {
        let mut m_s = Mat4::identity();
        m_s.m[0]  = v.v[0];
        m_s.m[5]  = v.v[1];
        m_s.m[10] = v.v[2];
    
        m_s * self
    }

    /// returns a scalar value with the determinant for a 4x4 matrix
    /// see
    /// http://www.euclideanspace.com/maths/algebra/matrix/functions/determinant/fourD/index.htm
    pub fn determinant(&self) -> f32 {
        self.m[12] * self.m[9]  * self.m[6]  * self.m[3]  -
        self.m[8]  * self.m[13] * self.m[6]  * self.m[3]  -
        self.m[12] * self.m[5]  * self.m[10] * self.m[3]  +
        self.m[4]  * self.m[13] * self.m[10] * self.m[3]  +
        self.m[8]  * self.m[5]  * self.m[14] * self.m[3]  -
        self.m[4]  * self.m[9]  * self.m[14] * self.m[3]  -
        self.m[12] * self.m[9]  * self.m[2]  * self.m[7]  +
        self.m[8]  * self.m[13] * self.m[2]  * self.m[7]  +
        self.m[12] * self.m[1]  * self.m[10] * self.m[7]  -
        self.m[0]  * self.m[13] * self.m[10] * self.m[7]  -
        self.m[8]  * self.m[1]  * self.m[14] * self.m[7]  +
        self.m[0]  * self.m[9]  * self.m[14] * self.m[7]  +
        self.m[12] * self.m[5]  * self.m[2]  * self.m[11] -
        self.m[4]  * self.m[13] * self.m[2]  * self.m[11] -
        self.m[12] * self.m[1]  * self.m[6]  * self.m[11] +
        self.m[0]  * self.m[13] * self.m[6]  * self.m[11] +
        self.m[4]  * self.m[1]  * self.m[14] * self.m[11] -
        self.m[0]  * self.m[5]  * self.m[14] * self.m[11] -
        self.m[8]  * self.m[5]  * self.m[2]  * self.m[15] +
        self.m[4]  * self.m[9]  * self.m[2]  * self.m[15] +
        self.m[8]  * self.m[1]  * self.m[6]  * self.m[15] -
        self.m[0]  * self.m[9]  * self.m[6]  * self.m[15] -
        self.m[4]  * self.m[1]  * self.m[10] * self.m[15] +
        self.m[0]  * self.m[5]  * self.m[10] * self.m[15]
    }

    // returns a perspective function mimicking the opengl projection style.
    pub fn perspective(fovy: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
        let fov_rad = fovy * ONE_DEG_IN_RAD;
        let range = f32::tan(fov_rad / 2.0) * near;
        let sx = (2.0 * near) / (range * aspect + range * aspect);
        let sy = near / range;
        let sz = -(far + near) / (far - near);
        let pz = -(2.0 * far * near) / (far - near);
        let mut m = Mat4::zero(); // make sure bottom-right corner is zero
        m.m[0] = sx;
        m.m[5] = sy;
        m.m[10] = sz;
        m.m[14] = pz;
        m.m[11] = -1.0;
        
        m
    }

    pub fn as_ptr(&self) -> *const f32 {
        self.m.as_ptr()
    }
}

impl fmt::Display for Mat4 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, 
            "\n[{:.2}][{:.2}][{:.2}][{:.2}]\n[{:.2}][{:.2}][{:.2}][{:.2}]\n[{:.2}][{:.2}][{:.2}][{:.2}]\n[{:.2}][{:.2}][{:.2}][{:.2}]", 
            self.m[0], self.m[4], self.m[8],  self.m[12],
            self.m[1], self.m[5], self.m[9],  self.m[13],
            self.m[2], self.m[6], self.m[10], self.m[14],
            self.m[3], self.m[7], self.m[11], self.m[15]
        )
    }
}

pub fn mat4(
        m11: f32, m12: f32, m13: f32, m14: f32, 
        m21: f32, m22: f32, m23: f32, m24: f32,
        m31: f32, m32: f32, m33: f32, m34: f32,
        m41: f32, m42: f32, m43: f32, m44: f32) -> Mat4 {

    Mat4::new(
        m11, m12, m13, m14, 
        m21, m22, m23, m24, 
        m31, m32, m33, m34, 
        m41, m42, m43, m44
    )
}

impl ops::Mul<Vec4> for Mat4 {
    type Output = Vec4;

    fn mul(self, other: Vec4) -> Self::Output {
        let x = self.m[0] * other.v[0] + self.m[4] * other.v[1] + self.m[8]  * other.v[2] + self.m[12] * other.v[3];
        let y = self.m[1] * other.v[0] + self.m[5] * other.v[1] + self.m[9]  * other.v[2] + self.m[13] * other.v[3];
        let z = self.m[2] * other.v[0] + self.m[6] * other.v[1] + self.m[10] * other.v[2] + self.m[14] * other.v[3];
        let w = self.m[3] * other.v[0] + self.m[7] * other.v[1] + self.m[11] * other.v[2] + self.m[15] * other.v[3];
        
        Vec4::new(x, y, z, w)
    }
}

impl<'a> ops::Mul<&'a Mat4> for Mat4 {
    type Output = Mat4;

    fn mul(self, other: &'a Mat4) -> Mat4 {
        let mut mm = Mat4::zero();

        mm.m[0]  = self.m[0]*other.m[0]  + self.m[4]*other.m[1]  + self.m[8]*other.m[2]   + self.m[12]*other.m[3];
        mm.m[1]  = self.m[1]*other.m[0]  + self.m[5]*other.m[1]  + self.m[9]*other.m[2]   + self.m[13]*other.m[3];
        mm.m[2]  = self.m[2]*other.m[0]  + self.m[6]*other.m[1]  + self.m[10]*other.m[2]  + self.m[14]*other.m[3];
        mm.m[3]  = self.m[3]*other.m[0]  + self.m[7]*other.m[1]  + self.m[11]*other.m[2]  + self.m[15]*other.m[3];
        mm.m[4]  = self.m[0]*other.m[4]  + self.m[4]*other.m[5]  + self.m[8]*other.m[6]   + self.m[12]*other.m[7];
        mm.m[5]  = self.m[1]*other.m[4]  + self.m[5]*other.m[5]  + self.m[9]*other.m[6]   + self.m[13]*other.m[7];
        mm.m[6]  = self.m[2]*other.m[4]  + self.m[6]*other.m[5]  + self.m[10]*other.m[6]  + self.m[14]*other.m[7];
        mm.m[7]  = self.m[3]*other.m[4]  + self.m[7]*other.m[5]  + self.m[11]*other.m[6]  + self.m[15]*other.m[7];
        mm.m[8]  = self.m[0]*other.m[8]  + self.m[4]*other.m[9]  + self.m[8]*other.m[10]  + self.m[12]*other.m[11];
        mm.m[9]  = self.m[1]*other.m[8]  + self.m[5]*other.m[9]  + self.m[9]*other.m[10]  + self.m[13]*other.m[11];
        mm.m[10] = self.m[2]*other.m[8]  + self.m[6]*other.m[9]  + self.m[10]*other.m[10] + self.m[14]*other.m[11];
        mm.m[11] = self.m[3]*other.m[8]  + self.m[7]*other.m[9]  + self.m[11]*other.m[10] + self.m[15]*other.m[11];
        mm.m[12] = self.m[0]*other.m[12] + self.m[4]*other.m[13] + self.m[8]*other.m[14]  + self.m[12]*other.m[15];
        mm.m[13] = self.m[1]*other.m[12] + self.m[5]*other.m[13] + self.m[9]*other.m[14]  + self.m[13]*other.m[15];
        mm.m[14] = self.m[2]*other.m[12] + self.m[6]*other.m[13] + self.m[10]*other.m[14] + self.m[14]*other.m[15];
        mm.m[15] = self.m[3]*other.m[12] + self.m[7]*other.m[13] + self.m[11]*other.m[14] + self.m[15]*other.m[15];

        mm
    }
}

impl<'a, 'b> ops::Mul<&'a Mat4> for &'b Mat4 {
    type Output = Mat4;

    fn mul(self, other: &'a Mat4) -> Mat4 {
        let mut mm = Mat4::zero();

        mm.m[0]  = self.m[0]*other.m[0]  + self.m[4]*other.m[1]  + self.m[8]*other.m[2]   + self.m[12]*other.m[3];
        mm.m[1]  = self.m[1]*other.m[0]  + self.m[5]*other.m[1]  + self.m[9]*other.m[2]   + self.m[13]*other.m[3];
        mm.m[2]  = self.m[2]*other.m[0]  + self.m[6]*other.m[1]  + self.m[10]*other.m[2]  + self.m[14]*other.m[3];
        mm.m[3]  = self.m[3]*other.m[0]  + self.m[7]*other.m[1]  + self.m[11]*other.m[2]  + self.m[15]*other.m[3];
        mm.m[4]  = self.m[0]*other.m[4]  + self.m[4]*other.m[5]  + self.m[8]*other.m[6]   + self.m[12]*other.m[7];
        mm.m[5]  = self.m[1]*other.m[4]  + self.m[5]*other.m[5]  + self.m[9]*other.m[6]   + self.m[13]*other.m[7];
        mm.m[6]  = self.m[2]*other.m[4]  + self.m[6]*other.m[5]  + self.m[10]*other.m[6]  + self.m[14]*other.m[7];
        mm.m[7]  = self.m[3]*other.m[4]  + self.m[7]*other.m[5]  + self.m[11]*other.m[6]  + self.m[15]*other.m[7];
        mm.m[8]  = self.m[0]*other.m[8]  + self.m[4]*other.m[9]  + self.m[8]*other.m[10]  + self.m[12]*other.m[11];
        mm.m[9]  = self.m[1]*other.m[8]  + self.m[5]*other.m[9]  + self.m[9]*other.m[10]  + self.m[13]*other.m[11];
        mm.m[10] = self.m[2]*other.m[8]  + self.m[6]*other.m[9]  + self.m[10]*other.m[10] + self.m[14]*other.m[11];
        mm.m[11] = self.m[3]*other.m[8]  + self.m[7]*other.m[9]  + self.m[11]*other.m[10] + self.m[15]*other.m[11];
        mm.m[12] = self.m[0]*other.m[12] + self.m[4]*other.m[13] + self.m[8]*other.m[14]  + self.m[12]*other.m[15];
        mm.m[13] = self.m[1]*other.m[12] + self.m[5]*other.m[13] + self.m[9]*other.m[14]  + self.m[13]*other.m[15];
        mm.m[14] = self.m[2]*other.m[12] + self.m[6]*other.m[13] + self.m[10]*other.m[14] + self.m[14]*other.m[15];
        mm.m[15] = self.m[3]*other.m[12] + self.m[7]*other.m[13] + self.m[11]*other.m[14] + self.m[15]*other.m[15];

        mm
    }
}


#[derive(Copy, Clone, Debug)]
struct Versor {
    q: [f32; 4],
}

impl Versor {
    pub fn normalize(&self) -> Versor {
        // normalize(q) = q / magnitude (q)
        // magnitude (q) = sqrt (w*w + x*x...)
        // only compute sqrt if interior sum != 1.0
        let sum = self.q[0] * self.q[0] + self.q[1] * self.q[1] + self.q[2] * self.q[2] + self.q[3] * self.q[3];
        // NB: Floats have min 6 digits of precision.
        let threshold = 0.0001;
        if f32::abs(1.0 - sum) < threshold {
            return *self;
        }

        let norm = f32::sqrt(sum);
        self / norm
    }

    pub fn dot(&self, r: &Versor) -> f32 {
        self.q[0] * r.q[0] + self.q[1] * r.q[1] + self.q[2] * r.q[2] + self.q[3] * r.q[3]
    }

    pub fn from_axis_rad(radians: f32, x: f32, y: f32, z: f32) -> Versor {
        Versor {
            q: [
                f32::cos(radians / 2.0),
                f32::sin(radians / 2.0) * x,
                f32::sin(radians / 2.0) * y,
                f32::sin(radians / 2.0) * z,
            ]
        }
    }

    pub fn from_axis_deg(degrees: f32, x: f32, y: f32, z: f32) -> Versor {
        Self::from_axis_rad(ONE_DEG_IN_RAD * degrees, x, y, z)
    }

    pub fn to_mat4(&self) -> Mat4 {
        let w = self.q[0];
        let x = self.q[1];
        let y = self.q[2];
        let z = self.q[3];
    
        Mat4::new(
            1.0 - 2.0 * y * y - 2.0 * z * z, 2.0 * x * y + 2.0 * w * z,       2.0 * x * z - 2.0 * w * y,       0.0, 
            2.0 * x * y - 2.0 * w * z,       1.0 - 2.0 * x * x - 2.0 * z * z, 2.0 * y * z + 2.0 * w * x,       0.0, 
            2.0 * x * z + 2.0 * w * y,       2.0 * y * z - 2.0 * w * x,       1.0 - 2.0 * x * x - 2.0 * y * y, 0.0, 
            0.0,                             0.0,                             0.0,                             1.0
        )
    }

    pub fn slerp(q: &mut Versor, r: &Versor, t: f32) -> Versor {
        // angle between q0-q1
        let mut cos_half_theta = q.dot(r);
        // as found here
        // http://stackoverflow.com/questions/2886606/flipping-issue-when-interpolating-rotations-using-quaternions
        // if dot product is negative then one quaternion should be negated, to make
        // it take the short way around, rather than the long way
        // yeah! and furthermore Susan, I had to recalculate the d.p. after this
        if cos_half_theta < 0.0 {
            q.q[0] *= -1.0;
            q.q[1] *= -1.0;
            q.q[2] *= -1.0;
            q.q[3] *= -1.0;

            cos_half_theta = q.dot(r);
        }
        // if qa=qb or qa=-qb then theta = 0 and we can return qa
        if f32::abs(cos_half_theta) >= 1.0 {
            return *q;
        }

        // Calculate temporary values
        let sin_half_theta = f32::sqrt(1.0 - cos_half_theta * cos_half_theta);
        // if theta = 180 degrees then result is not fully defined
        // we could rotate around any axis normal to qa or qb
        let mut result = Versor { q: [1.0, 0.0, 0.0, 0.0] };
        if f32::abs(sin_half_theta) < 0.001 {
            result.q[0] = (1.0 - t) * q.q[0] + t * r.q[0];
            result.q[1] = (1.0 - t) * q.q[1] + t * r.q[1];
            result.q[2] = (1.0 - t) * q.q[2] + t * r.q[2];
            result.q[3] = (1.0 - t) * q.q[3] + t * r.q[3];

            return result;
        }
        let half_theta = f32::acos(cos_half_theta);
        let a = f32::sin((1.0 - t) * half_theta) / sin_half_theta;
        let b = f32::sin(t * half_theta) / sin_half_theta;
        
        result.q[0] = q.q[0] * a + r.q[0] * b;
        result.q[1] = q.q[1] * a + r.q[1] * b;
        result.q[2] = q.q[2] * a + r.q[2] * b;
        result.q[3] = q.q[3] * a + r.q[3] * b;

        return result;
    }
}

impl fmt::Display for Versor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "[{:.2}, {:.2}, {:.2}, {:.2}]", self.q[0], self.q[1], self.q[2], self.q[3])
    }
}

impl ops::Div<f32> for Versor {
    type Output = Versor;

    fn div(self, other: f32) -> Versor {
        Versor {
            q: [
                self.q[0] / other, 
                self.q[1] / other, 
                self.q[2] / other, 
                self.q[3] / other,
            ]
        }
    }
}

impl<'a> ops::Div<f32> for &'a Versor {
    type Output = Versor;

    fn div(self, other: f32) -> Versor {
        Versor {
            q: [
                self.q[0] / other, 
                self.q[1] / other, 
                self.q[2] / other, 
                self.q[3] / other,
            ]
        }
    }
}

impl ops::Mul<f32> for Versor {
    type Output = Versor;

    fn mul(self, other: f32) -> Versor {
        Versor {
            q: [
                self.q[0] * other,
                self.q[1] * other,
                self.q[2] * other,
                self.q[3] * other,
            ]
        }
    }
}

impl<'a> ops::Mul<&'a Versor> for Versor {
    type Output = Versor;

    fn mul(self, other: &'a Versor) -> Self::Output {
        let result = Versor {
            q: [
                other.q[0] * self.q[0] - other.q[1] * self.q[1] - other.q[2] * self.q[2] - other.q[3] * self.q[3],
                other.q[0] * self.q[1] + other.q[1] * self.q[0] - other.q[2] * self.q[3] + other.q[3] * self.q[2],
                other.q[0] * self.q[2] + other.q[1] * self.q[3] + other.q[2] * self.q[0] - other.q[3] * self.q[1],
                other.q[0] * self.q[3] - other.q[1] * self.q[2] + other.q[2] * self.q[1] + other.q[3] * self.q[0],
            ]
        };
        // Renormalize in case of mangling.
        result.normalize()
    }
}

impl<'a> ops::Add<&'a Versor> for Versor {
    type Output = Versor;

    fn add(self, other: &'a Versor) -> Self::Output {
        let result = Versor {
            q: [
                other.q[0] + self.q[0],
                other.q[1] + self.q[1],
                other.q[2] + self.q[2],
                other.q[3] + self.q[3],
            ]
        };
        // Renormalize in case of mangling.
        result.normalize()
    }
}


