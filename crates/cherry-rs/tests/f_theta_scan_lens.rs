use cherry_rs::examples::f_theta_scan_lens::{field_specs, sequential_model};
use cherry_rs::{ApertureSpec, FieldSpec, ParaxialView, SamplingConfig, n, ray_trace_3d_view};

const WAVELENGTHS: [f64; 1] = [0.5876]; // He d line

fn init_tracing() {
    use tracing_subscriber::{EnvFilter, fmt};
    let _ = fmt()
        .pretty()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();
}

fn setup() -> (
    cherry_rs::SequentialModel,
    ApertureSpec,
    Vec<FieldSpec>,
    ParaxialView,
) {
    let model = sequential_model(n!(1.0), n!(1.84666), &WAVELENGTHS);
    let aperture_spec = ApertureSpec::EntrancePupil { semi_diameter: 0.5 };
    let field_specs = field_specs();
    let paraxial_view =
        ParaxialView::new(&model, &field_specs, false).expect("Could not create paraxial view");
    (model, aperture_spec, field_specs, paraxial_view)
}

#[test]
fn test_ray_trace_3d_on_axis() {
    init_tracing();
    let (model, aperture_spec, field_specs, paraxial_view) = setup();

    let results = ray_trace_3d_view(
        &aperture_spec,
        &field_specs,
        &model,
        &paraxial_view,
        SamplingConfig {
            n_fan_rays: 9,

            full_pupil_spacing: 0.1,
        },
    )
    .expect("Ray trace failed");

    assert!(!results.is_empty());
}

#[test]
fn test_ray_trace_3d_off_axis() {
    init_tracing();
    let (model, aperture_spec, _, paraxial_view) = setup();

    let off_axis_fields = vec![
        FieldSpec::Angle {
            chi: 5.0,
            phi: 90.0,
        },
        FieldSpec::Angle {
            chi: 10.0,
            phi: 90.0,
        },
        FieldSpec::Angle {
            chi: 20.0,
            phi: 90.0,
        },
    ];

    let results = ray_trace_3d_view(
        &aperture_spec,
        &off_axis_fields,
        &model,
        &paraxial_view,
        SamplingConfig {
            n_fan_rays: 9,

            full_pupil_spacing: 0.1,
        },
    )
    .expect("Ray trace failed");

    assert_eq!(results.len(), off_axis_fields.len());
}

#[test]
fn test_ray_trace_3d_square_grid() {
    init_tracing();
    let (model, aperture_spec, _, paraxial_view) = setup();

    let fields = vec![FieldSpec::Angle {
        chi: 10.0,
        phi: 90.0,
    }];

    let results = ray_trace_3d_view(
        &aperture_spec,
        &fields,
        &model,
        &paraxial_view,
        SamplingConfig {
            n_fan_rays: 9,

            full_pupil_spacing: 0.5,
        },
    )
    .expect("Ray trace failed");

    assert!(!results.is_empty());
}
