pub mod component_model;
pub mod rays;
pub mod sequential_model;
pub mod surface_types;
pub mod trace;

use std::f32::consts::PI;

use anyhow::Result;

use crate::math::mat3::Mat3;
use crate::math::vec3::Vec3;

use component_model::ComponentModel;
use sequential_model::{Gap, SequentialModel, SurfaceSpec};
use surface_types::{ObjectOrImagePlane, RefractingCircularConic, RefractingCircularFlat, Stop};

const INIT_DIAM: f32 = 25.0;

/// A model of an optical system.
#[derive(Debug)]
pub(crate) struct SystemModel {
    comp_model: ComponentModel,
    seq_model: SequentialModel,
    surfaces: Vec<Surface>,
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

        Self {
            comp_model: component_model,
            seq_model: SequentialModel::new(&surfaces),
            surfaces,
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

    pub fn aperture_spec(&self) -> &ApertureSpec {
        &self.aperture
    }

    pub fn set_aperture_spec(&mut self, aperture: ApertureSpec) -> Result<()> {
        if let ApertureSpec::EntrancePupilDiameter { diam } = aperture {
            if diam <= 0.0 {
                return Err(anyhow::anyhow!("Entrance pupil diameter must be positive"));
            }
        }
        
        self.aperture = aperture;

        Ok(())
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

impl From<(SurfaceSpec, &Gap)> for Surface {
    fn from((surf, gap): (SurfaceSpec, &Gap)) -> Self {
        let pos = Vec3::new(0.0, 0.0, 0.0);
        let dir = Vec3::new(0.0, 0.0, 0.0);

        match surf {
            SurfaceSpec::ObjectOrImagePlane { diam } => {
                let surf = Surface::new_obj_or_img_plane(pos, dir, diam);
                surf
            }
            SurfaceSpec::RefractingCircularConic { diam, roc, k } => {
                let surf = Surface::new_refr_circ_conic(pos, dir, diam, gap.n(), roc, k);
                surf
            }
            SurfaceSpec::RefractingCircularFlat { diam } => {
                let surf = Surface::new_refr_circ_flat(pos, dir, diam, gap.n());
                surf
            }
            SurfaceSpec::Stop { diam } => {
                let surf = Surface::new_stop(pos, dir, diam, gap.n());
                surf
            }
        }
    }
}

/// Specifies the aperture of an optical system.
///
/// For the moment, the entrance pupil is assumed to lie at the first surface, but this is not
/// valid in general.
#[derive(Debug)]
pub(crate) enum ApertureSpec {
    EntrancePupilDiameter { diam: f32 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_model() {
        let mut model = SystemModel::new();

        let surf = Surface::new_refr_circ_conic(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            10.0,
            1.5,
            10.0,
            0.0,
        );
        model.surfaces.push(surf);

        let surf = Surface::new_refr_circ_flat(
            Vec3::new(0.0, 0.0, 10.0),
            Vec3::new(0.0, 0.0, 1.0),
            10.0,
            1.5,
        );
        model.surfaces.push(surf);

        let surf = Surface::new_obj_or_img_plane(
            Vec3::new(0.0, 0.0, 20.0),
            Vec3::new(0.0, 0.0, 1.0),
            10.0,
        );
        model.surfaces.push(surf);

        let seq_model = SequentialModel::try_from(&model).unwrap();

        // 3 surfaces + object plane + image plane = 5 surfaces
        assert_eq!(seq_model.surfaces().len(), 5);
        assert_eq!(seq_model.gaps().len(), 4);
    }

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
        
        let result = model.set_aperture_spec(aperture);
        assert!(result.is_ok());

        let aperture = model.aperture_spec();

        if let ApertureSpec::EntrancePupilDiameter { diam } = aperture {
            assert_eq!(*diam, 10.0);
        } else {
            panic!("ApertureSpec is not EntrancePupilDiameter");
        }
    }

}
