use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::rc::Rc;

use cherry_rs::{
    ApertureSpec, FieldSpec, ParaxialView, RefractiveIndexSpec, SamplingConfig,
    examples::convexplano_lens::sequential_model, n, ray_trace_3d_view,
};

// Inputs
const WAVELENGTHS: [f64; 1] = [0.5876]; // He d line
const FIELD_SPECS: [FieldSpec; 2] = [
    FieldSpec::Angle {
        chi: 0.0,
        phi: 90.0,
    },
    FieldSpec::Angle {
        chi: 5.0,
        phi: 90.0,
    },
];
const APERTURE_SPEC: ApertureSpec = ApertureSpec::EntrancePupil { semi_diameter: 5.0 };

// Create a benchmark group to compare ray_trace_3d_view with
// ray_trace_3d_view_v2 Use c.benchmark_group
fn benchmark(c: &mut Criterion) {
    let n_air: Rc<dyn RefractiveIndexSpec> = n!(1.0);
    let n_nbk7: Rc<dyn RefractiveIndexSpec> = n!(1.515);
    let model = sequential_model(n_air, n_nbk7, &WAVELENGTHS);
    let paraxial_view = ParaxialView::new(&model, &FIELD_SPECS, false).unwrap();
    let mut group = c.benchmark_group("3D ray trace, convexplano lens");

    group.bench_function("ray_trace_3d_view", |b| {
        b.iter(|| {
            ray_trace_3d_view(
                black_box(&APERTURE_SPEC),
                black_box(&FIELD_SPECS),
                black_box(&model),
                black_box(&paraxial_view),
                black_box(SamplingConfig {
                    n_fan_rays: 9,
                    cross_section_n_fan_rays: 3,
                    full_pupil_spacing: 0.1,
                }),
            )
            .unwrap();
        });
    });
    group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
