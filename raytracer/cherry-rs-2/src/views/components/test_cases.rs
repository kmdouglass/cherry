use crate::{
    core::{
        sequential_model::{self, Gap},
        Float,
    },
    GapSpec, RefractiveIndexSpec, SequentialModel, SurfaceSpec,
};

const AIR: RefractiveIndexSpec = RefractiveIndexSpec {
    real: crate::RealSpec::Constant(1.0),
    imag: None,
};

const NBK7: RefractiveIndexSpec = RefractiveIndexSpec {
    real: crate::RealSpec::Constant(1.515),
    imag: None,
};

pub fn empty_system() -> SequentialModel {
    let surf_0 = SurfaceSpec::Object;
    let gap_0 = GapSpec {
        thickness: 1.0,
        refractive_index: RefractiveIndexSpec {
            real: crate::RealSpec::Constant(1.0),
            imag: None,
        },
    };
    let surf_1 = SurfaceSpec::Image;

    let surfaces = vec![surf_0, surf_1];
    let gaps = vec![gap_0];
    let wavelengths = vec![0.567];

    SequentialModel::new(&gaps, &surfaces, &wavelengths).unwrap()
}

pub fn silly_unpaired_surface() -> SequentialModel {
    // A silly system for edge case testing only.

    let surf_0 = SurfaceSpec::Object;
    let gap_0 = GapSpec {
        thickness: Float::INFINITY,
        refractive_index: AIR,
    };
    let surf_1 = SurfaceSpec::Conic {
        semi_diameter: 12.5,
        radius_of_curvature: 25.8,
        conic_constant: 0.0,
        surf_type: crate::SurfaceType::Refracting,
    };
    let gap_1 = GapSpec {
        thickness: 5.3,
        refractive_index: NBK7,
    };
    let surf_2 = SurfaceSpec::Conic {
        semi_diameter: 12.5,
        radius_of_curvature: Float::INFINITY,
        conic_constant: 0.0,
        surf_type: crate::SurfaceType::Refracting,
    };
    let gap_2 = GapSpec {
        thickness: 46.6,
        refractive_index: AIR,
    };
    let surf_3 = SurfaceSpec::Conic {
        semi_diameter: 12.5,
        radius_of_curvature: 25.8,
        conic_constant: 0.0,
        surf_type: crate::SurfaceType::Refracting,
    }; // Surface is unpaired
    let gap_3 = GapSpec {
        thickness: 20.0,
        refractive_index: NBK7,
    };
    let surf_4 = SurfaceSpec::Image;

    let surfaces = vec![surf_0, surf_1, surf_2, surf_3, surf_4];
    let gaps = vec![gap_0, gap_1, gap_2, gap_3];
    let wavelengths = vec![0.567];

    SequentialModel::new(&gaps, &surfaces, &wavelengths).unwrap()
}

pub fn silly_single_surface_and_stop() -> SequentialModel {
    // A silly system for edge case testing only.

    let surf_0 = SurfaceSpec::Object;
    let gap_0 = GapSpec {
        thickness: Float::INFINITY,
        refractive_index: AIR,
    };
    let surf_1 = SurfaceSpec::Conic {
        semi_diameter: 12.5,
        radius_of_curvature: 25.8,
        conic_constant: 0.0,
        surf_type: crate::SurfaceType::Refracting,
    };
    let gap_1 = GapSpec {
        thickness: 10.0,
        refractive_index: NBK7,
    };
    let surf_2 = SurfaceSpec::Stop {
        semi_diameter: 12.5,
    };
    let gap_2 = GapSpec {
        thickness: 10.0,
        refractive_index: AIR,
    };
    let surf_3 = SurfaceSpec::Image;

    let surfaces = vec![surf_0, surf_1, surf_2, surf_3];
    let gaps = vec![gap_0, gap_1, gap_2];
    let wavelengths = vec![0.567];

    SequentialModel::new(&gaps, &surfaces, &wavelengths).unwrap()
}

pub fn wollaston_landscape_lens() -> SequentialModel {
    // Wollaston landscape lens: https://www.youtube.com/watch?v=YN6gTqYVYcw
    // f/5, EFL = 50 mm
    // Aperture stop is a hard stop in front of the lens

    let surf_0 = SurfaceSpec::Object;
    let gap_0 = GapSpec::from_thickness_and_real_refractive_index(Float::INFINITY, 1.0);
    let surf_1 = SurfaceSpec::Stop { semi_diameter: 5.0 };
    let gap_1 = GapSpec::from_thickness_and_real_refractive_index(5.0, 1.0);
    let surf_2 = SurfaceSpec::Conic {
        semi_diameter: 6.882,
        radius_of_curvature: Float::INFINITY,
        conic_constant: 0.0,
        surf_type: crate::SurfaceType::Refracting,
    };
    let gap_2 = GapSpec::from_thickness_and_real_refractive_index(5.0, 1.515);
    let surf_3 = SurfaceSpec::Conic {
        semi_diameter: 7.367,
        radius_of_curvature: -25.84,
        conic_constant: 0.0,
        surf_type: crate::SurfaceType::Refracting,
    };
    let gap_3 = GapSpec::from_thickness_and_real_refractive_index(47.974, 1.0);
    let surf_4 = SurfaceSpec::Image;

    let surfaces = vec![surf_0, surf_1, surf_2, surf_3, surf_4];
    let gaps = vec![gap_0, gap_1, gap_2, gap_3];
    let wavelengths = vec![0.5876];

    SequentialModel::new(&gaps, &surfaces, &wavelengths).unwrap()
}

pub fn petzval_lens() -> SequentialModel {
    let surfaces = vec![
        SurfaceSpec::Object,
        SurfaceSpec::Conic {
            semi_diameter: 28.478,
            radius_of_curvature: 99.56266,
            conic_constant: 0.0,
            surf_type: crate::SurfaceType::Refracting,
        },
        SurfaceSpec::Conic {
            semi_diameter: 26.276,
            radius_of_curvature: -86.84002,
            conic_constant: 0.0,
            surf_type: crate::SurfaceType::Refracting,
        },
        SurfaceSpec::Conic {
            semi_diameter: 21.01,
            radius_of_curvature: -1187.63858,
            conic_constant: 0.0,
            surf_type: crate::SurfaceType::Refracting,
        },
        SurfaceSpec::Stop {
            semi_diameter: 33.262,
        },
        SurfaceSpec::Conic {
            semi_diameter: 20.543,
            radius_of_curvature: 57.47491,
            conic_constant: 0.0,
            surf_type: crate::SurfaceType::Refracting,
        },
        SurfaceSpec::Conic {
            semi_diameter: 20.074,
            radius_of_curvature: -54.61685,
            conic_constant: 0.0,
            surf_type: crate::SurfaceType::Refracting,
        },
        SurfaceSpec::Conic {
            semi_diameter: 16.492,
            radius_of_curvature: -614.68633,
            conic_constant: 0.0,
            surf_type: crate::SurfaceType::Refracting,
        },
        SurfaceSpec::Conic {
            semi_diameter: 17.297,
            radius_of_curvature: -38.17110,
            conic_constant: 0.0,
            surf_type: crate::SurfaceType::Refracting,
        },
        SurfaceSpec::Conic {
            semi_diameter: 18.94,
            radius_of_curvature: Float::INFINITY,
            conic_constant: 0.0,
            surf_type: crate::SurfaceType::Refracting,
        },
        SurfaceSpec::Image,
    ];
    let gaps = vec![
        GapSpec::from_thickness_and_real_refractive_index(Float::INFINITY, 1.0),
        GapSpec::from_thickness_and_real_refractive_index(13.0, 1.5168),
        GapSpec::from_thickness_and_real_refractive_index(4.0, 1.6645),
        GapSpec::from_thickness_and_real_refractive_index(40.0, 1.0),
        GapSpec::from_thickness_and_real_refractive_index(40.0, 1.0),
        GapSpec::from_thickness_and_real_refractive_index(12.0, 1.6074),
        GapSpec::from_thickness_and_real_refractive_index(3.0, 1.6727),
        GapSpec::from_thickness_and_real_refractive_index(46.82210, 1.0),
        GapSpec::from_thickness_and_real_refractive_index(2.0, 1.6727),
        GapSpec::from_thickness_and_real_refractive_index(1.87179, 1.0),
    ];
    let wavelengths = vec![0.5876];

    SequentialModel::new(&gaps, &surfaces, &wavelengths).unwrap()
}
