/// A parametric plane in R3.
use anyhow::Result;

use crate::core::math::vec3::Vec3;

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
struct ParametricPlane {
    /// A point on the plane.
    p0: Vec3,

    /// A vector in the plane.
    u: Vec3,

    /// Another vector in the plane, not parallel to `u`.
    v: Vec3,
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

    pub fn p0(&self) -> &Vec3 {
        &self.p0
    }

    pub fn u(&self) -> &Vec3 {
        &self.u
    }

    pub fn v(&self) -> &Vec3 {
        &self.v
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

        assert_eq!(plane.p0(), &p0);
        assert_eq!(plane.u(), &u);
        assert_eq!(plane.v(), &v);
    }
}
