use approx::assert_abs_diff_eq;

use cherry_rs::examples::wf_epi_excitation::sequential_model;
use cherry_rs::{FieldSpec, ParaxialView, n};

const WAVELENGTHS: [f64; 1] = [0.5876];
const FIELD_SPECS: [FieldSpec; 1] = [FieldSpec::PointSource { x: 0.0, y: 1.5 }];

// Paraxial property values
const APERTURE_STOP: usize = 3;
const BACK_FOCAL_DISTANCE: f64 = 5.1562;
const BACK_PRINCIPAL_PLANE: f64 = 7.0312;
const EFFECTIVE_FOCAL_LENGTH: f64 = -1.8750;
const ENTRANCE_PUPIL_LOCATION: f64 = -54.5455;
const ENTRANCE_PUPIL_SIZE: f64 = 1.5454;
const FRONT_FOCAL_DISTANCE: f64 = -55.0;
const FRONT_FOCAL_LENGTH: f64 = 1.25;
const FRONT_PRINCIPAL_PLANE: f64 = -56.25;
const IMAGE_LOCATION: f64 = 5.0000;
const IMAGE_SIZE: f64 = 0.125;
const IMAGE_SPACE_FNO: f64 = -0.6066;
const PARAXIAL_FNO: f64 = -0.3922;

#[test]
fn wf_epi_excitation_paraxial_aperture_stop() {
    let model = sequential_model(n!(1.0), n!(1.5), &WAVELENGTHS);
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for sub_view in view.iter() {
        let result = sub_view.aperture_stop();

        assert_eq!(APERTURE_STOP, *result)
    }
}

#[test]
fn wf_epi_excitation_paraxial_back_principal_plane() {
    let model = sequential_model(n!(1.0), n!(1.5), &WAVELENGTHS);
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for sub_view in view.iter() {
        let result = sub_view.back_principal_plane();

        assert_abs_diff_eq!(BACK_PRINCIPAL_PLANE, *result, epsilon = 1e-4)
    }
}

#[test]
fn wf_epi_excitation_paraxial_back_focal_distance() {
    let model = sequential_model(n!(1.0), n!(1.5), &WAVELENGTHS);
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for sub_view in view.iter() {
        let result = sub_view.back_focal_distance();

        assert_abs_diff_eq!(BACK_FOCAL_DISTANCE, *result, epsilon = 1e-4)
    }
}

#[test]
fn wf_epi_excitation_paraxial_effective_focal_length() {
    let model = sequential_model(n!(1.0), n!(1.5), &WAVELENGTHS);
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for sub_view in view.iter() {
        let result = sub_view.effective_focal_length();

        assert_abs_diff_eq!(EFFECTIVE_FOCAL_LENGTH, *result, epsilon = 1e-4)
    }
}

#[test]
fn wf_epi_excitation_entrance_pupil_location() {
    let model = sequential_model(n!(1.0), n!(1.5), &WAVELENGTHS);
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for sub_view in view.iter() {
        let entrance_pupil = sub_view.entrance_pupil();
        assert_abs_diff_eq!(
            entrance_pupil.location,
            ENTRANCE_PUPIL_LOCATION,
            epsilon = 1e-3
        );
        assert_abs_diff_eq!(
            entrance_pupil.semi_diameter,
            ENTRANCE_PUPIL_SIZE,
            epsilon = 1e-3
        );
    }
}

#[test]
fn wf_epi_excitation_paraxial_front_focal_distance() {
    let model = sequential_model(n!(1.0), n!(1.5), &WAVELENGTHS);
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for sub_view in view.iter() {
        let result = sub_view.front_focal_distance();

        assert_abs_diff_eq!(FRONT_FOCAL_DISTANCE, *result, epsilon = 1e-4)
    }
}

#[test]
fn wf_epi_excitation_paraxial_front_focal_length() {
    let model = sequential_model(n!(1.0), n!(1.5), &WAVELENGTHS);
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for sub_view in view.iter() {
        let result = sub_view.front_focal_length();

        assert_abs_diff_eq!(FRONT_FOCAL_LENGTH, *result, epsilon = 1e-4)
    }
}

#[test]
fn wf_epi_excitation_paraxial_front_principal_plane() {
    let model = sequential_model(n!(1.0), n!(1.5), &WAVELENGTHS);
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for sub_view in view.iter() {
        let result = sub_view.front_principal_plane();

        assert_abs_diff_eq!(FRONT_PRINCIPAL_PLANE, *result, epsilon = 1e-4)
    }
}

#[test]
fn wf_epi_excitation_paraxial_image_space_fno() {
    let model = sequential_model(n!(1.0), n!(1.5), &WAVELENGTHS);
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for sub_view in view.iter() {
        let result = sub_view.image_space_fno();

        assert_abs_diff_eq!(IMAGE_SPACE_FNO, result, epsilon = 1e-4)
    }
}

#[test]
fn wf_epi_excitation_paraxial_image_location() {
    let model = sequential_model(n!(1.0), n!(1.5), &WAVELENGTHS);
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for sub_view in view.iter() {
        let result = sub_view.paraxial_image_plane().location;
        assert_abs_diff_eq!(IMAGE_LOCATION, result, epsilon = 1e-4);
    }
}

#[test]
fn wf_epi_excitation_paraxial_image_size() {
    let model = sequential_model(n!(1.0), n!(1.5), &WAVELENGTHS);
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for sub_view in view.iter() {
        let result = sub_view.paraxial_image_plane().semi_diameter;
        assert_abs_diff_eq!(IMAGE_SIZE, result, epsilon = 1e-4);
    }
}

#[test]
fn wf_epi_excitation_paraxial_fno() {
    let model = sequential_model(n!(1.0), n!(1.5), &WAVELENGTHS);
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for sub_view in view.iter() {
        let result = sub_view.paraxial_fno();

        assert_abs_diff_eq!(PARAXIAL_FNO, result, epsilon = 1e-4)
    }
}
