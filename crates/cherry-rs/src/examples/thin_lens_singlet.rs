//! A single idealized thin lens with f = 100 mm, object at infinity.
use std::rc::Rc;

use crate::{GapSpec, RefractiveIndexSpec, Rotation3D, SequentialModel, SurfaceSpec, Vec3};

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
    let surf_1 = SurfaceSpec::ThinLens {
        semi_diameter: 12.5,
        focal_length: 100.0,
        rotation: Rotation3D::None,
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
