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

// Required placeholder when compiling for WASM; actual entry via
// #[wasm_bindgen(start)]
#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub async fn wasm_start() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).ok();

    use eframe::web_sys;
    use wasm_bindgen::JsCast;

    // SharedArrayBuffer requires cross-origin isolation (COOP + COEP headers).
    // If it is absent, threading cannot work at all — show a clear message
    // rather than panicking silently.
    let shared_array_buffer = js_sys::Reflect::get(
        &js_sys::global(),
        &wasm_bindgen::JsValue::from_str("SharedArrayBuffer"),
    )
    .unwrap_or(wasm_bindgen::JsValue::UNDEFINED);

    if shared_array_buffer.is_undefined() {
        let document = web_sys::window().unwrap().document().unwrap();
        let body = document.body().unwrap();
        let div = document.create_element("div").unwrap();
        div.set_attribute(
            "style",
            "font-family: sans-serif; padding: 2em; color: #c00;",
        )
        .unwrap();
        div.set_text_content(Some(
            "Cherry requires SharedArrayBuffer, which is only available in \
             cross-origin isolated contexts. Please ensure the page is served \
             with the required COOP and COEP headers.",
        ));
        body.append_child(&div).unwrap();
        return;
    }

    let canvas = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("cherry_canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();

    wasm_bindgen_futures::JsFuture::from(wasm_bindgen_rayon::init_thread_pool(
        web_sys::window()
            .unwrap()
            .navigator()
            .hardware_concurrency() as usize,
    ))
    .await
    .expect("failed to initialize rayon thread pool");

    eframe::WebRunner::new()
        .start(
            canvas,
            eframe::WebOptions::default(),
            Box::new(|cc| Ok(Box::new(CherryApp::new(cc)))),
        )
        .await
        .expect("eframe failed to start");
}
