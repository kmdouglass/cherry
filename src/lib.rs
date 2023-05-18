use wasm_bindgen::prelude::*;

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
        let diameter = 4f32;
        let surf1_axial_pos = 5f32;
        let thickness = 1f32;
        let n = 2.5f32; // Refractive index
        let roc = -1.67f32; // Radius of curvature
        let k = -6.25f32; // Conic constant

        let wavelength = 0.000633f32;

        let surf1 = surfaces::Surface::new_refr_circ_flat(surf1_axial_pos, diameter, n);
        let surf2 = surfaces::Surface::new_refr_circ_conic(
            surf1_axial_pos + thickness,
            diameter,
            n,
            roc,
            k,
        );

        panic!("TODO: Implement test")
    }
}
