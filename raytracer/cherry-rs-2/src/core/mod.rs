/// Shared data types for modeling ray tracing systems.
pub(super) mod math;
pub(crate) mod sequential_model;

use anyhow::{anyhow, Result};

use crate::specs::gaps::{ImagSpec, RealSpec, RefractiveIndexSpec};

pub(crate) use math::array::argmin;
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

#[derive(Debug, Clone, Copy)]
pub(crate) struct RefractiveIndex {
    eta: Complex<Float>,
}

impl Cursor {
    /// Create a new cursor at the given axial position in the global coordinate
    /// system.
    pub(crate) fn new(z: Float) -> Self {
        Self {
            pos: Vec3::new(0.0, 0.0, z),
            dir: Vec3::new(0.0, 0.0, 1.0),
        }
    }

    /// Advance the cursor by a given distance along the z-direction.
    pub fn advance(&mut self, distance: Float) {
        // Edge case for advancing from negative infinity to 0.
        if (self.pos.z() == Float::NEG_INFINITY) && (distance == Float::INFINITY) {
            self.pos.set_z(0.0);
            return;
        }
        self.pos += self.dir * distance;
    }

    /// Invert the direction of the cursor.
    pub fn invert(&mut self) {
        self.dir = -self.dir;
    }

    pub(super) fn pos(&self) -> Vec3 {
        self.pos
    }
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
            _ => !unimplemented!("Non-constant refractive indexes are not implemented."),
        };

        let k = match spec.imag {
            Some(ImagSpec::Constant(k)) => k,
            None => 0.0,
            _ => !unimplemented!("Non-constant refractive indexes are not implemented."),
        };

        Ok(Self::new(n, k))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cursor_advance() {
        let mut cursor = Cursor::new(0.0);
        cursor.advance(1.0);
        assert_eq!(cursor.pos(), Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn test_cursor_invert() {
        let mut cursor = Cursor::new(0.0);
        cursor.invert();
        cursor.advance(1.0);
        assert_eq!(cursor.pos(), Vec3::new(0.0, 0.0, -1.0));
    }

    #[test]
    fn test_cursor_start_from_neg_infinity() {
        let mut cursor = Cursor::new(Float::NEG_INFINITY);
        cursor.advance(Float::INFINITY);
        assert_eq!(cursor.pos(), Vec3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn test_refractive_index_try_from_spec() {
        let spec = RefractiveIndexSpec {
            real: RealSpec::Constant(1.5),
            imag: None,
        };

        let n = RefractiveIndex::try_from_spec(&spec, None).unwrap();
        assert_eq!(n.eta.real, 1.5);
        assert_eq!(n.eta.imag, 0.0);
    }
}
