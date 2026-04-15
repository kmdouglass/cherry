use cherry_rs::ParaxialView;
use cherry_rs::examples::petzval_lens::*;

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
    let view = paraxial_view();

    for sub_view in view.iter() {
        let result = sub_view.aperture_stop();

        assert_eq!(APERTURE_STOP, *result)
    }
}
