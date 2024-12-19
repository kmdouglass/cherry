//! Interface to RefractiveIndex.info materials data.

use anyhow::Result;
use lib_ria::Material;

use crate::{core::Float, RefractiveIndexSpec};

impl RefractiveIndexSpec for Material {
    fn n(&self, wavelength: Float) -> Result<Float> {
        let n = self.n(wavelength)?;
        Ok(n)
    }

    fn k(&self, wavelength: Float) -> Result<Float> {
        let k = self.k(wavelength)?;
        Ok(k)
    }
}
