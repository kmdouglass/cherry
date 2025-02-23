use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use std::rc::Rc;

use cherry_rs::{
    examples::convexplano_lens::sequential_model, n, ray_trace_3d_view, ApertureSpec, FieldSpec,
    ParaxialView, PupilSampling, RefractiveIndexSpec,
};

// Inputs
const WAVELENGTHS: [f64; 1] = [0.5876]; // He d line
const FIELD_SPECS: [FieldSpec; 2] = [
    FieldSpec::Angle {
        angle: 0.0,
        pupil_sampling: PupilSampling::ChiefAndMarginalRays,
    },
    FieldSpec::Angle {
        angle: 5.0,
        pupil_sampling: PupilSampling::ChiefAndMarginalRays,
    },
];
const APERTURE_SPEC: ApertureSpec = ApertureSpec::EntrancePupil { semi_diameter: 5.0 };

fn benchmark(c: &mut Criterion) {
    c.bench_function("3D ray trace, convexplano lens", |b| {
        let n_air: Rc<dyn RefractiveIndexSpec> = n!(1.0);
        let n_nbk7: Rc<dyn RefractiveIndexSpec> = n!(1.515);
        let model = sequential_model(n_air, n_nbk7, &WAVELENGTHS);
        let paraxial_view = ParaxialView::new(&model, &FIELD_SPECS, false).unwrap();

        b.iter(|| {
            ray_trace_3d_view(
                black_box(&APERTURE_SPEC),
                black_box(&FIELD_SPECS),
                black_box(&model),
                black_box(&paraxial_view),
                black_box(None),
            )
            .unwrap();
        })
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
