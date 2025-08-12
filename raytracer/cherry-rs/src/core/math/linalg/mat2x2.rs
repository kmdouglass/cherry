/// A 2 x 2 matrix.
use std::ops::Index;

use crate::core::{Float, math::vec2::Vec2};

#[derive(Debug)]
pub struct Mat2x2 {
    row_0: Vec2,
    row_1: Vec2,
}

impl Mat2x2 {
    pub fn new(e00: Float, e01: Float, e10: Float, e11: Float) -> Self {
        Self {
            row_0: Vec2 { x: e00, y: e01 },
            row_1: Vec2 { x: e10, y: e11 },
        }
    }

    /// Computes the determinant of the matrix.
    pub fn determinant(&self) -> Float {
        self.row_0.x * self.row_1.y - self.row_0.y * self.row_1.x
    }

    /// Computes the trace of the matrix.
    pub fn trace(&self) -> Float {
        self.row_0.x + self.row_1.y
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mat2_new() {
        let m = Mat2x2::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(m[0][0], 1.0);
        assert_eq!(m[0][1], 2.0);
        assert_eq!(m[1][0], 3.0);
        assert_eq!(m[1][1], 4.0);
    }

    #[test]
    fn mat2_determinant() {
        let m = Mat2x2::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(m.determinant(), -2.0);
    }
}
