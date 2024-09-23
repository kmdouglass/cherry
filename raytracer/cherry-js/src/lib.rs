mod system;

use std::collections::HashMap;

use anyhow::anyhow;
use wasm_bindgen::prelude::*;

use cherry_rs::{ApertureSpec, FieldSpec, GapSpec, Ray, SurfaceSpec};

use system::{System, SystemBuilder};

#[wasm_bindgen]
#[derive(Debug)]
pub struct OpticalSystem {
    builder: SystemBuilder,
    system_model: Option<System>,
}

#[wasm_bindgen]
#[allow(non_snake_case)]
impl OpticalSystem {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let builder = SystemBuilder::new();
        let system_model = Option::None;

        Self {
            builder,
            system_model,
        }
    }

    pub fn describe(&self) -> Result<JsValue, JsError> {
        let system_model = match self.system_model {
            Some(ref model) => model,
            None => return Err(JsError::new("System model is not built")),
        };

        let descr = system_model.describe();
        let descr = serde_wasm_bindgen::to_value(&descr)?;

        Ok(descr)
    }

    pub fn setSurfaces(&mut self, surfaces: JsValue) -> Result<(), JsError> {
        let surfaces: Vec<SurfaceSpec> = serde_wasm_bindgen::from_value(surfaces)?;
        self.builder.surface_specs(surfaces);
        Ok(())
    }

    pub fn setGaps(&mut self, gaps: JsValue) -> Result<(), JsError> {
        let gaps: Vec<GapSpec> = serde_wasm_bindgen::from_value(gaps)?;
        self.builder.gap_specs(gaps);
        Ok(())
    }

    pub fn setAperture(&mut self, aperture: JsValue) -> Result<(), JsError> {
        let aperture: ApertureSpec = serde_wasm_bindgen::from_value(aperture)?;
        self.builder.aperture_spec(aperture);
        Ok(())
    }

    pub fn setFields(&mut self, fields: JsValue) -> Result<(), JsError> {
        let fields: Vec<FieldSpec> = serde_wasm_bindgen::from_value(fields)?;
        for field in fields.iter() {
            field.validate().map_err(|e| JsError::new(&e.to_string()))?;
        }
        self.builder.field_specs(fields);
        Ok(())
    }

    pub fn setWavelengths(&mut self, wavelengths: JsValue) -> Result<(), JsError> {
        let wavelengths: Vec<f64> = serde_wasm_bindgen::from_value(wavelengths)?;
        self.builder.wavelengths(wavelengths);
        Ok(())
    }

    /// Build the system from the builder's components.
    ///
    /// We don't return the new system model to JS because it would be removed from Rust's memory
    /// management and cause memory leaks.
    pub fn build(&mut self) -> Result<(), JsError> {
        self.system_model = Option::Some(
            self.builder
                .build()
                .map_err(|e| JsError::new(&e.to_string()))?,
        );
        Ok(())
    }

    pub fn traceChiefAndMarginalRays(&self) -> Result<JsValue, JsError> {
        let system_model = match self.system_model {
            Some(ref model) => model,
            None => return Err(JsError::new("System model is not built")),
        };

        let mut results = HashMap::new();
        let raw_results = system_model.trace_chief_and_marginal_rays().map_err(|e| {
            JsError::new(&format!("Failed to trace chief and marginal rays: {}", e))
        })?;
        for (id, trace_results) in raw_results {
            let sanitized = Self::sanitize(trace_results);
            results.insert(id, sanitized);
        }

        Ok(serde_wasm_bindgen::to_value(&results)?)
    }

    pub fn trace(&self) -> Result<JsValue, JsError> {
        let system_model = match self.system_model {
            Some(ref model) => model,
            None => return Err(JsError::new("System model is not built")),
        };

        let mut results = HashMap::new();
        let raw_results = system_model
            .trace()
            .map_err(|e| JsError::new(&format!("Failed to trace rays: {}", e)))?;
        for (id, trace_results) in raw_results {
            let sanitized = Self::sanitize(trace_results);
            results.insert(id, sanitized);
        }

        Ok(serde_wasm_bindgen::to_value(&results)?)
    }

    // Loop over results and remove rays that did not result in an Error
    #[inline]
    fn sanitize(results: Vec<Vec<Result<Ray, anyhow::Error>>>) -> Vec<Vec<Option<Ray>>> {
        results
            .iter()
            .map(|surf_results| {
                surf_results
                    .iter()
                    .map(|res| match res {
                        Ok(ray) => Some(ray.clone()),
                        Err(_) => None,
                    })
                    .collect()
            })
            .collect()
    }
}
