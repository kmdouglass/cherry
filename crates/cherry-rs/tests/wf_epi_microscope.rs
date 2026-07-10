use approx::assert_abs_diff_eq;

use cherry_rs::examples::wf_epi_microscope::sequential_model;
use cherry_rs::{ApertureSpec, FieldSpec, ParaxialView, SamplingConfig, SequentialModel, n};

const EXCITATION_WAVELENGTHS: [f64; 1] = [0.488];
const EMISSION_WAVELENGTHS: [f64; 1] = [0.520];
const EXCITATION_FIELD_SPECS: [FieldSpec; 1] = [FieldSpec::PointSource { x: 0.0, y: 1.5 }];

/// Derives the emission path's own `FieldSpec` from the excitation arm's
/// computed transverse image height for its `y: 1.5` field point — the
/// emission path images the fluorescence spot the excitation path forms at
/// the specimen, not an independently chosen field point (FR-13).
fn emission_field_specs(model: &SequentialModel) -> Vec<FieldSpec> {
    let exc_view = ParaxialView::new(model, &[EXCITATION_FIELD_SPECS.to_vec(), vec![]], false)
        .expect("excitation paraxial view");
    let sub_view = exc_view.get_for_path(0, 0, 0).expect("path 0 subview");
    let image_height = sub_view.paraxial_image_plane().semi_diameter;
    vec![FieldSpec::PointSource {
        x: 0.0,
        y: image_height,
    }]
}

fn field_specs_by_path(model: &SequentialModel) -> Vec<Vec<FieldSpec>> {
    vec![EXCITATION_FIELD_SPECS.to_vec(), emission_field_specs(model)]
}

const EXC_EFFECTIVE_FOCAL_LENGTH: f64 = -1.8750;
const EXC_IMAGE_LOCATION: f64 = 5.0000;
const EXC_ENTRANCE_PUPIL_LOCATION: f64 = -54.5455;
const EXC_ENTRANCE_PUPIL_SIZE: f64 = 1.5454;
const EXC_LAGRANGE_INVARIANT: f64 = -0.1594;

const EMI_IMAGE_LOCATION: f64 = 199.8800167976479;

fn model() -> SequentialModel {
    sequential_model(
        n!(1.0),
        n!(1.5),
        &EXCITATION_WAVELENGTHS,
        &EMISSION_WAVELENGTHS,
    )
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
    let view = ParaxialView::new(&model, &field_specs_by_path(&model), false)
        .expect("Could not create paraxial view");
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
    let view = ParaxialView::new(&model, &field_specs_by_path(&model), false)
        .expect("Could not create paraxial view");
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
    let view = ParaxialView::new(&model, &field_specs_by_path(&model), false)
        .expect("Could not create paraxial view");
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
    let view = ParaxialView::new(&model, &field_specs_by_path(&model), false)
        .expect("Could not create paraxial view");
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
    let view = ParaxialView::new(&model, &field_specs_by_path(&model), false)
        .expect("Could not create paraxial view");
    let sub_view = view.get_for_path(0, 0, 0).expect("path 0 subview");
    for &h in sub_view.lagrange_invariants() {
        assert_abs_diff_eq!(EXC_LAGRANGE_INVARIANT, h, epsilon = 1e-4);
    }
}

#[test]
fn at_emission_paraxial_image_location() {
    let model = model();
    let view = ParaxialView::new(&model, &field_specs_by_path(&model), false)
        .expect("Could not create paraxial view");
    let tangential_vec_id = view.tangential_vec_id_for_phi(1, std::f64::consts::FRAC_PI_2);
    let sub_view = view
        .get_for_path(1, 0, tangential_vec_id)
        .expect("path 1 subview");
    assert_abs_diff_eq!(
        EMI_IMAGE_LOCATION,
        sub_view.paraxial_image_plane().location,
        epsilon = 1e-4
    );
}

#[test]
fn at_excitation_and_emission_wavelengths_differ() {
    let model = model();
    assert_eq!(model.wavelengths_for_path(0), &EXCITATION_WAVELENGTHS);
    assert_eq!(model.wavelengths_for_path(1), &EMISSION_WAVELENGTHS);
}

#[test]
fn at_ray_trace_succeeds_with_differing_wavelength_counts_per_path() {
    use cherry_rs::ray_trace_3d_view;

    let model = sequential_model(
        n!(1.0),
        n!(1.5),
        &[0.488],               // excitation: 1 wavelength
        &[0.500, 0.520, 0.540], // emission: 3 wavelengths
    );
    let field_specs = field_specs_by_path(&model);
    let view = ParaxialView::new(&model, &field_specs, false).unwrap();

    // AT-7: subview counts per path match each path's own wavelength count.
    let path0_subviews = view.iter().filter(|sv| sv.path_id() == 0).count();
    let path1_subviews = view.iter().filter(|sv| sv.path_id() == 1).count();
    assert_eq!(path1_subviews, 3 * path0_subviews);

    // AT-6: ray_trace_3d_view succeeds and produces the right result counts.
    let aperture = ApertureSpec::EntrancePupil { semi_diameter: 1.5 };
    let config = SamplingConfig {
        n_fan_rays: 5,
        full_pupil_spacing: 0.1,
    };
    let trace = ray_trace_3d_view(&[aperture, aperture], &field_specs, &model, &view, config)
        .expect("ray trace should succeed with differing per-path wavelength counts");

    assert_eq!(
        trace.iter().filter(|r| r.path_id() == 0).count(),
        field_specs[0].len()
    );
    assert_eq!(
        trace.iter().filter(|r| r.path_id() == 1).count(),
        3 * field_specs[1].len()
    );
}

/// FR-13/AT-8: the emission path's `FieldSpec` height must equal the
/// excitation path's own computed transverse image height for its `y: 1.5`
/// field point — not an independently chosen constant.
#[test]
fn at_emission_field_height_equals_excitation_image_height() {
    let model = model();
    let field_specs = field_specs_by_path(&model);
    let view = ParaxialView::new(&model, &field_specs, false).unwrap();
    let exc_sub_view = view.get_for_path(0, 0, 0).unwrap();
    let exc_image_height = exc_sub_view.paraxial_image_plane().semi_diameter;
    match field_specs[1][0] {
        FieldSpec::PointSource { y, .. } => {
            assert_abs_diff_eq!(y, exc_image_height, epsilon = 1e-9);
        }
        _ => panic!("expected PointSource"),
    }
}
