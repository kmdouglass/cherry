use approx::assert_abs_diff_eq;

use cherry_rs::examples::wf_epi_excitation::sequential_model;
use cherry_rs::{FieldSpec, ParaxialView, n};

const WAVELENGTHS: [f64; 1] = [0.5876];
const FIELD_SPECS: [FieldSpec; 1] = [FieldSpec::PointSource { x: 0.0, y: 0.0 }];

// Hand calculation: the stop (second thin lens) is 150 mm (100 mm + 50 mm,
// the fold mirror has no power) downstream of the first thin lens
// (f = 40 mm). Treating the stop as a real object and solving the thin lens
// equation 1/s' = 1/f - 1/s with s = 150 mm gives s' = 600/11 = 54.5455 mm,
// with the resulting image (the entrance pupil) on the object-space side of
// the first thin lens.
const ENTRANCE_PUPIL_LOCATION: f64 = -54.5455;

#[test]
fn wf_epi_excitation_entrance_pupil_location() {
    let model = sequential_model(n!(1.0), &WAVELENGTHS);
    let view =
        ParaxialView::new(&model, &FIELD_SPECS, false).expect("Could not create paraxial view");

    for sub_view in view.iter() {
        let entrance_pupil = sub_view.entrance_pupil();
        assert_abs_diff_eq!(
            entrance_pupil.location,
            ENTRANCE_PUPIL_LOCATION,
            epsilon = 1e-3
        );
    }
}
