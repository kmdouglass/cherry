use serde::{Deserialize, Serialize};

use crate::core::Float;

/// Specifies the system aperture.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum ApertureSpec {
    EntrancePupil { semi_diameter: Float },
}
