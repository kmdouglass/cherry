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
    ChiefMarginalRays,
}

/// Specifies an object field.
#[derive(Debug, Serialize, Deserialize)]
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

impl Default for PupilSampling {
    fn default() -> Self {
        Self::SquareGrid { spacing: 0.1 }
    }
}
