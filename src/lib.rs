use wasm_bindgen::prelude::*;

pub mod surfaces;
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
        let thickness = 1f32;
        let n = 2.5f32; // Refractive index
        let roc = -1.67f32; // Radius of curvature
        let k = -6.25f32; // Conic constant

        let wavelength = 0.000633f32;

        let surf = surfaces::Surface::new_refr_circ_conic(0.0, diameter, n, roc, k);

        panic!("TODO: Implement test")
    }
}