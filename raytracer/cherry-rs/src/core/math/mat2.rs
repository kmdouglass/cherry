/// A 2 x 2 matrix
use crate::core::Float;

#[derive(Debug)]
pub struct Mat2 {
    pub e: [[Float; 2]; 2],
}

impl Mat2 {
    pub fn new(e00: Float, e01: Float, e10: Float, e11: Float) -> Self {
        Self {
            e: [[e00, e01], [e10, e11]],
        }
    }

    /// Computes the determinant of the matrix.
    pub fn determinant(&self) -> Float {
        self.e[0][0] * self.e[1][1] - self.e[0][1] * self.e[1][0]
    }

    /// Computes the trace of the matrix.
    pub fn trace(&self) -> Float {
        self.e[0][0] + self.e[1][1]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mat2_new() {
        let m = Mat2::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(m.e[0][0], 1.0);
        assert_eq!(m.e[0][1], 2.0);
        assert_eq!(m.e[1][0], 3.0);
        assert_eq!(m.e[1][1], 4.0);
    }

    #[test]
    fn mat2_determinant() {
        let m = Mat2::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(m.determinant(), -2.0);
    }
}
