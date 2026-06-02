use approx::assert_abs_diff_eq;
use cherry_rs::{examples::beam_splitter, n};

const WAVELENGTHS: [f64; 1] = [0.5876];

// A 45° beam splitter (no optical power): both paths produce infinite EFL.

#[test]
fn transmitting_model_builds_without_error() {
    beam_splitter::transmitting_model(n!(1.0), &WAVELENGTHS);
}

#[test]
fn reflecting_model_builds_without_error() {
    beam_splitter::reflecting_model(n!(1.0), &WAVELENGTHS);
}

// In the transmitting model the cursor never changes direction: every
// axis_direction should be (0, 0, 1).
#[test]
fn transmitting_path_cursor_continues_along_z() {
    let model = beam_splitter::transmitting_model(n!(1.0), &WAVELENGTHS);
    for dir in model.axis_directions() {
        assert_abs_diff_eq!(dir.x(), 0.0, epsilon = 1e-10);
        assert_abs_diff_eq!(dir.y(), 0.0, epsilon = 1e-10);
        assert_abs_diff_eq!(dir.z(), 1.0, epsilon = 1e-10);
    }
}

// In the reflecting model the cursor is deflected 90° by the 45° mirror.
// After reflection the forward direction should be (0, 1, 0) (+y).
#[test]
fn reflecting_path_cursor_deflects_90_degrees() {
    let model = beam_splitter::reflecting_model(n!(1.0), &WAVELENGTHS);
    let dirs = model.axis_directions();

    // Before the beam splitter: along +z.
    assert_abs_diff_eq!(dirs[0].z(), 1.0, epsilon = 1e-10);

    // After the beam splitter: deflected 90° into -y (a -45° R-axis rotation
    // reflects the beam downward in the y direction).
    assert_abs_diff_eq!(dirs[2].x(), 0.0, epsilon = 1e-10);
    assert_abs_diff_eq!(dirs[2].y(), -1.0, epsilon = 1e-10);
    assert_abs_diff_eq!(dirs[2].z(), 0.0, epsilon = 1e-10);
}

// The image plane positions must differ between the two paths.
#[test]
fn transmitting_and_reflecting_image_planes_differ() {
    let tx = beam_splitter::transmitting_model(n!(1.0), &WAVELENGTHS);
    let rx = beam_splitter::reflecting_model(n!(1.0), &WAVELENGTHS);

    let tx_image = tx.cursor_positions().last().unwrap();
    let rx_image = rx.cursor_positions().last().unwrap();

    // Transmitting: image is 100 mm along +z from the BS at the origin.
    assert_abs_diff_eq!(tx_image.z(), 100.0, epsilon = 1e-10);

    // Reflecting: image is 100 mm along -y from the BS at the origin.
    assert_abs_diff_eq!(rx_image.y(), -100.0, epsilon = 1e-10);
    assert_abs_diff_eq!(rx_image.z(), 0.0, epsilon = 1e-6);
}
