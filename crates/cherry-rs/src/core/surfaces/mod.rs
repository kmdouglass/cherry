/// Surface logic and traits used by sequential models.
use anyhow::Result;

use crate::core::{Float, math::vec3::Vec3, ray::Ray};

use crate::specs::surfaces::{BoundaryType, Mask};

pub mod conic;
pub mod image;
pub mod iris;
pub mod object;
pub mod probe;
pub mod solvers;
pub mod surface_registry;

pub use conic::Conic;
pub use image::Image;
pub use iris::Iris;
pub use object::Object;
pub use probe::Probe;
pub use surface_registry::{SurfaceConstructor, SurfaceRegistry};

/// The role of a surface in the optical system.
///
/// Used by rendering and analysis code to distinguish surface roles that cannot
/// be inferred from geometry alone (e.g., Image vs. Probe vs. Object — all flat
/// with the same `boundary_type()`).
///
/// Library-provided surfaces return their specific kind. User-defined surfaces
/// should return [`SurfaceKind::Custom`] (the default).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceKind {
    Conic,
    Image,
    Iris,
    Object,
    Probe,
    Custom,
}

/// A surface is the primary unit of abstraction in sequential optical models.
/// It is a boundary between two media and can be used to model lenses, mirrors,
/// and other optical elements.
///
/// Surfaces are defined by their geometry, most notably their sag and normal
/// vector. By convention, the vertex of a curved surface lies at the origin of
/// its local coordinate system. A flat surface lies in the local xy-plane.
pub trait Surface: std::fmt::Debug + Send + Sync {
    /// Returns the surface boundary type (refracting, reflecting, etc.).
    fn boundary_type(&self) -> BoundaryType;

    /// Finds the intersection of a ray with the surface using Newton-Raphson
    /// iteration.
    ///
    /// Returns the intersection point and surface normal in the surface's local
    /// coordinate system. Returns an error if the iteration does not converge.
    ///
    /// Custom surface implementations may override this method to use a
    /// different intersection algorithm.
    ///
    /// # Arguments
    /// - `ray`: The ray to intersect with the surface, in the surface's local
    ///   coordinate system
    /// - `max_iter`: The maximum number of iterations to perform before giving
    ///   up
    fn intersect(&self, ray: &Ray, max_iter: usize) -> Result<(Vec3, Vec3)> {
        solvers::newton_raphson(ray, self, max_iter)
    }

    /// Returns a reference to the surface's clear-aperture mask.
    fn mask(&self) -> &Mask;

    /// Returns the radius of curvature of the base sphere of the surface.
    ///
    /// `azimuth_rad` is the angle in the surface's **local** xy-plane,
    /// measured from the local x-axis. Callers can obtain this angle by
    /// transforming a global-frame direction `v` via `Placement::rot_mat` and
    /// then computing `local_v.y().atan2(local_v.x())`.
    ///
    /// For circularly symmetric surfaces the argument is ignored and a single
    /// constant is returned. For non-circularly-symmetric surfaces such as
    /// cylinders and torics the curvature varies with azimuth.
    ///
    /// Flat surfaces should return [`Float::INFINITY`], which is
    /// the physically correct value and the default implementation.
    fn roc(&self, _azimuth_rad: Float) -> Float {
        Float::INFINITY
    }

    /// Returns the surface sag at a given position in local coordinates.
    fn sag(&self, pos: Vec3) -> Float;

    /// Returns the surface normal vector at a given position in local
    /// coordinates.
    ///
    /// The normal vector is not normalized. Its magnitude is important for
    /// Newton-Raphson ray tracing calculations.
    fn norm(&self, pos: Vec3) -> Vec3;

    /// Returns the role of this surface in the optical system.
    ///
    /// Used by rendering and analysis code to distinguish Object, Image, Probe,
    /// Conic, Iris, and Custom surfaces.
    ///
    /// User-defined surfaces should return [`SurfaceKind::Custom`].
    fn surface_kind(&self) -> SurfaceKind {
        SurfaceKind::Custom
    }

    /// Modifies the ray after it intersects this surface.
    ///
    /// The default implementation applies Snell's law for refracting surfaces,
    /// the law of reflection for reflecting surfaces, and is a no-op for NoOp
    /// surfaces. Custom surface implementations may override this method to
    /// also displace the ray (e.g., a cardinal lens that displaces rays between
    /// principal planes).
    ///
    /// All vectors are in the surface's **local** coordinate system.
    ///
    /// # Arguments
    /// - `ray`: The ray to modify, already displaced to the intersection point
    /// - `n_0`: Refractive index of the medium before the surface
    /// - `n_1`: Refractive index of the medium after the surface
    /// - `norm`: Surface normal at the intersection point (need not be
    ///   normalized)
    fn interact(&self, ray: &mut Ray, n_0: Float, n_1: Float, norm: Vec3) {
        let norm = norm.normalize();
        match self.boundary_type() {
            BoundaryType::Refracting => {
                let mu = n_0 / n_1;
                let cos_theta_1 = ray.dir().dot(&norm);
                let term_1 = norm * (1.0 - mu * mu * (1.0 - cos_theta_1 * cos_theta_1)).sqrt();
                let term_2 = (ray.dir() - norm * cos_theta_1) * mu;
                ray.set_dir(term_1 + term_2);
            }
            BoundaryType::Reflecting => {
                let cos_theta_1 = ray.dir().dot(&norm);
                ray.set_dir(ray.dir() - norm * (2.0 * cos_theta_1));
            }
            BoundaryType::NoOp => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::math::vec3::Vec3;
    use crate::core::surfaces::Conic;
    use crate::specs::surfaces::BoundaryType;

    fn make_ray(dir: Vec3) -> Ray {
        Ray::new(Vec3::new(0.0, 0.0, 0.0), dir)
    }

    #[test]
    fn test_interact_noop_leaves_direction_unchanged() {
        let surf = Conic::new(4.0, Float::INFINITY, 0.0, BoundaryType::NoOp);
        let dir = Vec3::new(0.0, 0.0, 1.0);
        let mut ray = make_ray(dir);
        let norm = Vec3::new(0.0, 0.0, 1.0);
        surf.interact(&mut ray, 1.0, 1.5, norm);
        assert_eq!(ray.dir(), dir);
    }

    #[test]
    fn test_interact_refracting_normal_incidence() {
        // Normal incidence: direction should not change (Snell's law, theta_i = 0)
        let surf = Conic::new(4.0, Float::INFINITY, 0.0, BoundaryType::Refracting);
        let dir = Vec3::new(0.0, 0.0, 1.0);
        let mut ray = make_ray(dir);
        let norm = Vec3::new(0.0, 0.0, 1.0);
        surf.interact(&mut ray, 1.0, 1.5, norm);
        assert!((ray.dir().x() - 0.0).abs() < 1e-10);
        assert!((ray.dir().y() - 0.0).abs() < 1e-10);
        assert!((ray.dir().z() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_interact_reflecting_reverses_normal_component() {
        let surf = Conic::new(4.0, Float::INFINITY, 0.0, BoundaryType::Reflecting);
        // Ray travelling in +z, flat surface normal in +z — reflected back in -z
        let dir = Vec3::new(0.0, 0.0, 1.0);
        let mut ray = make_ray(dir);
        let norm = Vec3::new(0.0, 0.0, 1.0);
        surf.interact(&mut ray, 1.0, 1.0, norm);
        assert!((ray.dir().z() - (-1.0)).abs() < 1e-10);
    }
}
