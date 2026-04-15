use approx::assert_abs_diff_eq;

use cherry_rs::examples::biconvex_lens_finite_object::sequential_model;
use cherry_rs::{FieldSpec, ImagePlane, ParaxialRayBundle, ParaxialView, Pupil, n};

// Inputs
const WAVELENGTHS: [f64; 1] = [0.5876]; // He d line
const FIELD_SPECS: [FieldSpec; 2] = [
    FieldSpec::PointSource { x: 0.0, y: 0.0 },
    FieldSpec::PointSource { x: 0.0, y: 5.0 },
];

// Paraxial property values
const APERTURE_STOP: usize = 1;
const BACK_FOCAL_DISTANCE: f64 = 98.4360;
const BACK_PRINCIPAL_PLANE: f64 = -1.1937;
const EFFECTIVE_FOCAL_LENGTH: f64 = 99.6297;
const ENTRANCE_PUPIL: Pupil = Pupil {
    location: 0.0,
    semi_diameter: 12.7,
};
const EXIT_PUPIL: Pupil = Pupil {
    location: -2.4019,
    semi_diameter: 12.8540,
};
const FRONT_FOCAL_DISTANCE: f64 = 98.4360;
const FRONT_PRINCIPAL_PLANE: f64 = 1.1937;

const PARAXIAL_IMAGE_PLANE: ImagePlane = ImagePlane {
    location: 199.7684,
    semi_diameter: 4.9048,
};

// For a 5 mm field point; expected (height, angle) per surface for first ray
// Paraxial angle = tan(field angle)
fn chief_ray_expected() -> Vec<(f64, f64)> {
    vec![
        (5.0, -0.025),
        (0.0, -0.01648),
        (-0.0593, -0.02470),
        (-4.9048, -0.02470),
    ]
}

fn marginal_ray_expected() -> Vec<(f64, f64)> {
    vec![
        (0.0, 0.0635),
        (12.7000, -0.0004088),
        (12.6985, -0.06473),
        (0.0, -0.06473),
    ]
}

fn assert_ray_results_approx_eq(actual: &ParaxialRayBundle, expected: &[(f64, f64)], epsilon: f64) {
    assert_eq!(
        actual.num_surfaces(),
        expected.len(),
        "Surface count mismatch"
    );
    for (surface_rays, (exp_h, exp_a)) in actual.iter_surfaces().zip(expected.iter()) {
        assert_abs_diff_eq!(surface_rays[0].height, *exp_h, epsilon = epsilon);
        assert_abs_diff_eq!(surface_rays[0].angle, *exp_a, epsilon = epsilon);
    }
}

#[test]
fn test_paraxial_view_aperture_stop() {
    let model = sequential_model(n!(1.0), n!(1.517), &WAVELENGTHS);
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for sub_view in view.iter() {
        let result = sub_view.aperture_stop();

        assert_eq!(APERTURE_STOP, *result)
    }
}

#[test]
fn test_paraxial_view_back_focal_distance() {
    let model = sequential_model(n!(1.0), n!(1.517), &WAVELENGTHS);
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for sub_view in view.iter() {
        let result = sub_view.back_focal_distance();

        assert_abs_diff_eq!(BACK_FOCAL_DISTANCE, *result, epsilon = 1e-4)
    }
}

#[test]
fn test_paraxial_view_back_principal_plane() {
    let model = sequential_model(n!(1.0), n!(1.517), &WAVELENGTHS);
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for sub_view in view.iter() {
        let result = sub_view.back_principal_plane();

        assert_abs_diff_eq!(BACK_PRINCIPAL_PLANE, *result, epsilon = 1e-4)
    }
}

#[test]
fn test_paraxial_view_entrance_pupil() {
    let model = sequential_model(n!(1.0), n!(1.517), &WAVELENGTHS);
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for sub_view in view.iter() {
        let result = sub_view.entrance_pupil();

        assert_eq!(ENTRANCE_PUPIL, *result)
    }
}

#[test]
fn test_paraxial_view_exit_pupil() {
    let model = sequential_model(n!(1.0), n!(1.517), &WAVELENGTHS);
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for sub_view in view.iter() {
        let result = sub_view.exit_pupil();

        assert_abs_diff_eq!(EXIT_PUPIL.location, result.location, epsilon = 1e-4);
        assert_abs_diff_eq!(
            EXIT_PUPIL.semi_diameter,
            result.semi_diameter,
            epsilon = 1e-4
        );
    }
}

#[test]
fn test_paraxial_view_effective_focal_length() {
    let model = sequential_model(n!(1.0), n!(1.517), &WAVELENGTHS);
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for sub_view in view.iter() {
        let result = sub_view.effective_focal_length();

        assert_abs_diff_eq!(EFFECTIVE_FOCAL_LENGTH, *result, epsilon = 1e-4)
    }
}

#[test]
fn test_paraxial_view_front_focal_distance() {
    let model = sequential_model(n!(1.0), n!(1.517), &WAVELENGTHS);
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for sub_view in view.iter() {
        let result = sub_view.front_focal_distance();

        assert_abs_diff_eq!(FRONT_FOCAL_DISTANCE, *result, epsilon = 1e-4)
    }
}

#[test]
fn test_paraxial_view_front_principal_plane() {
    let model = sequential_model(n!(1.0), n!(1.517), &WAVELENGTHS);
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for sub_view in view.iter() {
        let result = sub_view.front_principal_plane();

        assert_abs_diff_eq!(FRONT_PRINCIPAL_PLANE, *result, epsilon = 1e-4)
    }
}

#[test]
fn test_paraxial_view_image_plane() {
    let model = sequential_model(n!(1.0), n!(1.517), &WAVELENGTHS);
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for sub_view in view.iter() {
        let result = sub_view.paraxial_image_plane();

        assert_abs_diff_eq!(
            PARAXIAL_IMAGE_PLANE.location,
            result.location,
            epsilon = 1e-4
        );
        assert_abs_diff_eq!(
            PARAXIAL_IMAGE_PLANE.semi_diameter,
            result.semi_diameter,
            epsilon = 1e-4
        );
    }
}

#[test]
fn test_paraxial_view_marginal_ray() {
    let model = sequential_model(n!(1.0), n!(1.517), &WAVELENGTHS);
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for sub_view in view.iter() {
        assert_ray_results_approx_eq(sub_view.marginal_ray(), &marginal_ray_expected(), 1e-4);
    }
}

#[test]
fn test_paraxial_view_chief_ray() {
    let model = sequential_model(n!(1.0), n!(1.517), &WAVELENGTHS);
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for sub_view in view.iter() {
        assert_ray_results_approx_eq(sub_view.chief_ray(), &chief_ray_expected(), 1e-4);
    }
}
