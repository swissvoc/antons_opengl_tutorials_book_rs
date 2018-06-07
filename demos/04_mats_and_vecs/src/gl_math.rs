use std::fmt;
use std::ops;


pub struct Vec2 {
    v: [f32; 2],
}

impl Vec2 {
    fn new(x: f32, y: f32) -> Vec2 {
        Vec2 { v: [x, y] }
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

pub struct Vec3 {
    v: [f32; 3],
}

impl Vec3 {
    fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 { v: [x, y, z] }
    }

    fn zero() -> Vec3 {
        Vec3 { v: [0.0, 0.0, 0.0] }
    }
}

#[inline]
pub fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

impl fmt::Display for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:.2}, {:.2}, {:.2}]", self.v[0], self.v[1], self.v[2])
    }
}

fn length(v: &Vec3) -> f32 {
    f32::sqrt(v.v[0] * v.v[0] + v.v[1] * v.v[1] + v.v[2] * v.v[2])
}

// Squared length.
fn length2(v: &Vec3) -> f32 {
    v.v[0] * v.v[0] + v.v[1] * v.v[1] + v.v[2] * v.v[2]
}

fn normalize(v: &Vec3) -> Vec3 {
    let norm_v = length(v);
    if norm_v == 0.0 {
        return Vec3::zero();
    }

    Vec3::new(v.v[0] / norm_v, v.v[1] / norm_v, v.v[2] / norm_v)
}

fn dot(a: &Vec3, b: &Vec3) -> f32 {
    a.v[0] * b.v[0] + a.v[1] * b.v[1] + a.v[2] * b.v[2]
}

fn cross(a: &Vec3, b: &Vec3) -> Vec3 {
    let x = a.v[1] * b.v[2] - a.v[2] * b.v[1];
    let y = a.v[2] * b.v[0] - a.v[0] * b.v[2];
    let z = a.v[0] * b.v[1] - a.v[1] * b.v[0];
    
    Vec3::new(x, y, z)
}

fn get_squared_dist(from: Vec3, to: Vec3) -> f32 {
    let x = ( to.v[0] - from.v[0] ) * ( to.v[0] - from.v[0] );
    let y = ( to.v[1] - from.v[1] ) * ( to.v[1] - from.v[1] );
    let z = ( to.v[2] - from.v[2] ) * ( to.v[2] - from.v[2] );
    
    x + y + z
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

pub struct Vec4 {
    v: [f32; 4],
}

impl Vec4 {
    fn new(x: f32, y: f32, z: f32, w: f32) -> Vec4 {
        Vec4 { v: [x, y, z, w] }
    }
}

#[inline]
pub fn vec4(x: f32, y: f32, z: f32, w: f32) -> Vec4 {
    Vec4::new(x, y, z, w)
}

impl fmt::Display for Vec4 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:.2}, {:.2}, {:.2}, {:.2}]", self.v[0], self.v[1], self.v[2], self.v[3])
    }
}

///
/// The `Mat3` type represents 3x3 matrices in column-major order.
///
pub struct Mat3 {
    v: [f32; 9],
}

impl Mat3 {
    fn new(m11: f32, m12: f32, m13: f32, 
           m21: f32, m22: f32, m23: f32, 
           m31: f32, m32: f32, m33: f32) -> Mat3 {

        Mat3 {
            v: [
                m11, m12, m13, // Column 1
                m21, m22, m23, // Column 2
                m31, m32, m33  // Column 3
            ]
        }
    }

    fn zero() -> Mat3 {
        Mat3::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
    }

    fn identity() -> Mat3 {
        Mat3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0)
    }
}

impl fmt::Display for Mat3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, 
            "\n[{:.2}][{:.2}][{:.2}]\n[{:.2}][{:.2}][{:.2}]\n[{:.2}][{:.2}][{:.2}]", 
            self.v[0], self.v[3], self.v[6],
            self.v[1], self.v[4], self.v[7],
            self.v[2], self.v[5], self.v[8],
        )
    }
}

///
/// The `Mat4` type represents 4x4 matrices in column-major order.
///
pub struct Mat4 {
    v: [f32; 16],
}

impl Mat4 {
    fn new(m11: f32, m12: f32, m13: f32, m14: f32,
           m21: f32, m22: f32, m23: f32, m24: f32,
           m31: f32, m32: f32, m33: f32, m34: f32,
           m41: f32, m42: f32, m43: f32, m44: f32) -> Mat4 {

        Mat4 {
            v: [
                m11, m12, m13, m14, // Column 1
                m21, m22, m23, m24, // Column 2
                m31, m32, m33, m34, // Column 3
                m41, m42, m43, m44  // Column 4
            ]
        }
    }

    fn zero() -> Mat4 {
        Mat4::new(
            0.0, 0.0, 0.0, 0.0, 
            0.0, 0.0, 0.0, 0.0, 
            0.0, 0.0, 0.0, 0.0, 
            0.0, 0.0, 0.0, 0.0
        )
    }

    fn identity() -> Mat4 {
        Mat4::new(
            1.0, 0.0, 0.0, 0.0, 
            0.0, 1.0, 0.0, 0.0, 
            0.0, 0.0, 1.0, 0.0, 
            0.0, 0.0, 0.0, 1.0
        )
    }
}

impl fmt::Display for Mat4 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, 
            "\n[{:.2}][{:.2}][{:.2}][{:.2}]\n[{:.2}][{:.2}][{:.2}][{:.2}]\n[{:.2}][{:.2}][{:.2}][{:.2}]\n[{:.2}][{:.2}][{:.2}][{:.2}]", 
            self.v[0], self.v[4], self.v[8],  self.v[12],
            self.v[1], self.v[5], self.v[9],  self.v[13],
            self.v[2], self.v[6], self.v[10], self.v[14],
            self.v[3], self.v[7], self.v[11], self.v[15]
        )
    }
}
