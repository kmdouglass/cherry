//! A pair of flat mirrors at 30 degrees to the z-axis, forming a figure Z.
//!
//!  The mirrors are separated by 100 mm. Rays starting horizontal to the z-axis
//! emerge horizontal to it.
use std::rc::Rc;

use crate::{
    EulerAngles, GapSpec, RefractiveIndexSpec, Rotation, SequentialModel, SurfaceSpec, SurfaceType,
    core::Float,
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
        refractive_index: n_air.clone(),
    };
    let gap_2 = GapSpec {
        thickness: 50.0,
        refractive_index: n_air,
    };
    let gaps = vec![gap_0, gap_1, gap_2];

    let surf_0 = SurfaceSpec::Object;
    let surf_1 = SurfaceSpec::Conic {
        semi_diameter: 12.7,
        radius_of_curvature: Float::INFINITY,
        conic_constant: 0.0,
        surf_type: SurfaceType::Reflecting,
        rotation: Rotation::IntrinsicPassiveRUF(EulerAngles((30 as Float).to_radians(), 0.0, 0.0)),
    };
    let surf_2 = SurfaceSpec::Conic {
        semi_diameter: 12.7,
        radius_of_curvature: f64::INFINITY,
        conic_constant: 0.0,
        surf_type: SurfaceType::Reflecting,
        rotation: Rotation::IntrinsicPassiveRUF(EulerAngles((30 as Float).to_radians(), 0.0, 0.0)),
    };
    let surf_3 = SurfaceSpec::Image {
        rotation: Rotation::None,
    };
    let surfaces = vec![surf_0, surf_1, surf_2, surf_3];

    SequentialModel::new(&gaps, &surfaces, wavelengths).unwrap()
}
