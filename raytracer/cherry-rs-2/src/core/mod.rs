/// Shared data types for modeling ray tracing systems.
pub(super) mod math;
pub(crate) mod seq;

use crate::specs::gaps::{RIDataSpec, RefractiveIndexSpec};

use math::Complex;

pub(crate) type Float = f64;

#[derive(Debug)]
pub(crate) struct RefractiveIndex {
    eta: Complex<Float>,
}

impl RefractiveIndex {
    pub(crate) fn from_spec(spec: &RefractiveIndexSpec, wavelength: Float) -> Self {
        match spec {
            RefractiveIndexSpec::N { n } => {
                unimplemented!();
            }
            RefractiveIndexSpec::NAndKSeparate { n, k } => {
                unimplemented!();
            }
            RefractiveIndexSpec::NAndKTogether { nk } => {
                unimplemented!();
            }
        }
    }
}
