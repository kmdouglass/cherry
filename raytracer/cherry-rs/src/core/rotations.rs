/// Provides data structures and logic for rotations.
use serde::{Deserialize, Serialize};

use crate::core::Float;

/// Euler angles in radians.
/// The angles are in the order of the rotation that is applied; the exact
/// rotation sequence is specified in the [Rotation] enum.
#[derive(Debug, Serialize, Deserialize)]
pub struct EulerAngles(pub Float, pub Float, pub Float);

/// 3D rotation sequences represented by Euler angles.
///
/// The following conventions are used:
/// - Coordinate systems are right-handed
/// - Counterclockwise rotations are positive
/// - Angles are in radians
#[derive(Debug, Serialize, Deserialize)]
pub enum Rotation {
    /// No rotation is applied.
    None,

    /// Rotation around the z-axis, then y-axis, then x-axis of the global
    /// reference frame.
    ExtrinsicZYX(EulerAngles),
}
