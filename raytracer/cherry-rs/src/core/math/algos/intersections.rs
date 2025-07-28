/// Intersection algorithms for various geometric objects.
use anyhow::Result;

use crate::core::math::{
    curves::conic::Conic,
    surfaces::{parametric_plane::ParametricPlane, quadric::Quadric},
};

/// Computes the intersection curve of a Quadric and a ParametricPlane.
/// The parameterization of the plane is:
///
/// ```text
/// x(s, t) = p0.e[0] + s * u.e[0] + t * v.e[0]
/// y(s, t) = p0.e[1] + s * u.e[1] + t * v.e[1]
/// z(s, t) = p0.e[2] + s * u.e[2] + t * v.e[2]
/// ```
///
/// The quadric/plane intersection curve is obtained by substituting these
/// parameterizations into the quadric equation for x, y, and z:
///
/// ```text
/// Q(x, y, z) = Ax^2 + By^2 + Cz^2 + Dxy + Exz + Fyz + Gx + Hy + Iz + J = 0
/// ```
pub fn quadric_parametric_plane_intersection_curve(
    quadric: &Quadric,
    plane: &ParametricPlane,
) -> Result<Conic> {
    if !plane.is_basis_orthonormal() {
        return Err(anyhow::anyhow!(
            "The basis of the parametric plane must be orthonormal"
        ));
    }

    let (p0, u, v) = (plane.p0, plane.u, plane.v);
    let (a, b, c, d, e, f, g, h, i, j) = (
        quadric.a, quadric.b, quadric.c, quadric.d, quadric.e, quadric.f, quadric.g, quadric.h,
        quadric.i, quadric.j,
    );

    // Quadratic terms
    let ss = a * u.e[0] * u.e[0]
        + b * u.e[1] * u.e[1]
        + c * u.e[2] * u.e[2]
        + d * u.e[0] * u.e[1]
        + e * u.e[0] * u.e[2]
        + f * u.e[1] * u.e[2];
    let tt = a * v.e[0] * v.e[0]
        + b * v.e[1] * v.e[1]
        + c * v.e[2] * v.e[2]
        + d * v.e[0] * v.e[1]
        + e * v.e[0] * v.e[2]
        + f * v.e[1] * v.e[2];
    let st = 2.0 * (a * u.e[0] * v.e[0] + b * u.e[1] * v.e[1] + c * u.e[2] * v.e[2])
        + d * (u.e[0] * v.e[1] + u.e[1] * v.e[0])
        + e * (u.e[0] * v.e[2] + u.e[2] * v.e[0])
        + f * (u.e[1] * v.e[2] + u.e[2] * v.e[1]);

    // Linear terms
    let s = g * u.e[0]
        + h * u.e[1]
        + i * u.e[2]
        + 2.0 * a * p0.e[0] * u.e[0]
        + 2.0 * b * p0.e[1] * u.e[1]
        + 2.0 * c * p0.e[2] * u.e[2]
        + d * (p0.e[0] * u.e[1] + p0.e[1] * u.e[0])
        + e * (p0.e[0] * u.e[2] + p0.e[2] * u.e[0])
        + f * (p0.e[1] * u.e[2] + p0.e[2] * u.e[1]);
    let t = g * v.e[0]
        + h * v.e[1]
        + i * v.e[2]
        + 2.0 * a * p0.e[0] * v.e[0]
        + 2.0 * b * p0.e[1] * v.e[1]
        + 2.0 * c * p0.e[2] * v.e[2]
        + d * (p0.e[0] * v.e[1] + p0.e[1] * v.e[0])
        + e * (p0.e[0] * v.e[2] + p0.e[2] * v.e[0])
        + f * (p0.e[1] * v.e[2] + p0.e[2] * v.e[1]);

    // Constant term
    let c = a * p0.e[0] * p0.e[0]
        + b * p0.e[1] * p0.e[1]
        + c * p0.e[2] * p0.e[2]
        + d * p0.e[0] * p0.e[1]
        + e * p0.e[0] * p0.e[2]
        + f * p0.e[1] * p0.e[2]
        + g * p0.e[0]
        + h * p0.e[1]
        + i * p0.e[2]
        + j;

    Ok(Conic::new(ss, st, tt, s, t, c))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::math::{curves::conic::ConicType, vec3::Vec3};

    #[test]
    fn qppic_plane_basis_not_orthnormal() {
        let p0 = Vec3::new(0.0, 0.0, 0.0);
        let u = Vec3::new(1.0, 0.0, 0.0);
        let v = Vec3::new(1.0, 1.0, 0.0); // Not orthonormal with u

        let plane = ParametricPlane::new(p0, u, v).unwrap();
        let quadric = Quadric::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0);

        let result = quadric_parametric_plane_intersection_curve(&quadric, &plane);
        assert!(result.is_err(), "Expected error for non-orthonormal basis");
    }

    /// Tests the intersection of the unit sphere with the plane z = 0.5.
    ///
    /// The unit sphere is defined by the equation:
    ///
    /// ```text
    /// x^2 + y^2 + z^2 - 1 = 0
    /// ```
    #[test]
    fn qppic_unit_sphere() {
        let p0 = Vec3::new(0.0, 0.0, 0.5);
        let u = Vec3::new(1.0, 0.0, 0.0);
        let v = Vec3::new(0.0, 1.0, 0.0);
        let plane = ParametricPlane::new(p0, u, v).unwrap();
        let unit_sphere = Quadric::new(1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0);

        let result = quadric_parametric_plane_intersection_curve(&unit_sphere, &plane).unwrap();

        let conic_type = result.classify();
        assert_eq!(
            conic_type,
            ConicType::Ellipse,
            "Expected intersection to be an ellipse"
        );

        let expected_radius_squared = 0.75; // The radius squared of the intersection circle
        let radius_squared = -result.matrix().e[2][2];
        assert!(
            (radius_squared - expected_radius_squared).abs() < 1e-12,
            "Expected semi-major axis to be approximately 0.75"
        );
    }

    #[test]
    fn qqpic_unit_sphere_no_intersection() {
        let p0 = Vec3::new(0.0, 0.0, 2.0); // Plane z = 2.0 is above the sphere.
        let u = Vec3::new(1.0, 0.0, 0.0);
        let v = Vec3::new(0.0, 1.0, 0.0);
        let plane = ParametricPlane::new(p0, u, v).unwrap();
        let unit_sphere = Quadric::new(1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0);

        let result = quadric_parametric_plane_intersection_curve(&unit_sphere, &plane);

        let conic_type = result.unwrap().classify();
        assert_eq!(
            conic_type,
            ConicType::Empty,
            "Expected intersection to be empty"
        );
    }
}
