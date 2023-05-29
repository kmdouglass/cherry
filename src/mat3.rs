/// A 3 x 3 matrix
use std::ops;

use crate::vec3::Vec3;

#[derive(Debug)]
pub struct Mat3 {
    e: [[f32; 3]; 3],
}

impl Mat3 {
    pub fn new(
        e00: f32,
        e01: f32,
        e02: f32,
        e10: f32,
        e11: f32,
        e12: f32,
        e20: f32,
        e21: f32,
        e22: f32,
    ) -> Self {
        Self {
            e: [[e00, e01, e02], [e10, e11, e12], [e20, e21, e22]],
        }
    }

    pub fn transpose(&self) -> Self {
        Self::new(
            self.e[0][0],
            self.e[1][0],
            self.e[2][0],
            self.e[0][1],
            self.e[1][1],
            self.e[2][1],
            self.e[0][2],
            self.e[1][2],
            self.e[2][2],
        )
    }
}

/// Create a new 3x3 matrix in row-major order.
macro_rules! mat3 {
    ($e00:expr, $e01:expr, $e02:expr, $e10:expr, $e11:expr, $e12:expr, $e20:expr, $e21:expr, $e22:expr) => {
        Mat3::new($e00, $e01, $e02, $e10, $e11, $e12, $e20, $e21, $e22)
    };
}

impl std::ops::Mul<Vec3> for Mat3 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3::new(
            self.e[0][0] * rhs.x() + self.e[0][1] * rhs.y() + self.e[0][2] * rhs.z(),
            self.e[1][0] * rhs.x() + self.e[1][1] * rhs.y() + self.e[1][2] * rhs.z(),
            self.e[2][0] * rhs.x() + self.e[2][1] * rhs.y() + self.e[2][2] * rhs.z(),
        )
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_mat3_mul_vec3() {
        use super::*;

        let mat = mat3!(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7., 8.0, 9.0);
        let vec = Vec3::new(1.0, 2.0, 3.0);

        let res = mat * vec;

        assert_eq!(res, Vec3::new(14.0, 32.0, 50.0));
    }
}
