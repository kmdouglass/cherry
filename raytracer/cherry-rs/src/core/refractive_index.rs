//! A material's refractive index.
use anyhow::{anyhow, Result};

use crate::core::{math::Complex, Float};
use crate::specs::gaps::RefractiveIndexSpec;

#[derive(Debug, Clone, Copy)]
pub struct RefractiveIndex {
    eta: Complex<Float>,
}

impl RefractiveIndex {
    pub(crate) fn new(n: Float, k: Float) -> Self {
        Self {
            eta: Complex { real: n, imag: k },
        }
    }

    pub(crate) fn n(&self) -> Float {
        self.eta.real
    }

    pub(crate) fn k(&self) -> Float {
        self.eta.imag
    }

    pub(crate) fn try_from_spec(spec: &dyn RefractiveIndexSpec, wavelength: Float) -> Result<Self> {
        let n = spec.n(wavelength)?;
        let k = spec.k(wavelength)?;
        if n < 1.0 {
            return Err(anyhow!(
                "Refractive index must be greater than or equal to 1."
            ));
        }
        Ok(Self::new(n, k))
    }
}
