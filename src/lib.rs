use std::f32::consts::PI;

use wasm_bindgen::prelude::*;

mod math;
mod ray_tracing;
mod rendering;
mod surfaces;

#[wasm_bindgen]
#[derive(Debug)]
pub struct SystemModel {
    surfaces: Vec<surfaces::Surface>,
}

#[wasm_bindgen]
#[allow(non_snake_case)]
impl SystemModel {
    #[wasm_bindgen(constructor)]
    pub fn new() -> SystemModel {
        let surfaces = Vec::new();

        SystemModel { surfaces }
    }

    /// Trace a set of rays through the system.
    pub fn rayTrace(&self) -> JsValue {
        // Find the maximum diameter of the system
        let mut max_diam = 0.0;
        for surf in &self.surfaces {
            if surf.diam() > max_diam {
                max_diam = surf.diam();
            }
        }

        // Find the z-position of the first surface
        let mut first_surf_z = f32::INFINITY;
        for surf in &self.surfaces {
            if surf.pos().z() < first_surf_z {
                first_surf_z = surf.pos().z();
            }
        }

        let wavelength = 0.000532_f32;
        // Generate a ray fan with diameter equal to the maximum diameter of the system
        let rays = ray_tracing::rays::Ray::fan(3, max_diam / 2.0, PI / 2.0, first_surf_z, 0.0);

        let results = ray_tracing::ray_trace(&self.surfaces, rays, wavelength);

        // Loop over results and remove rays that did not result in an Error
        let sanitized: Vec<Vec<ray_tracing::rays::Ray>> = results
            .iter()
            .map(|surf_results| {
                surf_results
                    .iter()
                    .filter_map(|res| match res {
                        Ok(ray) => Some(ray.clone()),
                        Err(_) => None,
                    })
                    .collect()
            })
            .collect();


        serde_wasm_bindgen::to_value(&sanitized).unwrap()
    }

    /// Insert a refracting circular conic surface into the optical system.
    pub fn pushSurfRefrCircConic(&mut self, axial_pos: f32, diam: f32, n: f32, roc: f32, k: f32) {
        let surf = surfaces::Surface::new_refr_circ_conic(axial_pos, diam, n, roc, k);
        self.surfaces.push(surf);
    }

    /// Insert a refracting circular flat surface into the optical system.
    pub fn pushSurfRefrCircFlat(&mut self, axial_pos: f32, diam: f32, n: f32) {
        let surf = surfaces::Surface::new_refr_circ_flat(axial_pos, diam, n);
        self.surfaces.push(surf);
    }

    /// Insert an object or image plane into the optical system.
    pub fn pushSurfObjOrImgPlane(&mut self, axial_pos: f32, diam: f32) {
        let surf = surfaces::Surface::new_obj_or_img_plane(axial_pos, diam);
        self.surfaces.push(surf);
    }

    /// Return point samples from a surface in the optical system in local YZ plane.
    pub fn sampleSurfYZ(&self, surf_idx: usize, num_points: usize) -> JsValue {
        let surf = &self.surfaces[surf_idx];
        let samples = surf.sample_yz(num_points);

        serde_wasm_bindgen::to_value(&samples).unwrap()
    }

    /// Return the number of surfaces in the system.
    pub fn numSurfaces(&self) -> usize {
        self.surfaces.len()
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
        let img_surf = surfaces::Surface::new_obj_or_img_plane(surf1_axial_pos + thickness + efl, diameter);

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
