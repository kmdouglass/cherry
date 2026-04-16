use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use cherry_rs::{
    ApertureSpec, FieldSpec, ParaxialView, SamplingConfig,
    examples::f_theta_scan_lens::sequential_model, n, ray_trace_3d_view,
};

const WAVELENGTHS: [f64; 1] = [0.5876]; // He d line
const APERTURE_SPEC: ApertureSpec = ApertureSpec::EntrancePupil { semi_diameter: 0.5 };

fn benchmark(c: &mut Criterion) {
    let model = sequential_model(n!(1.0), n!(1.84666), &WAVELENGTHS);
    let field_specs = vec![FieldSpec::Angle {
        chi: 20.0,
        phi: 90.0,
    }];
    let paraxial_view = ParaxialView::new(&model, &field_specs, false).unwrap();
    let mut group = c.benchmark_group("3D ray trace, f-theta scan lens");

    group.bench_function("ray_trace_3d_view, 20 deg off-axis", |b| {
        b.iter(|| {
            ray_trace_3d_view(
                black_box(&APERTURE_SPEC),
                black_box(&field_specs),
                black_box(&model),
                black_box(&paraxial_view),
                black_box(SamplingConfig {
                    n_fan_rays: 9,
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
