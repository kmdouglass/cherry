use anyhow::{bail, Result};

use crate::math::mat3::Mat3;
use crate::math::vec3::Vec3;

use crate::ray_tracing::SystemModel;

struct SequentialModel<'a> {
    system_model: &'a SystemModel,
    gaps: Vec<Gap>,
    surfaces: Vec<SeqSurface>,
}

impl<'a> SequentialModel<'a> {
    fn new(system_model: &'a SystemModel) -> SequentialModel<'a> {
        let mut gaps = Vec::new();
        let mut surfaces = Vec::new();
        Self {
            system_model,
            gaps,
            surfaces,
        }
    }

    fn add_surface_and_gap(&mut self, idx: usize, surface: SeqSurface, gap: Gap) -> Result<()> {
        if idx == 0 {
            bail!("Cannot add surface before the object plane.");
        }

        if idx > self.surfaces.len() - 1 {
            bail!("Cannot add surface after the image plane.");
        }

        self.surfaces.insert(idx, surface);
        self.gaps.insert(idx, gap);

        Ok(())
    }
}

/// A gap between two surfaces in an optical system.
#[derive(Debug)]
pub struct Gap {
    n: f32,
    thickness: f32,
}

impl Gap {
    pub fn new(n: f32, thickness: f32) -> Self {
        Self { n, thickness }
    }
}

/// A surface in a sequential model of an optical system.
#[derive(Debug)]
pub enum SeqSurface {
    ObjectOrImagePlane { diam: f32 },
    RefractingCircularConic { diam: f32, n: f32, roc: f32, k: f32 },
    RefractingCircularFlat { diam: f32, n: f32 },
}
