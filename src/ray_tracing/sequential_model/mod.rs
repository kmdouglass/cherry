use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::ray_tracing::{Surface, SystemModel};

#[derive(Debug)]
pub struct SequentialModel {
    gaps: Vec<Gap>,
    surfaces: Vec<SeqSurface>,
}

impl SequentialModel {
    pub fn new(system_model: &SystemModel) -> SequentialModel {
        // Iterate over SurfacePairs and convert to SeqSurfaces and Gaps
        let mut gaps = Vec::new();
        let mut surfaces = Vec::new();
        for pair in SurfacePairIterator::new(&system_model.surfaces) {
            let (surf, gap) = pair.into();
            surfaces.push(surf);
            gaps.push(gap);
        }

        // Add the image plane
        surfaces.push(system_model.surfaces.last().unwrap().into());

        Self { gaps, surfaces }
    }

    pub fn surfaces(&self) -> &[SeqSurface] {
        &self.surfaces
    }

    pub fn gaps(&self) -> &[Gap] {
        &self.gaps
    }

    pub fn insert_surface_and_gap(
        &mut self,
        idx: usize,
        surface: SeqSurface,
        gap: Gap,
    ) -> Result<()> {
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

    pub fn remove_surface_and_gap(&mut self, idx: usize) -> Result<()> {
        if idx == 0 {
            bail!("Cannot remove the object plane.");
        }

        if idx > self.surfaces.len() - 1 {
            bail!("Cannot remove the image plane.");
        }

        self.surfaces.remove(idx);
        self.gaps.remove(idx - 1);

        Ok(())
    }
}

impl From<&SystemModel> for SequentialModel {
    fn from(value: &SystemModel) -> Self {
        Self::new(value)
    }
}

/// A gap between two surfaces in an optical system.
#[derive(Debug, Serialize, Deserialize)]
pub struct Gap {
    n: f32,
    thickness: f32,
}

impl Gap {
    pub fn new(n: f32, thickness: f32) -> Self {
        Self { n, thickness }
    }

    pub fn n(&self) -> f32 {
        self.n
    }

    pub fn thickness(&self) -> f32 {
        self.thickness
    }
}

/// A surface in a sequential model of an optical system.
#[derive(Debug, Serialize, Deserialize)]
pub enum SeqSurface {
    ObjectOrImagePlane { diam: f32 },
    RefractingCircularConic { diam: f32, n: f32, roc: f32, k: f32 },
    RefractingCircularFlat { diam: f32, n: f32 },
}

impl From<&Surface> for SeqSurface {
    fn from(value: &Surface) -> Self {
        match value {
            Surface::ObjectOrImagePlane(surf) => {
                let surf = SeqSurface::ObjectOrImagePlane { diam: surf.diam };
                surf
            }
            Surface::RefractingCircularConic(surf) => {
                let surf = SeqSurface::RefractingCircularConic {
                    diam: surf.diam,
                    n: surf.n,
                    roc: surf.roc,
                    k: surf.k,
                };
                surf
            }
            Surface::RefractingCircularFlat(surf) => {
                let surf = SeqSurface::RefractingCircularFlat {
                    diam: surf.diam,
                    n: surf.n,
                };
                surf
            }
        }
    }
}

struct SurfacePair(Surface, Surface);

impl From<SurfacePair> for (SeqSurface, Gap) {
    fn from(value: SurfacePair) -> Self {
        let thickness = value.1.pos().z() - value.0.pos().z();
        match value.0 {
            Surface::ObjectOrImagePlane(surf) => {
                let gap = Gap::new(surf.n, thickness);
                let surf = SeqSurface::ObjectOrImagePlane { diam: surf.diam };
                (surf, gap)
            }
            Surface::RefractingCircularConic(surf) => {
                let gap = Gap::new(surf.n, thickness);
                let surf = SeqSurface::RefractingCircularConic {
                    diam: surf.diam,
                    n: surf.n,
                    roc: surf.roc,
                    k: surf.k,
                };
                (surf, gap)
            }
            Surface::RefractingCircularFlat(surf) => {
                let gap = Gap::new(surf.n, thickness);
                let surf = SeqSurface::RefractingCircularFlat {
                    diam: surf.diam,
                    n: surf.n,
                };
                (surf, gap)
            }
        }
    }
}

struct SurfacePairIterator<'a> {
    surfaces: &'a [Surface],
    idx: usize,
}

impl<'a> SurfacePairIterator<'a> {
    fn new(surfaces: &'a [Surface]) -> Self {
        Self {
            surfaces: surfaces,
            idx: 0,
        }
    }
}

impl<'a> Iterator for SurfacePairIterator<'a> {
    type Item = SurfacePair;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx > self.surfaces.len() - 2 {
            return None;
        }

        let surf1 = self.surfaces[self.idx];
        let surf2 = self.surfaces[self.idx + 1];
        self.idx += 1;

        Some(SurfacePair(surf1, surf2))
    }
}
