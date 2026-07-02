use approx::assert_abs_diff_eq;

use cherry_rs::examples::wf_epi_microscope::sequential_model;
use cherry_rs::{FieldSpec, ParaxialView, SequentialModel, n};

const WAVELENGTHS: [f64; 1] = [0.5876];
const FIELD_SPECS: [FieldSpec; 1] = [FieldSpec::PointSource { x: 0.0, y: 1.5 }];

const EXC_EFFECTIVE_FOCAL_LENGTH: f64 = -1.8750;
const EXC_IMAGE_LOCATION: f64 = 5.0000;
const EXC_ENTRANCE_PUPIL_LOCATION: f64 = -54.5455;
const EXC_ENTRANCE_PUPIL_SIZE: f64 = 1.5454;
const EXC_LAGRANGE_INVARIANT: f64 = -0.1594;

const EMI_IMAGE_LOCATION: f64 = 199.8800167976479;

fn model() -> SequentialModel {
    sequential_model(n!(1.0), n!(1.5), &WAVELENGTHS)
}

#[test]
fn at_model_has_two_paths() {
    let model = model();
    assert_eq!(model.path_count(), 2);
}

#[test]
fn at_emission_object_position_equals_excitation_image_position() {
    let model = model();
    let exc_image = model.placements()[4].position;
    let emi_object = model.placements()[5].position;
    assert_abs_diff_eq!(exc_image.x(), emi_object.x(), epsilon = 1e-9);
    assert_abs_diff_eq!(exc_image.y(), emi_object.y(), epsilon = 1e-9);
    assert_abs_diff_eq!(exc_image.z(), emi_object.z(), epsilon = 1e-9);
}

#[test]
fn at_objective_in_both_paths() {
    let model = model();
    assert!(model.path_surface_indices(0).contains(&3));
    assert!(model.path_surface_indices(1).contains(&3));
}

#[test]
fn at_beamsplitter_in_both_paths() {
    let model = model();
    assert!(model.path_surface_indices(0).contains(&2));
    assert!(model.path_surface_indices(1).contains(&2));
}

#[test]
fn at_stop_surface_excitation() {
    let model = model();
    assert_eq!(model.stop_surface_for_path(0), Some(3));
}

#[test]
fn at_stop_surface_emission() {
    let model = model();
    assert_eq!(model.stop_surface_for_path(1), Some(3));
}

#[test]
fn at_excitation_paraxial_effective_focal_length() {
    let model = model();
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");
    let sub_view = view.get_for_path(0, 0, 0).expect("path 0 subview");
    assert_abs_diff_eq!(
        EXC_EFFECTIVE_FOCAL_LENGTH,
        sub_view.effective_focal_length(),
        epsilon = 1e-4
    );
}

#[test]
fn at_excitation_paraxial_image_location() {
    let model = model();
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");
    let sub_view = view.get_for_path(0, 0, 0).expect("path 0 subview");
    assert_abs_diff_eq!(
        EXC_IMAGE_LOCATION,
        sub_view.paraxial_image_plane().location,
        epsilon = 1e-4
    );
}

#[test]
fn at_excitation_paraxial_entrance_pupil_location() {
    let model = model();
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");
    let sub_view = view.get_for_path(0, 0, 0).expect("path 0 subview");
    assert_abs_diff_eq!(
        EXC_ENTRANCE_PUPIL_LOCATION,
        sub_view.entrance_pupil().location,
        epsilon = 1e-3
    );
}

#[test]
fn at_excitation_paraxial_entrance_pupil_size() {
    let model = model();
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");
    let sub_view = view.get_for_path(0, 0, 0).expect("path 0 subview");
    assert_abs_diff_eq!(
        EXC_ENTRANCE_PUPIL_SIZE,
        sub_view.entrance_pupil().semi_diameter,
        epsilon = 1e-3
    );
}

#[test]
fn at_excitation_paraxial_lagrange_invariant() {
    let model = model();
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");
    let sub_view = view.get_for_path(0, 0, 0).expect("path 0 subview");
    for &h in sub_view.lagrange_invariants() {
        assert_abs_diff_eq!(EXC_LAGRANGE_INVARIANT, h, epsilon = 1e-4);
    }
}

#[test]
fn at_emission_paraxial_image_location() {
    let model = model();
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");
    let sub_view = view.get_for_path(1, 0, 0).expect("path 1 subview");
    assert_abs_diff_eq!(
        EMI_IMAGE_LOCATION,
        sub_view.paraxial_image_plane().location,
        epsilon = 1e-4
    );
}
