use crate::{
    core::{Float, math::vec3::Vec3},
    specs::surfaces::{BoundaryType, Mask},
};

use super::{Surface, SurfaceKind};

/// The object plane — a flat surface with no optical effect on rays.
#[derive(Debug, Clone)]
pub struct Object {
    mask: Mask,
}

impl Object {
    pub fn new() -> Self {
        Self {
            mask: Mask::Unbounded,
        }
    }
}

impl Default for Object {
    fn default() -> Self {
        Self::new()
    }
}

impl Surface for Object {
    fn sag(&self, _pos: Vec3) -> Float {
        0.0
    }

    fn norm(&self, _pos: Vec3) -> Vec3 {
        Vec3::new(0.0, 0.0, 1.0)
    }

    fn mask(&self) -> &Mask {
        &self.mask
    }

    fn boundary_type(&self) -> BoundaryType {
        BoundaryType::NoOp
    }

    fn surface_kind(&self) -> SurfaceKind {
        SurfaceKind::Object
    }
}
