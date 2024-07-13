
use cherry_rs_2::examples::convexplano_lens::sequential_model;
use cherry_rs_2::ParaxialView;

fn paraxial_view() -> ParaxialView {
    let model = sequential_model();
    ParaxialView::new(&model)
}

#[test]
fn test_describe_paraxial_view() {
    let view = paraxial_view();
    let _ = view.describe();
}
