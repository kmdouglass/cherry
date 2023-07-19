use wasm_bindgen::prelude::*;

mod math;
mod ray_tracing;
mod rendering;
mod surfaces;

#[wasm_bindgen]
pub struct SystemModel {
    surfaces: Vec<surfaces::Surface>,
}

#[wasm_bindgen]
impl SystemModel {
    pub fn new() -> SystemModel {
        // Setup the optical system
        // From Mansuripur, "Abbe's Sine Condition," Optics and Photonics News 9(2), 56-60 (1998)
        let diameter = 4f32;
        let surf1_axial_pos = 5f32;
        let thickness = 1f32;
        let efl = 1.1133f32; // Effective focal length
        let n = 2.5f32; // Refractive index
        let roc = -1.67f32; // Radius of curvature
        let k = -6.25f32; // Conic constant

        let obj_surf = surfaces::Surface::new_obj_or_img_plane(f32::NEG_INFINITY, diameter);
        let surf1 = surfaces::Surface::new_refr_circ_flat(surf1_axial_pos, diameter, n);
        let surf2 = surfaces::Surface::new_refr_circ_conic(
            surf1_axial_pos + thickness,
            diameter,
            1.0,
            roc,
            k,
        );
        let img_surf =
            surfaces::Surface::new_obj_or_img_plane(surf1_axial_pos + thickness + efl, diameter);

        // Build the sequential optical system model
        let surfaces = vec![obj_surf, surf1, surf2, img_surf];

        SystemModel { surfaces }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ray_trace_planoconvex_lens() {
        use ray_tracing::rays::Ray;

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

        let obj_surf = surfaces::Surface::new_obj_or_img_plane(f32::NEG_INFINITY, diameter);
        let surf1 = surfaces::Surface::new_refr_circ_flat(surf1_axial_pos, diameter, n);
        let surf2 = surfaces::Surface::new_refr_circ_conic(
            surf1_axial_pos + thickness,
            diameter,
            1.0,
            roc,
            k,
        );
        let img_surf =
            surfaces::Surface::new_obj_or_img_plane(surf1_axial_pos + thickness + efl, diameter);

        // Build the sequential optical system model
        let surfaces = vec![obj_surf, surf1, surf2, img_surf];

        // Define the rays to trace
        let rays = vec![
            Ray::new(
                math::vec3::Vec3::new(0.0, -diameter / 2.0, 0.0),
                math::vec3::Vec3::new(0.0, 0.0, 1.0),
            )
            .unwrap(),
            Ray::new(
                math::vec3::Vec3::new(0.0, 0.0, 0.0),
                math::vec3::Vec3::new(0.0, 0.0, 1.0),
            )
            .unwrap(),
            Ray::new(
                math::vec3::Vec3::new(0.0, diameter / 2.0, 0.0),
                math::vec3::Vec3::new(0.0, 0.0, 1.0),
            )
            .unwrap(),
        ];

        // Trace the rays; skip the object plane
        let results = ray_tracing::ray_trace(&surfaces[1..], rays, wavelength);
        println!("{:?}", results);
    }
}
