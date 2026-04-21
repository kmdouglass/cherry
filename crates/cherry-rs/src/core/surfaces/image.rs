use crate::{
    core::{Float, math::vec3::Vec3},
    specs::surfaces::{BoundaryType, Mask},
};

use super::{Surface, SurfaceKind};

/// The image plane — a flat surface with no optical effect on rays.
#[derive(Debug, Clone)]
pub struct Image {
    mask: Mask,
}

impl Image {
    pub fn new() -> Self {
        Self {
            mask: Mask::Unbounded,
        }
    }
}

impl Default for Image {
    fn default() -> Self {
        Self::new()
    }
}

impl Surface for Image {
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
        SurfaceKind::Image
    }
}
