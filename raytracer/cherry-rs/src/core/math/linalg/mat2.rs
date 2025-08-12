/// A 2 x 2 matrix.
use std::ops::Index;

use crate::core::{Float, math::vec2::Vec2};

#[derive(Debug)]
pub struct Mat2 {
    col_1: Vec2,
    col_2: Vec2,
}

impl Mat2 {
    pub fn new(e00: Float, e01: Float, e10: Float, e11: Float) -> Self {
        Self {
            col_1: Vec2 { x: e00, y: e10 },
            col_2: Vec2 { x: e01, y: e11 },
        }
    }

    /// Computes the determinant of the matrix.
    pub fn determinant(&self) -> Float {
        self.col_1.x * self.col_2.y - self.col_1.y * self.col_2.x
    }

    /// Computes the trace of the matrix.
    pub fn trace(&self) -> Float {
        self.col_1.x + self.col_2.y
    }
}

impl Index<usize> for Mat2 {
    type Output = Vec2;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.col_1,
            1 => &self.col_2,
            _ => panic!("Index out of bounds for Mat2"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mat2_new() {
        let m = Mat2::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(m[0][0], 1.0);
        assert_eq!(m[0][1], 2.0);
        assert_eq!(m[1][0], 3.0);
        assert_eq!(m[1][1], 4.0);
    }

    #[test]
    fn mat2_determinant() {
        let m = Mat2::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(m.determinant(), -2.0);
    }
}
