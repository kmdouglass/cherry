use crate::core::{Float, RefractiveIndex};
use crate::specs::gaps::GapSpec;

/// Data types for modeling sequential ray tracing systems.
pub(crate) struct Gap {
    thickness: Float,
    refractive_index: RefractiveIndex,
}

impl Gap {
    fn from_spec(spec: &GapSpec, wavelength: Float) -> Self {
        let thickness = spec.thickness;
        let refractive_index = RefractiveIndex::from_spec(&spec.refractive_index, wavelength);
        Self {
            thickness,
            refractive_index,
        }
    }
}
