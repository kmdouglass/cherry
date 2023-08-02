pub mod component_model;
pub mod rays;
pub mod sequential_model;
pub mod surface_types;
pub mod trace;

use std::f32::INFINITY;

use crate::math::mat3::Mat3;
use crate::math::vec3::Vec3;
use anyhow::bail;
use sequential_model::{Gap, SurfaceSpec, SequentialModel};
use surface_types::{ObjectOrImagePlane, RefractingCircularConic, RefractingCircularFlat};

/// A model of an optical system.
///
/// A SystemModel can be built from a SequentialModel by iterating over (surface, gap) pairs,
/// creating the corresponding 3D surface objects in the process.
#[derive(Debug)]
pub struct SystemModel {
    surfaces: Vec<Surface>,
}

impl SystemModel {
    /// Creates a new SystemModel with an object plane and an image plane.
    ///
    /// By convention, the first non-object surface lies at z = 0.
    pub fn new() -> Self {
        let obj_plane = Surface::new_obj_or_img_plane(
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(0.0, 0.0, 1.0),
            INFINITY,
        );
        let img_plane = Surface::new_obj_or_img_plane(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            INFINITY,
        );

        let mut surfaces = Vec::new();

        surfaces.push(obj_plane);
        surfaces.push(img_plane);

        Self { surfaces }
    }
}

impl TryFrom<&SequentialModel> for SystemModel {
    type Error = anyhow::Error;

    fn try_from(value: &SequentialModel) -> Result<Self, Self::Error> {
        let mut surfaces = Vec::with_capacity(value.surfaces().len());

        // Find the starting point along the axis of the optical system.
        let t1 = value.gaps()[0].thickness();
        let mut pos: Vec3 = Vec3::new(0.0, 0.0, -t1);
        let dir: Vec3 = Vec3::new(0.0, 0.0, 1.0);

        // By convention, the number of gaps is one less than the number of sequential surfaces, so
        // the last surface  (the image plane) is ignored here.
        for (surf, gap) in value.surfaces().iter().zip(value.gaps().iter()) {
            let surf = match surf {
                SurfaceSpec::ObjectOrImagePlane { diam } => {
                    let surf = Surface::new_obj_or_img_plane(pos, dir, *diam);
                    surf
                }
                SurfaceSpec::RefractingCircularConic { diam, n, roc, k } => {
                    let surf = Surface::new_refr_circ_conic(pos, dir, *diam, *n, *roc, *k);
                    surf
                }
                SurfaceSpec::RefractingCircularFlat { diam, n } => {
                    let surf = Surface::new_refr_circ_flat(pos, dir, *diam, *n);
                    surf
                }
            };

            surfaces.push(surf);

            // Advance to the position of the next surface.
            pos += dir * gap.thickness();
        }

        // Add the image plane.
        if let SurfaceSpec::ObjectOrImagePlane { diam } = value.surfaces().last().unwrap() {
            let surf = Surface::new_obj_or_img_plane(pos, dir, *diam);
            surfaces.push(surf);
        } else {
            bail!("The last surface in the sequential model must be an image plane.")
        }

        Ok(Self { surfaces })
    }
}

/// A surface in an optical system that can interact with light rays.
#[derive(Debug, Clone, Copy)]
pub enum Surface {
    ObjectOrImagePlane(ObjectOrImagePlane),
    RefractingCircularConic(RefractingCircularConic),
    RefractingCircularFlat(RefractingCircularFlat),
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

    /// Compute the surface sag and surface normals at a given position.
    pub fn sag_norm(&self, pos: Vec3) -> (f32, Vec3) {
        match self {
            Self::ObjectOrImagePlane(surf) => surf.sag_norm(pos),
            Self::RefractingCircularConic(surf) => surf.sag_norm(pos),
            Self::RefractingCircularFlat(surf) => surf.sag_norm(pos),
        }
    }

    /// Return the position of the surface in the global coordinate system.
    #[inline]
    pub fn pos(&self) -> Vec3 {
        match self {
            Self::ObjectOrImagePlane(surf) => surf.pos,
            Self::RefractingCircularConic(surf) => surf.pos,
            Self::RefractingCircularFlat(surf) => surf.pos,
        }
    }

    pub fn set_pos(&mut self, pos: Vec3) {
        match self {
            Self::ObjectOrImagePlane(surf) => surf.pos = pos,
            Self::RefractingCircularConic(surf) => surf.pos = pos,
            Self::RefractingCircularFlat(surf) => surf.pos = pos,
        }
    }

    /// Return the rotation matrix from the global to the surface's coordinate system.
    #[inline]
    pub fn rot_mat(&self) -> Mat3 {
        match self {
            Self::ObjectOrImagePlane(surf) => surf.rot_mat,
            Self::RefractingCircularConic(surf) => surf.rot_mat,
            Self::RefractingCircularFlat(surf) => surf.rot_mat,
        }
    }

    /// Return the diameter of the surface.
    #[inline]
    pub fn diam(&self) -> f32 {
        match self {
            Self::ObjectOrImagePlane(surf) => surf.diam,
            Self::RefractingCircularConic(surf) => surf.diam,
            Self::RefractingCircularFlat(surf) => surf.diam,
        }
    }

    /// Return the refractive index of the surface.
    #[inline]
    pub fn n(&self) -> f32 {
        match self {
            Self::ObjectOrImagePlane(surf) => surf.n,
            Self::RefractingCircularConic(surf) => surf.n,
            Self::RefractingCircularFlat(surf) => surf.n,
        }
    }
}

impl From<(SurfaceSpec, Gap)> for Surface {
    fn from((surf, gap): (SurfaceSpec, Gap)) -> Self {
        let pos = Vec3::new(0.0, 0.0, 0.0);
        let dir = Vec3::new(0.0, 0.0, 1.0);

        match surf {
            SurfaceSpec::ObjectOrImagePlane { diam } => {
                let surf = Surface::new_obj_or_img_plane(pos, dir, diam);
                surf
            }
            SurfaceSpec::RefractingCircularConic { diam, n, roc, k } => {
                let surf = Surface::new_refr_circ_conic(pos, dir, diam, gap.n(), roc, k);
                surf
            }
            SurfaceSpec::RefractingCircularFlat { diam, n } => {
                let surf = Surface::new_refr_circ_flat(pos, dir, diam, gap.n());
                surf
            }
        }
    }
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
}
