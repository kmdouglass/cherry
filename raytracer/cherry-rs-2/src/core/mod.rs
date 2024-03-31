/// Shared data types for modeling ray tracing systems.
pub(super) mod math;

use math::Complex;

pub(crate) type Float = f64;

#[derive(Debug)]
pub(crate) struct RefractiveIndex {
    eta: Complex<Float>,
}

impl RefractiveIndex {
    pub(crate) fn new(n: Float, k: Float) -> Self {
        Self {
            eta: Complex { real: n, imag: k },
        }
    }
}
