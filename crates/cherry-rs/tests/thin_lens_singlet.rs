use cherry_rs::{FieldSpec, ParaxialView, examples::thin_lens_singlet, n};

const WAVELENGTHS: [f64; 1] = [0.5876];
const FIELD_SPECS: [FieldSpec; 1] = [FieldSpec::Angle { chi: 0.0, phi: 0.0 }];

// An idealized thin lens has no thickness, so its principal planes coincide
// with the lens itself — back focal distance equals the effective focal
// length exactly, for an object at infinity.

#[test]
fn thin_lens_efl_equals_focal_length() {
    let model = thin_lens_singlet::sequential_model(n!(1.0), &WAVELENGTHS);
    let view = ParaxialView::new(&model, &FIELD_SPECS, false).expect("paraxial view");
    for sub_view in view.iter() {
        approx::assert_abs_diff_eq!(*sub_view.effective_focal_length(), 100.0, epsilon = 1e-9);
    }
}

#[test]
fn thin_lens_bfd_equals_focal_length() {
    let model = thin_lens_singlet::sequential_model(n!(1.0), &WAVELENGTHS);
    let view = ParaxialView::new(&model, &FIELD_SPECS, false).expect("paraxial view");
    for sub_view in view.iter() {
        approx::assert_abs_diff_eq!(*sub_view.back_focal_distance(), 100.0, epsilon = 1e-9);
    }
}

#[test]
fn thin_lens_marginal_ray_crosses_axis_at_focal_plane() {
    let model = thin_lens_singlet::sequential_model(n!(1.0), &WAVELENGTHS);
    let view = ParaxialView::new(&model, &FIELD_SPECS, false).expect("paraxial view");
    for sub_view in view.iter() {
        let marginal = sub_view.marginal_ray();
        // Surface 0 = Object (at infinity), 1 = ThinLens, 2 = Image (placed at
        // the back focal distance by the example's gap thickness of 100 mm).
        let h_image = marginal.rays_at_surface(2)[0].height;
        approx::assert_abs_diff_eq!(h_image, 0.0, epsilon = 1e-9);
    }
}
