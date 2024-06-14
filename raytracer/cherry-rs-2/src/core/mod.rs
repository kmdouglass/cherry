/// Shared data types for modeling ray tracing systems.
pub(super) mod math;
pub(crate) mod models;
pub(crate) mod seq;

use anyhow::{anyhow, Result};

use crate::specs::gaps::{ImagSpec, RealSpec, RefractiveIndexSpec};

use math::vec3::Vec3;
use math::Complex;

pub(crate) type Float = f64;

pub(crate) const EPSILON: Float = Float::EPSILON;
pub(crate) const PI: Float = std::f64::consts::PI;

/// The cursor navigates through the optical system surface by surface, keeping
/// track of its position as it changes.
#[derive(Debug)]
pub(crate) struct Cursor {
    /// The position of the cursor.
    pos: Vec3,

    /// The direction of the cursor expressed as a unit vector.
    dir: Vec3,
}

#[derive(Debug)]
pub(crate) struct RefractiveIndex {
    eta: Complex<Float>,
}

impl Cursor {
    /// Create a new cursor at the origin of the global coordinate system.
    pub(crate) fn new() -> Self {
        Self {
            pos: Vec3::new(0.0, 0.0, 0.0),
            dir: Vec3::new(0.0, 0.0, 1.0),
        }
    }

    /// Advance the cursor by a given distance.
    pub fn advance(&mut self, distance: Float) {
        self.pos += self.dir * distance;
    }
}

impl RefractiveIndex {
    pub(crate) fn new(n: Float, k: Float) -> Self {
        Self {
            eta: Complex { real: n, imag: k },
        }
    }

    /// Creates a Gap instance from a GapSpec.
    ///
    /// A wavelength is required to compute the refractive index from the spec
    /// if the refractive index is specified as a function of wavelength.
    /// Otherwise, the real part of the refractive index is provided by the user
    /// as a constant value.
    pub(crate) fn try_from_spec(
        spec: &RefractiveIndexSpec,
        wavelength: Option<Float>,
    ) -> Result<Self> {
        if wavelength.is_none() && spec.depends_on_wavelength() {
            return Err(anyhow!(
                "The refractive index must be a constant when no wavelength is provided."
            ));
        }

        let n = match spec.real {
            RealSpec::Constant(n) => n,
            _ => !unimplemented!("Non-constant refractive index"),
        };

        let k = match spec.imag {
            Some(ImagSpec::Constant(k)) => k,
            _ => !unimplemented!("Non-constant refractive index"),
        };

        Ok(Self::new(n, k))
    }
}
