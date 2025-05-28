//! A f = +100 mm biconvex lens with an object at a finite distance.
//!
//! Thorlabs Part No.: LB1676-A
use std::rc::Rc;

use crate::{GapSpec, RefractiveIndexSpec, Rotation, SequentialModel, SurfaceSpec, SurfaceType};

pub fn sequential_model(
    n_air: Rc<dyn RefractiveIndexSpec>,
    n_glass: Rc<dyn RefractiveIndexSpec>,
    wavelengths: &[f64],
) -> SequentialModel {
    let gap_0 = GapSpec {
        thickness: 200.0,
        refractive_index: n_air.clone(),
    };
    let gap_1 = GapSpec {
        thickness: 3.6,
        refractive_index: n_glass,
    };
    let gap_2 = GapSpec {
        thickness: 196.1684,
        refractive_index: n_air,
    };
    let gaps = vec![gap_0, gap_1, gap_2];

    let surf_0 = SurfaceSpec::Object;
    let surf_1 = SurfaceSpec::Conic {
        semi_diameter: 12.7,
        radius_of_curvature: 102.4,
        conic_constant: 0.0,
        surf_type: SurfaceType::Refracting,
        rotation: Rotation::None,
    };
    let surf_2 = SurfaceSpec::Conic {
        semi_diameter: 12.7,
        radius_of_curvature: -102.4,
        conic_constant: 0.0,
        surf_type: SurfaceType::Refracting,
        rotation: Rotation::None,
    };
    let surf_3 = SurfaceSpec::Image {
        rotation: Rotation::None,
    };

    let surfaces = vec![surf_0, surf_1, surf_2, surf_3];

    SequentialModel::new(&gaps, &surfaces, wavelengths).unwrap()
}
