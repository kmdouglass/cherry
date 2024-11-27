use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::core::Float;

/// Specifies a pupil sampling method.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PupilSampling {
    /// A square grid of rays in the the entrance pupil.
    ///
    /// Spacing is the spacing between rays in the grid in normalized pupil
    /// distances, i.e. [0, 1]. A spacing of 1.0 means that one ray will lie
    /// at the pupil center (the chief ray), and the others will lie at the
    /// pupil edge (marginal rays).
    SquareGrid { spacing: Float },

    /// The chief and marginal rays.
    ChiefAndMarginalRays,
}

/// Specifies an object field.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FieldSpec {
    /// The angle the field makes with the optical axis, in degrees.
    Angle {
        angle: Float,
        pupil_sampling: PupilSampling,
    },

    /// The height of the field above the optical axis.
    ObjectHeight {
        height: Float,
        pupil_sampling: PupilSampling,
    },
}

impl PupilSampling {
    /// Validate the pupil sampling method.
    pub fn validate(&self) -> Result<()> {
        match self {
            PupilSampling::SquareGrid { spacing } => {
                if spacing.is_nan() {
                    anyhow::bail!("Pupil grid spacing must be a number");
                }
                if *spacing < 0.0 || *spacing > 1.0 {
                    anyhow::bail!("Pupil grid spacing must be in the range [0, 1]");
                }
            }
            PupilSampling::ChiefAndMarginalRays => {}
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
    /// Validate the field specification.
    pub fn validate(&self) -> Result<()> {
        match self {
            FieldSpec::Angle {
                angle,
                pupil_sampling,
            } => {
                if angle.is_nan() {
                    anyhow::bail!("Field angle must be a number");
                }
                if *angle < -90.0 || *angle > 90.0 {
                    anyhow::bail!("Field angle must be in the range [-90.0, 90]");
                }
                pupil_sampling.validate()?;
            }
            FieldSpec::ObjectHeight {
                height,
                pupil_sampling,
            } => {
                if height.is_nan() {
                    anyhow::bail!("Field height must be a number");
                }
                pupil_sampling.validate()?;
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
        let angle = FieldSpec::Angle {
            angle: 45.0,
            pupil_sampling: PupilSampling::SquareGrid { spacing: 0.1 },
        };
        assert!(angle.validate().is_ok());

        let angle = FieldSpec::Angle {
            angle: 95.0,
            pupil_sampling: PupilSampling::SquareGrid { spacing: 0.1 },
        };
        assert!(angle.validate().is_err());

        let angle = FieldSpec::Angle {
            angle: -5.0,
            pupil_sampling: PupilSampling::SquareGrid { spacing: 0.1 },
        };
        assert!(angle.validate().is_ok());

        let angle = FieldSpec::Angle {
            angle: 45.0,
            pupil_sampling: PupilSampling::SquareGrid { spacing: 1.1 },
        };
        assert!(angle.validate().is_err());

        let angle = FieldSpec::Angle {
            angle: 45.0,
            pupil_sampling: PupilSampling::SquareGrid { spacing: -0.1 },
        };
        assert!(angle.validate().is_err());

        let angle = FieldSpec::Angle {
            angle: 45.0,
            pupil_sampling: PupilSampling::SquareGrid {
                spacing: Float::NAN,
            },
        };
        assert!(angle.validate().is_err());

        let height = FieldSpec::ObjectHeight {
            height: 0.1,
            pupil_sampling: PupilSampling::SquareGrid { spacing: 0.1 },
        };
        assert!(height.validate().is_ok());

        let height = FieldSpec::ObjectHeight {
            height: 0.1,
            pupil_sampling: PupilSampling::SquareGrid { spacing: 1.1 },
        };
        assert!(height.validate().is_err());
    }
}
