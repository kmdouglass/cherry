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
