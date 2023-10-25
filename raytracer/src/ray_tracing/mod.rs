pub mod component_model;
mod paraxial_model;
pub mod rays;
pub mod sequential_model;
pub mod surface_types;
pub mod trace;

use std::f32::consts::PI;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::math::mat3::Mat3;
use crate::math::vec3::Vec3;

use component_model::ComponentModel;
use paraxial_model::ParaxialModel;
use rays::Ray;
use sequential_model::{SequentialModel, SurfaceSpec};
use surface_types::{
    ImagePlane, ObjectPlane, RefractingCircularConic, RefractingCircularFlat, Stop,
};

const INIT_DIAM: f32 = 25.0;

#[derive(Debug)]
pub(crate) struct SystemBuilder {
    surfaces: Vec<SurfaceSpec>,
    gaps: Vec<Gap>,
    aperture: Option<ApertureSpec>,
    fields: Vec<FieldSpec>,
    background: f32,
}

impl SystemBuilder {
    pub fn new() -> Self {
        Self {
            surfaces: Vec::new(),
            gaps: Vec::new(),
            aperture: None,
            fields: Vec::new(),
            background: 1.0, // The background refractive index; hardcoded to air for now
        }
    }

    pub fn surfaces(&mut self, surfaces: Vec<SurfaceSpec>) -> &mut Self {
        self.surfaces = surfaces;
        self
    }

    pub fn gaps(&mut self, gaps: Vec<Gap>) -> &mut Self {
        self.gaps = gaps;
        self
    }

    pub fn aperture(&mut self, aperture: ApertureSpec) -> &mut Self {
        self.aperture = Some(aperture);
        self
    }

    pub fn fields(&mut self, fields: Vec<FieldSpec>) -> &mut Self {
        self.fields = fields;
        self
    }

    pub fn build(&self) -> Result<SystemModel> {
        let aperture = self
            .aperture
            .ok_or(anyhow!("The system aperture must be specified."))?;
        let model = SystemModel::new(
            &self.surfaces,
            &self.gaps,
            &aperture,
            &self.fields,
            self.background,
        )?;

        Ok(model)
    }
}

/// A model of an optical system.
#[derive(Debug)]
pub struct SystemModel {
    comp_model: ComponentModel,
    parax_model: ParaxialModel,
    seq_model: SequentialModel,
    aperture: ApertureSpec,
    fields: Vec<FieldSpec>,
    background: f32,
}

impl SystemModel {
    pub fn new(
        surface_specs: &[SurfaceSpec],
        gaps: &[Gap],
        aperture: &ApertureSpec,
        fields: &[FieldSpec],
        background: f32,
    ) -> Result<SystemModel> {
        SystemModel::validate_surface_specs_and_gaps(surface_specs, gaps)?;
        let surfaces = Self::specs_to_surfs(surface_specs, gaps);

        let sequential_model = SequentialModel::new(&surfaces);
        let component_model = ComponentModel::new(&surfaces, background);
        let paraxial_model = ParaxialModel::from(sequential_model.surfaces());

        let model = Self {
            comp_model: component_model,
            seq_model: sequential_model,
            parax_model: paraxial_model,
            aperture: aperture.clone(),
            fields: fields.to_vec(),
            background: background,
        };

        Ok(model)
    }

    fn validate_surface_specs_and_gaps(surfaces: &[SurfaceSpec], gaps: &[Gap]) -> Result<()> {
        if surfaces.len() < 2 {
            return Err(anyhow::anyhow!("At least two surfaces are required"));
        }

        if surfaces.len() != gaps.len() + 1 {
            return Err(anyhow::anyhow!(
                "The number of gaps must be one less than the number of surfaces"
            ));
        }

        let first_surface = surfaces
            .first()
            .expect("At least two surfaces are required");
        if let SurfaceSpec::ObjectPlane { .. } = first_surface {
        } else {
            return Err(anyhow::anyhow!("The first surface must be an object plane"));
        }

        let last_surface = surfaces.last().expect("At least two surfaces are required");
        if let SurfaceSpec::ImagePlane { .. } = last_surface {
        } else {
            return Err(anyhow::anyhow!("The last surface must be an image plane"));
        }

        for surf in surfaces[1..surfaces.len() - 1].iter() {
            if let SurfaceSpec::ObjectPlane { .. } = surf {
                return Err(anyhow::anyhow!("There can only be one object surface"));
            }

            if let SurfaceSpec::ImagePlane { .. } = surf {
                return Err(anyhow::anyhow!("There can only be one image surface"));
            }
        }

        Ok(())
    }

    /// Convert surface specs into surface objects.
    fn specs_to_surfs(surface_specs: &[SurfaceSpec], gaps: &[Gap]) -> Vec<Surface> {
        // Construct surfaces from the surface specs and gaps
        let mut surfaces = Vec::new();
        for (surf, gap) in surface_specs.iter().zip(gaps.iter()) {
            let surface: Surface = Surface::from((surf, gap));
            surfaces.push(surface);
        }

        // Add the image plane, which was left out of the zip iterator above
        let pos = Vec3::new(0.0, 0.0, 0.0);
        let dir = Vec3::new(0.0, 0.0, 0.0);
        let diam = if let SurfaceSpec::ImagePlane { diam } = surface_specs.last().unwrap() {
            *diam
        } else {
            panic!("The last surface must be an image plane");
        };
        let img_plane = Surface::new_img_plane(pos, dir, diam);
        surfaces.push(img_plane);

        // Shift all surfaces so that the first non-object surface is at z = 0
        let obj_z = -gaps[0].thickness;
        surfaces[0].set_pos(Vec3::new(0.0, 0.0, obj_z));
        surfaces[1].set_pos(pos);
        let mut dist = 0.0;
        for (surf, gap) in surfaces[2..].iter_mut().zip(gaps[1..].iter()) {
            dist += gap.thickness;
            surf.set_pos(pos + Vec3::new(0.0, 0.0, dist));
        }

        surfaces
    }

    /// Creates a new SystemModel with an object plane and an image plane.
    ///
    /// By convention, the first non-object surface lies at z = 0.
    pub fn old() -> Self {
        let obj_plane = Surface::new_obj_plane(
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(0.0, 0.0, 0.0),
            INIT_DIAM,
        );
        let img_plane = Surface::new_img_plane(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            INIT_DIAM,
        );

        let mut surfaces = Vec::new();

        surfaces.push(obj_plane);
        surfaces.push(img_plane);

        let sequential_model = SequentialModel::new(&surfaces);
        let component_model = ComponentModel::new(&surfaces, 1.0f32);
        let paraxial_model = ParaxialModel::from(sequential_model.surfaces());

        let fields = vec![FieldSpec { angle: 0.0 }, FieldSpec { angle: 5.0 }];

        Self {
            comp_model: component_model,
            seq_model: SequentialModel::new(&surfaces),
            parax_model: paraxial_model,
            aperture: ApertureSpec::EntrancePupilDiameter { diam: INIT_DIAM },
            fields: fields,
            background: 1.0,
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

    pub fn insert_surface_and_gap(
        &mut self,
        idx: usize,
        surface_spec: SurfaceSpec,
        gap: Gap,
    ) -> Result<()> {
        let seq_model = self.seq_model_mut();
        let surface: Surface = Surface::from((&surface_spec, &gap));
        let preceding_surface = seq_model
            .surfaces()
            .get(idx - 1)
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
        self.seq_model.remove_surface_and_gap(idx)?;
        self.parax_model.remove_element_and_gap(idx)?;

        Ok(())
    }

    pub fn aperture(&self) -> &ApertureSpec {
        &self.aperture
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

    pub fn fields(&self) -> &[FieldSpec] {
        &self.fields
    }

    /// Return the background refractive index.
    pub fn background(&self) -> f32 {
        self.background
    }

    /// Determine the entrance pupil for the system.
    pub(crate) fn entrance_pupil(&self) -> Result<EntrancePupil> {
        // The diameter is the aperture diameter (until more aperture types are supported)
        let diam = match self.aperture() {
            ApertureSpec::EntrancePupilDiameter { diam } => *diam,
        };

        let entrance_pupil_dist = self.parax_model.entrance_pupil()?;

        let pos = self.seq_model.surfaces()[1].pos();
        let pos = Vec3::new(pos.x(), pos.y(), pos.z() - entrance_pupil_dist);

        Ok(EntrancePupil { pos, diam })
    }

    pub(crate) fn object_plane(&self) -> Surface {
        self.seq_model.surfaces()[0]
    }

    pub fn set_obj_space(&mut self, n: f32, thickness: f32) -> Result<()> {
        if thickness <= 0.0 {
            return Err(anyhow::anyhow!("Object space thickness must be positive"));
        };

        self.seq_model.set_obj_space(Gap::new(n, thickness));
        self.parax_model.set_obj_dist(thickness);

        Ok(())
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
    /// * `phi` - The angle of the ray w.r.t. the z-axis.
    pub(crate) fn pupil_ray_fan(&self, num_rays: usize, theta: f32, phi: f32) -> Result<Vec<Ray>> {
        let ep = self.entrance_pupil()?;
        let obj_z = self.object_plane().pos().z();
        let sur_z = self.seq_model.surfaces()[1].pos().z();
        let enp_z = ep.pos().z();

        let launch_point_z = self.axial_launch_point(obj_z, sur_z, enp_z);

        // Determine the radial distance from the axis at the launch point for the center of the
        // ray fan.
        let dz = enp_z - launch_point_z;
        let dr = -dz * phi.tan();

        let rays = Ray::fan(num_rays, ep.diam() / 2.0, theta, launch_point_z, phi, dr);

        Ok(rays)
    }

    /// Determine the axial launch point for the rays.
    ///
    /// If the object plane is at infinity, and if the first surface lies before the entrance
    /// pupil, then launch the rays from one unit to the left of the first surface. If the object
    /// plane is at infinity, and if it comes after the entrance pupil, then launch the rays from
    /// one unit in front of the entrance pupil. Otherwise, launch the rays from the object plane.
    fn axial_launch_point(&self, obj_z: f32, sur_z: f32, enp_z: f32) -> f32 {
        if obj_z == f32::NEG_INFINITY && sur_z <= enp_z {
            sur_z - 1.0
        } else if obj_z == f32::NEG_INFINITY && sur_z > enp_z {
            enp_z - 1.0
        } else {
            obj_z
        }
    }
}

/// A surface in an optical system that can interact with light rays.
#[derive(Debug, Clone, Copy)]
pub enum Surface {
    ImagePlane(ImagePlane),
    ObjectPlane(ObjectPlane),
    RefractingCircularConic(RefractingCircularConic),
    RefractingCircularFlat(RefractingCircularFlat),
    Stop(Stop),
}

impl Surface {
    pub fn new_img_plane(pos: Vec3, dir: Vec3, diam: f32) -> Self {
        let n = 1.0;
        Self::ImagePlane(ImagePlane::new(pos, dir, diam, n))
    }

    pub fn new_obj_plane(pos: Vec3, dir: Vec3, diam: f32) -> Self {
        let n = 1.0;
        Self::ObjectPlane(ObjectPlane::new(pos, dir, diam, n))
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
            Self::ImagePlane(surf) => surf.sag_norm(pos),
            Self::ObjectPlane(surf) => surf.sag_norm(pos),
            Self::RefractingCircularConic(surf) => surf.sag_norm(pos),
            Self::RefractingCircularFlat(surf) => surf.sag_norm(pos),
            Self::Stop(surf) => surf.sag_norm(pos),
        }
    }

    /// Return the position of the surface in the global coordinate system.
    #[inline]
    pub fn pos(&self) -> Vec3 {
        match self {
            Self::ImagePlane(surf) => surf.pos,
            Self::ObjectPlane(surf) => surf.pos,
            Self::RefractingCircularConic(surf) => surf.pos,
            Self::RefractingCircularFlat(surf) => surf.pos,
            Self::Stop(surf) => surf.pos,
        }
    }

    pub fn set_pos(&mut self, pos: Vec3) {
        match self {
            Self::ImagePlane(surf) => surf.pos = pos,
            Self::ObjectPlane(surf) => surf.pos = pos,
            Self::RefractingCircularConic(surf) => surf.pos = pos,
            Self::RefractingCircularFlat(surf) => surf.pos = pos,
            Self::Stop(surf) => surf.pos = pos,
        }
    }

    /// Return the rotation matrix from the global to the surface's coordinate system.
    #[inline]
    pub fn rot_mat(&self) -> Mat3 {
        match self {
            Self::ImagePlane(surf) => surf.rot_mat,
            Self::ObjectPlane(surf) => surf.rot_mat,
            Self::RefractingCircularConic(surf) => surf.rot_mat,
            Self::RefractingCircularFlat(surf) => surf.rot_mat,
            Self::Stop(surf) => surf.rot_mat,
        }
    }

    /// Return the diameter of the surface.
    #[inline]
    pub fn diam(&self) -> f32 {
        match self {
            Self::ImagePlane(surf) => surf.diam,
            Self::ObjectPlane(surf) => surf.diam,
            Self::RefractingCircularConic(surf) => surf.diam,
            Self::RefractingCircularFlat(surf) => surf.diam,
            Self::Stop(surf) => surf.diam,
        }
    }

    /// Return the refractive index of the surface.
    #[inline]
    pub fn n(&self) -> f32 {
        match self {
            Self::ImagePlane(surf) => surf.n,
            Self::ObjectPlane(surf) => surf.n,
            Self::RefractingCircularConic(surf) => surf.n,
            Self::RefractingCircularFlat(surf) => surf.n,
            Self::Stop(surf) => surf.n,
        }
    }

    /// Return the radius of curvature of the surface.
    #[inline]
    pub fn roc(&self) -> f32 {
        match self {
            Self::ImagePlane(_) => f32::INFINITY,
            Self::ObjectPlane(_) => f32::INFINITY,
            Self::RefractingCircularConic(surf) => surf.roc,
            Self::RefractingCircularFlat(_) => f32::INFINITY,
            Self::Stop(_) => f32::INFINITY,
        }
    }

    /// Determine sequential point samples on the surface in the y-z plane.
    pub fn sample_yz(&self, num_samples: usize) -> Vec<Vec3> {
        // Skip object or image planes at infinity
        match self {
            Self::ObjectPlane(surf) => {
                if surf.pos.z().abs() == f32::INFINITY {
                    return Vec::new();
                }
            }
            Self::ImagePlane(surf) => {
                if surf.pos.z().abs() == f32::INFINITY {
                    return Vec::new();
                }
            }
            _ => {}
        }

        let diam = self.diam();

        // Sample the surface in in the y,z plane by creating uniformally spaced (0,y,z) coordinates
        let sample_points = Vec3::fan(num_samples, diam / 2.0, PI / 2.0, 0.0, 0.0);

        let mut sample: Vec3;
        let mut rot_sample: Vec3;
        let mut samples = Vec::with_capacity(sample_points.len());
        for point in sample_points {
            let (sag, _) = match self {
                Self::ImagePlane(surf) => surf.sag_norm(point),
                Self::ObjectPlane(surf) => surf.sag_norm(point),
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
        // TODO Validate n and thickness
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
            SurfaceSpec::ImagePlane { diam } => {
                let surf = Surface::new_img_plane(pos, dir, *diam);
                surf
            }
            SurfaceSpec::ObjectPlane { diam } => {
                let surf = Surface::new_obj_plane(pos, dir, *diam);
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

impl From<SurfacePair> for (Surface, Gap) {
    fn from(value: SurfacePair) -> Self {
        let thickness = value.1.pos().z() - value.0.pos().z();
        match value.0 {
            Surface::ImagePlane(surf) => {
                let gap = Gap::new(surf.n, thickness);
                (value.0, gap)
            }
            Surface::ObjectPlane(surf) => {
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

/// Specifies a field.
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSpec {
    /// The angle the field makes with the optical axis, in degrees.
    angle: f32,
}

impl FieldSpec {
    pub fn new(angle: f32) -> Self {
        Self { angle }
    }

    /// Return the angle the field makes with the optical axis, in degrees.
    #[inline]
    pub fn angle(&self) -> f32 {
        self.angle
    }
}

/// Specifies the aperture of an optical system.
///
/// For the moment, the entrance pupil is assumed to lie at the first surface, but this is not
/// valid in general.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum ApertureSpec {
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

    struct ExpectedTestResults {
        entrance_pupil_pos: f32,
        entrance_pupil_diam: f32,
    }

    fn verification_planoconvex_lens_obj_at_inf() -> (SystemModel, ExpectedTestResults) {
        // A f = +50.1 mm planoconvex lens: https://www.thorlabs.com/thorproduct.cfm?partnumber=LA1255
        // Object is at infinity; aperture stop is the first surface.
        let surf_1 = SurfaceSpec::RefractingCircularConic {
            diam: 25.0,
            roc: 25.8,
            k: 0.0,
        };
        let gap_1 = Gap::new(1.515, 5.3);
        let surf_2 = SurfaceSpec::RefractingCircularFlat { diam: 25.0 };
        let gap_2 = Gap::new(1.0, 46.6);

        let mut model = SystemModel::old();
        model.insert_surface_and_gap(1, surf_1, gap_1).unwrap();
        model.insert_surface_and_gap(2, surf_2, gap_2).unwrap();
        model.set_obj_space(1.0, f32::INFINITY).unwrap();

        let expected = ExpectedTestResults {
            entrance_pupil_pos: 0.0,
            entrance_pupil_diam: 25.0,
        };

        (model, expected)
    }

    #[test]
    fn verify_planoconvex_lens_obj_at_inf() {
        let (model, expected) = verification_planoconvex_lens_obj_at_inf();

        let entrance_pupil = model.entrance_pupil().unwrap();

        assert_eq!(
            entrance_pupil.pos(),
            Vec3::new(0.0, 0.0, expected.entrance_pupil_pos)
        );
        assert_eq!(entrance_pupil.diam(), expected.entrance_pupil_diam);
    }

    fn verification_planoconvex_lens_finite_obj() -> (SystemModel, ExpectedTestResults) {
        // A f = +50.1 mm planoconvex lens: https://www.thorlabs.com/thorproduct.cfm?partnumber=LA1255
        // Object is at -46.6 mm from the flat surface; aperture stop is the second surface.
        let surf_1 = SurfaceSpec::RefractingCircularFlat { diam: 25.0 };
        let gap_1 = Gap::new(1.515, 5.3);
        let surf_2 = SurfaceSpec::RefractingCircularConic {
            diam: 25.0,
            roc: -25.8,
            k: 0.0,
        };
        let gap_2 = Gap::new(1.0, f32::INFINITY);

        let mut model = SystemModel::old();
        model.insert_surface_and_gap(1, surf_1, gap_1).unwrap();
        model.insert_surface_and_gap(2, surf_2, gap_2).unwrap();
        model.set_obj_space(1.0, 46.6).unwrap();

        let expected = ExpectedTestResults {
            entrance_pupil_pos: 3.4983,
            entrance_pupil_diam: 25.0,
        };

        (model, expected)
    }

    #[test]
    fn verify_planoconvex_lens_finite_obj() {
        let (model, expected) = verification_planoconvex_lens_finite_obj();

        let entrance_pupil = model.entrance_pupil().unwrap();

        assert_eq!(
            entrance_pupil.pos(),
            Vec3::new(0.0, 0.0, expected.entrance_pupil_pos)
        );
        assert_eq!(entrance_pupil.diam(), expected.entrance_pupil_diam);
    }

    #[test]
    fn test_sample_yz_object_plane_at_infinity() {
        let surf = Surface::new_obj_plane(
            Vec3::new(0.0, 0.0, f32::NEG_INFINITY),
            Vec3::new(0.0, 0.0, 1.0),
            4.0,
        );
        let samples = surf.sample_yz(20);
        assert_eq!(samples.len(), 0);
    }

    #[test]
    fn test_sample_yz_image_plane_at_infinity() {
        let surf = Surface::new_img_plane(
            Vec3::new(0.0, 0.0, f32::INFINITY),
            Vec3::new(0.0, 0.0, 1.0),
            4.0,
        );
        let samples = surf.sample_yz(20);
        assert_eq!(samples.len(), 0);
    }

    #[test]
    fn test_sample_yz_finite_object_plane() {
        let surf = Surface::new_obj_plane(Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 0.0, 1.0), 4.0);
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
        let mut model = SystemModel::old();
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

    #[test]
    fn test_axial_launch_point() {
        let model = SystemModel::old();

        let obj_z = f32::NEG_INFINITY;
        let sur_z = 0.0;
        let enp_z = 0.0;

        let launch_point = model.axial_launch_point(obj_z, sur_z, enp_z);
        assert_eq!(launch_point, sur_z - 1.0);

        let obj_z = f32::NEG_INFINITY;
        let sur_z = 0.0;
        let enp_z = 1.0;

        let launch_point = model.axial_launch_point(obj_z, sur_z, enp_z);
        assert_eq!(launch_point, sur_z - 1.0);

        let obj_z = f32::NEG_INFINITY;
        let sur_z = 0.0;
        let enp_z = -5.0;

        let launch_point = model.axial_launch_point(obj_z, sur_z, enp_z);
        assert_eq!(launch_point, enp_z - 1.0);

        let obj_z = -10.0;
        let sur_z = 0.0;
        let enp_z = 0.0;

        let launch_point = model.axial_launch_point(obj_z, sur_z, enp_z);
        assert_eq!(launch_point, obj_z);
    }
}
