pub(crate) mod parametric_plane;
pub(crate) mod quadric;

use anyhow::Result;

use crate::core::math::geometry::{
    curves::GeometricCurve,
    surfaces::{parametric_plane::ParametricPlane, quadric::Quadric},
};

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
    pub fn plane_intersection(&self, plane: ParametricPlane) -> Result<GeometricCurve> {
        match self {
            Self::Quadric(quadric) => quadric.parametric_plane_intersection(&plane),
            Self::ParametricPlane(parametric_plane) => Ok(parametric_plane.xy_plane_intersection()),
        }
    }
}
