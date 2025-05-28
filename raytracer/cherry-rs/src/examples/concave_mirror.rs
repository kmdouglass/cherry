//! A f=+100 mm concave mirror with infinite field points.
use std::rc::Rc;

use crate::{GapSpec, RefractiveIndexSpec, Rotation, SequentialModel, SurfaceSpec, SurfaceType};

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
    let surf_1 = SurfaceSpec::Conic {
        semi_diameter: 12.5,
        radius_of_curvature: -200.0,
        conic_constant: 0.0,
        surf_type: SurfaceType::Reflecting,
        rotation: Rotation::None,
    };
    let surf_2 = SurfaceSpec::Image {
        rotation: Rotation::None,
    };
    let surfaces = vec![surf_0, surf_1, surf_2];

    SequentialModel::new(&gaps, &surfaces, wavelengths).unwrap()
}
