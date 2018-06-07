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

pub struct Mat3 {
    v: [f32; 12],
}

pub struct Mat4 {
    v: [f32; 16],
}

