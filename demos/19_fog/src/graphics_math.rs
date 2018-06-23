use std::cmp;
use std::fmt;
use std::ops;
use std::convert::From;
use std::convert;


// Constants used to convert degrees into radians.
pub const M_PI: f32 = 3.14159265358979323846264338327950288;
pub const TAU: f32 = 2.0 * M_PI;
pub const ONE_DEG_IN_RAD: f32 = (2.0 * M_PI) / 360.0; // == 0.017444444
pub const ONE_RAD_IN_DEG: f32 = 360.0 / (2.0 * M_PI); // == 57.2957795
pub const EPSILON: f32 = 0.00001; 


///
/// A representation of two-dimensional vectors, with a
/// Euclidean metric.
///
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

///
/// A representation of three-dimensional vectors, with a
/// Euclidean metric.
///
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vec3 {
    pub v: [f32; 3],
}

impl Vec3 {
    ///
    /// Create a new vector.
    ///
    pub fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 { v: [x, y, z] }
    }

    ///
    /// Generate a zero vector.
    ///
    pub fn zero() -> Vec3 {
        Vec3 { v: [0.0, 0.0, 0.0] }
    }
    
    ///
    /// Compute the norm (length) of a vector.
    ///
    pub fn norm(&self) -> f32 {
        f32::sqrt(self.v[0] * self.v[0] + self.v[1] * self.v[1] + self.v[2] * self.v[2])
    }

    ///
    /// Compute the squared norm (length) of a vector.
    ///
    pub fn norm2(&self) -> f32 {
        self.v[0] * self.v[0] + self.v[1] * self.v[1] + self.v[2] * self.v[2]
    }

    ///
    /// Convert an arbitrary vector into a unit vector.
    ///
    pub fn normalize(&self) -> Vec3 {
        let norm_v = self.norm();
        if norm_v == 0.0 {
            return Vec3::zero();
        }

        Vec3::new(self.v[0] / norm_v, self.v[1] / norm_v, self.v[2] / norm_v)
    }

    ///
    /// Compute the dot product of two vectors.
    ///
    pub fn dot(&self, other: &Vec3) -> f32 {
        self.v[0] * other.v[0] + self.v[1] * other.v[1] + self.v[2] * other.v[2]
    }

    ///
    /// Compute the cross product of two three-dimensional vectors. Note that
    /// with the vectors used in computer graphics (two, three, and four dimensions),
    /// the cross product is defined only in three dimensions. Also note that the 
    /// cross product is the hodge dual of the corresponding 2-vector representing 
    /// the surface element that the crossed vector is normal to. That is, 
    /// given vectors u and v, u x v == *(u /\ v), where *(.) denotes the hodge dual.
    ///
    pub fn cross(&self, other: &Vec3) -> Vec3 {
        let x = self.v[1] * other.v[2] - self.v[2] * other.v[1];
        let y = self.v[2] * other.v[0] - self.v[0] * other.v[2];
        let z = self.v[0] * other.v[1] - self.v[1] * other.v[0];
    
        Vec3::new(x, y, z)
    }

    ///
    /// Compute the squared distance between two vectors.
    ///
    pub fn get_squared_dist(&self, to: &Vec3) -> f32 {
        let x = (to.v[0] - self.v[0]) * (to.v[0] - self.v[0]);
        let y = (to.v[1] - self.v[1]) * (to.v[1] - self.v[1]);
        let z = (to.v[2] - self.v[2]) * (to.v[2] - self.v[2]);
    
        x + y + z
    }
}

///
/// Construct a new three-dimensional vector in the style of
/// a GLSL vec3 constructor.
///
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

impl<'a> From<Vec4> for Vec3 {
    #[inline]
    fn from(v: Vec4) -> Vec3 {
        Vec3::new(v.v[0], v.v[1], v.v[2])
    }
}

impl<'a> From<&'a Vec4> for Vec3 {
    #[inline]
    fn from(v: &'a Vec4) -> Vec3 {
        Vec3::new(v.v[0], v.v[1], v.v[2])
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
    pub v: [f32; 4],
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

impl cmp::PartialEq for Vec4 {
    fn eq(&self, other: &Vec4) -> bool {
        (f32::abs(self.v[0] - other.v[0]) < EPSILON) &&
        (f32::abs(self.v[1] - other.v[1]) < EPSILON) &&
        (f32::abs(self.v[2] - other.v[2]) < EPSILON) &&
        (f32::abs(self.v[3] - other.v[3]) < EPSILON)
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

    pub fn as_mut_ptr(&mut self) -> *mut f32 {
        self.m.as_mut_ptr()
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

impl convert::AsRef<[f32; 9]> for Mat3 {
    fn as_ref(&self) -> &[f32; 9] {
        &self.m
    }
}

impl convert::AsMut<[f32; 9]> for Mat3 {
    fn as_mut(&mut self) -> &mut [f32; 9] {
        &mut self.m
    }
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

    pub fn is_invertible(&self) -> bool {
        self.determinant() != 0.0
    }

    /* returns a 16-element array that is the inverse of a 16-element array (4x4
    matrix). see
    http://www.euclideanspace.com/maths/algebra/matrix/functions/inverse/fourD/index.htm
    */
    pub fn inverse(&self) -> Mat4 {
        let det = self.determinant();
        /* there is no inverse if determinant is zero (not likely unless scale is
        broken) */
        if det == 0.0 {
            eprintln!("WARNING. Matrix has zero determinant. It cannot be inverted.");
            
            return *self;
        }

        let inv_det = 1.0 / det;

        return mat4(
            inv_det * ( self.m[9] * self.m[14] * self.m[7] - self.m[13] * self.m[10] * self.m[7] +
                                    self.m[13] * self.m[6] * self.m[11] - self.m[5] * self.m[14] * self.m[11] -
                                    self.m[9] * self.m[6] * self.m[15] + self.m[5] * self.m[10] * self.m[15] ),
            inv_det * ( self.m[13] * self.m[10] * self.m[3] - self.m[9] * self.m[14] * self.m[3] -
                                    self.m[13] * self.m[2] * self.m[11] + self.m[1] * self.m[14] * self.m[11] +
                                    self.m[9] * self.m[2] * self.m[15] - self.m[1] * self.m[10] * self.m[15] ),
            inv_det * ( self.m[5] * self.m[14] * self.m[3] - self.m[13] * self.m[6] * self.m[3] +
                                    self.m[13] * self.m[2] * self.m[7] - self.m[1] * self.m[14] * self.m[7] -
                                    self.m[5] * self.m[2] * self.m[15] + self.m[1] * self.m[6] * self.m[15] ),
            inv_det * ( self.m[9] * self.m[6] * self.m[3] - self.m[5] * self.m[10] * self.m[3] -
                                    self.m[9] * self.m[2] * self.m[7] + self.m[1] * self.m[10] * self.m[7] +
                                    self.m[5] * self.m[2] * self.m[11] - self.m[1] * self.m[6] * self.m[11] ),
            inv_det * ( self.m[12] * self.m[10] * self.m[7] - self.m[8] * self.m[14] * self.m[7] -
                                    self.m[12] * self.m[6] * self.m[11] + self.m[4] * self.m[14] * self.m[11] +
                                    self.m[8] * self.m[6] * self.m[15] - self.m[4] * self.m[10] * self.m[15] ),
            inv_det * ( self.m[8] * self.m[14] * self.m[3] - self.m[12] * self.m[10] * self.m[3] +
                                    self.m[12] * self.m[2] * self.m[11] - self.m[0] * self.m[14] * self.m[11] -
                                    self.m[8] * self.m[2] * self.m[15] + self.m[0] * self.m[10] * self.m[15] ),
            inv_det * ( self.m[12] * self.m[6] * self.m[3] - self.m[4] * self.m[14] * self.m[3] -
                                    self.m[12] * self.m[2] * self.m[7] + self.m[0] * self.m[14] * self.m[7] +
                                    self.m[4] * self.m[2] * self.m[15] - self.m[0] * self.m[6] * self.m[15] ),
            inv_det * ( self.m[4] * self.m[10] * self.m[3] - self.m[8] * self.m[6] * self.m[3] +
                                    self.m[8] * self.m[2] * self.m[7] - self.m[0] * self.m[10] * self.m[7] -
                                    self.m[4] * self.m[2] * self.m[11] + self.m[0] * self.m[6] * self.m[11] ),
            inv_det * ( self.m[8] * self.m[13] * self.m[7] - self.m[12] * self.m[9] * self.m[7] +
                                    self.m[12] * self.m[5] * self.m[11] - self.m[4] * self.m[13] * self.m[11] -
                                    self.m[8] * self.m[5] * self.m[15] + self.m[4] * self.m[9] * self.m[15] ),
            inv_det * ( self.m[12] * self.m[9] * self.m[3] - self.m[8] * self.m[13] * self.m[3] -
                                    self.m[12] * self.m[1] * self.m[11] + self.m[0] * self.m[13] * self.m[11] +
                                    self.m[8] * self.m[1] * self.m[15] - self.m[0] * self.m[9] * self.m[15] ),
            inv_det * ( self.m[4] * self.m[13] * self.m[3] - self.m[12] * self.m[5] * self.m[3] +
                                    self.m[12] * self.m[1] * self.m[7] - self.m[0] * self.m[13] * self.m[7] -
                                    self.m[4] * self.m[1] * self.m[15] + self.m[0] * self.m[5] * self.m[15] ),
            inv_det * ( self.m[8] * self.m[5] * self.m[3] - self.m[4] * self.m[9] * self.m[3] -
                                    self.m[8] * self.m[1] * self.m[7] + self.m[0] * self.m[9] * self.m[7] +
                                    self.m[4] * self.m[1] * self.m[11] - self.m[0] * self.m[5] * self.m[11] ),
            inv_det * ( self.m[12] * self.m[9] * self.m[6] - self.m[8] * self.m[13] * self.m[6] -
                                    self.m[12] * self.m[5] * self.m[10] + self.m[4] * self.m[13] * self.m[10] +
                                    self.m[8] * self.m[5] * self.m[14] - self.m[4] * self.m[9] * self.m[14] ),
            inv_det * ( self.m[8] * self.m[13] * self.m[2] - self.m[12] * self.m[9] * self.m[2] +
                                    self.m[12] * self.m[1] * self.m[10] - self.m[0] * self.m[13] * self.m[10] -
                                    self.m[8] * self.m[1] * self.m[14] + self.m[0] * self.m[9] * self.m[14] ),
            inv_det * ( self.m[12] * self.m[5] * self.m[2] - self.m[4] * self.m[13] * self.m[2] -
                                    self.m[12] * self.m[1] * self.m[6] + self.m[0] * self.m[13] * self.m[6] +
                                    self.m[4] * self.m[1] * self.m[14] - self.m[0] * self.m[5] * self.m[14] ),
            inv_det * ( self.m[4] * self.m[9] * self.m[2] - self.m[8] * self.m[5] * self.m[2] +
                                    self.m[8] * self.m[1] * self.m[6] - self.m[0] * self.m[9] * self.m[6] -
                                    self.m[4] * self.m[1] * self.m[10] + self.m[0] * self.m[5] * self.m[10] ) );
    }

    ///
    /// Compute the perspective matrix for converting from camera space to 
    /// normalized device coordinates.
    ///
    pub fn perspective(fovy: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
        let fov_rad = fovy * ONE_DEG_IN_RAD;
        let range = f32::tan(fov_rad * 0.5) * near;
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

    /// 
    /// Generate a pointer to the underlying array for passing a
    /// matrix to the graphics hardware.
    ///
    pub fn as_ptr(&self) -> *const f32 {
        self.m.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut f32 {
        self.m.as_mut_ptr()
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

impl convert::AsRef<[f32; 16]> for Mat4 {
    fn as_ref(&self) -> &[f32; 16] {
        &self.m
    }
}

impl convert::AsMut<[f32; 16]> for Mat4 {
    fn as_mut(&mut self) -> &mut [f32; 16] {
        &mut self.m
    }
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

impl ops::Mul<Mat4> for Mat4 {
    type Output = Mat4;

    fn mul(self, other: Mat4) -> Mat4 {
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

impl cmp::PartialEq for Mat4 {
    fn eq(&self, other: &Mat4) -> bool {
        for i in 0..self.m.len() {
            if f32::abs(self.m[i] - other.m[i]) > EPSILON {
                return false;
            }
        }

        true
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Versor {
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

    pub fn to_mut_mat4(&self, m: &mut Mat4) {
        let w = self.q[0];
        let x = self.q[1];
        let y = self.q[2];
        let z = self.q[3];
        m.m[0] = 1.0 - 2.0 * y * y - 2.0 * z * z;
        m.m[1] = 2.0 * x * y + 2.0 * w * z;
        m.m[2] = 2.0 * x * z - 2.0 * w * y;
        m.m[3] = 0.0;
        m.m[4] = 2.0 * x * y - 2.0 * w * z;
        m.m[5] = 1.0 - 2.0 * x * x - 2.0 * z * z;
        m.m[6] = 2.0 * y * z + 2.0 * w * x;
        m.m[7] = 0.0;
        m.m[8] = 2.0 * x * z + 2.0 * w * y;
        m.m[9] = 2.0 * y * z - 2.0 * w * x;
        m.m[10] = 1.0 - 2.0 * x * x - 2.0 * y * y;
        m.m[11] = 0.0;
        m.m[12] = 0.0;
        m.m[13] = 0.0;
        m.m[14] = 0.0;
        m.m[15] = 1.0;
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


mod vec2_tests {
    
}

mod vec3_tests {
    use std::slice::Iter;
    use super::Vec3;

    struct TestCase {
        c: f32,
        x: Vec3,
        y: Vec3,
    }

    struct Test {
        tests: Vec<TestCase>,
    }

    impl Test {
        fn iter(&self) -> TestIter {
            TestIter {
                inner: self.tests.iter()
            }
        }
    }

    struct TestIter<'a> {
        inner: Iter<'a, TestCase>,
    }

    impl<'a> Iterator for TestIter<'a> {
        type Item = &'a TestCase;

        fn next(&mut self) -> Option<Self::Item> {
            self.inner.next()
        }
    }

    fn test_cases() -> Test {
        Test {
            tests: vec![
                TestCase {
                    c: 802.3435169,
                    x: super::vec3((80.0,  23.43, 43.569)),
                    y: super::vec3((6.741, 426.1, 23.5724)),
                },
                TestCase {
                    c: 33.249539,
                    x: super::vec3((27.6189, 13.90, 4.2219)),
                    y: super::vec3((258.083, 31.70, 42.17))
                },
                TestCase {
                    c: 7.04217,
                    x: super::vec3((70.0,  49.0,  95.0)),
                    y: super::vec3((89.9138, 36.84, 427.46894)),
                },
                TestCase {
                    c: 61.891390,
                    x: super::vec3((8827.1983, 89.5049494, 56.31)),
                    y: super::vec3((89.0, 72.0, 936.5)),
                }
            ]
        }
    }

    #[test]
    fn test_addition() {
        for test in test_cases().iter() {
            let expected = super::vec3((test.x.v[0] + test.y.v[0], test.x.v[1] + test.y.v[1], test.x.v[2] + test.y.v[2]));
            let result = test.x + test.y;
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_subtraction() {
        for test in test_cases().iter() {
            let expected = super::vec3((test.x.v[0] - test.y.v[0], test.x.v[1] - test.y.v[1], test.x.v[2] - test.y.v[2]));
            let result = test.x - test.y;
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_scalar_multiplication() {
        for test in test_cases().iter() {
            let expected = super::vec3((test.c * test.x.v[0], test.c * test.x.v[1], test.c * test.x.v[2]));
            let result = test.x * test.c;
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_scalar_division() {
        for test in test_cases().iter() {
            let expected = super::vec3((test.x.v[0] / test.c, test.x.v[1] / test.c, test.x.v[2] / test.c));
            let result = test.x / test.c;
            assert_eq!(result, expected);
        }
    }
}

mod mat4_tests {
    use std::slice::Iter;
    use super::{Vec3, Mat4};

    struct TestCase {
        c: f32,
        a_mat: Mat4,
        b_mat: Mat4,
    }

    struct Test {
        tests: Vec<TestCase>,
    }

    impl Test {
        fn iter(&self) -> TestIter {
            TestIter {
                inner: self.tests.iter()
            }
        }
    }

    struct TestIter<'a> {
        inner: Iter<'a, TestCase>,
    }

    impl<'a> Iterator for TestIter<'a> {
        type Item = &'a TestCase;

        fn next(&mut self) -> Option<Self::Item> {
            self.inner.next()
        }
    }

    fn test_cases() -> Test {
        Test {
            tests: vec![
                TestCase {
                    c: 802.3435169,
                    a_mat: super::mat4(
                        80.0,   23.43,   43.569,  6.741, 
                        426.1,  23.5724, 27.6189, 13.90,
                        4.2219, 258.083, 31.70,   42.17, 
                        70.0,   49.0,    95.0,    89.9138
                    ),
                    b_mat: super::mat4(
                        36.84,   427.46894, 8827.1983, 89.5049494, 
                        7.04217, 61.891390, 56.31,     89.0, 
                        72.0,    936.5,     413.80,    50.311160,  
                        37.6985,  311.8,    60.81,     73.8393
                    ),
                },
                TestCase {
                    c: 6.2396,
                    a_mat: Mat4::identity(),
                    b_mat: Mat4::identity(),
                },
                TestCase {
                    c: 6.2396,
                    a_mat: Mat4::zero(),
                    b_mat: Mat4::zero(),
                },
                TestCase {
                    c:  14.5093,
                    a_mat: super::mat4(
                        68.32, 0.0,    0.0,   0.0,
                        0.0,   37.397, 0.0,   0.0,
                        0.0,   0.0,    9.483, 0.0,
                        0.0,   0.0,    0.0,   887.710
                    ),
                    b_mat: super::mat4(
                        57.72, 0.0,       0.0,       0.0, 
                        0.0,   9.5433127, 0.0,       0.0, 
                        0.0,   0.0,       86.731265, 0.0,
                        0.0,   0.0,       0.0,       269.1134546
                    )
                },
            ]
        }
    }

    #[test]
    fn test_mat_times_identity_equals_mat() {
        for test in test_cases().iter() {
            let a_mat_times_identity = test.a_mat * Mat4::identity();
            let b_mat_times_identity = test.b_mat * Mat4::identity();

            assert_eq!(a_mat_times_identity, test.a_mat);
            assert_eq!(b_mat_times_identity, test.b_mat);
        }
    }

    #[test]
    fn test_mat_times_zero_equals_zero() {
        for test in test_cases().iter() {
            let a_mat_times_zero = test.a_mat * Mat4::zero();
            let b_mat_times_zero = test.b_mat * Mat4::zero();

            assert_eq!(a_mat_times_zero, Mat4::zero());
            assert_eq!(b_mat_times_zero, Mat4::zero());
        }
    }

    #[test]
    fn test_zero_times_mat_equals_zero() {
        for test in test_cases().iter() {
            let zero_times_a_mat = Mat4::zero() * test.a_mat;
            let zero_times_b_mat = Mat4::zero() * test.b_mat;

            assert_eq!(zero_times_a_mat, Mat4::zero());
            assert_eq!(zero_times_b_mat, Mat4::zero());
        }
    }

    #[test]
    fn test_mat_times_identity_equals_identity_times_mat() {
        for test in test_cases().iter() {
            let a_mat_times_identity = test.a_mat * Mat4::identity();
            let identity_times_a_mat = Mat4::identity() * test.a_mat;
            let b_mat_times_identity = test.b_mat * Mat4::identity();
            let identity_times_b_mat = Mat4::identity() * test.b_mat;

            assert_eq!(a_mat_times_identity, identity_times_a_mat);
            assert_eq!(b_mat_times_identity, identity_times_b_mat);
        }
    }

    #[test]
    fn test_mat_times_mat_inverse_equals_identity() {
        for test in test_cases().iter() {
            let identity = Mat4::identity();
            if test.a_mat.is_invertible() {
                let a_mat_inverse = test.a_mat.inverse();
                assert_eq!(a_mat_inverse * test.a_mat, identity);
            }
            if test.b_mat.is_invertible() {
                let b_mat_inverse = test.b_mat.inverse();
                assert_eq!(b_mat_inverse * test.b_mat, identity);
            }
        }
    }

    #[test]
    fn test_mat_inverse_times_mat_equals_identity() {
        for test in test_cases().iter() {
            let identity = Mat4::identity();
            if test.a_mat.is_invertible() {
                let a_mat_inverse = test.a_mat.inverse();
                assert_eq!(test.a_mat * a_mat_inverse, identity);
            }
            if test.b_mat.is_invertible() {
                let b_mat_inverse = test.b_mat.inverse();
                assert_eq!(test.b_mat * b_mat_inverse, identity);
            }
        }
    }

    #[test]
    fn test_mat_transpose_transpose_equals_mat() {
        for test in test_cases().iter() {
            let a_mat_tr_tr = test.a_mat.transpose().transpose();
            let b_mat_tr_tr = test.b_mat.transpose().transpose();
            
            assert_eq!(a_mat_tr_tr, test.a_mat);
            assert_eq!(b_mat_tr_tr, test.b_mat);
        }
    }

    #[test]
    fn test_identity_transpose_equals_identity() {
        let identity = Mat4::identity();
        let identity_tr = identity.transpose();
            
        assert_eq!(identity, identity_tr);
    }

    #[test]
    fn test_identity_mat4_translates_vector_along_vector() {
        let v = super::vec3((2.0, 2.0, 2.0));
        let trans_mat = Mat4::identity().translate(&v);
        let zero_vec4 = super::vec4((0.0, 0.0, 0.0, 1.0));
        let zero_vec3 = super::vec3((0.0, 0.0, 0.0));

        let result = trans_mat * zero_vec4;
        assert_eq!(result, super::vec4((zero_vec3 + v, 1.0)));
    }
}

