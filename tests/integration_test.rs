fn refractive_index(_: f32) -> (f32, f32) {
    (2.5, 0.0)
}

#[test]
fn test_ray_trace_planoconvex_lens() {
    let radius = 2f32;
    let thickness = 1f32;
    let roc = -1.67f32; // Radius of curvature
    let k = -6.25f32; // Conic constant

    let wavelength = 0.000633f32;

    panic!("TODO: Implement test")
}
