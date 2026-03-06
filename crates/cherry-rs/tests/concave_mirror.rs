use approx::assert_abs_diff_eq;
use cherry_rs::examples::concave_mirror::sequential_model;
use cherry_rs::{FieldSpec, ImagePlane, ParaxialView, Pupil, PupilSampling, n};
use ndarray::{Array3, arr3};

// Inputs
const WAVELENGTHS: [f64; 1] = [0.5876]; // He d line
const FIELD_SPECS: [FieldSpec; 2] = [
    FieldSpec::Angle {
        angle: 0.0,
        pupil_sampling: PupilSampling::TangentialRayFan,
    },
    FieldSpec::Angle {
        angle: 5.0,
        pupil_sampling: PupilSampling::TangentialRayFan,
    },
];

// Paraxial property values
const APERTURE_STOP: usize = 1;
const BACK_FOCAL_DISTANCE: f64 = 100.0;
const BACK_PRINCIPAL_PLANE: f64 = 0.0;
const EFFECTIVE_FOCAL_LENGTH: f64 = 100.0;
const ENTRANCE_PUPIL: Pupil = Pupil {
    location: 0.0,
    semi_diameter: 12.5,
};
const EXIT_PUPIL: Pupil = Pupil {
    location: 0.0,
    semi_diameter: 12.5,
};
const FRONT_FOCAL_DISTANCE: f64 = 100.0;
const FRONT_PRINCIPAL_PLANE: f64 = 0.0;

// For a 5 degree field angle
const PARAXIAL_IMAGE_PLANE: ImagePlane = ImagePlane {
    location: -100.0,
    semi_diameter: 8.7489,
};

// For a 5 degree field angle
// Paraxial angle = tan(field angle)
fn chief_ray() -> Array3<f64> {
    arr3(&[
        [[0.0], [0.087489]],
        [[0.0], [-0.087489]],
        [[8.7489], [-0.087489]],
    ])
}

fn marginal_ray() -> Array3<f64> {
    arr3(&[
        [[12.5000], [0.0]],
        [[12.5000], [0.125]],
        [[0.0000], [0.125]],
    ])
}

#[test]
fn concave_mirror_paraxial_chief_ray() {
    let model = sequential_model(n!(1.0), &WAVELENGTHS);
    let sub_models = model.submodels();
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");
    let chief_ray = chief_ray();

    for sub_model_id in sub_models.keys() {
        let sub_view = view.subviews().get(sub_model_id).unwrap();
        let result = sub_view.chief_ray();

        assert_abs_diff_eq!(chief_ray, result, epsilon = 1e-4);
    }
}

#[test]
fn concave_mirror_paraxial_aperture_stop() {
    let model = sequential_model(n!(1.0), &WAVELENGTHS);
    let sub_models = model.submodels();
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for sub_model_id in sub_models.keys() {
        let sub_view = view.subviews().get(sub_model_id).unwrap();
        let result = sub_view.aperture_stop();

        assert_eq!(APERTURE_STOP, *result)
    }
}

#[test]
fn concave_mirror_paraxial_back_focal_distance() {
    let model = sequential_model(n!(1.0), &WAVELENGTHS);
    let sub_models = model.submodels();
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for (sub_model_id, _) in sub_models {
        let sub_view = view.subviews().get(sub_model_id).unwrap();
        let result = sub_view.back_focal_distance();

        assert_abs_diff_eq!(BACK_FOCAL_DISTANCE, *result, epsilon = 1e-4)
    }
}

#[test]
fn concave_mirror_paraxial_back_principal_plane() {
    let model = sequential_model(n!(1.0), &WAVELENGTHS);
    let sub_models = model.submodels();
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for (sub_model_id, _) in sub_models {
        let sub_view = view.subviews().get(sub_model_id).unwrap();
        let result = sub_view.back_principal_plane();

        assert_abs_diff_eq!(BACK_PRINCIPAL_PLANE, *result, epsilon = 1e-4)
    }
}

#[test]
fn concave_mirror_paraxial_entrance_pupil() {
    let model = sequential_model(n!(1.0), &WAVELENGTHS);
    let sub_models = model.submodels();
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for (sub_model_id, _) in sub_models {
        let sub_view = view.subviews().get(sub_model_id).unwrap();
        let result = sub_view.entrance_pupil();

        assert_eq!(ENTRANCE_PUPIL, *result)
    }
}

#[test]
fn concave_mirror_paraxial_exit_pupil() {
    let model = sequential_model(n!(1.0), &WAVELENGTHS);
    let sub_models = model.submodels();
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for (sub_model_id, _) in sub_models {
        let sub_view = view.subviews().get(sub_model_id).unwrap();
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
fn concave_mirror_paraxial_effective_focal_length() {
    let model = sequential_model(n!(1.0), &WAVELENGTHS);
    let sub_models = model.submodels();
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for (sub_model_id, _) in sub_models {
        let sub_view = view.subviews().get(sub_model_id).unwrap();
        let result = sub_view.effective_focal_length();

        assert_abs_diff_eq!(EFFECTIVE_FOCAL_LENGTH, *result, epsilon = 1e-4)
    }
}

#[test]
fn concave_mirror_paraxial_front_focal_distance() {
    let model = sequential_model(n!(1.0), &WAVELENGTHS);
    let sub_models = model.submodels();
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for (sub_model_id, _) in sub_models {
        let sub_view = view.subviews().get(sub_model_id).unwrap();
        let result = sub_view.front_focal_distance();

        assert_abs_diff_eq!(FRONT_FOCAL_DISTANCE, *result, epsilon = 1e-4)
    }
}

#[test]
fn concave_mirror_paraxial_front_principal_plane() {
    let model = sequential_model(n!(1.0), &WAVELENGTHS);
    let sub_models = model.submodels();
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for (sub_model_id, _) in sub_models {
        let sub_view = view.subviews().get(sub_model_id).unwrap();
        let result = sub_view.front_principal_plane();

        assert_abs_diff_eq!(FRONT_PRINCIPAL_PLANE, *result, epsilon = 1e-4)
    }
}

#[test]
fn concave_mirror_paraxial_image_plane() {
    let model = sequential_model(n!(1.0), &WAVELENGTHS);
    let sub_models = model.submodels();
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for (sub_model_id, _) in sub_models {
        let sub_view = view.subviews().get(sub_model_id).unwrap();
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
fn concave_mirror_paraxial_marginal_ray() {
    let model = sequential_model(n!(1.0), &WAVELENGTHS);
    let sub_models = model.submodels();
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");
    let marginal_ray = marginal_ray();

    for sub_model_id in sub_models.keys() {
        let sub_view = view.subviews().get(sub_model_id).unwrap();
        let result = sub_view.marginal_ray();

        assert_abs_diff_eq!(marginal_ray, result, epsilon = 1e-4);
    }
}
