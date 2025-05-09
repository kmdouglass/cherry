use crate::{FieldSpec, GapSpec, PupilSampling, SequentialModel, SurfaceSpec, SurfaceType, n};

pub fn sequential_model() -> SequentialModel {
    let air = n!(1.0);

    let gap_0 = GapSpec {
        thickness: f64::INFINITY,
        refractive_index: air.clone(),
    };
    let gap_1 = GapSpec {
        thickness: 13.0,
        refractive_index: n!(1.5168),
    };
    let gap_2 = GapSpec {
        thickness: 4.0,
        refractive_index: n!(1.6645),
    };
    let gap_3 = GapSpec {
        thickness: 40.0,
        refractive_index: air.clone(),
    };
    let gap_4 = GapSpec {
        thickness: 40.0,
        refractive_index: air.clone(),
    };
    let gap_5 = GapSpec {
        thickness: 12.0,
        refractive_index: n!(1.6074),
    };
    let gap_6 = GapSpec {
        thickness: 3.0,
        refractive_index: n!(1.6727),
    };
    let gap_7 = GapSpec {
        thickness: 46.82210,
        refractive_index: air.clone(),
    };
    let gap_8 = GapSpec {
        thickness: 2.0,
        refractive_index: n!(1.6727),
    };
    let gap_9 = GapSpec {
        thickness: 1.87179,
        refractive_index: air.clone(),
    };
    let gaps = vec![
        gap_0, gap_1, gap_2, gap_3, gap_4, gap_5, gap_6, gap_7, gap_8, gap_9,
    ];

    let surf_0 = SurfaceSpec::Object;
    let surf_1 = SurfaceSpec::Conic {
        semi_diameter: 28.478,
        radius_of_curvature: 99.56266,
        conic_constant: 0.0,
        surf_type: SurfaceType::Refracting,
    };
    let surf_2 = SurfaceSpec::Conic {
        semi_diameter: 26.276,
        radius_of_curvature: -86.84002,
        conic_constant: 0.0,
        surf_type: SurfaceType::Refracting,
    };
    let surf_3 = SurfaceSpec::Conic {
        semi_diameter: 21.02,
        radius_of_curvature: -1187.63858,
        conic_constant: 0.0,
        surf_type: SurfaceType::Refracting,
    };
    let surf_4 = SurfaceSpec::Stop {
        semi_diameter: 16.631,
    };
    let surf_5 = SurfaceSpec::Conic {
        semi_diameter: 20.543,
        radius_of_curvature: 57.47491,
        conic_constant: 0.0,
        surf_type: SurfaceType::Refracting,
    };
    let surf_6 = SurfaceSpec::Conic {
        semi_diameter: 20.074,
        radius_of_curvature: -54.61685,
        conic_constant: 0.0,
        surf_type: SurfaceType::Refracting,
    };
    let surf_7 = SurfaceSpec::Conic {
        semi_diameter: 20.074,
        radius_of_curvature: -614.68633,
        conic_constant: 0.0,
        surf_type: SurfaceType::Refracting,
    };
    let surf_8 = SurfaceSpec::Conic {
        semi_diameter: 17.297,
        radius_of_curvature: -38.17110,
        conic_constant: 0.0,
        surf_type: SurfaceType::Refracting,
    };
    let surf_9 = SurfaceSpec::Conic {
        semi_diameter: 18.94,
        radius_of_curvature: f64::INFINITY,
        conic_constant: 0.0,
        surf_type: SurfaceType::Refracting,
    };
    let surf_10 = SurfaceSpec::Image;
    let surfaces = vec![
        surf_0, surf_1, surf_2, surf_3, surf_4, surf_5, surf_6, surf_7, surf_8, surf_9, surf_10,
    ];

    let wavelengths: Vec<f64> = vec![0.567];

    SequentialModel::new(&gaps, &surfaces, &wavelengths).unwrap()
}

pub fn field_specs() -> Vec<FieldSpec> {
    vec![
        FieldSpec::Angle {
            angle: 0.0,
            pupil_sampling: PupilSampling::TangentialRayFan,
        },
        FieldSpec::Angle {
            angle: 5.0,
            pupil_sampling: PupilSampling::TangentialRayFan,
        },
    ]
}

// Paraxial View values
pub const APERTURE_STOP: usize = 4;
