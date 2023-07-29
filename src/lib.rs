use wasm_bindgen::prelude::*;

mod math;
mod ray_tracing;
mod rendering;

// Old interface
#[wasm_bindgen]
#[derive(Debug)]
pub struct SystemModel {}

#[wasm_bindgen]
#[allow(non_snake_case)]
impl SystemModel {
    #[wasm_bindgen(constructor)]
    pub fn new() -> SystemModel {
        SystemModel {}
    }
}
