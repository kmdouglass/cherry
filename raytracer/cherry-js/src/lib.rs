mod system;

use wasm_bindgen::prelude::*;

use cherry_rs::{ApertureSpec, FieldSpec, GapSpec, SurfaceSpec};

use system::{GapSpecConstantN, GapSpecMaterial, System, SystemBuilder};

/// Specifies the type of gap.
///
/// If `Material`, the gap is specified by a material specification and its dispersion data. If
/// `RefractiveIndex`, the gap is specified directly by a refractive index value.
#[wasm_bindgen]
#[derive(Copy, Clone, Debug)]
pub enum GapMode {
    Material = 0,
    RefractiveIndex = 1,
}

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

    pub fn setGaps(&mut self, gaps: JsValue, mode: GapMode) -> Result<(), JsError> {
        match mode {
            GapMode::Material => {
                let gaps: Vec<GapSpecMaterial> = serde_wasm_bindgen::from_value(gaps)?;
                let gaps: Vec<GapSpec> = gaps.into_iter().map(GapSpec::from).collect();
                self.builder.gap_specs(gaps);
            }
            GapMode::RefractiveIndex => {
                let gaps: Vec<GapSpecConstantN> = serde_wasm_bindgen::from_value(gaps)?;
                let gaps: Vec<GapSpec> = gaps.into_iter().map(GapSpec::from).collect();
                self.builder.gap_specs(gaps);
            }
        }
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

    pub fn traceTangentialRayFan(&self) -> Result<JsValue, JsError> {
        let system_model = match self.system_model {
            Some(ref model) => model,
            None => return Err(JsError::new("System model is not built")),
        };

        let results = system_model.trace_chief_and_marginal_rays().map_err(|e| {
            JsError::new(&format!("Failed to trace tangential ray fan: {}", e))
        })?;

        Ok(serde_wasm_bindgen::to_value(&results)?)
    }

    pub fn trace(&self) -> Result<JsValue, JsError> {
        let system_model = match self.system_model {
            Some(ref model) => model,
            None => return Err(JsError::new("System model is not built")),
        };

        let results = system_model
            .trace()
            .map_err(|e| JsError::new(&format!("Failed to trace rays: {}", e)))?;

        Ok(serde_wasm_bindgen::to_value(&results)?)
    }
}

impl Default for OpticalSystem {
    fn default() -> Self {
        Self::new()
    }
}
