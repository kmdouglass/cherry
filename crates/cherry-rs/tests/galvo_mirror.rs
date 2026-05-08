use cherry_rs::{FieldSpec, ParaxialView, examples::galvo_mirror, n};

const WAVELENGTHS: [f64; 1] = [0.5876];
const FIELD_SPECS: [FieldSpec; 1] = [FieldSpec::Angle {
    chi: 0.0,
    phi: 90.0,
}];

// A single flat galvo mirror has no optical power, so EFL and BFD are infinite.
// This is the key regression test for the paraxial RTM sign convention: the old
// `reflected` sign-toggle hack incorrectly negated the gap after the mirror,
// which corrupted EFL for folded (non-retroreflective) systems.

#[test]
fn galvo_mirror_efl_is_infinite() {
    let model = galvo_mirror::sequential_model(n!(1.0), &WAVELENGTHS);
    let view = ParaxialView::new(&model, &FIELD_SPECS, false).expect("paraxial view");
    for sub_view in view.iter() {
        assert!(
            sub_view.effective_focal_length().is_infinite(),
            "Expected infinite EFL for flat galvo mirror, got {}",
            sub_view.effective_focal_length()
        );
    }
}

#[test]
fn galvo_mirror_bfd_is_infinite() {
    let model = galvo_mirror::sequential_model(n!(1.0), &WAVELENGTHS);
    let view = ParaxialView::new(&model, &FIELD_SPECS, false).expect("paraxial view");
    for sub_view in view.iter() {
        assert!(
            sub_view.back_focal_distance().is_infinite(),
            "Expected infinite BFD for flat galvo mirror, got {}",
            sub_view.back_focal_distance()
        );
    }
}

#[test]
fn galvo_mirror_marginal_ray_height_unchanged() {
    let model = galvo_mirror::sequential_model(n!(1.0), &WAVELENGTHS);
    let view = ParaxialView::new(&model, &FIELD_SPECS, false).expect("paraxial view");
    for sub_view in view.iter() {
        let marginal = sub_view.marginal_ray();
        let h_at_mirror = marginal.rays_at_surface(1)[0].height;
        let h_at_image = marginal.rays_at_surface(2)[0].height;
        // A flat mirror with a collimated input (u=0) leaves height unchanged
        // after propagation (u stays 0, so h_image = h_mirror + 100*0 = h_mirror).
        approx::assert_abs_diff_eq!(h_at_image, h_at_mirror, epsilon = 1e-10);
    }
}
