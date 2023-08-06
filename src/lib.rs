use wasm_bindgen::prelude::*;

mod math;
mod ray_tracing;
mod rendering;

use ray_tracing::sequential_model::{Gap, SequentialModel, SurfaceSpec};
use ray_tracing::SystemModel;

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
        let surface: SurfaceSpec = serde_wasm_bindgen::from_value(surface)?;
        let gap: Gap = serde_wasm_bindgen::from_value(gap)?;
        self
            .seq_model_mut()
            .insert_surface_and_gap(idx, surface, gap)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(())
    }

    pub fn removeSurfaceAndGap(&mut self, idx: usize) -> Result<(), JsError> {
        self
            .seq_model_mut()
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

    fn seq_model(&self) -> &SequentialModel {
        self.system_model.seq_model()
    }

    fn seq_model_mut(&mut self) -> &mut SequentialModel {
        self.system_model.seq_model_mut()
    }
}
