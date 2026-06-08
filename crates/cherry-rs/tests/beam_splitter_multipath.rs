use approx::assert_abs_diff_eq;
use cherry_rs::{
    BoundaryKind, GapSpec, PathSpec, PathSurfaceRef, Rotation3D, SequentialModel,
    SequentialModelBuilder, SurfaceSpec, Vec3, examples::beam_splitter, n,
};

const WAVELENGTHS: [f64; 1] = [0.5876];

fn two_path_bs_model() -> SequentialModel {
    beam_splitter::two_path_model(n!(1.0), &WAVELENGTHS, 100.0, 100.0)
}

fn two_path_bs_model_with_gaps(gap_transmitted: f64, gap_reflected: f64) -> SequentialModel {
    beam_splitter::two_path_model(n!(1.0), &WAVELENGTHS, gap_transmitted, gap_reflected)
}

#[test]
fn at2_two_path_bs_model_has_two_paths() {
    let model = two_path_bs_model();
    assert_eq!(model.path_count(), 2);
}

// AT-4: transmitted arm image is at z ≈ 100 from BS.
#[test]
fn at4_transmitted_arm_image_along_forward_axis() {
    let model = two_path_bs_model();
    let img_z = model.path_placement(0, 2).position.z();
    assert_abs_diff_eq!(img_z, 100.0, epsilon = 1e-9);
}

// AT-5: reflected arm image is at y ≈ −100 from BS.
// A −45° R-axis rotation tilts the BS so that reflection deflects the cursor
// into the −y direction; the image plane lands at y = −100, z = 0.
#[test]
fn at5_reflected_arm_image_along_y_axis() {
    let model = two_path_bs_model();
    let img_y = model.path_placement(1, 2).position.y();
    assert_abs_diff_eq!(img_y, -100.0, epsilon = 1e-9);
}

// AT-3: Object and BS are shared (single store entries), so their placements
// are trivially identical between the two paths.
#[test]
fn at3_shared_surface_placements_are_identical() {
    let model = two_path_bs_model();

    // Both paths start with Object (store 0) and BS (store 1).
    assert_eq!(model.path_surface_indices(0)[0], 0);
    assert_eq!(model.path_surface_indices(1)[0], 0);
    assert_eq!(model.path_surface_indices(0)[1], 1);
    assert_eq!(model.path_surface_indices(1)[1], 1);

    // BS sits at the origin.
    assert_abs_diff_eq!(model.placements()[1].position.z(), 0.0, epsilon = 1e-10);
}

// AT-6: reflected-arm cursor is deflected at the BS.
#[test]
fn at6_reflected_arm_cursor_deflected_at_bs() {
    let model = two_path_bs_model();
    let img = model.path_placement(1, 2);
    assert_abs_diff_eq!(img.position.y(), -100.0, epsilon = 1e-9);
    assert_abs_diff_eq!(img.position.z(), 0.0, epsilon = 1e-9);
}

// AT-7: arm-specific gaps are independent.
#[test]
fn at7_arm_specific_gaps_are_independent() {
    let model = two_path_bs_model_with_gaps(50.0, 80.0);
    let img_t_z = model.path_placement(0, 2).position.z();
    let img_r_y = model.path_placement(1, 2).position.y();
    assert_abs_diff_eq!(img_t_z, 50.0, epsilon = 1e-9);
    assert_abs_diff_eq!(img_r_y, -80.0, epsilon = 1e-9);
}

// AT-13: a repeated store index (Shared revisiting a surface) is accepted.
#[test]
fn at13_repeated_store_index_is_permitted_via_pathspec() {
    // Path: [New(Obj), New(Sphere), Shared(1), New(Img)]
    // surface_indices = [0, 1, 1, 2]
    let path = PathSpec {
        surface_refs: vec![
            PathSurfaceRef::New(SurfaceSpec::Object),
            PathSurfaceRef::New(SurfaceSpec::Sphere {
                semi_diameter: 12.7,
                radius_of_curvature: 65.0,
                surf_kind: BoundaryKind::Refracting,
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            }),
            PathSurfaceRef::Shared(1),
            PathSurfaceRef::New(SurfaceSpec::Image {
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            }),
        ],
        gaps: vec![
            GapSpec {
                thickness: f64::INFINITY,
                refractive_index: n!(1.0),
            },
            GapSpec {
                thickness: 10.0,
                refractive_index: n!(1.0),
            },
            GapSpec {
                thickness: 10.0,
                refractive_index: n!(1.0),
            },
        ],
        beam_splitter_arms: vec![],
    };
    let result = SequentialModelBuilder::new()
        .paths(vec![path])
        .wavelengths(vec![0.587])
        .build();
    assert!(result.is_ok());
    let model = result.unwrap().model;
    assert_eq!(model.path_surface_indices(0), &[0, 1, 1, 2]);
}
