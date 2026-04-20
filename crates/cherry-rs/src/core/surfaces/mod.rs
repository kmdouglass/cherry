/// Surface logic and traits used by sequential models.
use crate::core::{Float, math::vec3::Vec3};

use crate::specs::surfaces::BoundaryType;

pub mod conic;
pub mod image;
pub mod object;
pub mod probe;
pub mod stop;
pub mod surface_registry;

pub use conic::Conic;
pub use image::Image;
pub use object::Object;
pub use probe::Probe;
pub use stop::Stop;
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
    Object,
    Probe,
    Stop,
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
    /// Returns the boundary type (refracting, reflecting, etc.).
    fn boundary_type(&self) -> BoundaryType;

    /// Determines whether a transverse point is outside the clear aperture of
    /// the surface.
    ///
    /// The axial z-position is ignored.
    fn outside_clear_aperture(&self, pos: Vec3) -> bool {
        let r_transv = pos.x() * pos.x() + pos.y() * pos.y();
        let r_max = self.semi_diameter();
        r_transv > r_max * r_max
    }

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

    /// Returns the semi-diameter of the surface's clear aperture.
    fn semi_diameter(&self) -> Float;

    /// Returns the role of this surface in the optical system.
    ///
    /// Used by rendering and analysis code to distinguish Object, Image, Probe,
    /// Conic, and Stop surfaces, which cannot all be differentiated from
    /// `boundary_type()` and `semi_diameter()` alone.
    ///
    /// User-defined surfaces should return [`SurfaceKind::Custom`].
    fn surface_kind(&self) -> SurfaceKind {
        SurfaceKind::Custom
    }
}
