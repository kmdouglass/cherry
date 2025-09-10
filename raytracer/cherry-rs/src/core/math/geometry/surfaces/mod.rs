pub(crate) mod parametric_plane;
pub(crate) mod quadric;

use crate::core::math::{
    geometry::surfaces::{parametric_plane::ParametricPlane, quadric::Quadric},
    vec3::Vec3,
};

type Curve = Vec<Vec3>;

/// A surface in 3D space.
///
/// Unlike optical surfaces for ray tracing, this trait encodes the behavior of
/// ideal, abstract surfaces in 3D Euclidean space. They are intended for
/// mathematical calculations only, not ray tracing.
pub(crate) enum GeometricSurface {
    Quadric(Quadric),
    ParametricPlane(ParametricPlane),
}

impl GeometricSurface {
    pub fn plane_intersection(&self) -> Curve {
        todo!("Implement plane intersection for geometric surfaces");
    }
}
