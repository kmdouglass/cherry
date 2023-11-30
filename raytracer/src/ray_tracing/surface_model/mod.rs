use anyhow::{bail, Result};

use crate::math::vec3::Vec3;
use crate::ray_tracing::{Gap, Surface, SurfacePairIterator};

#[derive(Debug)]
pub struct SurfaceModel {
    gaps: Vec<Gap>,
    surfaces: Vec<Surface>,
}

impl SurfaceModel {
    pub fn new(surfaces: &[Surface]) -> SurfaceModel {
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

    pub fn insert_surface_and_gap(&mut self, idx: usize, surface: Surface, gap: Gap) -> Result<()> {
        if idx == 0 {
            bail!("Cannot add surface before the object plane.");
        }

        if idx > self.surfaces.len() - 1 {
            bail!("Cannot add surface after the image plane.");
        }

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

    pub fn set_obj_space(&mut self, gap: Gap) {
        self.gaps[0] = gap;
        self.surfaces[0].set_pos(Vec3::new(0.0, 0.0, -gap.thickness()));
    }

    fn readjust_surfaces(&mut self, idx: usize) {
        // Loop over the surfaces starting at idx and all after it, adjusting their positions along the axis.
        let mut dist = self.surf_distance_from_origin(idx);

        // By convention, the first non-object surface is at z=0.
        if idx == 0 {
            dist = -dist;
        }

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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::ray_tracing::SystemModel;

    fn planoconvex_lens_obj_at_inf() -> SystemModel {
        // The image is located at the lens back focal plane.
        let mut system_model = SystemModel::old();
        let mut model = system_model.surf_model_mut();

        model
            .insert_surface_and_gap(
                1,
                Surface::new_refr_circ_conic(
                    Vec3::new(0.0, 0.0, 0.0),
                    Vec3::new(0.0, 0.0, 0.0),
                    25.8,
                    1.5,
                    25.8,
                    0.0,
                ),
                Gap::new(1.5, 5.3),
            )
            .unwrap();
        model
            .insert_surface_and_gap(
                2,
                Surface::new_refr_circ_flat(
                    Vec3::new(0.0, 0.0, 5.3),
                    Vec3::new(0.0, 0.0, 0.0),
                    25.0,
                    1.0,
                ),
                Gap::new(1.0, 46.6),
            )
            .unwrap();

        system_model
    }

    fn planoconvex_lens_img_at_inf() -> SystemModel {
        // The object is located at the lens front focal plane.
        let mut system_model = SystemModel::old();
        let mut model = system_model.surf_model_mut();

        model
            .insert_surface_and_gap(
                1,
                Surface::new_refr_circ_flat(
                    Vec3::new(0.0, 0.0, 0.0),
                    Vec3::new(0.0, 0.0, 0.0),
                    25.0,
                    1.5,
                ),
                Gap::new(1.5, 5.3),
            )
            .unwrap();
        model
            .insert_surface_and_gap(
                2,
                Surface::new_refr_circ_conic(
                    Vec3::new(0.0, 0.0, 0.0),
                    Vec3::new(0.0, 0.0, 0.0),
                    -25.8,
                    1.0,
                    -25.8,
                    0.0,
                ),
                Gap::new(1.0, f32::INFINITY),
            )
            .unwrap();

        system_model
    }

    #[test]
    fn test_insert_surface_and_gap() {
        let system_model = planoconvex_lens_obj_at_inf();
        let model = system_model.surf_model();

        // 2 surfaces + object plane + image plane
        assert_eq!(model.surfaces.len(), 4);
        assert_eq!(model.gaps.len(), 3);
        assert_eq!(model.surfaces[0].pos(), Vec3::new(0.0, 0.0, -1.0));
        assert_eq!(model.surfaces[1].pos(), Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(model.surfaces[2].pos(), Vec3::new(0.0, 0.0, 5.3));
        assert_eq!(model.surfaces[3].pos(), Vec3::new(0.0, 0.0, 51.9));
    }

    #[test]
    fn test_insert_surface_and_gap_before_another_surface() {
        let mut system_model = SystemModel::old();
        let mut model = system_model.surf_model_mut();

        model
            .insert_surface_and_gap(
                1,
                Surface::new_refr_circ_conic(
                    Vec3::new(0.0, 0.0, 0.0),
                    Vec3::new(0.0, 0.0, 0.0),
                    25.0,
                    1.0,
                    -1.0,
                    0.0,
                ),
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
                Surface::new_refr_circ_conic(
                    Vec3::new(0.0, 0.0, 0.0),
                    Vec3::new(0.0, 0.0, 0.0),
                    25.0,
                    1.0,
                    1.0,
                    0.0,
                ),
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
        let mut system_model = planoconvex_lens_obj_at_inf();
        let model = system_model.surf_model_mut();

        assert_eq!(model.surfaces.len(), 4);
        assert_eq!(model.gaps.len(), 3);

        model.remove_surface_and_gap(2).unwrap();

        assert_eq!(model.surfaces.len(), 3);
        assert_eq!(model.gaps.len(), 2);
        assert_eq!(model.surfaces[0].pos(), Vec3::new(0.0, 0.0, -1.0));
        assert_eq!(model.surfaces[1].pos(), Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(model.surfaces[2].pos(), Vec3::new(0.0, 0.0, 5.3));
    }

    #[test]
    fn test_surf_distance_from_origin() {
        let mut system_model = planoconvex_lens_obj_at_inf();
        let model = system_model.surf_model_mut();

        assert_eq!(model.surf_distance_from_origin(0), 1.0);
        assert_eq!(model.surf_distance_from_origin(1), 0.0);
        assert_eq!(model.surf_distance_from_origin(2), 5.3);
        assert_eq!(model.surf_distance_from_origin(3), 5.3 + 46.6);
    }
}
