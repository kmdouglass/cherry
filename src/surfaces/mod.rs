mod conics;
mod flats;

use crate::vec3::Vec3;

pub enum Surface {
    RefractingCircularConic(conics::RefractingCircularConic),
    RefractingCircularFlat(flats::RefractingCircularFlat),
}

impl Surface {
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
            Self::RefractingCircularConic(surf) => surf.sag_norm(pos),
            Self::RefractingCircularFlat(surf) => surf.sag_norm(pos),
        }
    }
}
