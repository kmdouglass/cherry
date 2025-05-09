use approx::assert_abs_diff_eq;
use ndarray::{Array3, arr3};

use cherry_rs::examples::biconvex_lens_finite_object::sequential_model;
use cherry_rs::{FieldSpec, ImagePlane, ParaxialView, Pupil, PupilSampling, n};

// Inputs
const WAVELENGTHS: [f64; 1] = [0.5876]; // He d line
const FIELD_SPECS: [FieldSpec; 2] = [
    FieldSpec::PointSource {
        x: 0.0,
        y: 0.0,
        pupil_sampling: PupilSampling::TangentialRayFan,
    },
    FieldSpec::PointSource {
        x: 0.0,
        y: 5.0,
        pupil_sampling: PupilSampling::TangentialRayFan,
    },
];

// Paraxial property values
const APERTURE_STOP: usize = 1;
const BACK_FOCAL_DISTANCE: f64 = 98.4360;
const BACK_PRINCIPAL_PLANE: f64 = 2.4063;
const EFFECTIVE_FOCAL_LENGTH: f64 = 99.6297;
const ENTRANCE_PUPIL: Pupil = Pupil {
    location: 0.0,
    semi_diameter: 12.7,
};
const EXIT_PUPIL: Pupil = Pupil {
    location: 1.1981,
    semi_diameter: 12.8540,
};
const FRONT_FOCAL_DISTANCE: f64 = -98.4360;
const FRONT_PRINCIPAL_PLANE: f64 = 1.1937;

const PARAXIAL_IMAGE_PLANE: ImagePlane = ImagePlane {
    location: 199.7684,
    semi_diameter: 4.9048,
};

// For a 5 mm field point
// Paraxial angle = tan(field angle)
fn chief_ray() -> Array3<f64> {
    arr3(&[
        [[5.0], [-0.025]],
        [[0.0], [-0.01648]],
        [[-0.0593], [-0.02470]],
        [[-4.9048], [-0.02470]],
    ])
}

fn marginal_ray() -> Array3<f64> {
    arr3(&[
        [[0.0], [0.0635]],
        [[12.7000], [-0.0004088]],
        [[12.6985], [-0.06473]],
        [[0.0], [-0.06473]],
    ])
}

#[test]
fn test_paraxial_view_aperture_stop() {
    let model = sequential_model(n!(1.0), n!(1.517), &WAVELENGTHS);
    let submodels = model.submodels();
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for submodel_id in submodels.keys() {
        let sub_view = view.subviews().get(submodel_id).unwrap();
        let result = sub_view.aperture_stop();

        assert_eq!(APERTURE_STOP, *result)
    }
}

#[test]
fn test_paraxial_view_back_focal_distance() {
    let model = sequential_model(n!(1.0), n!(1.517), &WAVELENGTHS);
    let submodels = model.submodels();
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for (submodel_id, _) in submodels {
        let sub_view = view.subviews().get(submodel_id).unwrap();
        let result = sub_view.back_focal_distance();

        assert_abs_diff_eq!(BACK_FOCAL_DISTANCE, *result, epsilon = 1e-4)
    }
}

#[test]
fn test_paraxial_view_back_principal_plane() {
    let model = sequential_model(n!(1.0), n!(1.517), &WAVELENGTHS);
    let submodels = model.submodels();
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for (submodel_id, _) in submodels {
        let sub_view = view.subviews().get(submodel_id).unwrap();
        let result = sub_view.back_principal_plane();

        assert_abs_diff_eq!(BACK_PRINCIPAL_PLANE, *result, epsilon = 1e-4)
    }
}

#[test]
fn test_paraxial_view_entrance_pupil() {
    let model = sequential_model(n!(1.0), n!(1.517), &WAVELENGTHS);
    let submodels = model.submodels();
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for (submodel_id, _) in submodels {
        let sub_view = view.subviews().get(submodel_id).unwrap();
        let result = sub_view.entrance_pupil();

        assert_eq!(ENTRANCE_PUPIL, *result)
    }
}

#[test]
fn test_paraxial_view_exit_pupil() {
    let model = sequential_model(n!(1.0), n!(1.517), &WAVELENGTHS);
    let submodels = model.submodels();
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for (submodel_id, _) in submodels {
        let sub_view = view.subviews().get(submodel_id).unwrap();
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
    let submodels = model.submodels();
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for (submodel_id, _) in submodels {
        let sub_view = view.subviews().get(submodel_id).unwrap();
        let result = sub_view.effective_focal_length();

        assert_abs_diff_eq!(EFFECTIVE_FOCAL_LENGTH, *result, epsilon = 1e-4)
    }
}

#[test]
fn test_paraxial_view_front_focal_distance() {
    let model = sequential_model(n!(1.0), n!(1.517), &WAVELENGTHS);
    let submodels = model.submodels();
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for (submodel_id, _) in submodels {
        let sub_view = view.subviews().get(submodel_id).unwrap();
        let result = sub_view.front_focal_distance();

        assert_abs_diff_eq!(FRONT_FOCAL_DISTANCE, *result, epsilon = 1e-4)
    }
}

#[test]
fn test_paraxial_view_front_principal_plane() {
    let model = sequential_model(n!(1.0), n!(1.517), &WAVELENGTHS);
    let sub_models = model.submodels();
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for (submodel_id, _) in sub_models {
        let sub_view = view.subviews().get(submodel_id).unwrap();
        let result = sub_view.front_principal_plane();

        assert_abs_diff_eq!(FRONT_PRINCIPAL_PLANE, *result, epsilon = 1e-4)
    }
}

#[test]
fn test_paraxial_view_image_plane() {
    let model = sequential_model(n!(1.0), n!(1.517), &WAVELENGTHS);
    let sub_models = model.submodels();
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for (submodel_id, _) in sub_models {
        let sub_view = view.subviews().get(submodel_id).unwrap();
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
    let sub_models = model.submodels();
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");
    let marginal_ray = marginal_ray();

    for submodel_id in sub_models.keys() {
        let sub_view = view.subviews().get(submodel_id).unwrap();
        let result = sub_view.marginal_ray();

        assert_abs_diff_eq!(marginal_ray, result, epsilon = 1e-4);
    }
}

#[test]
fn test_paraxial_view_chief_ray() {
    let model = sequential_model(n!(1.0), n!(1.517), &WAVELENGTHS);
    let sub_models = model.submodels();
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");
    let chief_ray = chief_ray();

    for submodel_id in sub_models.keys() {
        let sub_view = view.subviews().get(submodel_id).unwrap();
        let result = sub_view.chief_ray();

        assert_abs_diff_eq!(chief_ray, result, epsilon = 1e-4);
    }
}
