use serde::{Deserialize, Serialize};

use crate::core::Float;

/// Specifies an object field.
#[derive(Debug, Serialize, Deserialize)]
pub enum FieldSpec {
    /// The angle the field makes with the optical axis, in degrees.
    Angle { angle: Float },

    /// The height of the field above the optical axis.
    ObjectHeight { height: Float },
}
