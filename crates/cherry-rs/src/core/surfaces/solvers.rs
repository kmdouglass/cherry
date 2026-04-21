use anyhow::{Result, bail};
use tracing::{error, trace, trace_span};

use crate::core::{Float, math::vec3::Vec3, ray::Ray, surfaces::Surface};

/// Tolerance for convergence of the Newton-Raphson method in integer multiples
/// of the machine epsilon
pub(crate) const TOL: Float = 10.0 * Float::EPSILON;

/// Maximum number of bisections when backtracking to find a non-NaN sag value
/// during ray intersection
pub(crate) const MAX_BISECT: usize = 64;

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
    use crate::core::{Float, math::vec3::Vec3, ray::Ray, surfaces::Conic};
    use crate::specs::surfaces::BoundaryType;

    use super::*;

    #[test]
    fn flat_surface() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 0.0, 1.0));
        let surf = Conic::new(4.0, Float::INFINITY, 0.0, BoundaryType::Refracting);

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
}
