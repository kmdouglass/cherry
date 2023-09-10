#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat2 {
    e: [[f32; 2]; 2],
}

impl Mat2 {
    pub fn new(
        e00: f32,
        e01: f32,
        e10: f32,
        e11: f32,
    ) -> Self {
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
