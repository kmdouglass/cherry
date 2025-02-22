//! A f = 50 mm convexplano lens.
use std::rc::Rc;

use crate::{GapSpec, RefractiveIndexSpec, SequentialModel, SurfaceSpec, SurfaceType};

pub fn sequential_model(
    n_air: Rc<dyn RefractiveIndexSpec>,
    n_glass: Rc<dyn RefractiveIndexSpec>,
    wavelengths: &[f64],
) -> SequentialModel {
    let gap_0 = GapSpec {
        thickness: f64::INFINITY,
        refractive_index: n_air.clone(),
    };
    let gap_1 = GapSpec {
        thickness: 5.3,
        refractive_index: n_glass,
    };
    let gap_2 = GapSpec {
        thickness: 46.6,
        refractive_index: n_air,
    };
    let gaps = vec![gap_0, gap_1, gap_2];

    let surf_0 = SurfaceSpec::Object;
    let surf_1 = SurfaceSpec::Conic {
        semi_diameter: 12.5,
        radius_of_curvature: 25.8,
        conic_constant: 0.0,
        surf_type: SurfaceType::Refracting,
    };
    let surf_2 = SurfaceSpec::Conic {
        semi_diameter: 12.5,
        radius_of_curvature: f64::INFINITY,
        conic_constant: 0.0,
        surf_type: SurfaceType::Refracting,
    };
    let surf_3 = SurfaceSpec::Image;
    let surfaces = vec![surf_0, surf_1, surf_2, surf_3];

    SequentialModel::new(&gaps, &surfaces, wavelengths).unwrap()
}
