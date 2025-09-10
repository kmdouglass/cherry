/// A parametric plane in R3.
use anyhow::Result;

use crate::core::math::{geometry::curves::{GeometricCurve, line::Line}, linalg::mat3x3::Mat3x3, vec3::Vec3};

/// A parameterization of a plane in R3.
///
/// The parametric equation for the plane is given by:
///
/// ```text
/// P(s, t) = P0 + s * U + t * V
/// ```
/// where `P0` is a point on the plane, `U` and `V` are two non-parallel vectors
/// in the plane, and `s` and `t` are parameters that vary over the plane.
#[derive(Debug)]
pub struct ParametricPlane {
    /// A point on the plane.
    pub p0: Vec3,

    /// A vector in the plane.
    pub u: Vec3,

    /// Another vector in the plane, not parallel to `u`.
    pub v: Vec3,
}

impl ParametricPlane {
    /// Constructs a new `ParametricPlane` with the given point and vectors.
    ///
    /// # Arguments
    /// * `p0` - A point on the plane.
    /// * `u` - A vector in the plane.
    /// * `v` - Another vector in the plane, not parallel to `u`.
    pub fn new(p0: Vec3, u: Vec3, v: Vec3) -> Result<Self> {
        if u.is_parallel(&v) {
            return Err(anyhow::anyhow!("Vectors u and v must not be parallel"));
        }

        Ok(ParametricPlane { p0, u, v })
    }

    /// Determines whether the plane's basis is orthonormal.
    pub fn is_basis_orthonormal(&self) -> bool {
        self.u.is_orthogonal(&self.v) && self.u.is_unit() && self.v.is_unit()
    }

    pub fn rotate(&mut self, rotation_matrix: Mat3x3) {
        self.u = rotation_matrix * self.u;
        self.v = rotation_matrix * self.v;
    }

    pub fn translate(&mut self, translation: Vec3) {
        self.p0 = self.p0 + translation;
    }

    /// Finds the intersection curve of a ParametricPlane and the xy plane (z=0).
    /// 
    /// Don't bother with generic planes in any orientation since plane surfaces in the local
    /// CRS are always aligned with the xy plane. 
    pub fn xy_plane_intersection(&self) -> GeometricCurve {
        let (p0, u, v) = (self.p0, self.u, self.v);

        let m_x = v.e[0] - (v.e[2] * u.e[0]) / u.e[2];
        let b_x = p0.e[0] - (p0.e[2] * u.e[0]) / u.e[2];

        let m_y = v.e[1] - (v.e[2] * u.e[1]) / u.e[2];
        let b_y = p0.e[1] - (p0.e[2] * u.e[1]) / u.e[2];

        let point = Vec3::new(b_x, b_y, 0.0);
        let direction = Vec3::new(m_x, m_y, 0.0).normalize();

        GeometricCurve::Line(Line::new(point, direction))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parametric_plane_new_parallel_degenerate_basis() {
        let p0 = Vec3::new(0.0, 0.0, 0.0);
        let u = Vec3::new(1.0, 0.0, 0.0);
        let v = Vec3::new(2.0, 0.0, 0.0); // Parallel to u

        let result = ParametricPlane::new(p0, u, v);
        assert!(result.is_err(), "Expected error for parallel vectors");
    }

    #[test]
    fn parametric_plane_getters() {
        let p0 = Vec3::new(1.0, 2.0, 3.0);
        let u = Vec3::new(4.0, 5.0, 6.0);
        let v = Vec3::new(7.0, 8.0, 9.0);

        let plane = ParametricPlane::new(p0, u, v).unwrap();

        assert_eq!(plane.p0, p0);
        assert_eq!(plane.u, u);
        assert_eq!(plane.v, v);
    }
}
