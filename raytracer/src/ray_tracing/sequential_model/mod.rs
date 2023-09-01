use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::math::vec3::Vec3;
use crate::ray_tracing::{Surface, SurfacePair, SurfacePairIterator, SystemModel};

#[derive(Debug)]
pub struct SequentialModel {
    gaps: Vec<Gap>,
    surfaces: Vec<Surface>,
}

impl SequentialModel {
    pub fn new(surfaces: &[Surface]) -> SequentialModel {
        // Iterate over SurfacePairs and convert to Surfaces and Gaps
        let mut gaps = Vec::new();
        let mut surfs = Vec::new();
        for pair in SurfacePairIterator::new(surfaces) {
            let (surf, gap) = pair.into();
            surfs.push(surf);
            gaps.push(gap);
        }

        // Add the image plane
        surfs.push(surfaces.last().unwrap().clone());

        Self {
            gaps,
            surfaces: surfs,
        }
    }

    pub fn surfaces(&self) -> &[Surface] {
        &self.surfaces
    }

    pub fn gaps(&self) -> &[Gap] {
        &self.gaps
    }

    pub fn insert_surface_and_gap(
        &mut self,
        idx: usize,
        surf_spec: SurfaceSpec,
        gap: Gap,
    ) -> Result<()> {
        if idx == 0 {
            bail!("Cannot add surface before the object plane.");
        }

        if idx > self.surfaces.len() - 1 {
            bail!("Cannot add surface after the image plane.");
        }

        let surface = Surface::from((surf_spec, &gap));

        self.surfaces.insert(idx, surface);
        self.gaps.insert(idx, gap);

        // Readjust the positions of the surfaces after the inserted one.
        self.readjust_surfaces(idx);

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
        self.gaps.remove(idx);

        // Readjust the positions of the surfaces after the removed one.
        self.readjust_surfaces(idx);

        Ok(())
    }

    fn readjust_surfaces(&mut self, idx: usize) {
        // Loop over the surfaces starting at idx and all after it, adjusting their positions along the axis.
        let mut dist = self.surf_distance_from_origin(idx);
        for i in idx..self.gaps.len() {
            self.surfaces[i].set_pos(Vec3::new(0.0, 0.0, dist));
            dist += self.gaps[i].thickness();
        }
        self.surfaces
            .last_mut()
            .unwrap()
            .set_pos(Vec3::new(0.0, 0.0, dist));
    }

    fn surf_distance_from_origin(&self, idx: usize) -> f32 {
        // By convention, the first non-object surface is at z=0.
        if idx == 0 {
            return self.gaps[0].thickness();
        }

        let mut dist = 0.0;
        for i in 1..idx {
            dist += self.gaps[i].thickness();
        }
        dist
    }
}

impl From<&SystemModel> for SequentialModel {
    fn from(value: &SystemModel) -> Self {
        Self::new(&value.surfaces)
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
pub enum SurfaceSpec {
    ObjectOrImagePlane { diam: f32 },
    RefractingCircularConic { diam: f32, roc: f32, k: f32 },
    RefractingCircularFlat { diam: f32 },
    Stop { diam: f32 },
}

impl From<&Surface> for SurfaceSpec {
    fn from(value: &Surface) -> Self {
        match value {
            Surface::ObjectOrImagePlane(surf) => {
                let surf = SurfaceSpec::ObjectOrImagePlane { diam: surf.diam };
                surf
            }
            Surface::RefractingCircularConic(surf) => {
                let surf = SurfaceSpec::RefractingCircularConic {
                    diam: surf.diam,
                    roc: surf.roc,
                    k: surf.k,
                };
                surf
            }
            Surface::RefractingCircularFlat(surf) => {
                let surf = SurfaceSpec::RefractingCircularFlat { diam: surf.diam };
                surf
            }
            Surface::Stop(surf) => {
                let surf = SurfaceSpec::Stop { diam: surf.diam };
                surf
            }
        }
    }
}

impl From<SurfacePair> for (Surface, Gap) {
    fn from(value: SurfacePair) -> Self {
        let thickness = value.1.pos().z() - value.0.pos().z();
        match value.0 {
            Surface::ObjectOrImagePlane(surf) => {
                let gap = Gap::new(surf.n, thickness);
                (value.0, gap)
            }
            Surface::RefractingCircularConic(surf) => {
                let gap = Gap::new(surf.n, thickness);
                (value.0, gap)
            }
            Surface::RefractingCircularFlat(surf) => {
                let gap = Gap::new(surf.n, thickness);
                (value.0, gap)
            }
            Surface::Stop(surf) => {
                let gap = Gap::new(surf.n, thickness);
                (value.0, gap)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_surface_and_gap() {
        let mut system_model = SystemModel::new();
        let mut model = system_model.seq_model_mut();

        model
            .insert_surface_and_gap(
                1,
                SurfaceSpec::RefractingCircularConic {
                    diam: 25.0,
                    roc: 1.0,
                    k: 0.0,
                },
                Gap::new(1.0, 1.0),
            )
            .unwrap();
        model
            .insert_surface_and_gap(
                2,
                SurfaceSpec::RefractingCircularConic {
                    diam: 25.0,
                    roc: -1.0,
                    k: 0.0,
                },
                Gap::new(1.0, 10.0),
            )
            .unwrap();

        // 2 surfaces + object plane + image plane
        assert_eq!(model.surfaces.len(), 4);
        assert_eq!(model.gaps.len(), 3);
        assert_eq!(model.surfaces[0].pos(), Vec3::new(0.0, 0.0, -1.0));
        assert_eq!(model.surfaces[1].pos(), Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(model.surfaces[2].pos(), Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(model.surfaces[3].pos(), Vec3::new(0.0, 0.0, 11.0));
    }

    #[test]
    fn test_insert_surface_and_gap_before_another_surface() {
        let mut system_model = SystemModel::new();
        let mut model = system_model.seq_model_mut();

        model
            .insert_surface_and_gap(
                1,
                SurfaceSpec::RefractingCircularConic {
                    diam: 25.0,
                    roc: -1.0,
                    k: 0.0,
                },
                Gap::new(1.0, 10.0),
            )
            .unwrap();

        assert_eq!(model.surfaces.len(), 3);
        assert_eq!(model.gaps.len(), 2);
        assert_eq!(model.surfaces[0].pos(), Vec3::new(0.0, 0.0, -1.0));
        assert_eq!(model.surfaces[1].pos(), Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(model.surfaces[2].pos(), Vec3::new(0.0, 0.0, 10.0));

        // Thickness 1.0 goes before surface and gap with thickness 10.0
        model
            .insert_surface_and_gap(
                1,
                SurfaceSpec::RefractingCircularConic {
                    diam: 25.0,
                    roc: 1.0,
                    k: 0.0,
                },
                Gap::new(1.0, 1.0),
            )
            .unwrap();

        assert_eq!(model.surfaces.len(), 4);
        assert_eq!(model.gaps.len(), 3);
        assert_eq!(model.surfaces[0].pos(), Vec3::new(0.0, 0.0, -1.0));
        assert_eq!(model.surfaces[1].pos(), Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(model.surfaces[2].pos(), Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(model.surfaces[3].pos(), Vec3::new(0.0, 0.0, 11.0));
    }

    #[test]
    fn test_remove_surface_and_gap() {
        let mut system_model = SystemModel::new();
        let mut model = system_model.seq_model_mut();

        model
            .insert_surface_and_gap(
                1,
                SurfaceSpec::RefractingCircularConic {
                    diam: 25.0,
                    roc: 1.0,
                    k: 0.0,
                },
                Gap::new(1.5, 1.0),
            )
            .unwrap();
        model
            .insert_surface_and_gap(
                2,
                SurfaceSpec::RefractingCircularConic {
                    diam: 25.0,
                    roc: -1.0,
                    k: 0.0,
                },
                Gap::new(1.0, 10.0),
            )
            .unwrap();

        assert_eq!(model.surfaces.len(), 4);
        assert_eq!(model.gaps.len(), 3);

        model.remove_surface_and_gap(2).unwrap();

        assert_eq!(model.surfaces.len(), 3);
        assert_eq!(model.gaps.len(), 2);
        assert_eq!(model.surfaces[0].pos(), Vec3::new(0.0, 0.0, -1.0));
        assert_eq!(model.surfaces[1].pos(), Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(model.surfaces[2].pos(), Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn test_surf_distance_from_origin() {
        let mut system_model = SystemModel::new();
        let mut model = system_model.seq_model_mut();

        model
            .insert_surface_and_gap(
                1,
                SurfaceSpec::RefractingCircularConic {
                    diam: 25.0,
                    roc: 1.0,
                    k: 0.0,
                },
                Gap::new(1.5, 1.0),
            )
            .unwrap();
        model
            .insert_surface_and_gap(
                2,
                SurfaceSpec::RefractingCircularConic {
                    diam: 25.0,
                    roc: -1.0,
                    k: 0.0,
                },
                Gap::new(1.0, 10.0),
            )
            .unwrap();

        assert_eq!(model.surf_distance_from_origin(0), 1.0);
        assert_eq!(model.surf_distance_from_origin(1), 0.0);
        assert_eq!(model.surf_distance_from_origin(2), 1.0);
        assert_eq!(model.surf_distance_from_origin(3), 11.0);
    }
}
