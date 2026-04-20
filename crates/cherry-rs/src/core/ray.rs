use serde::{Deserialize, Serialize};

use crate::{
    BoundaryType,
    core::{Float, PI, math::vec3::Vec3, placement::Placement, sequential_model::Step},
};

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

    /// Returns the point along the ray at parameter `s`: `pos + dir * s`.
    pub fn pos_at(&self, s: Float) -> Vec3 {
        self.pos + self.dir * s
    }

    /// Returns the direction vector of the ray.
    pub fn dir(&self) -> Vec3 {
        self.dir
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

    // Return the direction cosine l of the ray
    pub fn l(&self) -> Float {
        self.dir.l()
    }

    // Return the direction cosine m of the ray
    pub fn m(&self) -> Float {
        self.dir.m()
    }

    // Return the direction cosine n of the ray
    pub fn n(&self) -> Float {
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
    #[allow(dead_code)]
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
mod tests {
    use crate::core::{
        Float, PI,
        math::vec3::Vec3,
        surfaces::{Conic, Surface},
    };
    use crate::specs::surfaces::BoundaryType;

    use super::*;

    #[test]
    fn test_ray_intersection_flat_surface() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 0.0, 1.0));
        let surf = Conic {
            semi_diameter: 4.0,
            radius_of_curvature: Float::INFINITY,
            conic_constant: 0.0,
            boundary_type: BoundaryType::Refracting,
        };

        let (p, _) = surf.intersect(&ray, 1000).unwrap();

        assert_eq!(p, Vec3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn test_ray_intersection_conic() {
        let l = (std::f64::consts::PI as Float / 4.0).sin();
        let m = (std::f64::consts::PI as Float / 4.0).cos();
        let ray = Ray::new(Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, l, m));
        let surf = Conic {
            semi_diameter: 4.0,
            radius_of_curvature: -1.0,
            conic_constant: 0.0,
            boundary_type: BoundaryType::Refracting,
        };

        let (p, _) = surf.intersect(&ray, 1000).unwrap();

        assert!((p.x() - 0.0_f64).abs() < 1e-4);
        assert!((p.y() - (PI / 4.0).sin()).abs() < 1e-4);
        assert!((p.z() - ((PI / 4.0).cos() - 1.0)).abs() < 1e-4);
    }
}
