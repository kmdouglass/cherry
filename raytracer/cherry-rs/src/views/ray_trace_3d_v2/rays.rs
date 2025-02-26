use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::core::{
    math::vec3::Vec3,
    sequential_model::{Step, Surface},
    Float, PI,
};

/// Tolerance for convergence of the Newton-Raphson method in integer mutliples
/// of the machine epsilon
const TOL: Float = Float::EPSILON;

/// A single ray to be traced through an optical system.
///
/// # Attributes
/// - pos: Position of the ray
/// - dir: Direction of the ray (direction cosines)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ray {
    pos: Vec3,
    dir: Vec3,
    terminated: bool,
}

impl Ray {
    pub fn new(pos: Vec3, dir: Vec3) -> Result<Self> {
        if !dir.is_unit() {
            bail!("Ray direction must be a unit vector");
        }
        Ok(Self {
            pos,
            dir,
            terminated: false,
        })
    }

    /// Finds the intersection point of a ray with a surface and the surface
    /// normal at that point.
    ///
    /// If no intersection is found, then this function returns an error.
    ///
    /// # Arguments
    /// - surf: Surface to intersect with
    /// - max_iter: Maximum number of iterations for the Newton-Raphson method
    pub fn intersect(&self, surf: &Surface, max_iter: usize) -> Result<(Vec3, Vec3)> {
        // Initial guess for the intersection point
        let mut s_1 = 0.0;

        // Find the distance along the ray to the z=0 plane; use this as the initial
        // value for s
        let mut s = -self.pos.z() / self.dir.m();

        let mut p: Vec3;
        let mut sag: Float;
        let mut norm: Vec3;
        for ctr in 0..max_iter {
            // Compute the current estimate of the intersection point from the distance s
            p = self.pos + self.dir * s;

            // Update the distance s using the Newton-Raphson method
            (sag, norm) = surf.sag_norm(p);
            s -= (p.z() - sag) / norm.dot(self.dir);

            // Check for convergence by comparing the current and previous values of s
            if (s - s_1).abs() / Float::max(s, s_1) < TOL {
                break;
            }

            if ctr == max_iter - 1 {
                bail!("Ray intersection did not converge");
            }

            s_1 = s;
        }

        // Compute the final intersection point and surface normal
        p = self.pos + self.dir * s;
        (_, norm) = surf.sag_norm(p);

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
        let (gap_0, surf, gap_1) = step;
        let n_0 = gap_0.refractive_index.n();
        let n_1 = if let Some(gap_1) = gap_1 {
            gap_1.refractive_index.n()
        } else {
            n_0
        };

        match surf {
            // Refracting surfaces
            //Surface::Conic(_) | Surface::Toric(_) => {
            Surface::Conic(_) => {
                let mu = n_0 / n_1;
                let cos_theta_1 = self.dir.dot(norm);
                let term_1 = norm * (1.0 - mu * mu * (1.0 - cos_theta_1 * cos_theta_1)).sqrt();
                let term_2 = (self.dir - norm * cos_theta_1) * mu;

                self.dir = term_1 + term_2;
            }
            // No-op surfaces
            Surface::Image(_) => {}
            Surface::Object(_) => {}
            Surface::Probe(_) => {}
            Surface::Stop(_) => {}
        }
    }

    /// Displace a ray to the given location.
    pub fn displace(&mut self, pos: Vec3) {
        self.pos = pos;
    }

    /// Transform a ray into the local coordinate system of a surface from the
    /// global system.
    pub fn transform(&mut self, surf: &Surface) {
        self.pos = surf.rot_mat() * (self.pos - surf.pos());
        self.dir = surf.rot_mat() * self.dir;
    }

    /// Transform a ray from the local coordinate system of a surface into the
    /// global system.
    pub fn i_transform(&mut self, surf: &Surface) {
        self.pos = surf.rot_mat().transpose() * (self.pos + surf.pos());
        self.dir = surf.rot_mat().transpose() * self.dir;
    }

    pub fn terminate(&mut self) {
        self.terminated = true;
    }

    #[inline]
    pub fn is_terminated(&self) -> bool {
        self.terminated
    }

    /// Create a fan of uniformly spaced rays in a given z-plane at an angle phi
    /// to the z-axis.
    ///
    /// The vectors have endpoints at an angle theta with respect to the x-axis
    /// and extend from distances -r to r from the point (0, 0, z). The rays
    /// are at an angle phi from the z-axis.
    ///
    /// # Arguments
    /// - n: Number of vectors to create
    /// - r: Radial span of vector endpoints from [-r, r]
    /// - theta: Angle of vectors with respect to x, radians
    /// - z: z-coordinate of endpoints
    /// - phi: Angle of vectors with respect to z, the optics axis, radians
    /// - radial_offset_x: Offset the radial position of the vectors by this
    ///   amount in x
    /// - radial_offset_y: Offset the radial position of the vectors by this
    ///   amount in y
    #[allow(clippy::too_many_arguments)]
    pub fn fan(
        n: usize,
        r: Float,
        theta: Float,
        z: Float,
        phi: Float,
        radial_offset_x: Float,
        radial_offset_y: Float,
    ) -> Vec<Ray> {
        let pos = Vec3::fan(n, r, theta, z, radial_offset_x, radial_offset_y);
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
            .map(|(p, d)| Ray::new(*p, *d).unwrap())
            .collect()
    }

    /// Create a square grid of uniformly spaced rays within a circle in a given
    /// z-plane.
    ///
    /// # Arguments
    /// - `radius`: Radius of the circle
    /// - `spacing`: Spacing between rays
    /// - `z`: z-coordinate of endpoints
    /// - `phi`: Angle of vectors with respect to z, the optics axis, radians
    /// - radial_offset_x: Offset the radial position of the vectors by this
    ///   amount in x
    /// - radial_offset_y: Offset the radial position of the vectors by this
    ///   amount in y
    pub(crate) fn sq_grid_in_circ(
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
            .map(|(p, d)| Ray::new(*p, *d).unwrap())
            .collect()
    }
}

#[cfg(test)]
mod test {
    use crate::specs::surfaces::{SurfaceSpec, SurfaceType};

    use super::*;
    // Test the constructor of Ray
    #[test]
    fn test_rays_new() {
        let pos = Vec3::new(0.0, 0.0, 0.0);
        let dir = Vec3::new(0.0, 0.0, 1.0);

        let rays = Ray::new(pos, dir);

        assert!(rays.is_ok());
    }

    // Test the constructor of Ray with a non-unit direction vector
    #[test]
    fn test_rays_new_non_unit_dir() {
        let pos = Vec3::new(0.0, 0.0, 0.0);
        let dir = Vec3::new(0.0, 0.0, 2.0);

        let rays = Ray::new(pos, dir);

        assert!(rays.is_err());
    }

    // Test the intersection of a ray with a flat surface
    #[test]
    fn test_ray_intersection_flat_surface() {
        let pos = Vec3::new(0.0, 0.0, 0.0);
        let ray = Ray::new(Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 0.0, 1.0)).unwrap();
        let surf_spec = SurfaceSpec::Conic {
            semi_diameter: 4.0,
            radius_of_curvature: Float::INFINITY,
            conic_constant: 0.0,
            surf_type: SurfaceType::Refracting,
        };
        let surf = Surface::from_spec(&surf_spec, pos);
        let max_iter = 1000;

        let (p, _) = ray.intersect(&surf, max_iter).unwrap();

        assert_eq!(p, Vec3::new(0.0, 0.0, 0.0));
    }

    // Test the intersection of a ray with a circular surface
    #[test]
    fn test_ray_intersection_conic() {
        let pos = Vec3::new(0.0, 0.0, 0.0);

        // Ray starts at z = -1.0 and travels at 45 degrees to the optics axis
        let l = 0.7071;
        let m = ((1.0 as Float) - 0.7071 * 0.7071).sqrt();
        let ray = Ray::new(Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, l, m)).unwrap();

        // Surface has radius of curvature -1.0 and conic constant 0.0, i.e. a circle
        let surf_spec = SurfaceSpec::Conic {
            semi_diameter: 4.0,
            radius_of_curvature: -1.0,
            conic_constant: 0.0,
            surf_type: SurfaceType::Refracting,
        };
        let surf = Surface::from_spec(&surf_spec, pos);
        let max_iter = 1000;

        let (p, _) = ray.intersect(&surf, max_iter).unwrap();

        assert!((p.x() - 0.0).abs() < 1e-4);
        assert!((p.y() - (PI / 4.0).sin()).abs() < 1e-4);
        assert!((p.z() - ((PI / 4.0).cos() - 1.0)).abs() < 1e-4);
    }
}
