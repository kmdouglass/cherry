use anyhow::{bail, Result};

use crate::surfaces;
use crate::vec3::Vec3;

#[derive(Debug)]
pub(crate) struct Ray {
    pos: Vec3,
    dir: Vec3,
}

impl Ray {
    pub fn new(pos: Vec3, dir: Vec3) -> Result<Self> {
        if !dir.is_unit() {
            bail!("Ray direction must be a unit vector");
        }
        Ok(Self { pos, dir })
    }

    /// Computes the point of intersection of a ray with a surface.
    ///
    /// If no intersection is found, this function returns an error.
    pub fn intersect(&self, surf: &surfaces::Surface, tol: f32, max_iter: usize) -> Result<Vec3> {
        // Initial guess for the intersection point
        let mut s_1 = 0.0;

        // Find the distance along the ray to the z=0 plane; use this as the initial value for s
        let mut s = -self.pos.z() / self.dir.m();

        let mut p: Vec3;
        let mut sag: f32;
        let mut norm: Vec3;
        for ctr in 0..max_iter {
            // Compute the current estimate of the intersection point from the distance s
            p = self.pos + self.dir * s;

            // Update the distance s using the Newton-Raphson method
            (sag, norm) = surf.sag_norm(p);
            s = s - (p.z() - sag) / norm.dot(self.dir);

            // Check for convergence by comparing the current and previous values of s
            if (s - s_1).abs() < tol {
                break;
            }

            if ctr == max_iter - 1 {
                bail!("Ray intersection did not converge");
            }

            s_1 = s;
        }

        Ok(self.pos + self.dir * s)
    }

    // Compute the direction cosines of the ray after interaction with a surface.
    //
    // This function accepts the surface normal at the intersection point as an argument to avoid
    // recomputing it.
    pub fn redirect(&self, surf: &surfaces::Surface, norm: Vec3, n1: f32, n2: f32) -> Vec3 {
        // Avoid matching on the wildcard "_" to ensure that this function is updated when new
        // surfaces are added
        match surf {
            // Refracting surfaces
            surfaces::Surface::RefractingCircularConic(_)
            | surfaces::Surface::RefractingCircularFlat(_) => {
                let mu = n1 / n2;
                let cos_theta_1 = self.dir.dot(norm);
                let term_1 = norm * (1.0 - mu * mu * (1.0 - cos_theta_1 * cos_theta_1)).sqrt();
                let term_2 = (self.dir - norm * cos_theta_1) * mu;

                term_1 + term_2
            }
            // No-op surfaces
            surfaces::Surface::ObjectOrImagePlane(_) => self.dir,
        }
    }
}

#[cfg(test)]
mod test {
    use std::f32::consts::PI;

    // Test the constructor of Ray
    #[test]
    fn test_rays_new() {
        use super::*;

        let pos = Vec3::new(0.0, 0.0, 0.0);
        let dir = Vec3::new(0.0, 0.0, 1.0);

        let rays = Ray::new(pos, dir);

        assert!(rays.is_ok());
    }

    // Test the constructor of Ray with a non-unit direction vector
    #[test]
    fn test_rays_new_non_unit_dir() {
        use super::*;

        let pos = Vec3::new(0.0, 0.0, 0.0);

        let dir = Vec3::new(0.0, 0.0, 2.0);

        let rays = Ray::new(pos, dir);

        assert!(rays.is_err());
    }

    // Test the intersection of a ray with a flat surface
    #[test]
    fn test_ray_intersection() {
        use super::*;
        let ray = Ray::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0)).unwrap();
        let surf = surfaces::Surface::new_refr_circ_flat(0.0, 4.0, 1.5);
        let tol = 1e-6;
        let max_iter = 1000;

        let p = ray.intersect(&surf, tol, max_iter).unwrap();

        assert_eq!(p, Vec3::new(0.0, 0.0, 0.0));
    }

    // Test the intersection of a ray with a circular surface
    #[test]
    fn test_ray_intersection_conic() {
        use super::*;

        // Ray starts at z = -1.0 and travels at 45 degrees to the optics axis
        let l = 0.7071;
        let m = (1.0_f32 - 0.7071 * 0.7071).sqrt();
        let ray = Ray::new(Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, l, m)).unwrap();
        println!("ray = {:?}", ray);
        // Surface has radius of curvature -1.0 and conic constant 0.0, i.e. a circle
        let surf = surfaces::Surface::new_refr_circ_conic(0.0, 2.0, 1.5, -1.0, 0.0);
        let tol = 1e-6;
        let max_iter = 1000;

        let p = ray.intersect(&surf, tol, max_iter).unwrap();

        assert_eq!(p, Vec3::new(0.0, (PI / 4.0).sin(), (PI / 4.0).cos() - 1.0));
    }
}