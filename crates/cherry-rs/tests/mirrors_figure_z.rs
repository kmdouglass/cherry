use std::f64::consts::FRAC_PI_2;

use approx::assert_abs_diff_eq;
use cherry_rs::{FieldSpec, ParaxialView, examples::mirrors_figure_z, n};

const WAVELENGTHS: [f64; 1] = [0.5876];
const FIELD_SPECS: [FieldSpec; 1] = [FieldSpec::Angle {
    chi: 0.0,
    phi: 90.0,
}];

// Aperture stop is the first mirror (surface 1, track = 0).
const APERTURE_STOP: usize = 1;

// Entrance pupil coincides with the aperture stop (Mirror 1, theta = 30°).
// phi=90° (v=Y): projected SD = r · cos(30°) (theta tilt foreshortens in Y).
// phi=0° (v=X): projected SD = r (theta tilt does not affect X).
const ENTRANCE_PUPIL_SD_U: f64 = 12.7 * 0.8660254037844387; // 12.7 · cos(30°)
const ENTRANCE_PUPIL_SD_R: f64 = 12.7;

// Exit pupil: image of Mirror 1 (track=0) formed by the flat Mirror 2
// (track=100). A flat mirror images an object 100 mm in front of it to 100 mm
// behind it, so the exit pupil is 100 mm past Mirror 2 (distance from last
// physical surface).
const EXIT_PUPIL_LOCATION: f64 = 100.0;

/// For a straight system track == z for every finite surface.
#[test]
fn track_equals_z_for_straight_system() {
    use cherry_rs::examples::convexplano_lens;
    let model = convexplano_lens::sequential_model(n!(1.0), n!(1.515), &WAVELENGTHS);
    for p in model.placements() {
        if p.position.z().is_finite() {
            assert!(
                (p.track - p.position.z()).abs() < 1e-10,
                "Expected track == z for straight system, got track={}, z={}",
                p.track,
                p.position.z()
            );
        }
    }
}

#[test]
fn mirrors_figure_z_paraxial_aperture_stop() {
    let model = mirrors_figure_z::sequential_model(n!(1.0), &WAVELENGTHS);
    let view = ParaxialView::new(&model, &FIELD_SPECS, false).expect("paraxial view");
    for sub_view in view.iter() {
        assert_eq!(*sub_view.aperture_stop(), APERTURE_STOP);
    }
}

#[test]
fn mirrors_figure_z_paraxial_exit_pupil() {
    let model = mirrors_figure_z::sequential_model(n!(1.0), &WAVELENGTHS);
    let view = ParaxialView::new(&model, &FIELD_SPECS, false).expect("paraxial view");
    for sub_view in view.iter() {
        assert_abs_diff_eq!(
            sub_view.exit_pupil().location,
            EXIT_PUPIL_LOCATION,
            epsilon = 1e-4
        );
    }
}

/// Marginal ray at the aperture stop equals the projected SD, not the raw SD.
///
/// For Mirror 1 tilted 30° about cursor-R, the effective clear aperture seen by
/// a paraxial beam along the cursor-U axis is `r · cos(30°)`.  The marginal ray
/// must be scaled to exactly fill this projected aperture.
#[test]
fn mirrors_figure_z_marginal_ray_uses_projected_sd() {
    let model = mirrors_figure_z::sequential_model(n!(1.0), &WAVELENGTHS);
    let view = ParaxialView::new(&model, &FIELD_SPECS, false).expect("paraxial view");
    let r = 12.7_f64;
    let projected_u = r * (30.0_f64.to_radians()).cos();

    // FIELD_SPECS has phi=90° → only one submodel with v=Y (foreshortened).
    let sub_view = view.iter().next().expect("at least one subview");
    assert_eq!(*sub_view.aperture_stop(), 1usize);
    let marginal_height_at_stop = sub_view.marginal_ray().rays_at_surface(1)[0].height;
    assert_abs_diff_eq!(marginal_height_at_stop, projected_u, epsilon = 1e-10);
}

/// A phi=90° field (v=Y) is foreshortened by the 30°-about-X mirror tilt.
/// The entrance pupil semi-diameter must equal 12.7·cos(30°) ≈ 10.9985.
#[test]
fn entrance_pupil_sd_phi_90_foreshortened() {
    let field_specs = [FieldSpec::Angle {
        chi: 0.0,
        phi: 90.0,
    }];
    let model = mirrors_figure_z::sequential_model(n!(1.0), &WAVELENGTHS);
    let view = ParaxialView::new(&model, &field_specs, false).expect("paraxial view");
    let tangential_vec_id = view.tangential_vec_id_for_phi(FRAC_PI_2);
    let ep = view.get(0, tangential_vec_id).unwrap().entrance_pupil();
    assert_abs_diff_eq!(ep.semi_diameter, ENTRANCE_PUPIL_SD_U, epsilon = 1e-4);
}

/// A phi=0° field (v=X) is not foreshortened by the 30°-about-X mirror tilt.
/// The entrance pupil semi-diameter must equal the raw 12.7.
#[test]
fn entrance_pupil_sd_phi_0_not_foreshortened() {
    let field_specs = [FieldSpec::Angle { chi: 0.0, phi: 0.0 }];
    let model = mirrors_figure_z::sequential_model(n!(1.0), &WAVELENGTHS);
    let view = ParaxialView::new(&model, &field_specs, false).expect("paraxial view");
    let tangential_vec_id = view.tangential_vec_id_for_phi(0.0);
    let ep = view.get(0, tangential_vec_id).unwrap().entrance_pupil();
    assert_abs_diff_eq!(ep.semi_diameter, ENTRANCE_PUPIL_SD_R, epsilon = 1e-4);
}

/// Each submodel uses only the field specs that match its phi angle for the
/// chief ray. The phi=90° submodel uses the 5° field; the phi=0° submodel uses
/// the 3° field.
#[test]
fn chief_ray_uses_matching_field_phi() {
    let field_specs = [
        FieldSpec::Angle {
            chi: 5.0,
            phi: 90.0,
        },
        FieldSpec::Angle { chi: 3.0, phi: 0.0 },
    ];
    let model = mirrors_figure_z::sequential_model(n!(1.0), &WAVELENGTHS);
    let view = ParaxialView::new(&model, &field_specs, false).expect("paraxial view");

    let v_phi90 = view.tangential_vec_id_for_phi(FRAC_PI_2);
    let v_phi0 = view.tangential_vec_id_for_phi(0.0);

    let angle_phi90 = view.get(0, v_phi90).unwrap().chief_ray().rays_at_surface(0)[0].angle;
    let angle_phi0 = view.get(0, v_phi0).unwrap().chief_ray().rays_at_surface(0)[0].angle;

    assert_abs_diff_eq!(angle_phi90, 5.0_f64.to_radians().tan(), epsilon = 1e-6);
    assert_abs_diff_eq!(angle_phi0, 3.0_f64.to_radians().tan(), epsilon = 1e-6);
}

/// Track accumulates path length regardless of fold direction.
#[test]
fn track_coordinates_folded_system() {
    let model = mirrors_figure_z::sequential_model(n!(1.0), &WAVELENGTHS);
    let placements = model.placements();

    // placements[0] Object: cursor initialized at -inf
    assert!(placements[0].track.is_infinite() && placements[0].track < 0.0);

    // placements[1] Mirror 1: special-case advance from -inf resets track to 0
    assert_eq!(placements[1].track, 0.0);

    // placements[2] Mirror 2: 0 + gap_1 (100 mm)
    assert!(
        (placements[2].track - 100.0).abs() < 1e-10,
        "Mirror 2 track should be 100, got {}",
        placements[2].track
    );

    // placements[3] Image: 100 + gap_2 (50 mm)
    assert!(
        (placements[3].track - 150.0).abs() < 1e-10,
        "Image track should be 150, got {}",
        placements[3].track
    );
}
