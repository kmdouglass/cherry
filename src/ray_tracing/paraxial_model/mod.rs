use std::f32::consts::PI;

use crate::get_id;
use crate::math::mat2::{mat2, Mat2};
use crate::math::vec2::Vec2;
use crate::ray_tracing::{Gap, Surface, SurfacePair, SurfacePairIterator};
use anyhow::{anyhow, bail, Result};

/// The initial angle of the ray in radians to find the entrance pupil.
const INIT_ANGLE: f32 = 5.0 * PI / 180.0;

/// The initial radius of the ray to find the entrance pupil.
const INIT_RADIUS: f32 = 1.0;

/// A paraxial element in an optical system.
///
/// The ray transfer matrices (RTM) are stored with each element to facilitate ray tracing.
///
/// A surface radius is the distance from the optical axis to its greatest extent.
#[derive(Debug, Clone, Copy)]
enum ParaxElem {
    Gap { id: usize, rtm: Mat2 },
    ImagePlane { id: usize, radius: f32, rtm: Mat2 },
    ObjectPlane { id: usize, radius: f32, rtm: Mat2 },
    Surf { id: usize, radius: f32, rtm: Mat2 },
    ThinLens { id: usize, radius: f32, rtm: Mat2 },
}

impl ParaxElem {
    fn new_gap(d: f32) -> Self {
        ParaxElem::Gap {
            id: get_id(),
            rtm: mat2!(1.0, d, 0.0, 1.0),
        }
    }

    fn new_refr_curved_surf(radius: f32, n_0: f32, n_1: f32, roc: f32) -> Self {
        let a = 1.0;
        let b = 0.0;
        let c = (n_0 - n_1) / (roc * n_1);
        let d = n_0 / n_1;

        ParaxElem::Surf {
            id: get_id(),
            radius,
            rtm: mat2!(a, b, c, d),
        }
    }

    fn new_refr_flat_surf(radius: f32, n_0: f32, n_1: f32) -> Self {
        let a = 1.0;
        let b = 0.0;
        let c = 0.0;
        let d = n_0 / n_1;

        ParaxElem::Surf {
            id: get_id(),
            radius,
            rtm: mat2!(a, b, c, d),
        }
    }

    fn new_img_plane(radius: f32) -> Self {
        ParaxElem::ImagePlane {
            id: get_id(),
            radius,
            rtm: Mat2::eye(),
        }
    }

    fn new_no_op_surf(radius: f32) -> Self {
        ParaxElem::Surf {
            id: get_id(),
            radius,
            rtm: Mat2::eye(),
        }
    }

    fn new_obj_plane(radius: f32) -> Self {
        ParaxElem::ObjectPlane {
            id: get_id(),
            radius,
            rtm: Mat2::eye(),
        }
    }

    fn new_thin_lens(radius: f32, focal_length: f32) -> Self {
        let a = 1.0;
        let b = 0.0;
        let c = -1.0 / focal_length;
        let d = 1.0;

        ParaxElem::ThinLens {
            id: get_id(),
            radius,
            rtm: mat2!(a, b, c, d),
        }
    }
}

// Implement PartialEq on ParaxElem, ignoring the ID field
impl PartialEq for ParaxElem {
    fn eq(&self, other: &Self) -> bool {
        // If variant is different, return false.
        if std::mem::discriminant(self) != std::mem::discriminant(other) {
            return false;
        }

        match (self, other) {
            (ParaxElem::Gap { id: _, rtm: m1 }, ParaxElem::Gap { id: _, rtm: m2 }) => m1 == m2,
            (
                ParaxElem::ImagePlane {
                    id: _,
                    radius: r1,
                    rtm: m1,
                },
                ParaxElem::ImagePlane {
                    id: _,
                    radius: r2,
                    rtm: m2,
                },
            ) => r1 == r2 && m1 == m2,
            (
                ParaxElem::ObjectPlane {
                    id: _,
                    radius: r1,
                    rtm: m1,
                },
                ParaxElem::ObjectPlane {
                    id: _,
                    radius: r2,
                    rtm: m2,
                },
            ) => r1 == r2 && m1 == m2,
            (
                ParaxElem::Surf {
                    id: _,
                    radius: r1,
                    rtm: m1,
                },
                ParaxElem::Surf {
                    id: _,
                    radius: r2,
                    rtm: m2,
                },
            ) => r1 == r2 && m1 == m2,
            (
                ParaxElem::ThinLens {
                    id: _,
                    radius: r1,
                    rtm: m1,
                },
                ParaxElem::ThinLens {
                    id: _,
                    radius: r2,
                    rtm: m2,
                },
            ) => r1 == r2 && m1 == m2,
            _ => false,
        }
    }
}

/// The result of tracing a ray through a paraxial model.
#[derive(Debug)]
struct ParaxTraceResult {
    ray: Vec2,
    surf_id: usize,
    surf_radius: f32,
}

/// A paraxial model of an optical system.
///
/// The paraxial model comprises a sequence of paraxial elements. The most important part of each
/// element is its ray transfer matrices (RTMs), which describes how a paraxial ray is transformed
/// as it propagates through the element.
#[derive(Debug)]
pub struct ParaxialModel {
    parax_elems: Vec<ParaxElem>,
}

impl ParaxialModel {
    /// Creates a new paraxial model.
    pub fn new() -> Self {
        let mut parax_elems = Vec::new();
        parax_elems.push(ParaxElem::new_obj_plane(f32::INFINITY));
        parax_elems.push(ParaxElem::new_gap(0.0));
        parax_elems.push(ParaxElem::new_img_plane(f32::INFINITY));

        Self { parax_elems }
    }

    fn elements(&self) -> &[ParaxElem] {
        &self.parax_elems
    }

    fn insert_element_and_gap(
        &mut self,
        idx: usize,
        elem: ParaxElem,
        gap: ParaxElem,
    ) -> Result<()> {
        if idx == 0 {
            bail!("Cannot add element before the object plane.");
        }

        if idx > self.parax_elems.len() / 2 {
            bail!("Cannot add element after the image plane.");
        }

        // Convert element/gap index into Vec<ParaxElem> index.
        let idx = idx * 2;
        self.parax_elems.insert(idx, gap);
        self.parax_elems.insert(idx, elem);

        Ok(())
    }

    fn trace(parax_elems: &[ParaxElem], mut ray: Vec2) -> Vec<ParaxTraceResult> {
        // There's always one more non-gap elements than there are gaps.
        let num_non_gaps = parax_elems.len() / 2 + 1;
        let mut results = Vec::with_capacity(num_non_gaps);

        // Trace the ray through the paraxial model and save the results at each surface.
        for elem in parax_elems {
            match elem {
                ParaxElem::Gap { id: _, rtm } => {
                    ray = rtm * &ray;
                }
                ParaxElem::ImagePlane { id, radius, rtm } => {
                    ray = rtm * &ray;
                    results.push(ParaxTraceResult {
                        ray: ray.clone(),
                        surf_id: *id,
                        surf_radius: *radius,
                    });
                }
                ParaxElem::ObjectPlane { id, radius, rtm } => {
                    ray = rtm * &ray;
                    results.push(ParaxTraceResult {
                        ray: ray.clone(),
                        surf_id: *id,
                        surf_radius: *radius,
                    });
                }
                ParaxElem::Surf { id, radius, rtm } => {
                    ray = rtm * &ray;
                    results.push(ParaxTraceResult {
                        ray: ray.clone(),
                        surf_id: *id,
                        surf_radius: *radius,
                    });
                }
                ParaxElem::ThinLens { id, radius, rtm } => {
                    ray = rtm * &ray;
                    results.push(ParaxTraceResult {
                        ray: ray.clone(),
                        surf_id: *id,
                        surf_radius: *radius,
                    });
                }
            }
        }

        results
    }

    /// Find the ID of the surface that is the aperture stop of the paraxial model.
    pub fn find_aperture_stop(&self) -> Result<usize> {
        let init_ray = self.init_ray()?;
        let results = ParaxialModel::trace(&self.parax_elems, init_ray);

        // Find the ID of the non-gap element with the smallest ratio of surface radius to ray height.
        let mut min_ratio = f32::MAX;
        let mut min_id = 0;
        for result in results.iter() {
            let ratio = result.surf_radius / result.ray.y();
            if ratio < min_ratio {
                min_ratio = ratio;
                min_id = result.surf_id;
            }
        }

        Ok(min_id)
    }

    /// Find the distance of the entrance pupil from the first optical surface.
    pub fn find_entrance_pupil_dist(&self) -> Result<f32> {
        let aperture_stop_id = self.find_aperture_stop()?;

        // Find the index of the element that is the aperture stop.
        let idx = self
            .parax_elems
            .iter()
            .position(|elem| {
                if let ParaxElem::Surf { id, .. } = elem {
                    *id == aperture_stop_id
                } else if let ParaxElem::ThinLens { id, .. } = elem {
                    *id == aperture_stop_id
                } else {
                    false
                }
            })
            .ok_or(anyhow!("The aperture stop ID was not found."))?;

        // Launch a ray from the aperture stop from the axis backwards through the system.
        let surfs = &self.parax_elems[0..idx + 1];
        let surfs_reversed = ParaxialModel::reverse_surfaces(surfs);
        let init_ray = Vec2::new(0.0, INIT_ANGLE);
        let results = ParaxialModel::trace(&surfs_reversed, init_ray);

        // Get the next-to-last result, which is the ray traveling in the object space from the
        // first surface to the object plane.
        let obj_ray = &results[results.len() - 1].ray;

        // Find the intersection of the object space ray with the optical axis.
        let t = -obj_ray.y() / obj_ray.theta();

        Ok(t)
    }

    /// Find the initial ray to trace through the paraxial model.
    ///
    /// If the object is at infinity, the initial ray is parallel to, but not colinear with, the
    /// optical axis. Otherwise, it starts on the axis with a small angle.
    fn init_ray(&self) -> Result<Vec2> {
        // Get the first gap (second element) in the paraxial model, which is the object space.
        let obj_space_dist = if let ParaxElem::Gap { id: _, rtm } = self.parax_elems[1] {
            rtm[0][1]
        } else {
            bail!("The second element in the paraxial model must be the object space gap.");
        };

        if obj_space_dist.is_infinite() {
            Ok(Vec2::new(INIT_RADIUS, 0.0))
        } else {
            Ok(Vec2::new(0.0, INIT_ANGLE))
        }
    }

    /// Reverses a sequence of paraxial elements.
    fn reverse_surfaces(surfs: &[ParaxElem]) -> Vec<ParaxElem> {
        let mut reversed_surfs = Vec::with_capacity(surfs.len());
        reversed_surfs.extend(surfs.iter().rev().cloned());
        reversed_surfs
    }
}

impl From<&[Surface]> for ParaxialModel {
    fn from(surfs: &[Surface]) -> Self {
        let obj_plane_radius = surfs.first().unwrap().diam() / 2.0;
        let mut rtms = Vec::new();

        // The object plane RTM does not do anything.
        rtms.push(ParaxElem::new_no_op_surf(obj_plane_radius));

        for pair in SurfacePairIterator::new(surfs) {
            // The ray transfer matrix for the second surface in the pair.
            let surf_rtm = pair.parax_surf();
            let (_, gap) = pair.into();

            rtms.push(gap.parax_surf());
            rtms.push(surf_rtm);
        }

        Self { parax_elems: rtms }
    }
}

impl SurfacePair {
    /// Return the paraxial surface equivalent for the second surface in the pair.
    fn parax_surf(&self) -> ParaxElem {
        let surf = self.1;
        let n_0 = self.0.n();
        let n_1 = self.1.n();

        match surf {
            Surface::RefractingCircularConic(surf) => {
                ParaxElem::new_refr_curved_surf(surf.diam / 2.0, n_0, n_1, surf.roc)
            }
            Surface::RefractingCircularFlat(surf) => {
                ParaxElem::new_refr_flat_surf(surf.diam / 2.0, n_0, n_1)
            }
            Surface::ObjectOrImagePlane(surf) => ParaxElem::new_no_op_surf(surf.diam / 2.0),
            Surface::Stop(surf) => ParaxElem::new_no_op_surf(surf.diam / 2.0),
        }
    }
}

impl Gap {
    /// Return the ray transfer matrix for a gap.
    fn parax_surf(&self) -> ParaxElem {
        let d = self.thickness();
        ParaxElem::new_gap(d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A two lens system used for verification testing of the paraixal model.
    fn two_lens_system_verification() {
        // Object space is 100 mm.
        // Image space is 75 mm.
        let lens_1 = ParaxElem::new_thin_lens(100.0, 100.0);
        let gap_1 = ParaxElem::new_gap(25.0);
        let stop = ParaxElem::new_no_op_surf(10.0);
        let gap_2 = ParaxElem::new_gap(25.0);
        let lens_2 = ParaxElem::new_thin_lens(100.0, 75.0);
    }

    #[test]
    fn test_paraxial_model_insert_elemement_and_gap() {
        let mut parax_model = ParaxialModel::new();
        let lens = ParaxElem::new_thin_lens(100.0, 100.0);
        let gap = ParaxElem::new_gap(100.0);

        parax_model.insert_element_and_gap(1, lens, gap).unwrap();

        let elements = parax_model.elements();

        // Object plane, object space, lens, image space, image plane.
        assert_eq!(elements.len(), 5);
        assert_eq!(elements[2], ParaxElem::new_thin_lens(100.0, 100.0));
        assert_eq!(elements[3], ParaxElem::new_gap(100.0));
    }

    #[test]
    fn test_paraxial_model_insert_elemement_and_gap_before_object_plane() {
        let mut parax_model = ParaxialModel::new();
        let lens = ParaxElem::new_thin_lens(100.0, 100.0);
        let gap = ParaxElem::new_gap(100.0);

        let result = parax_model.insert_element_and_gap(0, lens, gap);
        assert!(result.is_err());
    }

    #[test]
    fn test_paraxial_model_insert_elemement_and_gap_after_image_plane() {
        let mut parax_model = ParaxialModel::new();
        let lens = ParaxElem::new_thin_lens(100.0, 100.0);
        let gap = ParaxElem::new_gap(100.0);

        let result = parax_model.insert_element_and_gap(2, lens, gap);
        assert!(result.is_err());
    }
}