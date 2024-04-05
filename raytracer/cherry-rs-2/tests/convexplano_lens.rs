use std::vec;

use cherry_rs_2::specs::{
    aperture::ApertureSpec,
    fields::FieldSpec,
    gaps::{GapSpec, RIDataSpec, RefractiveIndexSpec},
    surfaces::{SurfaceSpec, SurfaceType},
};
use cherry_rs_2::systems::{SeqSys, SeqSysBuilder};

fn setup() -> SeqSys {
    let aperture = ApertureSpec::EntrancePupil {
        semi_diameter: 12.5,
    };

    let air = RefractiveIndexSpec::N {
        n: RIDataSpec::Constant(1.0),
    };
    // Glass: N-BK7
    let nbk7 = RefractiveIndexSpec::NAndKSeparate {
        n: RIDataSpec::Formula2 {
            wavelength_range: [0.3, 2.5],
            coefficients: vec![
                0.0,
                1.03961212,
                0.00600069867,
                0.231792344,
                0.0200179144,
                1.01046945,
                103.560653,
            ],
        },
        k: RIDataSpec::TabulatedK {
            data: vec![
                [0.3, 2.8607e-6],
                [0.31, 1.3679e-6],
                [0.32, 6.6608e-7],
                [0.334, 2.6415e-7],
                [0.35, 9.2894e-8],
                [0.365, 3.4191e-8],
                [0.37, 2.7405e-8],
                [0.38, 2.0740e-8],
                [0.39, 1.3731e-8],
                [0.4, 1.0227e-8],
                [0.405, 9.0558e-9],
                [0.42, 9.3912e-9],
                [0.436, 1.1147e-8],
                [0.46, 1.0286e-8],
                [0.5, 9.5781e-9],
                [0.546, 6.9658e-9],
                [0.58, 9.2541e-9],
                [0.62, 1.1877e-8],
                [0.66, 1.2643e-8],
                [0.7, 8.9305e-9],
                [1.06, 1.0137e-8],
                [1.53, 9.8390e-8],
                [1.97, 1.0933e-6],
                [2.325, 4.2911e-6],
                [2.5, 8.1300e-6],
            ],
        },
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

    let fields: Vec<FieldSpec> = vec![
        FieldSpec::Angle { angle: 0.0 },
        FieldSpec::Angle { angle: 5.0 },
    ];

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

    let wavelengths: Vec<f64> = vec![0.567, 0.632];

    SeqSysBuilder::new()
        .aperture(aperture)
        .fields(fields)
        .gaps(gaps)
        .surfaces(surfaces)
        .wavelengths(wavelengths)
        .build()
        .unwrap()
}

#[test]
fn test_setup() {
    setup();
}
