use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use tracing::{error, trace, trace_span};

use crate::{
    BoundaryType,
    core::{
        Float, PI, math::vec3::Vec3, placement::Placement, sequential_model::Step,
        surfaces::Surface,
    },
};

/// Tolerance for convergence of the Newton-Raphson method in integer mutliples
/// of the machine epsilon
const TOL: Float = 10.0 * Float::EPSILON;

/// Maximum number of bisections when backtracking to find a non-NaN sag value
/// during ray intersection
const MAX_BISECT: usize = 64;

/// A single ray to be traced through an optical system.
///
/// # Attributes
/// - pos: Position of the ray
/// - dir: Direction of the ray (direction cosines)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ray {
    pos: Vec3,
    dir: Vec3,
}

impl Ray {
    pub fn new(pos: Vec3, dir: Vec3) -> Self {
        // We no longer require the direction vector to be normalized since this led to
        // difficulties due to floating point errors
        Self { pos, dir }
    }

    /// Create a bundle of rays with default values.
    pub fn new_bundle(num: usize) -> Vec<Self> {
        vec![
            Self {
                pos: Vec3::new(0.0, 0.0, 0.0),
                dir: Vec3::new(0.0, 0.0, 1.0),
            };
            num
        ]
    }

    /// Finds the intersection point of a ray with a surface and the surface
    /// normal at that point.
    ///
    /// If no intersection is found, then this function returns an error.
    ///
    /// # Arguments
    /// - surf: Surface to intersect with
    /// - max_iter: Maximum number of iterations for the Newton-Raphson method
    pub fn intersect(&self, surf: &dyn Surface, max_iter: usize) -> Result<(Vec3, Vec3)> {
        let _intersect_span = trace_span!("intersect").entered();

        // Initial guess for the intersection point
        let mut s_1 = 0.0;

        // Find the distance along the ray to the z=0 plane; use this as the initial
        // value for s
        let mut s = -self.pos.z() / self.dir.n();

        trace!(
            pos_x = self.pos.x(),
            pos_y = self.pos.y(),
            pos_z = self.pos.z(),
            dir_l = self.dir.l(),
            dir_m = self.dir.m(),
            dir_n = self.dir.n(),
            s_init = s,
            "intersect_init"
        );

        let mut p: Vec3;
        let mut sag: Float;
        let mut norm: Vec3;
        for ctr in 0..max_iter {
            // Compute the current estimate of the intersection point from the distance s
            p = self.pos + self.dir * s;

            // Update the distance s using the Newton-Raphson method
            sag = surf.sag(p);
            norm = surf.norm(p);

            // sag can be NaN for points outside the clear aperture due to sqrt of a
            // negative number Bisect s backwards until we find a point where
            // sag is not NaN, or until we reach the initial guess
            let mut bisect_counter: usize = 0;
            while sag.is_nan() {
                s /= 2.0;
                p = self.pos + self.dir * s;
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
            let denom = norm.dot(&self.dir);

            if denom == 0.0 {
                error!(
                    ctr,
                    s,
                    residual,
                    "nr_failed: denominator is zero (ray perpendicular to surface normal)"
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

            // Check for convergence by comparing the current and previous values of s,
            // or by checking that the residual is at machine epsilon (handles near-grazing
            // rays where a small residual still produces a step larger than TOL * |s|)
            if (s - s_1).abs() / s.abs().max(s_1.abs()) < TOL || residual.abs() < TOL {
                break;
            }

            if ctr == max_iter - 1 {
                error!(ctr, s, residual, "Ray intersection did not converge");
                bail!("Ray intersection did not converge");
            }

            s_1 = s;
        }

        // Compute the final intersection point and surface normal
        p = self.pos + self.dir * s;
        norm = surf.norm(p);

        Ok((p, norm))
    }

    // Redirect the ray by computing the direction cosines of the ray after
    // interaction with a surface.
    //
    // This function accepts the surface normal at the intersection point as an
    // argument to avoid recomputing it.
    pub fn redirect(&mut self, step: &Step, norm: Vec3) {
        // Do not match on the wildcard "_" to ensure that this function is updated when
        // new surfaces are added
        let Step {
            gap_before: gap_0,
            surface: surf,
            gap_after: gap_1,
            ..
        } = step;
        let n_0 = gap_0.refractive_index.n();
        let n_1 = if let Some(gap_1) = gap_1 {
            gap_1.refractive_index.n()
        } else {
            n_0
        };

        // Ensure the normal vector is normalized for the redirect calculations.
        let norm = norm.normalize();

        match surf.boundary_type() {
            BoundaryType::Refracting => {
                let mu = n_0 / n_1;
                let cos_theta_1 = self.dir.dot(&norm);
                let term_1 = norm * (1.0 - mu * mu * (1.0 - cos_theta_1 * cos_theta_1)).sqrt();
                let term_2 = (self.dir - norm * cos_theta_1) * mu;

                self.dir = term_1 + term_2;
            }
            BoundaryType::Reflecting => {
                let cos_theta_1 = self.dir.dot(&norm);
                self.dir = self.dir - norm * (2.0 * cos_theta_1);
            }
            BoundaryType::NoOp => {}
        }
    }

    /// Displace a ray to the given location.
    pub fn displace(&mut self, pos: Vec3) {
        self.pos = pos;
    }

    /// Transform a ray into the local coordinate system of a surface from the
    /// global system.
    pub fn transform(&mut self, placement: &Placement) {
        self.pos = placement.rotation_matrix * (self.pos - placement.position);
        self.dir = placement.rotation_matrix * self.dir;
    }

    /// Transform a ray from the local coordinate system of a surface into the
    /// global system.
    pub fn i_transform(&mut self, placement: &Placement) {
        self.pos = (placement.inv_rotation_matrix * self.pos) + placement.position;
        self.dir = placement.inv_rotation_matrix * self.dir;
    }

    // Return the x-coordinate of the ray position
    pub fn x(&self) -> Float {
        self.pos.x()
    }

    // Return the y-coordinate of the ray position
    pub fn y(&self) -> Float {
        self.pos.y()
    }

    // Return the z-coordinate of the ray position
    pub fn z(&self) -> Float {
        self.pos.z()
    }

    // Return the direction cosine k of the ray
    pub fn k(&self) -> Float {
        self.dir.l()
    }

    // Return the direction cosine l of the ray
    pub fn l(&self) -> Float {
        self.dir.m()
    }

    // Return the direction cosine m of the ray
    pub fn m(&self) -> Float {
        self.dir.n()
    }

    /// Create a fan of uniformly spaced rays in a given z-plane at a zenith
    /// angle chi to the z-axis.
    ///
    /// The vectors have endpoints at an azimuthal angle spread_phi with respect
    /// to the x-axis and extend from distances -r to r from the point (0, 0,
    /// z). All rays share the same direction, given by field_phi and chi.
    ///
    /// # Arguments
    /// * `n`: Number of vectors to create
    /// * `r`: Radial span of vector endpoints from [-r, r]
    /// * `z`: z-coordinate of endpoints
    /// * `spread_phi`: Azimuthal angle of the fan spread in the x-y plane,
    ///   radians.
    /// * `field_phi`: Azimuthal angle of the field direction, radians.
    /// * `chi`: Zenith angle of vectors with respect to z, the optics axis,
    ///   radians.
    /// * `radial_offset_x`: Offset the radial position of the vectors by this
    ///   amount in x
    /// * `radial_offset_y`: Offset the radial position of the vectors by this
    ///   amount in y
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn parallel_ray_fan(
        n: usize,
        r: Float,
        z: Float,
        spread_phi: Float,
        field_phi: Float,
        chi: Float,
        radial_offset_x: Float,
        radial_offset_y: Float,
    ) -> Vec<Ray> {
        let pos = Vec3::fan(n, r, z, spread_phi, radial_offset_x, radial_offset_y);
        let dir: Vec<Vec3> = pos
            .iter()
            .map(|_| {
                let l = chi.sin() * field_phi.cos();
                let m = chi.sin() * field_phi.sin();
                let n = chi.cos();
                Vec3::new(l, m, n)
            })
            .collect();

        pos.iter()
            .zip(dir.iter())
            .map(|(p, d)| Ray::new(*p, *d))
            .collect()
    }

    /// Creates a bundle of parallel rays on a square grid.
    ///
    /// The rays are uniformly spaced within a circle in a given z-plane.
    ///
    /// # Arguments
    /// * `radius`: Radius of the circle
    /// * `spacing`: Spacing between rays
    /// * `z`: z-coordinate of endpoints
    /// * `phi`: Angle of vectors with respect to z, the optics axis, radians
    /// * `radial_offset_x`: Offset the radial position of the vectors by this
    ///   amount in x
    /// * `radial_offset_y`: Offset the radial position of the vectors by this
    ///   amount in y
    pub(crate) fn parallel_ray_bundle_on_sq_grid(
        radius: Float,
        spacing: Float,
        z: Float,
        phi: Float,
        radial_offset_x: Float,
        radial_offset_y: Float,
    ) -> Vec<Ray> {
        let theta = PI / 2.0; // TODO: For now rays are rotated about x only

        let pos: Vec<Vec3> =
            Vec3::sq_grid_in_circ(radius, spacing, z, radial_offset_x, radial_offset_y);
        let dir: Vec<Vec3> = pos
            .iter()
            .map(|_| {
                let l = phi.sin() * theta.cos();
                let m = phi.sin() * theta.sin();
                let n = phi.cos();
                Vec3::new(l, m, n)
            })
            .collect();

        pos.iter()
            .zip(dir.iter())
            .map(|(p, d)| Ray::new(*p, *d))
            .collect()
    }
}

#[cfg(test)]
mod test {
    use crate::core::surfaces::Conic;
    use crate::specs::surfaces::BoundaryType;

    use super::*;
    // Test the intersection of a ray with a flat surface
    #[test]
    fn test_ray_intersection_flat_surface() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 0.0, 1.0));
        let surf = Conic {
            semi_diameter: 4.0,
            radius_of_curvature: Float::INFINITY,
            conic_constant: 0.0,
            boundary_type: BoundaryType::Refracting,
        };
        let max_iter = 1000;

        let (p, _) = ray.intersect(&surf, max_iter).unwrap();

        assert_eq!(p, Vec3::new(0.0, 0.0, 0.0));
    }

    // Test the intersection of a ray with a circular surface
    #[test]
    fn test_ray_intersection_conic() {
        // Ray starts at z = -1.0 and travels at 45 degrees to the optics axis
        let l = (std::f64::consts::PI as Float / 4.0).sin();
        let m = (std::f64::consts::PI as Float / 4.0).cos();
        let ray = Ray::new(Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, l, m));

        // Surface has radius of curvature -1.0 and conic constant 0.0, i.e. a circle
        let surf = Conic {
            semi_diameter: 4.0,
            radius_of_curvature: -1.0,
            conic_constant: 0.0,
            boundary_type: BoundaryType::Refracting,
        };
        let max_iter = 1000;

        let (p, _) = ray.intersect(&surf, max_iter).unwrap();

        assert!((p.x() - 0.0).abs() < 1e-4);
        assert!((p.y() - (PI / 4.0).sin()).abs() < 1e-4);
        assert!((p.z() - ((PI / 4.0).cos() - 1.0)).abs() < 1e-4);
    }
}
