use crate::ray_tracing::{ApertureSpec, FieldSpec, Gap, SurfaceSpec, SystemBuilder, SystemModel};

pub fn empty_system() -> SystemModel {
    let surf_0 = SurfaceSpec::ObjectPlane { diam: 25.0 };
    let gap_0 = Gap::new(1.0, 1.0);
    let surf_1 = SurfaceSpec::ImagePlane { diam: 25.0 };

    let surfaces = vec![surf_0, surf_1];
    let gaps = vec![gap_0];
    let aperture = ApertureSpec::EntrancePupilDiameter { diam: 25.0 };
    let fields = vec![FieldSpec::new(0.0)];

    let mut builder = SystemBuilder::new();
    let model = builder
        .surfaces(surfaces)
        .gaps(gaps)
        .aperture(aperture)
        .fields(fields)
        .build()
        .unwrap();

    model
}

pub fn planoconvex_lens_obj_at_inf() -> SystemModel {
    // A f = +50.1 mm planoconvex lens: https://www.thorlabs.com/thorproduct.cfm?partnumber=LA1255
    // Object is at infinity; aperture stop is the first surface.
    // There are two fields: 0 and +5 degrees.
    let surf_0 = SurfaceSpec::ObjectPlane { diam: 25.0 };
    let gap_0 = Gap::new(1.0, f32::NEG_INFINITY);
    let surf_1 = SurfaceSpec::RefractingCircularConic {
        diam: 25.0,
        roc: 25.8,
        k: 0.0,
    };
    let gap_1 = Gap::new(1.515, 5.3);
    let surf_2 = SurfaceSpec::RefractingCircularFlat { diam: 25.0 };
    let gap_2 = Gap::new(1.0, 46.6);
    let surf_3 = SurfaceSpec::ImagePlane { diam: 25.0 };

    let surfaces = vec![surf_0, surf_1, surf_2, surf_3];
    let gaps = vec![gap_0, gap_1, gap_2];
    let aperture = ApertureSpec::EntrancePupilDiameter { diam: 25.0 };
    let fields = vec![FieldSpec::new(0.0), FieldSpec::new(5.0)];

    let mut builder = SystemBuilder::new();
    let model = builder
        .surfaces(surfaces)
        .gaps(gaps)
        .aperture(aperture)
        .fields(fields)
        .build()
        .unwrap();

    model
}

pub fn silly_unpaired_surface() -> SystemModel {
    // A silly system for edge case testing only.

    let surf_0 = SurfaceSpec::ObjectPlane { diam: 25.0 };
    let gap_0 = Gap::new(1.0, f32::NEG_INFINITY);
    let surf_1 = SurfaceSpec::RefractingCircularConic {
        diam: 25.0,
        roc: 25.8,
        k: 0.0,
    };
    let gap_1 = Gap::new(1.515, 5.3);
    let surf_2 = SurfaceSpec::RefractingCircularFlat { diam: 25.0 };
    let gap_2 = Gap::new(1.0, 46.6);
    let surf_3 = SurfaceSpec::RefractingCircularConic {
        diam: 25.0,
        roc: 25.8,
        k: 0.0,
    }; // Surface is unpaired
    let gap_3 = Gap::new(1.5, 20.0);
    let surf_4 = SurfaceSpec::ImagePlane { diam: 25.0 };

    let surfaces = vec![surf_0, surf_1, surf_2, surf_3, surf_4];
    let gaps = vec![gap_0, gap_1, gap_2, gap_3];
    let aperture = ApertureSpec::EntrancePupilDiameter { diam: 25.0 };
    let fields = vec![FieldSpec::new(0.0), FieldSpec::new(5.0)];

    let mut builder = SystemBuilder::new();
    let model = builder
        .surfaces(surfaces)
        .gaps(gaps)
        .aperture(aperture)
        .fields(fields)
        .build()
        .unwrap();

    model
}

pub fn silly_single_surface_and_stop() -> SystemModel {
    // A silly system for edge case testing only.

    let surf_0 = SurfaceSpec::ObjectPlane { diam: 25.0 };
    let gap_0 = Gap::new(1.0, f32::NEG_INFINITY);
    let surf_1 = SurfaceSpec::RefractingCircularConic {
        diam: 25.0,
        roc: 25.8,
        k: 0.0,
    };
    let gap_1 = Gap::new(1.515, 10.0);
    let surf_2 = SurfaceSpec::Stop { diam: 25.0 };
    let gap_2 = Gap::new(1.0, 10.0);
    let surf_3 = SurfaceSpec::ImagePlane { diam: 25.0 };

    let surfaces = vec![surf_0, surf_1, surf_2, surf_3];
    let gaps = vec![gap_0, gap_1, gap_2];
    let aperture = ApertureSpec::EntrancePupilDiameter { diam: 25.0 };
    let fields = vec![FieldSpec::new(0.0)];

    let mut builder = SystemBuilder::new();
    let model = builder
        .surfaces(surfaces)
        .gaps(gaps)
        .aperture(aperture)
        .fields(fields)
        .build()
        .unwrap();

    model
}

pub fn wollaston_landscape_lens() -> SystemModel {
    // Wollaston landscape lens: https://www.youtube.com/watch?v=YN6gTqYVYcw
    // f/5, EFL = 50 mm
    // Aperture stop is a hard stop in front of the lens
    // 10 degree field of view

    let surf_0 = SurfaceSpec::ObjectPlane { diam: 10.0 };
    let gap_0 = Gap::new(1.0, f32::NEG_INFINITY);
    let surf_1 = SurfaceSpec::Stop { diam: 10.0 };
    let gap_1 = Gap::new(1.0, 5.0);
    let surf_2 = SurfaceSpec::RefractingCircularFlat { diam: 13.764 };
    let gap_2 = Gap::new(1.515, 5.0);
    let surf_3 = SurfaceSpec::RefractingCircularConic {
        diam: 14.734,
        roc: -25.84,
        k: 0.0,
    };
    let gap_3 = Gap::new(1.0, 47.974);
    let surf_4 = SurfaceSpec::ImagePlane { diam: 17.714 };

    let surfaces = vec![surf_0, surf_1, surf_2, surf_3, surf_4];
    let gaps = vec![gap_0, gap_1, gap_2, gap_3];
    let aperture = ApertureSpec::EntrancePupilDiameter { diam: 10.0 };
    let fields = vec![FieldSpec::new(0.0), FieldSpec::new(10.0)];

    let mut builder = SystemBuilder::new();
    let model = builder
        .surfaces(surfaces)
        .gaps(gaps)
        .aperture(aperture)
        .fields(fields)
        .build()
        .unwrap();

    model
}

pub fn petzval_lens() -> SystemModel {
    let surfaces = vec![
        SurfaceSpec::ObjectPlane { diam: 50.0 },
        SurfaceSpec::RefractingCircularConic {
            diam: 56.956,
            roc: 99.56266,
            k: 0.0,
        },
        SurfaceSpec::RefractingCircularConic {
            diam: 52.552,
            roc: -86.84002,
            k: 0.0,
        },
        SurfaceSpec::RefractingCircularConic {
            diam: 42.02,
            roc: -1187.63858,
            k: 0.0,
        },
        SurfaceSpec::Stop { diam: 33.262 },
        SurfaceSpec::RefractingCircularConic {
            diam: 41.086,
            roc: 57.47491,
            k: 0.0,
        },
        SurfaceSpec::RefractingCircularConic {
            diam: 40.148,
            roc: -54.61685,
            k: 0.0,
        },
        SurfaceSpec::RefractingCircularConic {
            diam: 32.984,
            roc: -614.68633,
            k: 0.0,
        },
        SurfaceSpec::RefractingCircularConic {
            diam: 34.594,
            roc: -38.17110,
            k: 0.0,
        },
        SurfaceSpec::RefractingCircularFlat { diam: 37.88 },
        SurfaceSpec::ImagePlane { diam: 50.00 },
    ];
    let gaps = vec![
        Gap::new(1.0, f32::INFINITY),
        Gap::new(1.5168, 13.0),
        Gap::new(1.6645, 4.0),
        Gap::new(1.0, 40.0),
        Gap::new(1.0, 40.0),
        Gap::new(1.6074, 12.0),
        Gap::new(1.6727, 3.0),
        Gap::new(1.0, 46.82210),
        Gap::new(1.6727, 2.0),
        Gap::new(1.0, 1.87179),
    ];
    let aperture = ApertureSpec::EntrancePupilDiameter { diam: 20.0 };
    let fields = vec![FieldSpec::new(0.0), FieldSpec::new(5.0)];

    let mut builder = SystemBuilder::new();
    let model = builder
        .surfaces(surfaces)
        .gaps(gaps)
        .aperture(aperture)
        .fields(fields)
        .build()
        .unwrap();

    model
}
