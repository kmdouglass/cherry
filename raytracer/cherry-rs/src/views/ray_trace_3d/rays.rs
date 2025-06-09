use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

use crate::{
    SurfaceType,
    core::{
        Float, PI,
        math::vec3::Vec3,
        sequential_model::{Step, Surface},
    },
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
            //Surface::Conic(_) | Surface::Toric(_) => {
            Surface::Conic(_) => {
                match surf.surface_type() {
                    SurfaceType::Refracting => {
                        let mu = n_0 / n_1;
                        let cos_theta_1 = self.dir.dot(norm);
                        let term_1 =
                            norm * (1.0 - mu * mu * (1.0 - cos_theta_1 * cos_theta_1)).sqrt();
                        let term_2 = (self.dir - norm * cos_theta_1) * mu;

                        self.dir = term_1 + term_2;
                    }
                    SurfaceType::Reflecting => {
                        let cos_theta_1 = self.dir.dot(norm);
                        self.dir = self.dir - norm * (2.0 * cos_theta_1);
                    }
                    SurfaceType::NoOp => {}
                };
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
        todo!("Cache inverse rotation matrix");
        self.pos = surf.rot_mat().transpose() * (self.pos + surf.pos());
        self.dir = surf.rot_mat().transpose() * self.dir;
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
        self.dir.k()
    }

    // Return the direction cosine l of the ray
    pub fn l(&self) -> Float {
        self.dir.l()
    }

    // Return the direction cosine m of the ray
    pub fn m(&self) -> Float {
        self.dir.m()
    }

    /// Create a fan of uniformly spaced rays in a given z-plane at an angle phi
    /// to the z-axis.
    ///
    /// The vectors have endpoints at an angle theta with respect to the x-axis
    /// and extend from distances -r to r from the point (0, 0, z). The rays
    /// are at an angle phi from the z-axis.
    ///
    /// # Arguments
    /// * `n`: Number of vectors to create
    /// * `r`: Radial span of vector endpoints from [-r, r]
    /// * `z`: z-coordinate of endpoints
    /// * `theta` : The polar angle of the ray fan in the x-y plane.
    /// * `phi``: Angle of vectors with respect to z, the optics axis, radians
    /// * `radial_offset_x`: Offset the radial position of the vectors by this
    ///   amount in x
    /// * `radial_offset_y`: Offset the radial position of the vectors by this
    ///   amount in y
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn parallel_ray_fan(
        n: usize,
        r: Float,
        z: Float,
        theta: Float,
        phi: Float,
        radial_offset_x: Float,
        radial_offset_y: Float,
    ) -> Vec<Ray> {
        let pos = Vec3::fan(n, r, z, theta, radial_offset_x, radial_offset_y);
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
    use crate::{
        Rotation,
        core::reference_frames::Cursor,
        specs::surfaces::{SurfaceSpec, SurfaceType},
    };

    use super::*;
    // Test the intersection of a ray with a flat surface
    #[test]
    fn test_ray_intersection_flat_surface() {
        let cursor = Cursor::new(0.0);
        let ray = Ray::new(Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 0.0, 1.0));
        let surf_spec = SurfaceSpec::Conic {
            semi_diameter: 4.0,
            radius_of_curvature: Float::INFINITY,
            conic_constant: 0.0,
            surf_type: SurfaceType::Refracting,
            rotation: Rotation::None,
        };
        let surf = Surface::from_spec(&surf_spec, &cursor);
        let max_iter = 1000;

        let (p, _) = ray.intersect(&surf, max_iter).unwrap();

        assert_eq!(p, Vec3::new(0.0, 0.0, 0.0));
    }

    // Test the intersection of a ray with a circular surface
    #[test]
    fn test_ray_intersection_conic() {
        let cursor = Cursor::new(0.0);

        // Ray starts at z = -1.0 and travels at 45 degrees to the optics axis
        let l = 0.7071;
        let m = ((1.0 as Float) - 0.7071 * 0.7071).sqrt();
        let ray = Ray::new(Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, l, m));

        // Surface has radius of curvature -1.0 and conic constant 0.0, i.e. a circle
        let surf_spec = SurfaceSpec::Conic {
            semi_diameter: 4.0,
            radius_of_curvature: -1.0,
            conic_constant: 0.0,
            surf_type: SurfaceType::Refracting,
            rotation: Rotation::None,
        };
        let surf = Surface::from_spec(&surf_spec, &cursor);
        let max_iter = 1000;

        let (p, _) = ray.intersect(&surf, max_iter).unwrap();

        assert!((p.x() - 0.0).abs() < 1e-4);
        assert!((p.y() - (PI / 4.0).sin()).abs() < 1e-4);
        assert!((p.z() - ((PI / 4.0).cos() - 1.0)).abs() < 1e-4);
    }
}
