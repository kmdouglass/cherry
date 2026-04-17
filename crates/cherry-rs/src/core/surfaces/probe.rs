use crate::{
    core::{Float, math::vec3::Vec3},
    specs::surfaces::BoundaryType,
};

use super::{Surface, SurfaceKind};

/// A probe surface — a flat, non-optical surface used to measure ray positions.
#[derive(Debug, Clone)]
pub struct Probe;

impl Surface for Probe {
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
        SurfaceKind::Probe
    }
}
