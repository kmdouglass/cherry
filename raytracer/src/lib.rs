mod math;
mod ray_tracing;
pub(crate) mod test_cases;

use std::f32::consts::PI;
use std::sync::atomic::{AtomicUsize, Ordering};

use wasm_bindgen::prelude::*;

use ray_tracing::description::SystemDescription;
use ray_tracing::surface_model::SurfaceModel;
use ray_tracing::{ApertureSpec, Gap, SurfaceSpec, SystemBuilder, SystemModel};

static COUNTER: AtomicUsize = AtomicUsize::new(1);

/// Returns new unique IDs.
fn get_id() -> usize {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct WasmSystemModel {
    builder: SystemBuilder,
    system_model: SystemModel,
}

#[wasm_bindgen]
#[allow(non_snake_case)]
impl WasmSystemModel {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmSystemModel {
        let builder = SystemBuilder::new();
        let system_model = SystemModel::old();

        WasmSystemModel {
            builder,
            system_model,
        }
    }

    pub fn describe(&self) -> Result<JsValue, JsError> {
        let descr = SystemDescription::new(&self.system_model);
        let descr = serde_wasm_bindgen::to_value(&descr)?;

        Ok(descr)
    }

    pub fn setSurfaces(&mut self, surfaces: JsValue) -> Result<(), JsError> {
        let surfaces: Vec<SurfaceSpec> = serde_wasm_bindgen::from_value(surfaces)?;
        self.builder.surfaces(surfaces);
        Ok(())
    }

    pub fn setGaps(&mut self, gaps: JsValue) -> Result<(), JsError> {
        let gaps: Vec<Gap> = serde_wasm_bindgen::from_value(gaps)?;
        self.builder.gaps(gaps);
        Ok(())
    }

    pub fn setApertureV2(&mut self, aperture: JsValue) -> Result<(), JsError> {
        // TODO Rename this once the old setAperture is removed
        let aperture: ApertureSpec = serde_wasm_bindgen::from_value(aperture)?;
        self.builder.aperture(aperture);
        Ok(())
    }

    pub fn setFields(&mut self, fields: JsValue) -> Result<(), JsError> {
        let fields: Vec<ray_tracing::FieldSpec> = serde_wasm_bindgen::from_value(fields)?;
        self.builder.fields(fields);
        Ok(())
    }

    /// Build the system model from the builder's components.
    ///
    /// We don't return the new system model to JS because it would be removed from Rust's memory
    /// management, likely causing memory leaks.
    pub fn build(&mut self) -> Result<(), JsError> {
        self.system_model = self
            .builder
            .build()
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(())
    }

    pub fn surfaces(&self) -> JsValue {
        let mut surface_specs: Vec<SurfaceSpec> =
            Vec::with_capacity(self.surf_model().surfaces().len());
        for surface in self.surf_model().surfaces() {
            surface_specs.push(surface.into());
        }
        serde_wasm_bindgen::to_value(&surface_specs).unwrap()
    }

    pub fn gaps(&self) -> JsValue {
        serde_wasm_bindgen::to_value(self.surf_model().gaps()).unwrap()
    }

    pub fn aperture(&self) -> JsValue {
        serde_wasm_bindgen::to_value(self.system_model.aperture_spec()).unwrap()
    }

    pub fn rayTrace(&self) -> Result<JsValue, JsError> {
        let wavelength = 0.000532_f32;

        // Generate a ray fan for each field to fill the entrance pupil
        let num_rays = 3;
        let fields = self.system_model.field_specs();
        let mut rays = Vec::with_capacity(num_rays * fields.len());

        for field in fields {
            let ray_fan = self
                .system_model
                .pupil_ray_fan(num_rays, PI / 2.0, field.angle() * PI / 180.0)
                .map_err(|e| JsError::new(&e.to_string()))?;

            rays.extend(ray_fan);
        }

        let results = ray_tracing::trace::trace(&self.surf_model().surfaces(), rays, wavelength);

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

    fn surf_model(&self) -> &SurfaceModel {
        self.system_model.surf_model()
    }

    fn surf_model_mut(&mut self) -> &mut SurfaceModel {
        self.system_model.surf_model_mut()
    }
}
