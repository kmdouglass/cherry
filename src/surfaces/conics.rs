use std::f32::consts::PI;

use crate::mat3::Mat3;
use crate::vec3::Vec3;

/// A refracting conic surface with a circular cross-section.
pub struct RefractingCircularConic {
    // Position of the center of the lens relative to the global reference frame
    pub pos: Vec3,

    // Euler angles of the optics axis through the lens relative to the global reference frame
    pub dir: Vec3,

    // Rotation matrix from the global reference frame to the surface reference frame
    pub rot_mat: Mat3,

    // Diameter of the lens
    diam: f32,

    // Refractive index
    pub n: f32,

    // Radius of curvature
    roc: f32,

    // Conic constant
    k: f32,
}

impl RefractingCircularConic {
    pub fn new(pos: Vec3, dir: Vec3, diam: f32, n: f32, roc: f32, k: f32) -> Self {
        let rot_mat = Mat3::from_euler_angles(dir.x(), dir.y(), dir.z());
        Self {
            pos,
            dir,
            rot_mat,
            diam,
            n,
            roc,
            k,
        }
    }

    pub fn sag_norm(&self, pos: Vec3) -> (f32, Vec3) {
        // Convert to polar coordinates in x, y plane
        let r = (pos.x().powi(2) + pos.y().powi(2)).sqrt();
        let theta = pos.y().atan2(pos.x());

        // Compute surface sag
        let a = r.powi(2) / self.roc;
        let sag = a / (1.0 + (1.0 - (1.0 + self.k) * a / self.roc).sqrt());

        // Compute surface normal
        let denom = (self.roc.powi(4) - (1.0 + self.k) * (r * self.roc).powi(2)).sqrt();
        let dfdx = -r * self.roc * theta.cos() / denom;
        let dfdy = -r * self.roc * theta.sin() / denom;
        let dfdz = 1f32;
        let norm = Vec3::new(dfdx, dfdy, dfdz).normalize();

        (sag, norm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sag_norm_on_axis() {
        let pos = Vec3::new(0.0, 0.0, 0.0);
        let lens = RefractingCircularConic::new(pos, pos, 1.0, 1.0, 1.0, 1.0);
        let (sag, norm) = lens.sag_norm(pos);
        assert_eq!(sag, 0.0);
        assert_eq!(norm, Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    // Test for a concave hemisphere of ROC = 1.0
    // A ray at 45 degrees to the optical axis and z = -1.0 should intersect the surface at (x, y, z) = (0.0, 0.7071, -0.2929)
    fn test_sag_norm_concave_hemisphere() {
        let pos = Vec3::new(0.0, 0.0, 0.0);
        let lens = RefractingCircularConic::new(pos, pos, 1.0, 1.0, -1.0, 0.0);

        let s = -(1_f32 - (PI / 4_f32).cos());
        let y = 0.7071_f32;
        let z = (1.0_f32 - y * y).sqrt();
        let (sag, norm) = lens.sag_norm(Vec3::new(0.0, y, z));
        assert!((sag - s).abs() < 1e-3);
        assert_eq!(norm, Vec3::new(0.0, y, z));
    }
}
