/// Paraxial ray trace engine.
///
/// # Sign conventions
/// - Distances in front of elements are negative; distances after an element are positive.
/// - Rays counter-clockwise from the optical axis are positive; rays clockwise are negative.
use std::borrow::Borrow;
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
    elem_id: usize,
    elem_radius: f32,
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

    /// Inserts the second surface of a pair and a the following gap into the paraxial model.
    pub fn insert_surface_and_gap(
        &mut self,
        idx: usize,
        surface_0: Surface,
        surface_1: Surface,
        gap: Gap,
    ) -> Result<()> {
        // TODO Investigate why the refractive index isn't set correctly for flats
        let pair = SurfacePair(surface_0, surface_1);
        let parax_surf = ParaxElem::from(&pair);
        let parax_gap = ParaxElem::from(gap);

        self.insert_element_and_gap(idx, parax_surf, parax_gap)?;

        Ok(())
    }

    pub fn remove_element_and_gap(&mut self, idx: usize) -> Result<()> {
        if idx == 0 {
            bail!("Cannot remove the object plane.");
        }

        if idx > self.parax_elems.len() / 2 {
            bail!("Cannot remove the image plane.");
        }

        // Convert element/gap index into Vec<ParaxElem> index.
        let idx = idx * 2;
        self.parax_elems.remove(idx);
        self.parax_elems.remove(idx);

        Ok(())
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

    fn elements(&self) -> &[ParaxElem] {
        &self.parax_elems
    }

    fn is_empty(&self) -> bool {
        // The paraxial model always has an object plane, object space, and image plane.
        self.parax_elems.len() == 3
    }

    fn is_obj_at_infinity(&self) -> bool {
        if let ParaxElem::Gap { id: _, rtm } = self.parax_elems[1] {
            rtm[0][1].is_infinite()
        } else {
            false
        }
    }

    /// Set the distance of the object plane from the first optical surface.
    pub fn set_obj_dist(&mut self, dist: f32) {
        if let ParaxElem::Gap { id: _, rtm } = &mut self.parax_elems[1] {
            rtm[0][1] = dist;
        }
    }

    fn trace(parax_elems: &[ParaxElem], mut ray: Vec2) -> Vec<ParaxTraceResult> {
        // There's always one more non-gap element than there are gaps.
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
                        elem_id: *id,
                        elem_radius: *radius,
                    });
                }
                ParaxElem::ObjectPlane { id, radius, rtm } => {
                    ray = rtm * &ray;
                    results.push(ParaxTraceResult {
                        ray: ray.clone(),
                        elem_id: *id,
                        elem_radius: *radius,
                    });
                }
                ParaxElem::Surf { id, radius, rtm } => {
                    ray = rtm * &ray;
                    results.push(ParaxTraceResult {
                        ray: ray.clone(),
                        elem_id: *id,
                        elem_radius: *radius,
                    });
                }
                ParaxElem::ThinLens { id, radius, rtm } => {
                    ray = rtm * &ray;
                    results.push(ParaxTraceResult {
                        ray: ray.clone(),
                        elem_id: *id,
                        elem_radius: *radius,
                    });
                }
            }
        }

        results
    }

    /// Find the ID of the surface that is the aperture stop of the system.
    ///
    /// This algorithm works by launching a ray from the object space the system and finds the
    /// surface where the ray is the closest to the optical axis. For objects at infinity, a ray
    /// parallel to the axis is traced starting from the first surface.
    pub fn aperture_stop(&self) -> Result<usize> {
        if self.is_empty() {
            bail!("The paraxial model is empty.");
        };

        let init_ray = self.init_ray()?;
        let results = if self.is_obj_at_infinity() {
            // Don't trace the ray through the object space.
            ParaxialModel::trace(&self.parax_elems[2..], init_ray)
        } else {
            ParaxialModel::trace(&self.parax_elems, init_ray)
        };

        // Find the ID of the non-gap, non-image plane element with the smallest ratio of surface radius to ray height.
        let mut min_ratio = f32::MAX;
        let mut min_id = 0;

        // Don't include the last result, which is the ray striking the image plane.
        for result in results.iter().take(results.len() - 1) {
            let ratio = result.elem_radius / result.ray.y();
            if ratio < min_ratio {
                min_ratio = ratio;
                min_id = result.elem_id;
            }
        }

        Ok(min_id)
    }

    /// Find the distance of the entrance pupil from the first optical surface.
    ///
    /// Following this module's sign conventions, a positive distance means the entrance pupil is
    /// virtual, i.e. to the right of the first optical surface. A negative distance means the
    /// entrance pupil is a real image of the aperture stop, i.e. to the left of the first optical
    /// surface.
    pub fn entrance_pupil(&self) -> Result<f32> {
        let aperture_stop_id = self.aperture_stop()?;

        // Find the index of the element that is the aperture stop.
        let idx = self.id_to_index(aperture_stop_id)?;

        // If the element is the first surface after the object and object space gap, return 0.0.
        if idx == 2 {
            return Ok(0.0);
        }

        // Launch a ray from the aperture stop from the axis backwards through the system.
        let elems = &self.parax_elems[0..idx];
        let elems_reversed = ParaxialModel::reverse_surfaces(elems);
        let init_ray = Vec2::new(0.0, INIT_ANGLE);
        let results = ParaxialModel::trace(&elems_reversed, init_ray);

        // Get the next-to-last result, which is the ray traveling in the object space from the
        // first surface to the object plane.
        let obj_ray = &results[results.len() - 2].ray;

        // Find the intersection of the object space ray with the optical axis.
        let t = obj_ray.y() / obj_ray.theta();

        Ok(t)
    }

    /// Return the index of the paraxial element that corresponds to the given element ID.
    fn id_to_index(&self, id: usize) -> Result<usize> {
        let idx = self
            .parax_elems
            .iter()
            .position(|elem| {
                if let ParaxElem::Surf { id: elem_id, .. } = elem {
                    *elem_id == id
                } else if let ParaxElem::ThinLens { id: elem_id, .. } = elem {
                    *elem_id == id
                } else {
                    false
                }
            })
            .ok_or(anyhow!("The element ID was not found."))?;

        Ok(idx)
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
        let mut elems = Vec::new();

        // The object plane RTM does not do anything.
        elems.push(ParaxElem::new_no_op_surf(obj_plane_radius));

        for pair in SurfacePairIterator::new(surfs) {
            // The ray transfer matrix for the second surface in the pair.
            let surf = ParaxElem::from(&pair);
            let (_, gap) = pair.into();

            elems.push(ParaxElem::from(gap));
            elems.push(surf);
        }

        Self { parax_elems: elems }
    }
}

impl From<&SurfacePair> for ParaxElem {
    /// Return the paraxial element equivalent for the second surface in the pair.
    fn from(surface_pair: &SurfacePair) -> ParaxElem {
        let surf = surface_pair.1;
        let n_0 = surface_pair.0.n();
        let n_1 = surface_pair.1.n();

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

impl From<Gap> for ParaxElem {
    fn from(gap: Gap) -> Self {
        let d = gap.thickness();
        ParaxElem::new_gap(d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f32 = 0.001;

    fn assert_almost_equal(a: f32, b: f32) {
        assert!((a - b).abs() < TOL);
    }

    struct ExpectedTestResults {
        aperture_stop_idx: usize,
        entrance_pupil: f32,
    }

    /// A two lens system used for verification testing of the paraixal model.
    fn two_lens_system_verification() -> (ParaxialModel, ExpectedTestResults) {
        // Object space is 100 mm.
        let lens_1 = ParaxElem::new_thin_lens(100.0, 100.0);
        let gap_1 = ParaxElem::new_gap(25.0);
        let stop = ParaxElem::new_no_op_surf(10.0);
        let gap_2 = ParaxElem::new_gap(25.0);
        let lens_2 = ParaxElem::new_thin_lens(100.0, 75.0);
        let img_space = ParaxElem::new_gap(75.0);

        let mut parax_model = ParaxialModel::new();
        parax_model
            .insert_element_and_gap(1, lens_1, gap_1)
            .unwrap();
        parax_model.insert_element_and_gap(2, stop, gap_2).unwrap();
        parax_model
            .insert_element_and_gap(3, lens_2, img_space)
            .unwrap();

        parax_model.set_obj_dist(100.0);

        let expected = ExpectedTestResults {
            aperture_stop_idx: 4,
            entrance_pupil: 33.3333,
        };

        (parax_model, expected)
    }

    #[test]
    fn verify_two_lens_system_aperture_stop() {
        let (parax_model, expected) = two_lens_system_verification();
        let idx = if let ParaxElem::Surf { id, .. } =
            parax_model.elements()[expected.aperture_stop_idx]
        {
            id
        } else {
            panic!("The aperture stop should be a surface.");
        };
        let result = parax_model.aperture_stop().unwrap();

        assert_eq!(result, idx);
    }

    #[test]
    fn verify_two_lens_system_entrance_pupil() {
        let (parax_model, expected) = two_lens_system_verification();
        let result = parax_model.entrance_pupil().unwrap();

        assert_almost_equal(result, expected.entrance_pupil);
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

    #[test]
    fn test_paraxial_model_remove_element_and_gap() {
        let mut parax_model = ParaxialModel::new();
        let lens = ParaxElem::new_thin_lens(100.0, 100.0);
        let gap = ParaxElem::new_gap(100.0);

        parax_model.insert_element_and_gap(1, lens, gap).unwrap();
        parax_model.remove_element_and_gap(1).unwrap();

        let elements = parax_model.elements();

        // Object plane, object space, image plane.
        assert_eq!(elements.len(), 3);
    }

    #[test]
    fn test_paraxial_model_set_obj_dist() {
        let mut parax_model = ParaxialModel::new();
        let lens = ParaxElem::new_thin_lens(100.0, 100.0);
        let gap = ParaxElem::new_gap(100.0);

        parax_model.insert_element_and_gap(1, lens, gap).unwrap();
        parax_model.set_obj_dist(200.0);

        let elements = parax_model.elements();

        // Object plane, object space, lens, image space, image plane.
        assert_eq!(elements.len(), 5);
        assert_eq!(elements[1], ParaxElem::new_gap(200.0));
    }
}