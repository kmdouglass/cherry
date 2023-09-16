pub mod component_model;
mod paraxial_model;
pub mod rays;
pub mod sequential_model;
pub mod surface_types;
pub mod trace;

use std::{borrow::BorrowMut, f32::consts::PI};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::math::mat3::Mat3;
use crate::math::vec3::Vec3;

use component_model::ComponentModel;
use paraxial_model::ParaxialModel;
use rays::Ray;
use sequential_model::{SequentialModel, SurfaceSpec};
use surface_types::{ObjectOrImagePlane, RefractingCircularConic, RefractingCircularFlat, Stop};

const INIT_DIAM: f32 = 25.0;

/// A model of an optical system.
#[derive(Debug)]
pub(crate) struct SystemModel {
    comp_model: ComponentModel,
    parax_model: ParaxialModel,
    seq_model: SequentialModel,
    aperture: ApertureSpec,
}

impl SystemModel {
    /// Creates a new SystemModel with an object plane and an image plane.
    ///
    /// By convention, the first non-object surface lies at z = 0.
    pub fn new() -> Self {
        let obj_plane = Surface::new_obj_or_img_plane(
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(0.0, 0.0, 0.0),
            INIT_DIAM,
        );
        let img_plane = Surface::new_obj_or_img_plane(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            INIT_DIAM,
        );

        let mut surfaces = Vec::new();

        surfaces.push(obj_plane);
        surfaces.push(img_plane);

        let sequential_model = SequentialModel::new(&surfaces);
        let component_model = ComponentModel::from(&sequential_model);
        let paraxial_model = ParaxialModel::from(sequential_model.surfaces());

        Self {
            comp_model: component_model,
            seq_model: SequentialModel::new(&surfaces),
            parax_model: paraxial_model,
            aperture: ApertureSpec::EntrancePupilDiameter { diam: INIT_DIAM },
        }
    }

    pub fn comp_model(&self) -> &ComponentModel {
        &self.comp_model
    }

    pub fn seq_model(&self) -> &SequentialModel {
        &self.seq_model
    }

    pub fn seq_model_mut(&mut self) -> &mut SequentialModel {
        &mut self.seq_model
    }

    pub fn aperture(&self) -> &ApertureSpec {
        &self.aperture
    }

    pub fn insert_surface_and_gap(
        &mut self,
        idx: usize,
        surface_spec: SurfaceSpec,
        gap: Gap,
    ) -> Result<()> {
        // TODO Can this be made atomic across all the submodels?
        let seq_model = self.seq_model_mut();
        let surface: Surface = Surface::from((&surface_spec, &gap));
        let preceding_surface = seq_model
            .surfaces()
            .get(idx)
            .ok_or(anyhow!(
                "Surface index is out of bounds: {} >= {}",
                idx,
                seq_model.surfaces().len()
            ))?
            .clone();

        seq_model.insert_surface_and_gap(idx, surface, gap)?;
        self.parax_model
            .insert_surface_and_gap(idx, preceding_surface, surface, gap)?;

        Ok(())
    }

    pub fn remove_surface_and_gap(&mut self, idx: usize) -> Result<()> {
        // TODO Can this be made atomic across all the submodels?
        self.seq_model.remove_surface_and_gap(idx)?;
        self.parax_model.remove_element_and_gap(idx)?;

        Ok(())
    }

    pub fn set_aperture(&mut self, aperture: ApertureSpec) -> Result<()> {
        if let ApertureSpec::EntrancePupilDiameter { diam } = aperture {
            if diam <= 0.0 {
                return Err(anyhow::anyhow!("Entrance pupil diameter must be positive"));
            }
        }

        self.aperture = aperture;

        Ok(())
    }

    /// Determine the entrance pupil for the system.
    pub(crate) fn entrance_pupil(&self) -> Result<EntrancePupil> {
        // The diameter is the aperture diameter (until more aperture types are supported)
        let diam = match self.aperture() {
            ApertureSpec::EntrancePupilDiameter { diam } => *diam,
        };

        let entrance_pupil_dist = self.parax_model.entrance_pupil()?;

        let pos = self.seq_model.surfaces()[1].pos();
        let pos = Vec3::new(pos.x(), pos.y(), pos.z() + entrance_pupil_dist);

        Ok(EntrancePupil { pos, diam })
    }

    pub(crate) fn object_plane(&self) -> Surface {
        self.seq_model.surfaces()[0]
    }

    pub(crate) fn image_plane(&self) -> Surface {
        self.seq_model.surfaces()[self.seq_model.surfaces().len() - 1]
    }

    /// Create a linear ray fan that passes through the entrance pupil.
    ///
    /// # Arguments
    ///
    /// * `num_rays` - The number of rays in the fan.
    /// * `theta` - The polar angle of the ray fan in the x-y plane.
    /// * `phi` - The angle of the ray w.rt. the z-axis.
    pub(crate) fn pupil_ray_fan(&self, num_rays: usize, theta: f32, phi: f32) -> Result<Vec<Ray>> {
        // TODO Handle off-axis rays
        let ep = self.entrance_pupil()?;

        // If the object plane is at infinity, launch the rays from one unit in front of the first surface
        let launch_point_z = if self.object_plane().pos().z() == f32::NEG_INFINITY {
            self.seq_model.surfaces()[1].pos().z() - 1.0
        } else {
            self.object_plane().pos().z()
        };

        let rays = Ray::fan(num_rays, ep.diam() / 2.0, theta, launch_point_z, phi);

        Ok(rays)
    }
}

/// A surface in an optical system that can interact with light rays.
#[derive(Debug, Clone, Copy)]
pub enum Surface {
    ObjectOrImagePlane(ObjectOrImagePlane),
    RefractingCircularConic(RefractingCircularConic),
    RefractingCircularFlat(RefractingCircularFlat),
    Stop(Stop),
}

impl Surface {
    pub fn new_obj_or_img_plane(pos: Vec3, dir: Vec3, diam: f32) -> Self {
        let n = 1.0;
        Self::ObjectOrImagePlane(ObjectOrImagePlane::new(pos, dir, diam, n))
    }

    pub fn new_refr_circ_conic(pos: Vec3, dir: Vec3, diam: f32, n: f32, roc: f32, k: f32) -> Self {
        Self::RefractingCircularConic(RefractingCircularConic::new(pos, dir, diam, n, roc, k))
    }

    pub fn new_refr_circ_flat(pos: Vec3, dir: Vec3, diam: f32, n: f32) -> Self {
        Self::RefractingCircularFlat(RefractingCircularFlat::new(pos, dir, diam, n))
    }

    pub fn new_stop(pos: Vec3, dir: Vec3, diam: f32, n: f32) -> Self {
        Self::Stop(Stop::new(pos, dir, diam, n))
    }

    /// Compute the surface sag and surface normals at a given position.
    pub fn sag_norm(&self, pos: Vec3) -> (f32, Vec3) {
        match self {
            Self::ObjectOrImagePlane(surf) => surf.sag_norm(pos),
            Self::RefractingCircularConic(surf) => surf.sag_norm(pos),
            Self::RefractingCircularFlat(surf) => surf.sag_norm(pos),
            Self::Stop(surf) => surf.sag_norm(pos),
        }
    }

    /// Return the position of the surface in the global coordinate system.
    #[inline]
    pub fn pos(&self) -> Vec3 {
        match self {
            Self::ObjectOrImagePlane(surf) => surf.pos,
            Self::RefractingCircularConic(surf) => surf.pos,
            Self::RefractingCircularFlat(surf) => surf.pos,
            Self::Stop(surf) => surf.pos,
        }
    }

    pub fn set_pos(&mut self, pos: Vec3) {
        match self {
            Self::ObjectOrImagePlane(surf) => surf.pos = pos,
            Self::RefractingCircularConic(surf) => surf.pos = pos,
            Self::RefractingCircularFlat(surf) => surf.pos = pos,
            Self::Stop(surf) => surf.pos = pos,
        }
    }

    /// Return the rotation matrix from the global to the surface's coordinate system.
    #[inline]
    pub fn rot_mat(&self) -> Mat3 {
        match self {
            Self::ObjectOrImagePlane(surf) => surf.rot_mat,
            Self::RefractingCircularConic(surf) => surf.rot_mat,
            Self::RefractingCircularFlat(surf) => surf.rot_mat,
            Self::Stop(surf) => surf.rot_mat,
        }
    }

    /// Return the diameter of the surface.
    #[inline]
    pub fn diam(&self) -> f32 {
        match self {
            Self::ObjectOrImagePlane(surf) => surf.diam,
            Self::RefractingCircularConic(surf) => surf.diam,
            Self::RefractingCircularFlat(surf) => surf.diam,
            Self::Stop(surf) => surf.diam,
        }
    }

    /// Return the refractive index of the surface.
    #[inline]
    pub fn n(&self) -> f32 {
        match self {
            Self::ObjectOrImagePlane(surf) => surf.n,
            Self::RefractingCircularConic(surf) => surf.n,
            Self::RefractingCircularFlat(surf) => surf.n,
            Self::Stop(surf) => surf.n,
        }
    }

    /// Return the radius of curvature of the surface.
    #[inline]
    pub fn roc(&self) -> f32 {
        match self {
            Self::ObjectOrImagePlane(_) => f32::INFINITY,
            Self::RefractingCircularConic(surf) => surf.roc,
            Self::RefractingCircularFlat(_) => f32::INFINITY,
            Self::Stop(_) => f32::INFINITY,
        }
    }

    /// Determine sequential point samples on the surface in the y-z plane.
    pub fn sample_yz(&self, num_samples: usize) -> Vec<Vec3> {
        // Skip object or image planes at infinity
        if let Self::ObjectOrImagePlane(surf) = self {
            if surf.pos.z().abs() == f32::INFINITY {
                return Vec::new();
            }
        }

        let diam = self.diam();

        // Sample the surface in in the y,z plane by creating uniformally spaced (0,y,z) coordinates
        let sample_points = Vec3::fan(num_samples, diam / 2.0, PI / 2.0, 0.0);

        let mut sample: Vec3;
        let mut rot_sample: Vec3;
        let mut samples = Vec::with_capacity(sample_points.len());
        for point in sample_points {
            let (sag, _) = match self {
                Self::ObjectOrImagePlane(surf) => surf.sag_norm(point),
                Self::RefractingCircularConic(surf) => surf.sag_norm(point),
                Self::RefractingCircularFlat(surf) => surf.sag_norm(point),
                Self::Stop(surf) => surf.sag_norm(point),
            };

            // Transform the sample into the global coordinate system.
            sample = Vec3::new(point.x(), point.y(), sag);
            rot_sample = self.rot_mat().transpose() * (sample + self.pos());

            samples.push(rot_sample);
        }

        samples
    }
}

/// A gap between two surfaces in an optical system.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

impl From<(&SurfaceSpec, &Gap)> for Surface {
    fn from((surf, gap): (&SurfaceSpec, &Gap)) -> Self {
        let pos = Vec3::new(0.0, 0.0, 0.0);
        let dir = Vec3::new(0.0, 0.0, 0.0);

        match surf {
            SurfaceSpec::ObjectOrImagePlane { diam } => {
                let surf = Surface::new_obj_or_img_plane(pos, dir, *diam);
                surf
            }
            SurfaceSpec::RefractingCircularConic { diam, roc, k } => {
                let surf = Surface::new_refr_circ_conic(pos, dir, *diam, gap.n(), *roc, *k);
                surf
            }
            SurfaceSpec::RefractingCircularFlat { diam } => {
                let surf = Surface::new_refr_circ_flat(pos, dir, *diam, gap.n());
                surf
            }
            SurfaceSpec::Stop { diam } => {
                let surf = Surface::new_stop(pos, dir, *diam, gap.n());
                surf
            }
        }
    }
}

/// A sequential pair of surfaces in an optical system.
struct SurfacePair(Surface, Surface);

impl SurfacePair {
    /// Compute the axial distance between the two surfaces.
    fn axial_dist(&self) -> f32 {
        let pos1 = self.0.pos();
        let pos2 = self.1.pos();

        (pos2.z() - pos1.z()).abs()
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
        // Skip object and image planes
        if self.idx > self.surfaces.len() - 2 {
            return None;
        }

        let surf1 = self.surfaces[self.idx];
        let surf2 = self.surfaces[self.idx + 1];
        self.idx += 1;

        Some(SurfacePair(surf1, surf2))
    }
}

/// Specifies the aperture of an optical system.
///
/// For the moment, the entrance pupil is assumed to lie at the first surface, but this is not
/// valid in general.
#[derive(Debug, Deserialize, Serialize)]
pub(crate) enum ApertureSpec {
    EntrancePupilDiameter { diam: f32 },
}

/// The system's entrance pupil.
///
/// For now, this is assumed to lie at the first surface.
#[derive(Debug)]
pub(crate) struct EntrancePupil {
    pos: Vec3,
    diam: f32,
}

impl EntrancePupil {
    /// Return the position of the entrance pupil in the global coordinate system.
    #[inline]
    pub fn pos(&self) -> Vec3 {
        self.pos
    }

    /// Return the diameter of the entrance pupil.
    #[inline]
    pub fn diam(&self) -> f32 {
        self.diam
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_yz_object_plane_at_infinity() {
        let surf = Surface::new_obj_or_img_plane(
            Vec3::new(0.0, 0.0, f32::NEG_INFINITY),
            Vec3::new(0.0, 0.0, 1.0),
            4.0,
        );
        let samples = surf.sample_yz(20);
        assert_eq!(samples.len(), 0);
    }

    #[test]
    fn test_sample_yz_image_plane_at_infinity() {
        let surf = Surface::new_obj_or_img_plane(
            Vec3::new(0.0, 0.0, f32::INFINITY),
            Vec3::new(0.0, 0.0, 1.0),
            4.0,
        );
        let samples = surf.sample_yz(20);
        assert_eq!(samples.len(), 0);
    }

    #[test]
    fn test_sample_yz_finite_object_plane() {
        let surf =
            Surface::new_obj_or_img_plane(Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 0.0, 1.0), 4.0);
        let samples = surf.sample_yz(20);
        assert_eq!(samples.len(), 20);
    }

    #[ignore]
    #[test]
    fn test_sample_yz_x_values_are_zero() {
        let surf = Surface::new_refr_circ_conic(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            25.0,
            1.515,
            25.8,
            0.0,
        );
        let samples = surf.sample_yz(20);
        for sample in samples {
            assert_eq!(sample.x(), 0.0);
        }
    }

    // Test set_aperture_spec
    #[test]
    fn test_set_aperture_spec() {
        let mut model = SystemModel::new();
        let aperture = ApertureSpec::EntrancePupilDiameter { diam: 10.0 };

        let result = model.set_aperture(aperture);
        assert!(result.is_ok());

        let aperture = model.aperture();

        if let ApertureSpec::EntrancePupilDiameter { diam } = aperture {
            assert_eq!(*diam, 10.0);
        } else {
            panic!("ApertureSpec is not EntrancePupilDiameter");
        }
    }
}
