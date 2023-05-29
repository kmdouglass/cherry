use wasm_bindgen::prelude::*;

mod ray_tracing;
mod surfaces;
mod vec3;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ray_trace_planoconvex_lens() {
        // Setup the optical system
        // From Mansuripur, "Abbe's Sine Condition," Optics and Photonics News 9(2), 56-60 (1998)
        let diameter = 4f32;
        let surf1_axial_pos = 5f32;
        let thickness = 1f32;
        let efl = 1.1133f32; // Effective focal length
        let n = 2.5f32; // Refractive index
        let roc = -1.67f32; // Radius of curvature
        let k = -6.25f32; // Conic constant

        let wavelength = 0.000633f32;

        let obj_surf = surfaces::Surface::new_obj_or_img_plane(0.0);
        let surf1 = surfaces::Surface::new_refr_circ_flat(surf1_axial_pos, diameter, n);
        let surf2 = surfaces::Surface::new_refr_circ_conic(
            surf1_axial_pos + thickness,
            diameter,
            1.0,
            roc,
            k,
        );
        let img_surf = surfaces::Surface::new_obj_or_img_plane(surf1_axial_pos + thickness + efl);

        // Build the sequential optical system model
        let surfaces = vec![obj_surf, surf1, surf2, img_surf];

        // Define the rays to trace
        let rays = vec![
            ray_tracing::Ray::new(
                vec3::Vec3::new(0.0, -diameter / 2.0, 0.0),
                vec3::Vec3::new(0.0, 0.0, 1.0),
            )
            .unwrap(),
            ray_tracing::Ray::new(
                vec3::Vec3::new(0.0, 0.0, 0.0),
                vec3::Vec3::new(0.0, 0.0, 1.0),
            )
            .unwrap(),
            ray_tracing::Ray::new(
                vec3::Vec3::new(0.0, diameter / 2.0, 0.0),
                vec3::Vec3::new(0.0, 0.0, 1.0),
            )
            .unwrap(),
        ];

        panic!("TODO: Implement test")
    }
}
