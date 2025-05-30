/// Provides data structures and logic for rotations.
use serde::{Deserialize, Serialize};

use crate::core::{Float, math::mat3::Mat3};

/// Euler angles in radians.
///
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

    /// Intrinsic and passive rotation around the right axis, then intermediate
    /// up axis, then second intermediate forward axis.
    IntrinsicPassiveRUF(EulerAngles),
}

impl Rotation {
    /// Returns the 3D rotation matrix corresponding to the rotation.
    pub fn to_matrix(&self) -> Mat3 {
        match self {
            Rotation::None => Mat3::identity(),
            Rotation::IntrinsicPassiveRUF(euler_angles) => {
                let (theta, psi, phi) = (euler_angles.0, euler_angles.1, euler_angles.2);
                let (s_theta, c_theta) = theta.sin_cos();
                let (s_psi, c_psi) = psi.sin_cos();
                let (s_phi, c_phi) = phi.sin_cos();

                Mat3::new(
                    c_phi * c_psi,
                    s_phi * c_psi,
                    -s_psi,
                    -s_phi * c_theta + c_phi * s_psi * s_theta,
                    c_phi * c_theta + s_phi * s_psi * s_theta,
                    s_theta * c_psi,
                    s_phi * s_theta + c_phi * s_psi * c_theta,
                    -c_phi * s_theta + s_phi * s_psi * c_theta,
                    c_theta * c_psi,
                )
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::{Float, math::mat3::Mat3};

    const TOL: Float = 1e-6;

    #[test]
    fn intrinsic_passive_ruf_rotation_30_deg_about_r() {
        let rotation =
            Rotation::IntrinsicPassiveRUF(EulerAngles((30.0_f64).to_radians(), 0.0, 0.0));
        let matrix = rotation.to_matrix();

        let expected = Mat3::new(
            1.0,
            0.0,
            0.0,
            0.0,
            0.8660254037844387,
            0.5,
            0.0,
            -0.5,
            0.8660254037844387,
        );

        assert!(
            matrix.approx_eq(&expected, TOL),
            "Rotation matrix does not match expected value."
        );
    }
}
