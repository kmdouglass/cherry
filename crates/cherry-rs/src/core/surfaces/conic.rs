use crate::{
    core::{Float, math::vec3::Vec3},
    specs::surfaces::BoundaryType,
};

use super::{Surface, SurfaceKind};

/// A conic surface (sphere, paraboloid, hyperboloid, etc.).
#[derive(Debug, Clone)]
pub struct Conic {
    pub semi_diameter: Float,
    pub radius_of_curvature: Float,
    pub conic_constant: Float,
    pub boundary_type: BoundaryType,
}

impl Conic {
    pub fn new(
        semi_diameter: Float,
        radius_of_curvature: Float,
        conic_constant: Float,
        boundary_type: BoundaryType,
    ) -> Self {
        Self {
            semi_diameter,
            radius_of_curvature,
            conic_constant,
            boundary_type,
        }
    }
}

impl Surface for Conic {
    fn roc(&self, _azimuth_rad: Float) -> Float {
        self.radius_of_curvature
    }

    fn sag_norm(&self, pos: Vec3) -> (Float, Vec3) {
        if self.radius_of_curvature.is_infinite() {
            return (0.0, Vec3::new(0.0, 0.0, 1.0));
        }

        // Convert to polar coordinates in the xy-plane
        let r = (pos.x().powi(2) + pos.y().powi(2)).sqrt();
        let theta = pos.y().atan2(pos.x());

        // Compute surface sag using the standard conic equation
        let a = r.powi(2) / self.radius_of_curvature;
        let sag =
            a / (1.0 + (1.0 - (1.0 + self.conic_constant) * a / self.radius_of_curvature).sqrt());

        // Compute surface normal (not normalized — magnitude matters for
        // Newton-Raphson)
        let denom = (self.radius_of_curvature.powi(4)
            - (1.0 + self.conic_constant) * (r * self.radius_of_curvature).powi(2))
        .sqrt();
        let dfdx = -r * self.radius_of_curvature * theta.cos() / denom;
        let dfdy = -r * self.radius_of_curvature * theta.sin() / denom;
        let dfdz = 1.0_f64 as Float;

        (sag, Vec3::new(dfdx, dfdy, dfdz))
    }

    fn semi_diameter(&self) -> Float {
        self.semi_diameter
    }

    fn boundary_type(&self) -> BoundaryType {
        self.boundary_type
    }

    fn surface_kind(&self) -> SurfaceKind {
        SurfaceKind::Conic
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    fn sphere(roc: Float, semi_diameter: Float) -> Conic {
        Conic::new(semi_diameter, roc, 0.0, BoundaryType::Refracting)
    }

    #[test]
    fn flat_surface_sag_is_zero() {
        let conic = Conic::new(10.0, Float::INFINITY, 0.0, BoundaryType::Refracting);
        let (sag, norm) = conic.sag_norm(Vec3::new(3.0, 4.0, 0.0));
        assert_eq!(sag, 0.0);
        assert_abs_diff_eq!(norm.x(), 0.0);
        assert_abs_diff_eq!(norm.y(), 0.0);
        assert_abs_diff_eq!(norm.z(), 1.0);
    }

    #[test]
    fn flat_surface_at_origin_sag_is_zero() {
        let conic = Conic::new(10.0, Float::INFINITY, 0.0, BoundaryType::Refracting);
        let (sag, norm) = conic.sag_norm(Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(sag, 0.0);
        assert_abs_diff_eq!(norm.z(), 1.0);
    }

    #[test]
    fn sphere_sag_at_vertex_is_zero() {
        let conic = sphere(100.0, 10.0);
        let (sag, _) = conic.sag_norm(Vec3::new(0.0, 0.0, 0.0));
        assert_abs_diff_eq!(sag, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn sphere_sag_matches_analytic_formula() {
        // For a sphere: sag = R - sqrt(R² - r²)
        let roc = 50.0;
        let conic = sphere(roc, 10.0);
        let r = 5.0;
        let (sag, _) = conic.sag_norm(Vec3::new(r, 0.0, 0.0));
        let expected_sag = roc - (roc * roc - r * r).sqrt();
        assert_abs_diff_eq!(sag, expected_sag, epsilon = 1e-10);
    }

    #[test]
    fn sphere_sag_is_symmetric_in_xy() {
        let conic = sphere(50.0, 15.0);
        let r = 5.0;
        let (sag_x, _) = conic.sag_norm(Vec3::new(r, 0.0, 0.0));
        let (sag_y, _) = conic.sag_norm(Vec3::new(0.0, r, 0.0));
        assert_abs_diff_eq!(sag_x, sag_y, epsilon = 1e-12);
    }

    #[test]
    fn sphere_normal_at_vertex_points_along_z() {
        let conic = sphere(50.0, 10.0);
        let (_, norm) = conic.sag_norm(Vec3::new(0.001, 0.0, 0.0));
        // Near the vertex the normal should be nearly (0, 0, 1)
        assert_abs_diff_eq!(norm.x() / norm.z(), 0.0, epsilon = 1e-3);
        assert_abs_diff_eq!(norm.y() / norm.z(), 0.0, epsilon = 1e-3);
    }

    #[test]
    fn roc_returns_configured_value() {
        let conic = Conic::new(10.0, 77.3, 0.0, BoundaryType::Refracting);
        assert_abs_diff_eq!(conic.roc(0.0), 77.3);
        // Circularly symmetric — azimuth does not change roc
        assert_abs_diff_eq!(conic.roc(1.23), 77.3);
    }

    #[test]
    fn roc_flat_returns_infinity() {
        let conic = Conic::new(10.0, Float::INFINITY, 0.0, BoundaryType::Refracting);
        assert!(conic.roc(0.0).is_infinite());
    }

    #[test]
    fn semi_diameter_round_trips() {
        let conic = Conic::new(12.5, 50.0, 0.0, BoundaryType::Refracting);
        assert_abs_diff_eq!(conic.semi_diameter(), 12.5);
    }

    #[test]
    fn boundary_type_round_trips() {
        let r = Conic::new(5.0, 30.0, 0.0, BoundaryType::Refracting);
        let m = Conic::new(5.0, 30.0, 0.0, BoundaryType::Reflecting);
        assert!(matches!(r.boundary_type(), BoundaryType::Refracting));
        assert!(matches!(m.boundary_type(), BoundaryType::Reflecting));
    }

    #[test]
    fn outside_clear_aperture_default_impl() {
        let conic = Conic::new(10.0, Float::INFINITY, 0.0, BoundaryType::Refracting);
        assert!(!conic.outside_clear_aperture(Vec3::new(5.0, 0.0, 0.0)));
        assert!(conic.outside_clear_aperture(Vec3::new(11.0, 0.0, 0.0)));
    }
}
