use approx::assert_abs_diff_eq;

use cherry_rs::examples::convexplano_lens::*;
use cherry_rs::ParaxialView;

fn paraxial_view() -> ParaxialView {
    let model = sequential_model();
    ParaxialView::new(&model, false).expect("Could not create paraxial view")
}

#[test]
fn test_describe_paraxial_view() {
    let view = paraxial_view();
    let _ = view.describe();
}

#[test]
fn test_paraxial_view_aperture_stop() {
    let model = sequential_model();
    let sub_models = model.submodels();
    let view = paraxial_view();

    for sub_model_id in sub_models.keys() {
        let sub_view = view.subviews.get(sub_model_id).unwrap();
        let result = sub_view.aperture_stop();

        assert_eq!(APERTURE_STOP, *result)
    }
}

#[test]
fn test_paraxial_view_back_focal_distance() {
    let model = sequential_model();
    let sub_models = model.submodels();
    let view = paraxial_view();

    for (sub_model_id, _) in sub_models {
        let sub_view = view.subviews.get(sub_model_id).unwrap();
        let result = sub_view.back_focal_distance();

        assert_abs_diff_eq!(BACK_FOCAL_DISTANCE, *result, epsilon = 1e-4)
    }
}

#[test]
fn test_paraxial_view_back_principal_plane() {
    let model = sequential_model();
    let sub_models = model.submodels();
    let view = paraxial_view();

    for (sub_model_id, _) in sub_models {
        let sub_view = view.subviews.get(sub_model_id).unwrap();
        let result = sub_view.back_principal_plane();

        assert_abs_diff_eq!(BACK_PRINCIPAL_PLANE, *result, epsilon = 1e-4)
    }
}

#[test]
fn test_paraxial_view_entrance_pupil() {
    let model = sequential_model();
    let sub_models = model.submodels();
    let view = paraxial_view();

    for (sub_model_id, _) in sub_models {
        let sub_view = view.subviews.get(sub_model_id).unwrap();
        let result = sub_view.entrance_pupil();

        assert_eq!(ENTRANCE_PUPIL, *result)
    }
}

#[test]
fn test_paraxial_view_exit_pupil() {
    let model = sequential_model();
    let sub_models = model.submodels();
    let view = paraxial_view();

    for (sub_model_id, _) in sub_models {
        let sub_view = view.subviews.get(sub_model_id).unwrap();
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
    let model = sequential_model();
    let sub_models = model.submodels();
    let view = paraxial_view();

    for (sub_model_id, _) in sub_models {
        let sub_view = view.subviews.get(sub_model_id).unwrap();
        let result = sub_view.effective_focal_length();

        assert_abs_diff_eq!(EFFECTIVE_FOCAL_LENGTH, *result, epsilon = 1e-4)
    }
}

#[test]
fn test_paraxial_view_front_focal_distance() {
    let model = sequential_model();
    let sub_models = model.submodels();
    let view = paraxial_view();

    for (sub_model_id, _) in sub_models {
        let sub_view = view.subviews.get(sub_model_id).unwrap();
        let result = sub_view.front_focal_distance();

        assert_abs_diff_eq!(FRONT_FOCAL_DISTANCE, *result, epsilon = 1e-4)
    }
}

#[test]
fn test_paraxial_view_front_principal_plane() {
    let model = sequential_model();
    let sub_models = model.submodels();
    let view = paraxial_view();

    for (sub_model_id, _) in sub_models {
        let sub_view = view.subviews.get(sub_model_id).unwrap();
        let result = sub_view.front_principal_plane();

        assert_abs_diff_eq!(FRONT_PRINCIPAL_PLANE, *result, epsilon = 1e-4)
    }
}
