use wasm_bindgen::prelude::*;

mod math;
mod ray_tracing;
mod rendering;

use ray_tracing::SystemModel;
use ray_tracing::sequential_model::SequentialModel;

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
}
