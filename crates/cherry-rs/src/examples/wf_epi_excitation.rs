//! A reduced version of the widefield epifluorescence microscope excitation
//! path: an object, a thin lens, a flat fold mirror, and a second thin lens
//! that is the aperture stop.
//!
//! The fold mirror has infinite radius of curvature, so it contributes no
//! optical power and can be ignored when computing paraxial quantities by
//! hand; only the unfolded track length matters.
//!
//! By hand: the stop (second thin lens) sits 150 mm (100 mm + 50 mm) after
//! the first thin lens (f = 40 mm). Treating the stop as a real object for
//! the first thin lens and solving 1/s' = 1/f - 1/s with s = 150 mm gives
//! s' = 600 / 11 = 54.5455 mm, with the image (entrance pupil) on the
//! object-space side of the first thin lens, i.e. at location -54.5455 mm
//! relative to it.
use std::rc::Rc;

use crate::{
    BoundaryKind, EulerAngles, GapSpec, RefractiveIndexSpec, Rotation3D, SequentialModel,
    SurfaceSpec, Vec3, core::Float,
};

pub fn sequential_model(
    n_air: Rc<dyn RefractiveIndexSpec>,
    n_oil: Rc<dyn RefractiveIndexSpec>,
    wavelengths: &[f64],
) -> SequentialModel {
    let gap_0 = GapSpec {
        thickness: 40.0,
        refractive_index: n_air.clone(),
    };
    let gap_1 = GapSpec {
        thickness: 100.0,
        refractive_index: n_air.clone(),
    };
    let gap_2 = GapSpec {
        thickness: 50.0,
        refractive_index: n_air,
    };
    let gap_3 = GapSpec {
        thickness: 1.0,
        refractive_index: n_oil,
    };
    let gaps = vec![gap_0, gap_1, gap_2, gap_3];

    let surf_0 = SurfaceSpec::Object;
    let surf_1 = SurfaceSpec::ThinLens {
        semi_diameter: 25.0,
        focal_length: 40.0,
        rotation: Rotation3D::None,
        decenter: Vec3::new(0.0, 0.0, 0.0),
        rotation_offset: Rotation3D::None,
    };
    let surf_2 = SurfaceSpec::Conic {
        semi_diameter: 36.0,
        radius_of_curvature: Float::INFINITY,
        conic_constant: 0.0,
        surf_kind: BoundaryKind::Reflecting,
        rotation: Rotation3D::IntrinsicPassiveRUF(EulerAngles(
            (45.0 as Float).to_radians(),
            0.0,
            0.0,
        )),
        decenter: Vec3::new(0.0, 0.0, 0.0),
        rotation_offset: Rotation3D::None,
    };
    let surf_3 = SurfaceSpec::ThinLens {
        semi_diameter: 4.25,
        focal_length: 3.3333,
        rotation: Rotation3D::None,
        decenter: Vec3::new(0.0, 0.0, 0.0),
        rotation_offset: Rotation3D::None,
    };
    let surf_4 = SurfaceSpec::Image {
        rotation: Rotation3D::None,
        decenter: Vec3::new(0.0, 0.0, 0.0),
        rotation_offset: Rotation3D::None,
    };
    let surfaces = vec![surf_0, surf_1, surf_2, surf_3, surf_4];

    SequentialModel::from_surface_specs(&gaps, &surfaces, wavelengths, Some(3)).unwrap()
}
