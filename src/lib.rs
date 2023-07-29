use wasm_bindgen::prelude::*;

mod math;
mod ray_tracing;
mod rendering;

use ray_tracing::SystemModel;

#[wasm_bindgen]
#[derive(Debug)]
pub enum Mode {
    Sequential,
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
        let mode = Mode::Sequential;

        WasmSystemModel { system_model, mode }
    }
}
