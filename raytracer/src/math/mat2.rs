use super::vec2::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat2 {
    e: [[f32; 2]; 2],
}

impl Mat2 {
    pub fn new(e00: f32, e01: f32, e10: f32, e11: f32) -> Self {
        Self {
            e: [[e00, e01], [e10, e11]],
        }
    }

    /// Return the identity matrix.
    pub fn eye() -> Self {
        Self {
            e: [[1.0, 0.0], [0.0, 1.0]],
        }
    }
}

macro_rules! mat2 {
    ($e00:expr, $e01:expr, $e10:expr, $e11:expr) => {
        Mat2::new($e00, $e01, $e10, $e11)
    };
}

pub(crate) use mat2;

impl std::ops::Mul<&Vec2> for &Mat2 {
    type Output = Vec2;

    fn mul(self, rhs: &Vec2) -> Vec2 {
        let e00 = self.e[0][0];
        let e01 = self.e[0][1];
        let e10 = self.e[1][0];
        let e11 = self.e[1][1];

        let y = e00 * rhs.y() + e01 * rhs.theta();
        let theta = e10 * rhs.y() + e11 * rhs.theta();

        Vec2::new(y, theta)
    }
}

impl std::ops::Index<usize> for Mat2 {
    type Output = [f32];

    fn index(&self, row_index: usize) -> &[f32] {
        assert!(row_index < 2);

        &self.e[row_index]
    }
}