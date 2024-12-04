use cherry_rs::examples::petzval_lens::*;
use cherry_rs::{CutawayView, ParaxialView};

fn paraxial_view() -> ParaxialView {
    let model = sequential_model();
    ParaxialView::new(&model, &field_specs(), false).expect("Could not create paraxial view")
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

/// Regression test for https://github.com/kmdouglass/cherry/issues/144
#[test]
fn test_stop_type() {
    let model = sequential_model();
    let cutaway_view = CutawayView::new(&model, 20);
    let surface_types = cutaway_view.surface_types;

    assert_eq!("Stop", surface_types[&APERTURE_STOP]);
}
