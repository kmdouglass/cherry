use std::collections::HashMap;

use anyhow::anyhow;
use wasm_bindgen::prelude::*;

use cherry_rs::description::SystemDescription;
use cherry_rs::rays::Ray as RayOld;
use cherry_rs::surface_model::SurfaceModel;
use cherry_rs::trace::trace;
use cherry_rs::{
    ApertureSpec as ApertureSpecOld, FieldSpec as FieldSpecOld, Gap as GapSpecOld,
    PupilSampling as PupilSamplingOld, SurfaceSpec as SurfaceSpecOld,
    SystemBuilder as SystemBuilderOld, SystemModel,
};

#[wasm_bindgen]
#[derive(Debug)]
pub struct WasmSystemModel {
    builder: SystemBuilderOld,
    system_model: Option<SystemModel>,
}

#[wasm_bindgen]
#[allow(non_snake_case)]
impl WasmSystemModel {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmSystemModel {
        let builder = SystemBuilderOld::new();
        let system_model = Option::None;

        WasmSystemModel {
            builder,
            system_model,
        }
    }

    pub fn describe(&self) -> Result<JsValue, JsError> {
        let system_model = match self.system_model {
            Some(ref model) => model,
            None => return Err(JsError::new("System model is not built")),
        };
        let descr = SystemDescription::new(&system_model);
        let descr = serde_wasm_bindgen::to_value(&descr)?;

        Ok(descr)
    }

    pub fn setSurfaces(&mut self, surfaces: JsValue) -> Result<(), JsError> {
        let surfaces: Vec<SurfaceSpecOld> = serde_wasm_bindgen::from_value(surfaces)?;
        self.builder.surfaces(surfaces);
        Ok(())
    }

    pub fn setGaps(&mut self, gaps: JsValue) -> Result<(), JsError> {
        let gaps: Vec<GapSpecOld> = serde_wasm_bindgen::from_value(gaps)?;
        self.builder.gaps(gaps);
        Ok(())
    }

    pub fn setAperture(&mut self, aperture: JsValue) -> Result<(), JsError> {
        let aperture: ApertureSpecOld = serde_wasm_bindgen::from_value(aperture)?;
        self.builder.aperture(aperture);
        Ok(())
    }

    pub fn setFields(&mut self, fields: JsValue) -> Result<(), JsError> {
        let fields: Vec<FieldSpecOld> = serde_wasm_bindgen::from_value(fields)?;
        self.builder.fields(fields);
        Ok(())
    }

    /// Build the system model from the builder's components.
    ///
    /// We don't return the new system model to JS because it would be removed from Rust's memory
    /// management, likely causing memory leaks.
    pub fn build(&mut self) -> Result<(), JsError> {
        self.system_model = Option::Some(
            self.builder
                .build()
                .map_err(|e| JsError::new(&e.to_string()))?,
        );
        Ok(())
    }

    pub fn surfaces(&self) -> Result<JsValue, JsError> {
        let surf_model = self
            .surf_model()
            .map_err(|e| JsError::new(&e.to_string()))?;
        let mut surface_specs: Vec<SurfaceSpecOld> =
            Vec::with_capacity(surf_model.surfaces().len());
        for surface in surf_model.surfaces() {
            surface_specs.push(surface.into());
        }
        Ok(serde_wasm_bindgen::to_value(&surface_specs)?)
    }

    pub fn gaps(&self) -> Result<JsValue, JsError> {
        let surf_model = self
            .surf_model()
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(serde_wasm_bindgen::to_value(surf_model.gaps())?)
    }

    pub fn aperture(&self) -> Result<JsValue, JsError> {
        let system_model = match self.system_model {
            Some(ref model) => model,
            None => return Err(JsError::new("System model is not built")),
        };
        Ok(serde_wasm_bindgen::to_value(system_model.aperture_spec())?)
    }

    pub fn traceChiefAndMarginalRays(&self) -> Result<JsValue, JsError> {
        let system_model = match self.system_model {
            Some(ref model) => model,
            None => return Err(JsError::new("System model is not built")),
        };

        let surf_model = self
            .surf_model()
            .map_err(|e| JsError::new(&e.to_string()))?;

        let rays = system_model
            .rays(Some(PupilSamplingOld::ChiefMarginalRays))
            .map_err(|e| JsError::new(&e.to_string()))?;

        let results = trace(&surf_model.surfaces(), rays);
        let sanitized: Vec<Vec<Option<RayOld>>> = WasmSystemModel::sanitize(results);

        Ok(serde_wasm_bindgen::to_value(&sanitized)?)
    }

    pub fn trace(&self) -> Result<JsValue, JsError> {
        let system_model = match self.system_model {
            Some(ref model) => model,
            None => return Err(JsError::new("System model is not built")),
        };

        let surf_model = self
            .surf_model()
            .map_err(|e| JsError::new(&e.to_string()))?;

        let rays = system_model
            .rays(None)
            .map_err(|e| JsError::new(&e.to_string()))?;

        let results: Vec<Vec<Result<RayOld, anyhow::Error>>> = trace(&surf_model.surfaces(), rays);
        let sanitized: Vec<Vec<Option<RayOld>>> = WasmSystemModel::sanitize(results);

        Ok(serde_wasm_bindgen::to_value(&sanitized)?)
    }

    // Loop over results and remove rays that did not result in an Error
    #[inline]
    fn sanitize(results: Vec<Vec<Result<RayOld, anyhow::Error>>>) -> Vec<Vec<Option<RayOld>>> {
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

    fn surf_model(&self) -> Result<&SurfaceModel, anyhow::Error> {
        let system_model = match self.system_model {
            Some(ref model) => model,
            None => return Err(anyhow!("System model is not built")),
        };
        Ok(system_model.surf_model())
    }
}

//-------------------------------------------------------------------------
// New implementation
//-------------------------------------------------------------------------
mod system;

use cherry_rs_2::{ApertureSpec, FieldSpec, GapSpec, Ray, SurfaceSpec};

use system::{System, SystemBuilder};
use web_sys::console;

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
        let raw_results = system_model.trace_chief_and_marginal_rays();
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
        let raw_results = system_model.trace();
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
