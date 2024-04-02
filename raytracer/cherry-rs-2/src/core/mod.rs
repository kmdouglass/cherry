/// Shared data types for modeling ray tracing systems.
pub(super) mod math;

use crate::specs::materials::{DataSpec, RefractiveIndexSpec};

use math::Complex;

pub(crate) type Float = f64;

#[derive(Debug)]
pub(crate) struct RefractiveIndex {
    eta: Complex<Float>,
}

impl RefractiveIndex {
    pub(crate) fn from_spec(spec: &RefractiveIndexSpec, wavelength: Float) -> Self {
        unimplemented!("TODO: Implement RefractiveIndex::from_spec")
    }
}
