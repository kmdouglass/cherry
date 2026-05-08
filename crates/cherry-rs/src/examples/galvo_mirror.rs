//! A single flat galvo mirror at −45° (a 90° fold) with no downstream optics.
//!
//! Object is at infinity. The mirror has no optical power, so EFL and BFD are
//! both infinite and all ray heights pass through unchanged.
use std::rc::Rc;

use crate::{
    BoundaryKind, EulerAngles, GapSpec, RefractiveIndexSpec, Rotation3D, SequentialModel,
    SurfaceSpec, Vec3, core::Float,
};

pub fn sequential_model(
    n_air: Rc<dyn RefractiveIndexSpec>,
    wavelengths: &[f64],
) -> SequentialModel {
    let gap_0 = GapSpec {
        thickness: f64::INFINITY,
        refractive_index: n_air.clone(),
    };
    let gap_1 = GapSpec {
        thickness: 100.0,
        refractive_index: n_air,
    };
    let gaps = vec![gap_0, gap_1];

    let surf_0 = SurfaceSpec::Object;
    let surf_1 = SurfaceSpec::Sphere {
        semi_diameter: 2.0,
        radius_of_curvature: Float::INFINITY,
        surf_kind: BoundaryKind::Reflecting,
        rotation: Rotation3D::IntrinsicPassiveRUF(EulerAngles(
            (-45 as Float).to_radians(),
            0.0,
            0.0,
        )),
        decenter: Vec3::new(0.0, 0.0, 0.0),
        rotation_offset: Rotation3D::None,
    };
    let surf_2 = SurfaceSpec::Image {
        rotation: Rotation3D::None,
        decenter: Vec3::new(0.0, 0.0, 0.0),
        rotation_offset: Rotation3D::None,
    };
    let surfaces = vec![surf_0, surf_1, surf_2];

    SequentialModel::from_surface_specs(&gaps, &surfaces, wavelengths, None).unwrap()
}
