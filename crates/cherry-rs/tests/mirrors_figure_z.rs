use approx::assert_abs_diff_eq;
use cherry_rs::{Axis, FieldSpec, ParaxialView, PupilSampling, examples::mirrors_figure_z, n};

const WAVELENGTHS: [f64; 1] = [0.5876];
const FIELD_SPECS: [FieldSpec; 1] = [FieldSpec::Angle {
    angle: 0.0,
    pupil_sampling: PupilSampling::TangentialRayFan {
        n: 3,
        axis: cherry_rs::Axis::U,
    },
}];

// Aperture stop is the first mirror (surface 1, track = 0).
const APERTURE_STOP: usize = 1;

// Entrance pupil coincides with the aperture stop (Mirror 1, theta = 30°).
// U axis: projected SD = r · cos(30°) (theta tilt foreshortens in U).
// R axis: projected SD = r (theta tilt does not affect R).
const ENTRANCE_PUPIL_LOCATION: f64 = 0.0;
const ENTRANCE_PUPIL_SD_U: f64 = 12.7 * 0.8660254037844387; // 12.7 · cos(30°)
const ENTRANCE_PUPIL_SD_R: f64 = 12.7;

// Exit pupil: image of Mirror 1 (track=0) formed by the flat Mirror 2
// (track=100). A flat mirror images an object 100 mm in front of it to 100 mm
// behind it, so the exit pupil is 100 mm past Mirror 2 (distance from last
// physical surface).
const EXIT_PUPIL_LOCATION: f64 = 100.0;

/// For a straight system track == z for every finite surface.
#[test]
fn track_equals_z_for_straight_system() {
    use cherry_rs::examples::convexplano_lens;
    let model = convexplano_lens::sequential_model(n!(1.0), n!(1.515), &WAVELENGTHS);
    let surfaces = model.surfaces();
    for surf in surfaces {
        if surf.z().is_finite() {
            assert!(
                (surf.track() - surf.z()).abs() < 1e-10,
                "Expected track == z for straight system, got track={}, z={}",
                surf.track(),
                surf.z()
            );
        }
    }
}

#[test]
fn mirrors_figure_z_paraxial_aperture_stop() {
    let model = mirrors_figure_z::sequential_model(n!(1.0), &WAVELENGTHS);
    let view = ParaxialView::new(&model, &FIELD_SPECS, false).expect("paraxial view");
    for sub_view in model.submodels().keys().map(|id| &view.subviews()[id]) {
        assert_eq!(*sub_view.aperture_stop(), APERTURE_STOP);
    }
}

#[test]
fn mirrors_figure_z_paraxial_entrance_pupil() {
    let model = mirrors_figure_z::sequential_model(n!(1.0), &WAVELENGTHS);
    let view = ParaxialView::new(&model, &FIELD_SPECS, false).expect("paraxial view");
    for id in model.submodels().keys() {
        let sub_view = &view.subviews()[id];
        let ep = sub_view.entrance_pupil();
        let expected_sd = if id.1 == Axis::U {
            ENTRANCE_PUPIL_SD_U
        } else {
            ENTRANCE_PUPIL_SD_R
        };
        assert_abs_diff_eq!(ep.location, ENTRANCE_PUPIL_LOCATION, epsilon = 1e-10);
        assert_abs_diff_eq!(ep.semi_diameter, expected_sd, epsilon = 1e-10);
    }
}

#[test]
fn mirrors_figure_z_paraxial_exit_pupil() {
    let model = mirrors_figure_z::sequential_model(n!(1.0), &WAVELENGTHS);
    let view = ParaxialView::new(&model, &FIELD_SPECS, false).expect("paraxial view");
    for sub_view in model.submodels().keys().map(|id| &view.subviews()[id]) {
        assert_abs_diff_eq!(
            sub_view.exit_pupil().location,
            EXIT_PUPIL_LOCATION,
            epsilon = 1e-4
        );
    }
}

/// Marginal ray at the aperture stop equals the projected SD, not the raw SD.
///
/// For Mirror 1 tilted 30° about cursor-R, the effective clear aperture seen by
/// a paraxial beam along the cursor-U axis is `r · cos(30°)`.  The marginal ray
/// must be scaled to exactly fill this projected aperture.
#[test]
fn mirrors_figure_z_marginal_ray_uses_projected_sd() {
    let model = mirrors_figure_z::sequential_model(n!(1.0), &WAVELENGTHS);
    let view = ParaxialView::new(&model, &FIELD_SPECS, false).expect("paraxial view");
    let r = 12.7_f64;
    let projected_u = r * (30.0_f64.to_radians()).cos();
    let projected_r = r;

    for id in model.submodels().keys() {
        let sub_view = &view.subviews()[id];
        let expected = if id.1 == Axis::U {
            projected_u
        } else {
            projected_r
        };
        // The aperture stop is Mirror 1 (surface index 1).
        assert_eq!(*sub_view.aperture_stop(), 1usize);
        // The marginal ray height at the aperture stop should equal the projected SD.
        let marginal_height_at_stop = sub_view.marginal_ray()[[1, 0, 0]];
        assert_abs_diff_eq!(marginal_height_at_stop, expected, epsilon = 1e-10);
    }
}

/// Track accumulates path length regardless of fold direction.
#[test]
fn track_coordinates_folded_system() {
    let model = mirrors_figure_z::sequential_model(n!(1.0), &WAVELENGTHS);
    let surfaces = model.surfaces();

    // surf[0] Object: cursor initialized at -inf
    assert!(surfaces[0].track().is_infinite() && surfaces[0].track() < 0.0);

    // surf[1] Mirror 1: special-case advance from -inf resets track to 0
    assert_eq!(surfaces[1].track(), 0.0);

    // surf[2] Mirror 2: 0 + gap_1 (100 mm)
    assert!(
        (surfaces[2].track() - 100.0).abs() < 1e-10,
        "Mirror 2 track should be 100, got {}",
        surfaces[2].track()
    );

    // surf[3] Image: 100 + gap_2 (50 mm)
    assert!(
        (surfaces[3].track() - 150.0).abs() < 1e-10,
        "Image track should be 150, got {}",
        surfaces[3].track()
    );
}
