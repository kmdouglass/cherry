#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use cherry_rs::gui::CherryApp;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([640.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Cherry",
        native_options,
        Box::new(|cc| Ok(Box::new(CherryApp::new(cc)))),
    )
}

// Required placeholder when compiling for WASM; actual entry via #[wasm_bindgen(start)]
#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub async fn wasm_start() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).ok();

    use eframe::web_sys;
    use wasm_bindgen::JsCast;
    let canvas = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("cherry_canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();

    eframe::WebRunner::new()
        .start(
            canvas,
            eframe::WebOptions::default(),
            Box::new(|cc| Ok(Box::new(CherryApp::new(cc)))),
        )
        .await
        .expect("eframe failed to start");
}
