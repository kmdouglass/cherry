//! Interface to RefractiveIndex.info materials data.

use anyhow::Result;
use lib_ria::Material;

use crate::{RefractiveIndexSpec, core::Float};

impl RefractiveIndexSpec for Material {
    fn n(&self, wavelength: Float) -> Result<Float> {
        let n = self.n(wavelength)?;
        Ok(n)
    }

    fn k(&self, _wavelength: Float) -> Result<Float> {
        // TODO: Implement this.
        //let k = self.k(wavelength)?;
        //match k {
        //    Some(k) => Ok(k),
        //    None => Ok(0.0),
        //}
        Ok(0.0)
    }
}
