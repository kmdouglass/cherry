use serde::{Deserialize, Serialize};

use crate::core::Float;

/// Specifies the system aperture.
///
#[derive(Debug, Serialize, Deserialize)]
pub enum ApertureSpec {
    EntrancePupil { semi_diameter: Float },
}
