use crate::{
    core::{Float, math::vec3::Vec3},
    specs::surfaces::{BoundaryType, Mask},
};

use super::{Surface, SurfaceKind, solvers::spherical_surface};

/// A spherical surface (conic constant = 0).
#[derive(Debug, Clone)]
pub struct Sphere {
    pub radius_of_curvature: Float,
    pub boundary_type: BoundaryType,
    mask: Mask,
}

impl Sphere {
    pub fn new(
        semi_diameter: Float,
        radius_of_curvature: Float,
        boundary_type: BoundaryType,
    ) -> Self {
        Self {
            radius_of_curvature,
            boundary_type,
            mask: Mask::Circular { semi_diameter },
        }
    }
}

impl Surface for Sphere {
    fn boundary_type(&self) -> BoundaryType {
        self.boundary_type
    }

    fn intersect(
        &self,
        ray: &crate::core::ray::Ray,
        max_iter: usize,
    ) -> anyhow::Result<(Vec3, Vec3)> {
        spherical_surface(ray, self, max_iter)
    }

    fn mask(&self) -> &Mask {
        &self.mask
    }

    fn norm(&self, pos: Vec3) -> Vec3 {
        if self.radius_of_curvature.is_infinite() {
            return Vec3::new(0.0, 0.0, 1.0);
        }

        let r = (pos.x().powi(2) + pos.y().powi(2)).sqrt();
        let theta = pos.y().atan2(pos.x());

        let denom =
            (self.radius_of_curvature.powi(4) - (r * self.radius_of_curvature).powi(2)).sqrt();
        let dfdx = -r * self.radius_of_curvature * theta.cos() / denom;
        let dfdy = -r * self.radius_of_curvature * theta.sin() / denom;
        let dfdz = 1.0_f64 as Float;

        Vec3::new(dfdx, dfdy, dfdz)
    }

    fn roc(&self, _azimuth_rad: Float) -> Float {
        self.radius_of_curvature
    }

    fn sag(&self, pos: Vec3) -> Float {
        if self.radius_of_curvature.is_infinite() {
            return 0.0;
        }

        let r_sq = pos.x().powi(2) + pos.y().powi(2);
        let a = r_sq / self.radius_of_curvature;
        a / (1.0 + (1.0 - a / self.radius_of_curvature).sqrt())
    }

    fn surface_kind(&self) -> SurfaceKind {
        SurfaceKind::Sphere
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::surfaces::Conic;
    use approx::assert_abs_diff_eq;

    fn sphere(roc: Float, semi_diameter: Float) -> Sphere {
        Sphere::new(semi_diameter, roc, BoundaryType::Refracting)
    }

    fn equiv_conic(roc: Float, semi_diameter: Float) -> Conic {
        Conic::new(semi_diameter, roc, 0.0, BoundaryType::Refracting)
    }

    #[test]
    fn flat_surface_sag_is_zero() {
        let s = Sphere::new(10.0, Float::INFINITY, BoundaryType::Refracting);
        assert_eq!(s.sag(Vec3::new(3.0, 4.0, 0.0)), 0.0);
        let norm = s.norm(Vec3::new(3.0, 4.0, 0.0));
        assert_abs_diff_eq!(norm.x(), 0.0);
        assert_abs_diff_eq!(norm.y(), 0.0);
        assert_abs_diff_eq!(norm.z(), 1.0);
    }

    #[test]
    fn flat_surface_at_origin_sag_is_zero() {
        let s = Sphere::new(10.0, Float::INFINITY, BoundaryType::Refracting);
        assert_eq!(s.sag(Vec3::new(0.0, 0.0, 0.0)), 0.0);
        assert_abs_diff_eq!(s.norm(Vec3::new(0.0, 0.0, 0.0)).z(), 1.0);
    }

    #[test]
    fn sag_at_vertex_is_zero() {
        let s = sphere(100.0, 10.0);
        assert_abs_diff_eq!(s.sag(Vec3::new(0.0, 0.0, 0.0)), 0.0, epsilon = 1e-12);
    }

    #[test]
    fn sag_matches_analytic_formula() {
        let roc = 50.0;
        let s = sphere(roc, 10.0);
        let r = 5.0;
        let sag = s.sag(Vec3::new(r, 0.0, 0.0));
        let expected = roc - (roc * roc - r * r).sqrt();
        assert_abs_diff_eq!(sag, expected, epsilon = 1e-10);
    }

    #[test]
    fn sag_matches_conic_k0() {
        let roc = 50.0;
        let s = sphere(roc, 15.0);
        let c = equiv_conic(roc, 15.0);
        for &r in &[0.0, 1.0, 3.0, 5.0, 7.0] {
            let pos = Vec3::new(r, 0.0, 0.0);
            assert_abs_diff_eq!(s.sag(pos), c.sag(pos), epsilon = 1e-12);
        }
    }

    #[test]
    fn norm_matches_conic_k0() {
        let roc = 50.0;
        let s = sphere(roc, 15.0);
        let c = equiv_conic(roc, 15.0);
        for &r in &[0.001, 1.0, 3.0, 5.0] {
            let pos = Vec3::new(r, 0.0, 0.0);
            let ns = s.norm(pos);
            let nc = c.norm(pos);
            assert_abs_diff_eq!(ns.x(), nc.x(), epsilon = 1e-12);
            assert_abs_diff_eq!(ns.y(), nc.y(), epsilon = 1e-12);
            assert_abs_diff_eq!(ns.z(), nc.z(), epsilon = 1e-12);
        }
    }

    #[test]
    fn sag_is_symmetric_in_xy() {
        let s = sphere(50.0, 15.0);
        let r = 5.0;
        assert_abs_diff_eq!(
            s.sag(Vec3::new(r, 0.0, 0.0)),
            s.sag(Vec3::new(0.0, r, 0.0)),
            epsilon = 1e-12
        );
    }

    #[test]
    fn roc_returns_configured_value() {
        let s = Sphere::new(10.0, 77.3, BoundaryType::Refracting);
        assert_abs_diff_eq!(s.roc(0.0), 77.3);
        assert_abs_diff_eq!(s.roc(1.23), 77.3);
    }

    #[test]
    fn boundary_type_round_trips() {
        let r = Sphere::new(5.0, 30.0, BoundaryType::Refracting);
        let m = Sphere::new(5.0, 30.0, BoundaryType::Reflecting);
        assert!(matches!(r.boundary_type(), BoundaryType::Refracting));
        assert!(matches!(m.boundary_type(), BoundaryType::Reflecting));
    }

    #[test]
    fn surface_kind_is_sphere() {
        let s = Sphere::new(5.0, 30.0, BoundaryType::Refracting);
        assert!(matches!(s.surface_kind(), SurfaceKind::Sphere));
    }

    #[test]
    fn mask_preserves_semi_diameter() {
        let s = Sphere::new(12.5, 50.0, BoundaryType::Refracting);
        assert_abs_diff_eq!(s.mask().semi_diameter(), 12.5);
    }
}
