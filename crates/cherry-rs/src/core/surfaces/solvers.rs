use anyhow::{Result, bail};
use tracing::{error, trace, trace_span};

use crate::core::{Float, math::vec3::Vec3, ray::Ray, surfaces::Surface};

/// Tolerance for convergence of the Newton-Raphson method in integer multiples
/// of the machine epsilon
pub(crate) const TOL: Float = 10.0 * Float::EPSILON;

/// Maximum number of bisections when backtracking to find a non-NaN sag value
/// during ray intersection
pub(crate) const MAX_BISECT: usize = 64;

/// Finds the intersection of a ray with a flat surface.
///
/// By convention, a flat surface lies in the z=0 plane of the surface's local
/// coordinate system. Its normal vector points in the positive z-direction.
///
/// Returns the intersection point and the surface normal at that point, both in
/// the surface's local coordinate system.
pub fn flat_surface<S: Surface + ?Sized>(
    ray: &Ray,
    surf: &S,
    _max_iter: usize,
) -> Result<(Vec3, Vec3)> {
    let s = -ray.z() / ray.n();
    let r = ray.pos_at(s);
    let norm = surf.norm(r);
    Ok((r, norm))
}

/// Finds the intersection of a ray with the surface of a sphere.
///
/// Returns the intersection point and the surface normal at that point, both in
/// the surface's local coordinate system.
pub fn spherical_surface<S: Surface + ?Sized>(
    ray: &Ray,
    surf: &S,
    _max_iter: usize,
) -> Result<(Vec3, Vec3)> {
    // Fallback to flat surface if ROC is infinite
    if surf.roc(0.0).is_infinite() {
        return flat_surface(ray, surf, 0);
    }

    let p = ray.pos();
    let dir = ray.dir();

    // Solve the quadratic equation for ray-sphere intersection using Vieta's
    // formulas to improve numerical stability.
    let b = p.dot(&dir) - surf.roc(0.0) * dir.z();
    let c = p.dot(&p) - 2.0 * surf.roc(0.0) * p.z();

    // TODO: Use fused multiply-add on desktop
    let discriminant = b * b - c;

    if discriminant < 0.0 {
        error!(b, c, discriminant, "Ray does not intersect the sphere");
        bail!("Ray does not intersect the sphere");
    };

    let s_1 = -b - b.signum() * Float::sqrt(discriminant);
    let s_2 = c / s_1;

    let s = if surf.roc(0.0) > 0.0 {
        s_1.min(s_2)
    } else {
        s_1.max(s_2)
    };

    let r = ray.pos_at(s);
    let norm = surf.norm(r);
    Ok((r, norm))
}

/// Finds the intersection of a ray with a surface using Newton-Raphson
/// iteration.
///
/// Returns the intersection point and the surface normal at that point, both in
/// the surface's local coordinate system. Returns an error if the iteration
/// does not converge within `max_iter` steps.
pub fn newton_raphson<S: Surface + ?Sized>(
    ray: &Ray,
    surf: &S,
    max_iter: usize,
) -> Result<(Vec3, Vec3)> {
    let _intersect_span = trace_span!("intersect").entered();

    // s_1: Initial guess for intersection point
    // s: Initial distance along the ray to the z=0 plane
    let mut s_1 = 0.0;
    let mut s = -ray.z() / ray.n();

    trace!(
        pos_x = ray.x(),
        pos_y = ray.y(),
        pos_z = ray.z(),
        dir_l = ray.l(),
        dir_m = ray.m(),
        dir_n = ray.n(),
        s_init = s,
        "intersect_init"
    );

    let mut p: Vec3;
    let mut sag: Float;
    let mut norm: Vec3;
    for ctr in 0..max_iter {
        p = ray.pos_at(s);

        sag = surf.sag(p);
        norm = surf.norm(p);

        let mut bisect_counter: usize = 0;
        while sag.is_nan() {
            s /= 2.0;
            p = ray.pos_at(s);
            sag = surf.sag(p);
            norm = surf.norm(p);

            trace!(
                ctr,
                s_original = s * 2.0,
                s_bisected = s,
                "sag was NaN at current point, bisected s to find a point with non-NaN sag"
            );

            bisect_counter += 1;
            if bisect_counter > MAX_BISECT {
                error!(
                    ctr,
                    s, "Ray intersection did not converge: sag is NaN for all bisected points"
                );
                bail!(
                    "Ray intersection did not converge: sag is NaN (point may be outside clear aperture)"
                );
            }
        }

        let residual = p.z() - sag;

        trace!(
            ctr,
            norm_x = norm.x(),
            norm_y = norm.y(),
            norm_z = norm.z(),
            "normal at current point"
        );
        let denom = norm.dot(&ray.dir());

        if denom == 0.0 {
            error!(
                ctr,
                s, residual, "nr_failed: denominator is zero (ray perpendicular to surface normal)"
            );
            bail!("Ray intersection did not converge: denominator is zero");
        }

        s -= residual / denom;

        trace!(
            ctr,
            s,
            s_1,
            p_x = p.x(),
            p_y = p.y(),
            p_z = p.z(),
            sag,
            residual,
            denom,
            "newton-raphson iteration data"
        );

        if (s - s_1).abs() / s.abs().max(s_1.abs()) < TOL || residual.abs() < TOL {
            break;
        }

        if ctr == max_iter - 1 {
            error!(ctr, s, residual, "Ray intersection did not converge");
            bail!("Ray intersection did not converge");
        }

        s_1 = s;
    }

    p = ray.pos_at(s);
    norm = surf.norm(p);

    Ok((p, norm))
}

#[cfg(test)]
mod tests {
    use crate::core::{
        Float,
        math::vec3::Vec3,
        ray::Ray,
        surfaces::{Conic, Probe, Sphere},
    };
    use crate::specs::surfaces::BoundaryType;

    use super::*;

    #[test]
    fn analytical_flat_surface() {
        let ray = Ray::new(
            Vec3::new(0.0, 0.0, -10.0),
            Vec3::new(0.0, Float::sqrt(2.0) / 2.0, Float::sqrt(2.0) / 2.0),
        );
        let surf = Probe::new();

        let (p, norm) = flat_surface(&ray, &surf, 0).unwrap();

        assert_eq!(p, Vec3::new(0.0, 10.0, 0.0));
        assert_eq!(norm, Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn newton_raphson_flat_surface() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 0.0, 1.0));
        let surf = Sphere::new(4.0, Float::INFINITY, BoundaryType::Refracting);

        let (p, _) = newton_raphson(&ray, &surf, 1000).unwrap();

        assert_eq!(p, Vec3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn conic_surface() {
        let l = (std::f64::consts::PI as Float / 4.0).sin();
        let m = (std::f64::consts::PI as Float / 4.0).cos();
        let ray = Ray::new(Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, l, m));
        let surf = Conic::new(4.0, -1.0, 0.0, BoundaryType::Refracting);

        let (p, _) = newton_raphson(&ray, &surf, 1000).unwrap();

        use crate::core::PI;
        assert!((p.x() - 0.0).abs() < 1e-4);
        assert!((p.y() - (PI / 4.0).sin()).abs() < 1e-4);
        assert!((p.z() - ((PI / 4.0).cos() - 1.0)).abs() < 1e-4);
    }

    #[test]
    fn spherical_surface_infinite_roc_is_flat() {
        let surf = Sphere::new(12.5, Float::INFINITY, BoundaryType::Refracting);
        let ray = Ray::new(
            Vec3::new(0.0, 3.0, -10.0),
            Vec3::new(0.0, Float::sqrt(2.0) / 2.0, Float::sqrt(2.0) / 2.0),
        );
        let (p, norm) = spherical_surface(&ray, &surf, 0).unwrap();
        assert!((p.z()).abs() < 1e-12, "z should be 0, got {}", p.z());
        assert!((norm.z() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn spherical_surface_positive_roc_matches_newton_raphson() {
        let roc = 102.4;
        let surf = Sphere::new(12.7, roc, BoundaryType::Refracting);
        let ray = Ray::new(Vec3::new(0.0, 5.0, -200.0), Vec3::new(0.0, 0.0, 1.0));

        let (p_analytical, _) = spherical_surface(&ray, &surf, 0).unwrap();
        let (p_nr, _) = newton_raphson(&ray, &surf, 1000).unwrap();

        assert!(
            (p_analytical.x() - p_nr.x()).abs() < 1e-8,
            "x: {} vs {}",
            p_analytical.x(),
            p_nr.x()
        );
        assert!(
            (p_analytical.y() - p_nr.y()).abs() < 1e-8,
            "y: {} vs {}",
            p_analytical.y(),
            p_nr.y()
        );
        assert!(
            (p_analytical.z() - p_nr.z()).abs() < 1e-8,
            "z: {} vs {}",
            p_analytical.z(),
            p_nr.z()
        );
    }

    #[test]
    fn spherical_surface_negative_roc_matches_newton_raphson() {
        let roc = -102.4;
        let surf = Sphere::new(12.7, roc, BoundaryType::Refracting);
        let ray = Ray::new(Vec3::new(0.0, 5.0, -3.6), Vec3::new(0.0, 0.0, 1.0));

        let (p_analytical, _) = spherical_surface(&ray, &surf, 0).unwrap();
        let (p_nr, _) = newton_raphson(&ray, &surf, 1000).unwrap();

        assert!(
            (p_analytical.x() - p_nr.x()).abs() < 1e-8,
            "x: {} vs {}",
            p_analytical.x(),
            p_nr.x()
        );
        assert!(
            (p_analytical.y() - p_nr.y()).abs() < 1e-8,
            "y: {} vs {}",
            p_analytical.y(),
            p_nr.y()
        );
        assert!(
            (p_analytical.z() - p_nr.z()).abs() < 1e-8,
            "z: {} vs {}",
            p_analytical.z(),
            p_nr.z()
        );
    }

    #[test]
    fn spherical_surface_on_axis_matches_newton_raphson() {
        let roc = 50.0;
        let surf = Sphere::new(10.0, roc, BoundaryType::Refracting);
        let ray = Ray::new(Vec3::new(0.0, 0.0, -100.0), Vec3::new(0.0, 0.0, 1.0));

        let (p_analytical, _) = spherical_surface(&ray, &surf, 0).unwrap();
        let (p_nr, _) = newton_raphson(&ray, &surf, 1000).unwrap();

        assert!(
            (p_analytical.x() - p_nr.x()).abs() < 1e-8,
            "x: {} vs {}",
            p_analytical.x(),
            p_nr.x()
        );
        assert!(
            (p_analytical.y() - p_nr.y()).abs() < 1e-8,
            "y: {} vs {}",
            p_analytical.y(),
            p_nr.y()
        );
        assert!(
            (p_analytical.z() - p_nr.z()).abs() < 1e-8,
            "z: {} vs {}",
            p_analytical.z(),
            p_nr.z()
        );
    }
}
