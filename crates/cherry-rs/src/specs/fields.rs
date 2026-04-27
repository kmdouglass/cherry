use anyhow::Result;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::core::{Float, PI, math::vec3::Vec3};

/// Specifies a pupil sampling method.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PupilSampling {
    /// A pupil consisting of only a chief ray that pierces the pupil center.
    ChiefRay,

    /// A square grid of rays in the the entrance pupil.
    ///
    /// Spacing is the spacing between rays in the grid in normalized pupil
    /// distances, i.e. [0, 1]. A spacing of 1.0 means that one ray will lie
    /// at the pupil center (the chief ray), and the others will lie at the
    /// pupil edge (marginal rays).
    SquareGrid { spacing: Float },

    /// A tangential ray fan of `n` evenly-spaced rays from one pupil edge to
    /// the other. The fan lies in the meridional plane, whose orientation is
    /// determined by the field spec via [`FieldSpec::tangential_fan_phi`].
    TangentialRayFan { n: usize },

    /// A sagittal ray fan of `n` evenly-spaced rays from one pupil edge to
    /// the other. The fan lies in the sagittal plane, perpendicular to the
    /// meridional plane, whose orientation is determined by the field spec
    /// via [`FieldSpec::sagittal_fan_phi`].
    SagittalRayFan { n: usize },
}

/// Specifies an object field.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FieldSpec {
    /// The 2D direction of the field, specified in spherical coordinates.
    ///
    /// - `chi`: polar (zenith) angle from the optical axis, in degrees. Range
    ///   `[-90, 90]`. Negative chi places the field on the opposite side of the
    ///   phi direction, which allows smooth scanning through zero without
    ///   flipping phi.
    /// - `phi`: azimuthal angle in the RU (XY) plane, in degrees. Range `(-180,
    ///   180]`. Consistent with phi = rotation about F (z-axis). φ = 0 → R/XZ
    ///   plane; φ = 90 → U/YZ plane (default).
    ///
    /// Chief ray direction: `(sin χ · cos φ, sin χ · sin φ, cos χ)`.
    Angle { chi: Float, phi: Float },

    /// The x, y position of the object field point.
    ///
    /// (0, 0) corresponds to the optical axis.
    PointSource { x: Float, y: Float },
}

/// Returns the unique tangential direction vectors for a set of field specs,
/// sorted by ascending phi (radians).
///
/// Each `FieldSpec` contributes one phi key via `tangential_fan_phi()`. Keys
/// are deduplicated by exact float equality (bit-identical values share a
/// submodel). Returns `vec![(0, 1, 0)]` (v = Y, phi = 90°) when `field_specs`
/// is empty.
pub fn unique_tangential_vecs(field_specs: &[FieldSpec]) -> Vec<Vec3> {
    if field_specs.is_empty() {
        return vec![Vec3::new(0.0, 1.0, 0.0)];
    }
    let mut phis: Vec<Float> = field_specs.iter().map(|f| f.tangential_fan_phi()).collect();
    phis.sort_by(|a, b| a.total_cmp(b));
    phis.dedup();
    phis.iter()
        .map(|&phi| Vec3::new(phi.cos(), phi.sin(), 0.0))
        .collect()
}

impl PupilSampling {
    /// Validate the pupil sampling method.
    pub fn validate(&self) -> Result<()> {
        match self {
            PupilSampling::ChiefRay => {}
            PupilSampling::SquareGrid { spacing } => {
                if spacing.is_nan() {
                    anyhow::bail!("Pupil grid spacing must be a number");
                }
                if *spacing < 0.0 || *spacing > 1.0 {
                    anyhow::bail!("Pupil grid spacing must be in the range [0, 1]");
                }
            }
            PupilSampling::TangentialRayFan { .. } => {}
            PupilSampling::SagittalRayFan { .. } => {}
        }
        Ok(())
    }
}

impl Default for PupilSampling {
    fn default() -> Self {
        Self::SquareGrid { spacing: 0.1 }
    }
}

impl FieldSpec {
    /// Returns the azimuthal angle (in radians) of the tangential ray fan in
    /// the pupil plane.
    ///
    /// The tangential fan lies in the meridional plane, which is defined by the
    /// optical axis and the field direction. For an `Angle` field this is
    /// simply `phi` converted to radians. For a `PointSource` field it is
    /// `atan2(y, x)`, with a fallback of `π/2` (U/YZ plane) when the source
    /// is on-axis.
    pub fn tangential_fan_phi(&self) -> Float {
        match self {
            FieldSpec::Angle { chi: _, phi } => phi.to_radians(),
            FieldSpec::PointSource { x, y } => {
                if *x == 0.0 && *y == 0.0 {
                    PI / 2.0
                } else {
                    y.atan2(*x)
                }
            }
        }
    }

    /// Returns the azimuthal angle (in radians) of the sagittal ray fan in the
    /// pupil plane.
    ///
    /// The sagittal fan is perpendicular to the tangential fan, so this is
    /// `tangential_fan_phi() + π/2`.
    pub fn sagittal_fan_phi(&self) -> Float {
        self.tangential_fan_phi() + PI / 2.0
    }

    /// Validate the field specification.
    pub fn validate(&self) -> Result<()> {
        match self {
            FieldSpec::Angle { chi, phi } => {
                if chi.is_nan() {
                    anyhow::bail!("Field chi must be a number");
                }
                if *chi < -90.0 || *chi > 90.0 {
                    anyhow::bail!("Field chi must be in the range [-90, 90]");
                }
                if phi.is_nan() {
                    anyhow::bail!("Field phi must be a number");
                }
                if *phi <= -180.0 || *phi > 180.0 {
                    anyhow::bail!("Field phi must be in the range (-180, 180]");
                }
            }
            FieldSpec::PointSource { x, y } => {
                if x.is_nan() || y.is_nan() {
                    anyhow::bail!("Point source field locations must be numbers");
                }

                if x.is_infinite() || y.is_infinite() {
                    anyhow::bail!("Point source field locations must be finite");
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_pupil_sampling_validate() {
        let square_grid = PupilSampling::SquareGrid { spacing: 0.1 };
        assert!(square_grid.validate().is_ok());

        let square_grid = PupilSampling::SquareGrid { spacing: 1.1 };
        assert!(square_grid.validate().is_err());

        let square_grid = PupilSampling::SquareGrid { spacing: -0.1 };
        assert!(square_grid.validate().is_err());

        let square_grid = PupilSampling::SquareGrid {
            spacing: Float::NAN,
        };
        assert!(square_grid.validate().is_err());
    }

    #[test]
    fn test_field_spec_validate() {
        assert!(
            FieldSpec::Angle {
                chi: 5.0,
                phi: 90.0
            }
            .validate()
            .is_ok()
        );
        assert!(
            FieldSpec::Angle {
                chi: 95.0,
                phi: 90.0
            }
            .validate()
            .is_err()
        );
        assert!(
            FieldSpec::Angle {
                chi: -5.0,
                phi: 90.0
            }
            .validate()
            .is_ok()
        );
        assert!(
            FieldSpec::Angle {
                chi: 5.0,
                phi: 90.0
            }
            .validate()
            .is_ok()
        );
        assert!(FieldSpec::PointSource { x: 0.0, y: 0.1 }.validate().is_ok());
        assert!(
            FieldSpec::PointSource {
                x: Float::NAN,
                y: 0.1
            }
            .validate()
            .is_err()
        );
        assert!(
            FieldSpec::PointSource {
                x: 0.0,
                y: Float::NAN
            }
            .validate()
            .is_err()
        );
        assert!(
            FieldSpec::PointSource {
                x: Float::INFINITY,
                y: 0.1
            }
            .validate()
            .is_err()
        );
        assert!(
            FieldSpec::PointSource {
                x: 0.0,
                y: Float::INFINITY
            }
            .validate()
            .is_err()
        );
    }

    #[test]
    fn test_tangential_fan_phi() {
        use std::f64::consts::{FRAC_PI_2, PI};

        approx::assert_abs_diff_eq!(
            FieldSpec::Angle {
                chi: 5.0,
                phi: 90.0
            }
            .tangential_fan_phi(),
            FRAC_PI_2,
            epsilon = 1e-10
        );
        approx::assert_abs_diff_eq!(
            FieldSpec::Angle { chi: 5.0, phi: 0.0 }.tangential_fan_phi(),
            0.0,
            epsilon = 1e-10
        );
        approx::assert_abs_diff_eq!(
            FieldSpec::Angle {
                chi: 5.0,
                phi: 45.0
            }
            .tangential_fan_phi(),
            PI / 4.0,
            epsilon = 1e-10
        );
        approx::assert_abs_diff_eq!(
            FieldSpec::Angle {
                chi: -5.0,
                phi: 90.0
            }
            .tangential_fan_phi(),
            FRAC_PI_2,
            epsilon = 1e-10
        );
        // sagittal is tangential + π/2
        approx::assert_abs_diff_eq!(
            FieldSpec::Angle {
                chi: 5.0,
                phi: 90.0
            }
            .sagittal_fan_phi(),
            FRAC_PI_2 + FRAC_PI_2,
            epsilon = 1e-10
        );
        // PointSource: atan2(y, x)
        approx::assert_abs_diff_eq!(
            FieldSpec::PointSource { x: 0.0, y: 1.0 }.tangential_fan_phi(),
            FRAC_PI_2,
            epsilon = 1e-10
        );
        approx::assert_abs_diff_eq!(
            FieldSpec::PointSource { x: 1.0, y: 0.0 }.tangential_fan_phi(),
            0.0,
            epsilon = 1e-10
        );
        // PointSource at origin: fallback to π/2
        approx::assert_abs_diff_eq!(
            FieldSpec::PointSource { x: 0.0, y: 0.0 }.tangential_fan_phi(),
            FRAC_PI_2,
            epsilon = 1e-10
        );
    }

    #[test]
    fn test_field_spec_angle_chi_phi_validate() {
        assert!(
            FieldSpec::Angle {
                chi: 5.0,
                phi: 90.0
            }
            .validate()
            .is_ok()
        );
        assert!(
            FieldSpec::Angle {
                chi: 95.0,
                phi: 90.0
            }
            .validate()
            .is_err()
        );
        assert!(
            FieldSpec::Angle {
                chi: -91.0,
                phi: 90.0
            }
            .validate()
            .is_err()
        );
        assert!(
            FieldSpec::Angle {
                chi: -5.0,
                phi: 90.0
            }
            .validate()
            .is_ok()
        );
        assert!(
            FieldSpec::Angle {
                chi: 0.0,
                phi: 180.0
            }
            .validate()
            .is_ok()
        );
        assert!(
            FieldSpec::Angle {
                chi: 0.0,
                phi: -180.0
            }
            .validate()
            .is_err()
        );
        assert!(
            FieldSpec::Angle {
                chi: Float::NAN,
                phi: 90.0
            }
            .validate()
            .is_err()
        );
        assert!(
            FieldSpec::Angle {
                chi: 5.0,
                phi: Float::NAN
            }
            .validate()
            .is_err()
        );
    }
}
