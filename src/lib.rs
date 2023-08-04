use wasm_bindgen::prelude::*;

mod math;
mod ray_tracing;
mod rendering;

use ray_tracing::sequential_model::{Gap, SequentialModel, SurfaceSpec};
use ray_tracing::SystemModel;

#[derive(Debug)]
pub enum Mode {
    Sequential(SequentialModel),
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct WasmSystemModel {
    system_model: SystemModel,
    mode: Mode,
}

#[wasm_bindgen]
#[allow(non_snake_case)]
impl WasmSystemModel {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmSystemModel {
        let system_model = SystemModel::new();
        let sequential_model: SequentialModel = SequentialModel::from(&system_model);
        let mode = Mode::Sequential(sequential_model);

        WasmSystemModel { system_model, mode }
    }

    pub fn insertSurfaceAndGap(
        &mut self,
        idx: usize,
        surface: JsValue,
        gap: JsValue,
    ) -> Result<(), JsError> {
        match self.mode {
            Mode::Sequential(ref mut model) => {
                let surface: SurfaceSpec = serde_wasm_bindgen::from_value(surface)?;
                let gap: Gap = serde_wasm_bindgen::from_value(gap)?;
                model
                    .insert_surface_and_gap(idx, surface, gap)
                    .map_err(|e| JsError::new(&e.to_string()))?;
                Ok(())
            }
            _ => Err(JsError::new(
                "Cannot add surface when the model is not in Sequential mode.",
            )),
        }
    }

    pub fn removeSurfaceAndGap(&mut self, idx: usize) -> Result<(), JsError> {
        match self.mode {
            Mode::Sequential(ref mut model) => {
                model
                    .remove_surface_and_gap(idx)
                    .map_err(|e| JsError::new(&e.to_string()))?;
                Ok(())
            }
            _ => Err(JsError::new(
                "Cannot remove surface when the model is not in Sequential mode.",
            )),
        }
    }

    pub fn surfaces(&self) -> JsValue {
        match self.mode {
            Mode::Sequential(ref model) => {
                let mut surface_specs: Vec<SurfaceSpec> = Vec::with_capacity(model.surfaces().len());
                for surface in model.surfaces() {
                    surface_specs.push(surface.into());
                }
                serde_wasm_bindgen::to_value(&surface_specs).unwrap()
            }
        }
    }

    pub fn gaps(&self) -> JsValue {
        match self.mode {
            Mode::Sequential(ref model) => {
                serde_wasm_bindgen::to_value(&model.gaps()).unwrap()
            }
        }
    }
}
