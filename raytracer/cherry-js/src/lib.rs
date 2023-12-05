use std::f32::consts::PI;

use anyhow::anyhow;
use wasm_bindgen::prelude::*;

use cherry_rs::description::SystemDescription;
use cherry_rs::surface_model::SurfaceModel;
use cherry_rs::{ApertureSpec, FieldSpec, Gap, SurfaceSpec, SystemBuilder, SystemModel};



#[wasm_bindgen]
#[derive(Debug)]
pub struct WasmSystemModel {
    builder: SystemBuilder,
    system_model: Option<SystemModel>,
}

#[wasm_bindgen]
#[allow(non_snake_case)]
impl WasmSystemModel {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmSystemModel {
        let builder = SystemBuilder::new();
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
        let surfaces: Vec<SurfaceSpec> = serde_wasm_bindgen::from_value(surfaces)?;
        self.builder.surfaces(surfaces);
        Ok(())
    }

    pub fn setGaps(&mut self, gaps: JsValue) -> Result<(), JsError> {
        let gaps: Vec<Gap> = serde_wasm_bindgen::from_value(gaps)?;
        self.builder.gaps(gaps);
        Ok(())
    }

    pub fn setAperture(&mut self, aperture: JsValue) -> Result<(), JsError> {
        let aperture: ApertureSpec = serde_wasm_bindgen::from_value(aperture)?;
        self.builder.aperture(aperture);
        Ok(())
    }

    pub fn setFields(&mut self, fields: JsValue) -> Result<(), JsError> {
        let fields: Vec<FieldSpec> = serde_wasm_bindgen::from_value(fields)?;
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
        let mut surface_specs: Vec<SurfaceSpec> = Vec::with_capacity(surf_model.surfaces().len());
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

    pub fn rayTrace(&self) -> Result<JsValue, JsError> {
        let system_model = match self.system_model {
            Some(ref model) => model,
            None => return Err(JsError::new("System model is not built")),
        };

        let surf_model = self
            .surf_model()
            .map_err(|e| JsError::new(&e.to_string()))?;

        let wavelength = 0.000532_f32;

        // Generate a ray fan for each field to fill the entrance pupil
        let num_rays = 3;
        let fields = system_model.field_specs();
        let mut rays = Vec::with_capacity(num_rays * fields.len());

        for field in fields {
            let ray_fan = system_model
                .pupil_ray_fan(num_rays, PI / 2.0, field.angle().to_radians())
                .map_err(|e| JsError::new(&e.to_string()))?;

            rays.extend(ray_fan);
        }

        let results = cherry_rs::trace::trace(&surf_model.surfaces(), rays, wavelength);

        // Loop over results and remove rays that did not result in an Error
        let sanitized: Vec<Vec<Option<cherry_rs::rays::Ray>>> = results
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
            .collect();

        Ok(serde_wasm_bindgen::to_value(&sanitized)?)
    }

    fn surf_model(&self) -> Result<&SurfaceModel, anyhow::Error> {
        let system_model = match self.system_model {
            Some(ref model) => model,
            None => return Err(anyhow!("System model is not built")),
        };
        Ok(system_model.surf_model())
    }
}
