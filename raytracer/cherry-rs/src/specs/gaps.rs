use std::rc::Rc;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::core::Float;

/// Creates a real refractive index spec.
#[macro_export]
macro_rules! n {
    ($n:expr_2021) => {
        std::rc::Rc::new($crate::ConstantRefractiveIndex::new($n, 0.0))
    };
    () => {};
}

/// Specifies a gap in a sequential optical system model.
#[derive(Debug)]
pub struct GapSpec {
    pub thickness: Float,
    pub refractive_index: Rc<dyn RefractiveIndexSpec>,
}

/// Specifies the refractive index of a material.
///
/// This is a trait rather than a definite type to allow for different materials
/// databases to be used.
pub trait RefractiveIndexSpec: std::fmt::Debug {
    ///
    /// # Arguments
    /// * `wavelength` - The wavelength of the light in micrometers.
    ///
    /// # Returns
    /// The real part of the refractive index.
    ///
    /// # Errors
    /// If the wavelength is not within the valid range for the material.
    fn n(&self, wavelength: Float) -> Result<Float>;

    /// The imaginary part of a material's refractive index.
    ///
    /// # Arguments
    /// * `wavelength` - The wavelength of the light in micrometers.
    ///
    /// # Returns
    /// The imaginary part of the refractive index.
    ///
    /// # Errors
    /// If the wavelength is not within the valid range for the material.
    fn k(&self, wavelength: Float) -> Result<Float>;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConstantRefractiveIndex {
    n: Float,
    k: Float,
}

impl ConstantRefractiveIndex {
    /// Creates a new constant refractive index.
    ///
    /// # Arguments
    /// * `n` - The real part of the refractive index.
    /// * `k` - The imaginary part of the refractive index.
    pub fn new(n: Float, k: Float) -> Self {
        Self { n, k }
    }
}

impl RefractiveIndexSpec for ConstantRefractiveIndex {
    fn n(&self, _wavelength: Float) -> Result<Float> {
        Ok(self.n)
    }

    fn k(&self, _wavelength: Float) -> Result<Float> {
        Ok(self.k)
    }
}
