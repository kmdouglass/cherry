/// A 2 x 2 matrix
use crate::core::Float;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat2x2 {
    pub e: [[Float; 2]; 2],
}

impl Mat2x2 {
    pub fn new(e00: Float, e01: Float, e10: Float, e11: Float) -> Self {
        Self {
            e: [[e00, e01], [e10, e11]],
        }
    }
}

impl std::ops::Mul<Mat2x2> for Mat2x2 {
    type Output = Mat2x2;

    fn mul(self, rhs: Mat2x2) -> Mat2x2 {
        Mat2x2::new(
            self.e[0][0] * rhs.e[0][0] + self.e[0][1] * rhs.e[1][0],
            self.e[0][0] * rhs.e[0][1] + self.e[0][1] * rhs.e[1][1],
            self.e[1][0] * rhs.e[0][0] + self.e[1][1] * rhs.e[1][0],
            self.e[1][0] * rhs.e[0][1] + self.e[1][1] * rhs.e[1][1],
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mat2_mul_mat2() {
        let mat1 = Mat2x2::new(1.0, 2.0, 3.0, 4.0);
        let mat2 = Mat2x2::new(5.0, 6.0, 7.0, 8.0);
        let res = mat1 * mat2;
        assert_eq!(res, Mat2x2::new(19.0, 22.0, 43.0, 50.0));
    }

    #[test]
    fn mat2_identity_mul() {
        let identity = Mat2x2::new(1.0, 0.0, 0.0, 1.0);
        let mat = Mat2x2::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(mat * identity, mat);
        assert_eq!(identity * mat, mat);
    }
}
