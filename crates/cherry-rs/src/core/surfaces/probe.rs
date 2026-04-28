use anyhow::Result;

use crate::{
    core::{Float, math::vec3::Vec3, ray::Ray},
    specs::surfaces::{BoundaryType, Mask},
};

use super::{Surface, SurfaceKind, solvers::flat_surface};

/// A probe surface — a flat, non-optical surface used to measure ray positions.
#[derive(Debug, Clone)]
pub struct Probe {
    mask: Mask,
}

impl Probe {
    pub fn new() -> Self {
        Self {
            mask: Mask::Unbounded,
        }
    }
}

impl Default for Probe {
    fn default() -> Self {
        Self::new()
    }
}

impl Surface for Probe {
    fn boundary_type(&self) -> BoundaryType {
        BoundaryType::NoOp
    }

    fn intersect(&self, ray: &Ray, _max_iter: usize) -> Result<(Vec3, Vec3)> {
        flat_surface(ray, self, 0)
    }

    fn mask(&self) -> &Mask {
        &self.mask
    }

    fn norm(&self, _pos: Vec3) -> Vec3 {
        Vec3::new(0.0, 0.0, 1.0)
    }

    fn sag(&self, _pos: Vec3) -> Float {
        0.0
    }

    fn surface_kind(&self) -> SurfaceKind {
        SurfaceKind::Probe
    }
}
