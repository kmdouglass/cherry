use ndarray::{arr3, Array3};

use crate::{
    GapSpec, Pupil, RealSpec, RefractiveIndexSpec, SequentialModel, SurfaceSpec, SurfaceType,
};

pub fn sequential_model() -> SequentialModel {
    let air = RefractiveIndexSpec {
        real: RealSpec::Constant(1.0),
        imag: None,
    };
    // Glass: N-BK7
    let nbk7 = RefractiveIndexSpec {
        real: RealSpec::Constant(1.515),
        imag: None,
    };

    let gap_0 = GapSpec {
        thickness: f64::INFINITY,
        refractive_index: air.clone(),
    };
    let gap_1 = GapSpec {
        thickness: 5.3,
        refractive_index: nbk7,
    };
    let gap_2 = GapSpec {
        thickness: 46.6,
        refractive_index: air,
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

    let wavelengths: Vec<f64> = vec![0.567];

    SequentialModel::new(&gaps, &surfaces, &wavelengths).unwrap()
}

// Paraxial View values
pub const APERTURE_STOP: usize = 1;

pub const BACK_FOCAL_DISTANCE: f64 = 46.5987;

pub const BACK_PRINCIPAL_PLANE: f64 = 1.8017;

pub const EFFECTIVE_FOCAL_LENGTH: f64 = 50.097;

pub const ENTRANCE_PUPIL: Pupil = Pupil {
    location: 0.0,
    semi_diameter: 12.5,
};

pub const FRONT_PRINCIPAL_PLANE: f64 = 0.0;

pub fn marginal_ray() -> Array3<f64> {
    arr3(&[
        [[12.5000], [0.0]],
        [[12.5000], [-0.1647]],
        [[11.6271], [-0.2495]],
        [[-0.0003], [-0.2495]],
    ])
}
