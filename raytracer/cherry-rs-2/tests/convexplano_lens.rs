use std::vec;

use cherry_rs_2::specs::{
    aperture::ApertureSpec,
    fields::FieldSpec,
    gaps::{GapSpec, RealSpec, RefractiveIndexSpec},
    surfaces::{SurfaceSpec, SurfaceType},
};
use cherry_rs_2::systems::SeqSys;

fn setup() -> SeqSys {
    let aperture = ApertureSpec::EntrancePupil {
        semi_diameter: 12.5,
    };

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

    SeqSys::new(aperture, fields, gaps, surfaces, wavelengths).unwrap()
}

#[test]
fn test_setup() {
    setup();
}
