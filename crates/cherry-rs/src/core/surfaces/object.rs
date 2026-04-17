use crate::{
    core::{Float, math::vec3::Vec3},
    specs::surfaces::BoundaryType,
};

use super::{Surface, SurfaceKind};

/// The object plane — a flat surface with no optical effect on rays.
#[derive(Debug, Clone)]
pub struct Object;

impl Surface for Object {
    fn sag_norm(&self, _pos: Vec3) -> (Float, Vec3) {
        (0.0, Vec3::new(0.0, 0.0, 1.0))
    }

    fn semi_diameter(&self) -> Float {
        Float::INFINITY
    }

    fn boundary_type(&self) -> BoundaryType {
        BoundaryType::NoOp
    }

    fn surface_kind(&self) -> SurfaceKind {
        SurfaceKind::Object
    }
}
