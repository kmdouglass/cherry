use approx::assert_abs_diff_eq;

use cherry_rs::examples::michelson_interferometer::{
    BS_TO_LENS, LENS_F, LENS_TO_DETECTOR, sequential_model,
};
use cherry_rs::{
    ApertureSpec, FieldSpec, ParaxialView, SamplingConfig, SequentialModel, SurfaceKind,
    components_view, cross_section_view, n, ray_trace_3d_view,
};

const WAVELENGTHS: [f64; 1] = [0.6328];
const ARM_1: f64 = 100.0;
const ARM_2: f64 = 110.0;

// Store indices fixed by path 0's New steps, then path 1's M2.
const IRIS: usize = 1;
const BS: usize = 2;
const M1: usize = 3;
const DETECTOR: usize = 5;
const M2: usize = 6;

fn model() -> SequentialModel {
    sequential_model(n!(1.0), &WAVELENGTHS, ARM_1, ARM_2)
}

fn field_specs() -> Vec<Vec<FieldSpec>> {
    let fields = vec![
        FieldSpec::Angle {
            chi: 0.0,
            phi: 90.0,
        },
        FieldSpec::Angle {
            chi: 1.0,
            phi: 90.0,
        },
    ];
    vec![fields.clone(), fields]
}

/// AT-1: two paths recombining on one shared detector Image.
#[test]
fn at1_both_paths_end_on_the_same_image() {
    let model = model();
    assert_eq!(model.path_count(), 2);
    let last_0 = *model.path_surface_indices(0).last().unwrap();
    let last_1 = *model.path_surface_indices(1).last().unwrap();
    assert_eq!(last_0, last_1);
    assert_eq!(model.surfaces()[last_0].surface_kind(), SurfaceKind::Image);
}

/// AT-2: each arm visits the beam splitter twice.
#[test]
fn at2_each_path_visits_the_beam_splitter_twice() {
    let model = model();
    for path in 0..2 {
        let visits = model
            .path_surface_indices(path)
            .iter()
            .filter(|&&i| i == BS)
            .count();
        assert_eq!(visits, 2, "path {path}");
    }
    assert_eq!(model.path_surface_indices(0), &[0, 1, 2, 3, 2, 4, 5]);
    assert_eq!(model.path_surface_indices(1), &[0, 1, 2, 6, 2, 4, 5]);
}

/// AT-3: M1 straight ahead (+z), M2 folded up (+y), detector folded down
/// (−y).
#[test]
fn at3_layout_matches_the_textbook_picture() {
    let model = model();
    let p = |i: usize| model.placements()[i].position;
    let bs = p(BS);

    let m1 = p(M1);
    assert_abs_diff_eq!(m1.z() - bs.z(), ARM_1, epsilon = 1e-9);
    assert_abs_diff_eq!(m1.y() - bs.y(), 0.0, epsilon = 1e-9);

    let m2 = p(M2);
    assert_abs_diff_eq!(m2.y() - bs.y(), ARM_2, epsilon = 1e-9);
    assert_abs_diff_eq!(m2.z() - bs.z(), 0.0, epsilon = 1e-9);

    let det = p(DETECTOR);
    assert_abs_diff_eq!(
        det.y() - bs.y(),
        -(BS_TO_LENS + LENS_TO_DETECTOR),
        epsilon = 1e-9
    );
    assert_abs_diff_eq!(det.z() - bs.z(), 0.0, epsilon = 1e-9);
}

/// AT-4: every Shared step's cursor lands on the shared surface's stored
/// placement — i.e. the restated gaps agree with the shared geometry.
#[test]
fn at4_shared_steps_are_geometrically_consistent() {
    let model = model();
    for path in 0..2 {
        let indices = model.path_surface_indices(path);
        let steps = model.path_steps(path);
        // Skip step 0 (the Object at infinity).
        for (step, &store_idx) in indices.iter().enumerate().skip(1) {
            let cursor = steps[step].cursor_position;
            let placed = model.placements()[store_idx].position;
            assert_abs_diff_eq!(cursor.x(), placed.x(), epsilon = 1e-9);
            assert_abs_diff_eq!(cursor.y(), placed.y(), epsilon = 1e-9);
            assert_abs_diff_eq!(cursor.z(), placed.z(), epsilon = 1e-9);
        }
    }
}

/// AT-5: both arms focus at the lens focal plane with the iris as stop.
#[test]
fn at5_paraxial_properties_match_in_both_arms() {
    let model = model();
    let view = ParaxialView::new(&model, &field_specs(), false).unwrap();
    for path in 0..2 {
        let sub_views: Vec<_> = view.iter().filter(|sv| sv.path_id() == path).collect();
        assert!(!sub_views.is_empty(), "path {path} has no subviews");
        for sub_view in sub_views {
            assert_abs_diff_eq!(*sub_view.effective_focal_length(), LENS_F, epsilon = 1e-9);
            assert_abs_diff_eq!(
                sub_view.paraxial_image_plane().location,
                LENS_TO_DETECTOR,
                epsilon = 1e-9
            );
            assert_eq!(
                model.path_surface_indices(path)[*sub_view.aperture_stop()],
                IRIS
            );
        }
    }
}

/// AT-6: no ray terminates early in either arm.
#[test]
fn at6_ray_trace_no_ray_terminates_either_path() {
    let model = model();
    let fields = field_specs();
    let view = ParaxialView::new(&model, &fields, false).unwrap();
    let aperture = ApertureSpec::EntrancePupil { semi_diameter: 2.0 };
    let config = SamplingConfig {
        n_fan_rays: 5,
        full_pupil_spacing: 0.25,
    };
    let trace = ray_trace_3d_view(&[aperture, aperture], &fields, &model, &view, config)
        .expect("ray trace should succeed");

    assert_eq!(trace.iter().count(), 4);
    for r in trace.iter() {
        assert!(r.chief_ray_reached_image(), "path {}", r.path_id());
        for (name, bundle) in [
            ("tangential", r.tangential_fan()),
            ("sagittal", r.sagittal_fan()),
            ("full pupil", r.full_pupil()),
        ] {
            assert!(
                bundle.terminated().iter().all(|&t| t == 0),
                "path {} {name} ray(s) terminated early: {:?}",
                r.path_id(),
                bundle.reason_for_termination()
            );
        }
    }
}

/// AT-7: component and cross-section views build.
#[test]
fn at7_cross_section_view_builds() {
    let model = model();
    let components = components_view(&model, n!(1.0)).unwrap();
    let _ = cross_section_view(&model, None, &components);
}
