mod math;
mod ray_tracing;

use std::f32::consts::PI;
use std::sync::atomic::{AtomicUsize, Ordering};

use wasm_bindgen::prelude::*;

use ray_tracing::sequential_model::{SequentialModel, SurfaceSpec};
use ray_tracing::{ApertureSpec, Gap, SystemModel};

static COUNTER: AtomicUsize = AtomicUsize::new(1);

/// Returns new unique IDs.
fn get_id() -> usize {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct WasmSystemModel {
    system_model: SystemModel,
}

#[wasm_bindgen]
#[allow(non_snake_case)]
impl WasmSystemModel {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmSystemModel {
        let system_model = SystemModel::new();

        WasmSystemModel { system_model }
    }

    pub fn insertSurfaceAndGap(
        &mut self,
        idx: usize,
        surface: JsValue,
        gap: JsValue,
    ) -> Result<(), JsError> {
        let surf_spec: SurfaceSpec = serde_wasm_bindgen::from_value(surface)?;
        let gap: Gap = serde_wasm_bindgen::from_value(gap)?;
        self.system_model
            .insert_surface_and_gap(idx, surf_spec, gap)
            .map_err(|e| JsError::new(&e.to_string()))?;

        Ok(())
    }

    pub fn removeSurfaceAndGap(&mut self, idx: usize) -> Result<(), JsError> {
        self.seq_model_mut()
            .remove_surface_and_gap(idx)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(())
    }

    pub fn surfaces(&self) -> JsValue {
        let mut surface_specs: Vec<SurfaceSpec> =
            Vec::with_capacity(self.seq_model().surfaces().len());
        for surface in self.seq_model().surfaces() {
            surface_specs.push(surface.into());
        }
        serde_wasm_bindgen::to_value(&surface_specs).unwrap()
    }

    pub fn gaps(&self) -> JsValue {
        serde_wasm_bindgen::to_value(self.seq_model().gaps()).unwrap()
    }

    /// Return point samples from a surface in the optical system along the y-z plane.
    pub fn sampleSurfYZ(&self, surf_idx: usize, num_points: usize) -> JsValue {
        let surf = self.seq_model().surfaces()[surf_idx];
        let samples = surf.sample_yz(num_points);

        serde_wasm_bindgen::to_value(&samples).unwrap()
    }

    pub fn aperture(&self) -> JsValue {
        serde_wasm_bindgen::to_value(self.system_model.aperture()).unwrap()
    }

    pub fn setAperture(&mut self, aperture: JsValue) -> Result<(), JsError> {
        let aperture: ApertureSpec = serde_wasm_bindgen::from_value(aperture)?;
        self.system_model
            .set_aperture(aperture)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(())
    }

    pub fn setObjectSpace(&mut self, n: f32, thickness: f32) -> Result<(), JsError> {
        self.system_model
            .set_obj_space(n, thickness)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(())
    }

    pub fn rayTrace(&self) -> Result<JsValue, JsError> {
        let wavelength = 0.000532_f32;

        // Generate a ray fan to fill the entrance pupil
        let num_rays = 5;
        let rays = self
            .system_model
            .pupil_ray_fan(num_rays, PI / 2.0, 0.0)
            .map_err(|e| JsError::new(&e.to_string()))?;

        let results = ray_tracing::trace::trace(&self.seq_model().surfaces(), rays, wavelength);

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

        Ok(serde_wasm_bindgen::to_value(&sanitized).unwrap())
    }

    fn seq_model(&self) -> &SequentialModel {
        self.system_model.seq_model()
    }

    fn seq_model_mut(&mut self) -> &mut SequentialModel {
        self.system_model.seq_model_mut()
    }
}
