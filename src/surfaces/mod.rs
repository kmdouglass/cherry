mod conics;
mod flats;
mod object_or_image;

use crate::mat3::Mat3;
use crate::vec3::Vec3;

/// A surface in an optical system that can interact with light rays.
pub enum Surface {
    ObjectOrImagePlane(object_or_image::ObjectOrImagePlane),
    RefractingCircularConic(conics::RefractingCircularConic),
    RefractingCircularFlat(flats::RefractingCircularFlat),
}

impl Surface {
    pub fn new_obj_or_img_plane(axial_pos: f32) -> Self {
        let pos = Vec3::new(0.0, 0.0, axial_pos);
        let dir = Vec3::new(0.0, 0.0, 1.0);
        let n = 1.0;

        Self::ObjectOrImagePlane(object_or_image::ObjectOrImagePlane::new(pos, dir, n))
    }

    pub fn new_refr_circ_conic(axial_pos: f32, diam: f32, n: f32, roc: f32, k: f32) -> Self {
        let pos = Vec3::new(0.0, 0.0, axial_pos);
        let dir = Vec3::new(0.0, 0.0, 1.0);

        Self::RefractingCircularConic(conics::RefractingCircularConic::new(
            pos, dir, diam, n, roc, k,
        ))
    }

    pub fn new_refr_circ_flat(axial_pos: f32, diam: f32, n: f32) -> Self {
        let pos = Vec3::new(0.0, 0.0, axial_pos);
        let dir = Vec3::new(0.0, 0.0, 1.0);

        Self::RefractingCircularFlat(flats::RefractingCircularFlat::new(pos, dir, diam, n))
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

    /// Return the rotation matrix from the global to the surface's coordinate system.
    #[inline]
    pub fn rot_mat(&self) -> Mat3 {
        match self {
            Self::ObjectOrImagePlane(surf) => surf.rot_mat,
            Self::RefractingCircularConic(surf) => surf.rot_mat,
            Self::RefractingCircularFlat(surf) => surf.rot_mat,
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
