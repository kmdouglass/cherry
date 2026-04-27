#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::core::Float;

/// Specifies the system aperture.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ApertureSpec {
    EntrancePupil { semi_diameter: Float },
}
