/// A 3 x 3 matrix
use crate::core::{Float, math::vec3::Vec3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat3 {
    e: [[Float; 3]; 3],
}

impl Mat3 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        e00: Float,
        e01: Float,
        e02: Float,
        e10: Float,
        e11: Float,
        e12: Float,
        e20: Float,
        e21: Float,
        e22: Float,
    ) -> Self {
        Self {
            e: [[e00, e01, e02], [e10, e11, e12], [e20, e21, e22]],
        }
    }

    /// Determines whether all elements of a matrix are approximately equal to
    /// another.
    pub fn approx_eq(&self, other: &Self, tol: Float) -> bool {
        self.e
            .iter()
            .zip(other.e.iter())
            .all(|(row_self, row_other)| {
                row_self
                    .iter()
                    .zip(row_other.iter())
                    .all(|(a, b)| (a - b).abs() < tol)
            })
    }

    /// Create a 3x3 identity matrix.
    pub fn identity() -> Self {
        Self::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0)
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

    /// Compute a 3x3 rotation matrix from Tait-Bryan Euler angles.
    ///
    /// The following conventions are used:
    /// - Intrinsic rotations are used
    /// - Rotations are specified by the Tait-Bryan triplet z-y'-x'' where the
    ///   first rotation is about the z-axis, the second rotation is about the
    ///   y'-axis, and the third rotation is about the x''-axis.
    /// - Right-handed coordinate systems are used
    /// - Counterclockwise rotations are positive
    /// - Active rotations are used
    pub fn from_euler_angles(k: Float, l: Float, m: Float) -> Self {
        let (sin_k, cos_k) = k.sin_cos();
        let (sin_l, cos_l) = l.sin_cos();
        let (sin_m, cos_m) = m.sin_cos();

        Self::new(
            cos_l * cos_m,
            sin_k * sin_l * cos_m - cos_k * sin_m,
            cos_k * sin_l * cos_m + sin_k * sin_m,
            cos_l * sin_m,
            sin_k * sin_l * sin_m + cos_k * cos_m,
            cos_k * sin_l * sin_m - sin_k * cos_m,
            -sin_l,
            sin_k * cos_l,
            cos_k * cos_l,
        )
    }
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
    use super::*;

    #[test]
    fn test_mat3_mul_vec3() {
        use super::*;

        let mat = Mat3::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7., 8.0, 9.0);
        let vec = Vec3::new(1.0, 2.0, 3.0);

        let res = mat * vec;

        assert_eq!(res, Vec3::new(14.0, 32.0, 50.0));
    }

    #[test]
    fn test_mat3_transpose() {
        use super::*;

        let mat = Mat3::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7., 8.0, 9.0);

        let res = mat.transpose();

        assert_eq!(res, Mat3::new(1.0, 4.0, 7.0, 2.0, 5.0, 8.0, 3., 6.0, 9.0));
    }

    #[test]
    fn test_mat3_from_euler_angles() {
        let (k, l, m) = (0.0, 0.0, 0.0); // no rotation
        let mat = Mat3::from_euler_angles(k, l, m);
        assert_eq!(mat, Mat3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0));
    }

    #[test]
    fn test_mat3_identity() {
        let mat = Mat3::identity();
        assert_eq!(mat, Mat3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0));
    }
}
