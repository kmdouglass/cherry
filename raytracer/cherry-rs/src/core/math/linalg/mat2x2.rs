/// A 2 x 2 matrix.
use std::ops::{Index, Mul};

use anyhow::{Result, anyhow};

use crate::core::{
    Float,
    math::{constants::ZERO_TOL, vec2::Vec2},
};

use super::quadratic::NormalizedQuadratic;

#[derive(Debug, Clone, Copy)]
pub struct Mat2x2 {
    row_0: Vec2,
    row_1: Vec2,
}

impl Mat2x2 {
    /// Creates a new 2x2 matrix with the specified elements.
    pub fn new(e00: Float, e01: Float, e10: Float, e11: Float) -> Self {
        Self {
            row_0: Vec2 { x: e00, y: e01 },
            row_1: Vec2 { x: e10, y: e11 },
        }
    }

    /// Determines whether all elements of a matrix are approximately equal to
    /// another.
    pub fn approx_eq(&self, other: &Self, tol: Float) -> bool {
        self.row_0.approx_eq(&other.row_0, tol) && self.row_1.approx_eq(&other.row_1, tol)
    }

    /// Computes the determinant of the matrix.
    pub fn determinant(&self) -> Float {
        self.row_0.x * self.row_1.y - self.row_0.y * self.row_1.x
    }

    /// Computes the eigenvalues and eigenvectors of the matrix.
    ///
    /// The smaller eigenvalue is returned first.
    ///
    /// # Returns
    /// A tuple containing:
    ///
    /// - The smaller eigenvalue
    /// - The larger eigenvalue
    /// - A 2x2 matrix with the rows being the normalized eigenvectors
    ///   corresponding to the eigenvalues.
    pub fn eig(&self) -> Result<(Float, Float, Mat2x2)> {
        let characteristic_polynomial = NormalizedQuadratic::new(-self.trace(), self.determinant());

        let (lambda1, lambda2) = characteristic_polynomial.roots()?;

        let (eigenvector1, eigenvector2) = (self.eigenvector(lambda1)?, self.eigenvector(lambda2)?);

        Ok((
            lambda1,
            lambda2,
            Mat2x2 {
                row_0: eigenvector1,
                row_1: eigenvector2,
            },
        ))
    }

    fn eigenvector(&self, eigenvalue: Float) -> Result<Vec2> {
        let eigenvalue_minus_a_00 = eigenvalue - self.row_0.x;
        let a_01 = self.row_0.y;
        let a_10 = self.row_1.x;
        let eigenvalue_minus_a_11 = eigenvalue - self.row_1.y;

        // Use the first available non-zero entry to compute the eigenvector.
        let (x, y) = if a_01.abs() > ZERO_TOL {
            (a_01 / eigenvalue_minus_a_00, 1.0)
        } else if a_10.abs() > ZERO_TOL {
            (1.0, a_10 / eigenvalue_minus_a_11)
        } else if eigenvalue_minus_a_00.abs() > ZERO_TOL {
            (1.0, 0.0)
        } else if eigenvalue_minus_a_11.abs() > ZERO_TOL {
            (0.0, 1.0)
        } else {
            return Err(anyhow!("Cannot compute eigenvector for zero eigenvalue"));
        };

        let mut eigenvector = Vec2 { x, y };
        eigenvector.normalize();
        Ok(eigenvector)
    }

    /// Returns the 2x2 identity matrix.
    ///
    /// ```text
    /// | 1.0, 0.0 |
    /// | 0.0, 1.0 |
    /// ```
    pub fn identity() -> Self {
        Self {
            row_0: Vec2 { x: 1.0, y: 0.0 },
            row_1: Vec2 { x: 0.0, y: 1.0 },
        }
    }

    /// Determines whether the matrix is invertible.
    pub fn is_invertible(&self) -> bool {
        self.determinant().abs() > ZERO_TOL
    }

    /// Determines whether the matrix is orthonormal.
    pub fn is_orthonormal(&self) -> bool {
        let row_0_length_squared = self.row_0.length_squared();
        let row_1_length_squared = self.row_1.length_squared();

        let is_orthogonal = self.row_0.dot(&self.row_1).abs() < ZERO_TOL;

        (row_0_length_squared - 1.0).abs() < ZERO_TOL
            && (row_1_length_squared - 1.0).abs() < ZERO_TOL
            && is_orthogonal
    }

    /// Determines whether the matrix rows are normalized.
    pub fn is_row_normalized(&self) -> bool {
        let row_0_length_squared = self.row_0.length_squared();
        let row_1_length_squared = self.row_1.length_squared();

        (row_0_length_squared - 1.0).abs() < ZERO_TOL
            && (row_1_length_squared - 1.0).abs() < ZERO_TOL
    }

    /// Computes the trace of the matrix.
    pub fn trace(&self) -> Float {
        self.row_0.x + self.row_1.y
    }

    /// Computes the transpose of the matrix.
    pub fn transpose(&self) -> Self {
        Self {
            row_0: Vec2 {
                x: self.row_0.x,
                y: self.row_1.x,
            },
            row_1: Vec2 {
                x: self.row_0.y,
                y: self.row_1.y,
            },
        }
    }
}

impl Index<usize> for Mat2x2 {
    type Output = Vec2;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.row_0,
            1 => &self.row_1,
            _ => panic!("Index out of bounds for Mat2"),
        }
    }
}

impl Mul<Vec2> for Mat2x2 {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self.row_0.x * rhs.x + self.row_0.y * rhs.y,
            y: self.row_1.x * rhs.x + self.row_1.y * rhs.y,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mat2x2_new() {
        let m = Mat2x2::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(m[0][0], 1.0);
        assert_eq!(m[0][1], 2.0);
        assert_eq!(m[1][0], 3.0);
        assert_eq!(m[1][1], 4.0);
    }

    #[test]
    fn mat2x2_determinant() {
        let m = Mat2x2::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(m.determinant(), -2.0);
    }

    #[test]
    fn mat2x2_eig() {
        const TOL: Float = 1e8;
        let m = Mat2x2::new(4.0, 2.0, 1.0, 3.0);
        let expected_eigenvalues = (2.0, 5.0);
        let expected_eigenvectors = (
            Vec2 {
                x: -0.70710678,
                y: 0.70710678,
            },
            Vec2 {
                x: 0.89442719,
                y: 0.4472136,
            },
        );

        let (eigenvalue0, eigenvalue1, eigenvectors) = m.eig().unwrap();

        assert!((eigenvalue0 - expected_eigenvalues.0).abs() < TOL);
        assert!((eigenvalue1 - expected_eigenvalues.1).abs() < TOL);

        assert!((eigenvectors.row_0.x - expected_eigenvectors.0.x).abs() < TOL);
        assert!((eigenvectors.row_0.y - expected_eigenvectors.0.y).abs() < TOL);
        assert!((eigenvectors.row_1.x - expected_eigenvectors.1.x).abs() < TOL);
        assert!((eigenvectors.row_1.y - expected_eigenvectors.1.y).abs() < TOL);
    }

    #[test]
    fn mat2x2_eigenvector() {
        const TOL: Float = 1e8;
        let m = Mat2x2::new(4.0, 2.0, 1.0, 3.0);
        let eigenvalue = 5.0;
        let expected = (0.89442719, 0.4472136);

        let eigenvector = m.eigenvector(eigenvalue).unwrap();

        assert!((eigenvector.x - expected.0).abs() < TOL);
        assert!((eigenvector.y - expected.1).abs() < TOL);
    }

    #[test]
    fn mat2x2_eigenvectors_are_normalized() {
        let m = Mat2x2::new(4.0, 2.0, 1.0, 3.0);
        let (_eigenvalue0, _eigenvalue1, eigenvectors) = m.eig().unwrap();

        assert!(eigenvectors.is_row_normalized());
    }

    #[test]
    fn mat2x2_is_invertible() {
        let m = Mat2x2::new(1.0, 2.0, 3.0, 4.0);
        assert!(m.is_invertible());

        let m2 = Mat2x2::new(1.0, 2.0, 2.0, 4.0);
        assert!(!m2.is_invertible());
    }

    #[test]
    fn mat2x2_is_orthonormal() {
        let m = Mat2x2::new(1.0, 0.0, 0.0, 1.0);
        assert!(m.is_orthonormal());

        let m2 = Mat2x2::new(0.0, 1.0, -1.0, 0.0);
        assert!(m2.is_orthonormal());

        let m3 = Mat2x2::new(1.0, 1.0, 1.0, 1.0);
        assert!(!m3.is_orthonormal());

        let m4 = Mat2x2::new(
            2.0_f64.sqrt() / 2.0,
            -2.0_f64.sqrt() / 2.0,
            2.0_f64.sqrt() / 2.0,
            2.0_f64.sqrt() / 2.0,
        );
        assert!(m4.is_orthonormal());
    }

    #[test]
    fn mat2x2_multiply_with_vec2() {
        let m = Mat2x2::new(1.0, 2.0, 3.0, 4.0);
        let v = Vec2 { x: 5.0, y: 6.0 };
        let result = m * v;

        assert_eq!(result.x, 17.0); // 1*5 + 2*6
        assert_eq!(result.y, 39.0); // 3*5 + 4*6
    }

    #[test]
    fn mat2x2_trace() {
        let m = Mat2x2::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(m.trace(), 5.0);
    }
}
