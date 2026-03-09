use std::sync::mpsc::{Receiver, Sender, channel};

#[cfg(feature = "ri-info")]
use std::{collections::HashMap, rc::Rc};

use crate::gui::{
    compute::{ComputeRequest, compute_loop, spawn_compute_thread},
    examples,
    model::SystemSpecs,
    result_package::ResultPackage,
    windows::{
        CrossSectionWindow, ParaxialWindow, SpecsWindow, SpotDiagramWindow, WindowVisibility,
    },
};

#[cfg(feature = "ri-info")]
use crate::gui::panels;

#[cfg(feature = "ri-info")]
use crate::gui::windows::MaterialsWindow;

/// Serialized subset of app state, persisted across sessions.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct AppState {
    specs: SystemSpecs,
    input_id: u64,
    windows: WindowVisibility,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            specs: SystemSpecs::default(),
            input_id: 0,
            windows: WindowVisibility::default(),
        }
    }
}

pub struct CherryApp {
    // Persisted
    specs: SystemSpecs,
    input_id: u64,
    windows: WindowVisibility,

    // Runtime channels
    compute_tx: Sender<ComputeRequest>,
    result_rx: Receiver<ResultPackage>,

    // Latest compute result
    latest_result: Option<ResultPackage>,

    // Window state
    specs_window: SpecsWindow,
    spot_diagram_window: SpotDiagramWindow,
    cross_section_window: CrossSectionWindow,

    // ri-info: material browser data loaded on main thread for UI
    #[cfg(feature = "ri-info")]
    materials: HashMap<String, Rc<lib_ria::Material>>,
    #[cfg(feature = "ri-info")]
    material_index: panels::MaterialIndex,
    #[cfg(feature = "ri-info")]
    material_browser: panels::MaterialBrowserState,

    // WASM: async-loaded pending material index (swapped in during update())
    #[cfg(all(feature = "ri-info", target_arch = "wasm32"))]
    pending_material_index: std::sync::Arc<std::sync::Mutex<Option<panels::MaterialIndex>>>,

    // WASM: pending specs loaded asynchronously from file open dialog
    #[cfg(target_arch = "wasm32")]
    pending_specs: std::sync::Arc<std::sync::Mutex<Option<SystemSpecs>>>,
}

#[cfg(all(feature = "ri-info", not(target_arch = "wasm32")))]
fn load_material_store() -> anyhow::Result<HashMap<String, Rc<lib_ria::Material>>> {
    let filename = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data/rii.db");
    let data = std::fs::read(&filename)
        .map_err(|e| anyhow::anyhow!("Cannot read {}: {e}", filename.display()))?;
    let mut store: lib_ria::Store = bitcode::deserialize(&data)
        .map_err(|e| anyhow::anyhow!("Cannot deserialize material database: {e}"))?;
    let keys: Vec<String> = store.keys().cloned().collect();
    let mut materials = HashMap::with_capacity(keys.len());
    for key in keys {
        if let Some(mat) = store.remove(&key) {
            materials.insert(key, Rc::new(mat));
        }
    }
    Ok(materials)
}

/// Fetch bytes from a URL (WASM only).
#[cfg(target_arch = "wasm32")]
async fn fetch_bytes(url: &str) -> anyhow::Result<Vec<u8>> {
    let resp = gloo_net::http::Request::get(url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    resp.binary().await.map_err(|e| anyhow::anyhow!("{e}"))
}

impl CherryApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let state: AppState = cc
            .storage
            .and_then(|s| eframe::get_value(s, eframe::APP_KEY))
            .unwrap_or_default();

        let (compute_tx, compute_rx) = channel::<ComputeRequest>();
        let (result_tx, result_rx) = channel::<ResultPackage>();

        // On WASM+ri-info, create a one-shot channel for the raw database bytes.
        #[cfg(all(feature = "ri-info", target_arch = "wasm32"))]
        let (materials_bytes_tx, materials_bytes_rx) = std::sync::mpsc::channel::<Vec<u8>>();

        spawn_compute_thread(move || {
            compute_loop(
                compute_rx,
                result_tx,
                #[cfg(all(feature = "ri-info", target_arch = "wasm32"))]
                materials_bytes_rx,
            )
        });

        // Send the initial compute request.
        let initial_id = state.input_id;
        compute_tx
            .send(ComputeRequest {
                id: initial_id,
                specs: state.specs.clone(),
            })
            .ok();

        // Native: load materials synchronously from disk.
        #[cfg(all(feature = "ri-info", not(target_arch = "wasm32")))]
        let (materials, material_index) = match load_material_store() {
            Ok(mats) => {
                let index = panels::MaterialIndex::build_from_keys(mats.keys());
                (mats, index)
            }
            Err(e) => {
                log::error!("Failed to load material database: {e}");
                (HashMap::new(), panels::MaterialIndex::default())
            }
        };

        // WASM: start empty; materials are fetched asynchronously below.
        #[cfg(all(feature = "ri-info", target_arch = "wasm32"))]
        let (materials, material_index) = (HashMap::new(), panels::MaterialIndex::default());

        // WASM: Arc shared with the async loading task.
        #[cfg(all(feature = "ri-info", target_arch = "wasm32"))]
        let pending_material_index =
            std::sync::Arc::new(std::sync::Mutex::new(None::<panels::MaterialIndex>));

        // WASM: Arc shared with the async file-open task.
        #[cfg(target_arch = "wasm32")]
        let pending_specs = std::sync::Arc::new(std::sync::Mutex::new(None::<SystemSpecs>));

        // WASM+ri-info: kick off two-phase async load of the materials database.
        #[cfg(all(feature = "ri-info", target_arch = "wasm32"))]
        {
            let pending_index = std::sync::Arc::clone(&pending_material_index);
            wasm_bindgen_futures::spawn_local(async move {
                // Phase 1: fetch the small index JSON and populate the browser.
                match fetch_bytes("assets/rii-index.json").await {
                    Ok(bytes) => match serde_json::from_slice::<Vec<String>>(&bytes) {
                        Ok(keys) => {
                            let idx = panels::MaterialIndex::build_from_keys(keys.iter());
                            *pending_index.lock().unwrap() = Some(idx);
                        }
                        Err(e) => {
                            log::error!("Failed to parse rii-index.json: {e}")
                        }
                    },
                    Err(e) => {
                        log::error!("Failed to fetch rii-index.json: {e}")
                    }
                }

                // Phase 2: fetch the full database and send bytes to the
                // compute thread for deserialization.
                match fetch_bytes("assets/rii.db").await {
                    Ok(bytes) => {
                        let _ = materials_bytes_tx.send(bytes);
                    }
                    Err(e) => log::error!("Failed to fetch rii.db: {e}"),
                }
            });
        }

        Self {
            specs: state.specs,
            input_id: initial_id,
            windows: state.windows,
            compute_tx,
            result_rx,
            latest_result: None,
            specs_window: SpecsWindow::default(),
            spot_diagram_window: SpotDiagramWindow::default(),
            cross_section_window: CrossSectionWindow::default(),
            #[cfg(feature = "ri-info")]
            materials,
            #[cfg(feature = "ri-info")]
            material_index,
            #[cfg(feature = "ri-info")]
            material_browser: panels::MaterialBrowserState::default(),
            #[cfg(all(feature = "ri-info", target_arch = "wasm32"))]
            pending_material_index,
            #[cfg(target_arch = "wasm32")]
            pending_specs,
        }
    }

    /// Increment the input id and dispatch a new compute request.
    fn bump_input_id(&mut self) {
        self.input_id = self.input_id.wrapping_add(1);
        self.compute_tx
            .send(ComputeRequest {
                id: self.input_id,
                specs: self.specs.clone(),
            })
            .ok();
    }

    fn load_specs(&mut self, specs: SystemSpecs) {
        self.specs = specs;
        self.bump_input_id();
    }

    fn save_to_file(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("Save System")
                .add_filter("JSON", &["json"])
                .save_file()
            {
                let json = match serde_json::to_string_pretty(&self.specs) {
                    Ok(j) => j,
                    Err(e) => {
                        log::error!("Failed to serialize specs: {e}");
                        return;
                    }
                };
                if let Err(e) = std::fs::write(&path, json) {
                    log::error!("Failed to write file: {e}");
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            let specs = self.specs.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let json = match serde_json::to_string_pretty(&specs) {
                    Ok(j) => j,
                    Err(e) => {
                        log::error!("Failed to serialize specs: {e}");
                        return;
                    }
                };
                if let Some(handle) = rfd::AsyncFileDialog::new()
                    .set_title("Save System")
                    .add_filter("JSON", &["json"])
                    .save_file()
                    .await
                {
                    if let Err(e) = handle.write(json.as_bytes()).await {
                        log::error!("Failed to write file: {e}");
                    }
                }
            });
        }
    }

    fn export_cross_section_svg(&self, ctx: &egui::Context) {
        let Some(result) = &self.latest_result else {
            return;
        };
        let Some(cs) = &result.cross_section else {
            return;
        };
        let dark_mode = ctx.style().visuals.dark_mode;
        let Some(svg) = self.cross_section_window.export_svg_string(cs, dark_mode) else {
            return;
        };

        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("Export Cross-Section SVG")
                .add_filter("SVG", &["svg"])
                .save_file()
            {
                if let Err(e) = std::fs::write(&path, svg.as_bytes()) {
                    log::error!("Failed to write SVG: {e}");
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                if let Some(handle) = rfd::AsyncFileDialog::new()
                    .set_title("Export Cross-Section SVG")
                    .add_filter("SVG", &["svg"])
                    .save_file()
                    .await
                {
                    if let Err(e) = handle.write(svg.as_bytes()).await {
                        log::error!("Failed to write SVG: {e}");
                    }
                }
            });
        }
    }

    fn open_from_file(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("Open System")
                .add_filter("JSON", &["json"])
                .pick_file()
            {
                let contents = match std::fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(e) => {
                        log::error!("Failed to read file: {e}");
                        return;
                    }
                };
                match serde_json::from_str::<SystemSpecs>(&contents) {
                    Ok(specs) => self.load_specs(specs),
                    Err(e) => log::error!("Failed to parse file: {e}"),
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            let pending = std::sync::Arc::clone(&self.pending_specs);
            wasm_bindgen_futures::spawn_local(async move {
                if let Some(handle) = rfd::AsyncFileDialog::new()
                    .set_title("Open System")
                    .add_filter("JSON", &["json"])
                    .pick_file()
                    .await
                {
                    let bytes = handle.read().await;
                    match serde_json::from_slice::<SystemSpecs>(&bytes) {
                        Ok(specs) => *pending.lock().unwrap() = Some(specs),
                        Err(e) => log::error!("Failed to parse file: {e}"),
                    }
                }
            });
        }
    }
}

impl eframe::App for CherryApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let state = AppState {
            specs: self.specs.clone(),
            input_id: self.input_id,
            windows: WindowVisibility {
                specs: self.windows.specs,
                materials: self.windows.materials,
                paraxial_summary: self.windows.paraxial_summary,
                spot_diagram: self.windows.spot_diagram,
                cross_section: self.windows.cross_section,
            },
        };
        eframe::set_value(storage, eframe::APP_KEY, &state);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // WASM: swap in the material index once the async load completes.
        #[cfg(all(feature = "ri-info", target_arch = "wasm32"))]
        {
            let maybe_idx = self
                .pending_material_index
                .try_lock()
                .ok()
                .and_then(|mut g| g.take());
            if let Some(idx) = maybe_idx {
                self.material_index = idx;
            }
        }

        // WASM: apply specs loaded asynchronously from the file open dialog.
        #[cfg(target_arch = "wasm32")]
        {
            let maybe_specs = self
                .pending_specs
                .try_lock()
                .ok()
                .and_then(|mut g| g.take());
            if let Some(specs) = maybe_specs {
                self.load_specs(specs);
            }
        }

        // Poll for new results from the compute thread.
        while let Ok(result) = self.result_rx.try_recv() {
            if self
                .latest_result
                .as_ref()
                .map_or(true, |prev| result.id >= prev.id)
            {
                self.latest_result = Some(result);
            }
        }

        let result_ref = self.latest_result.as_ref();
        let is_computing = result_ref.map_or(true, |r| r.id < self.input_id);

        // Top menu bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open\u{2026}").clicked() {
                        ui.close();
                        self.open_from_file();
                    }
                    if ui.button("Save\u{2026}").clicked() {
                        ui.close();
                        self.save_to_file();
                    }
                    ui.separator();
                    let can_export_svg = self
                        .latest_result
                        .as_ref()
                        .and_then(|r| r.cross_section.as_ref())
                        .is_some();
                    ui.add_enabled_ui(can_export_svg, |ui| {
                        if ui.button("Export SVG\u{2026}").clicked() {
                            ui.close();
                            self.export_cross_section_svg(ctx);
                        }
                    });
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.add_space(16.0);

                ui.menu_button("Examples", |ui| {
                    if ui.button("Convexplano Lens").clicked() {
                        self.load_specs(SystemSpecs::default());
                        ui.close();
                    }
                    if ui.button("Petzval Lens").clicked() {
                        self.load_specs(examples::petzval_lens());
                        ui.close();
                    }
                    if ui.button("Concave Mirror").clicked() {
                        self.load_specs(examples::concave_mirror());
                        ui.close();
                    }
                });
                ui.add_space(16.0);

                egui::widgets::global_theme_preference_buttons(ui);

                if is_computing {
                    ui.add_space(8.0);
                    ui.spinner();
                }
            });
        });

        // Right panel: window toggle list
        egui::SidePanel::right("window_list")
            .default_width(160.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.label(egui::RichText::new("Input").strong());
                ui.separator();

                ui.toggle_value(&mut self.windows.specs, "Specs");
                #[cfg(feature = "ri-info")]
                ui.toggle_value(&mut self.windows.materials, "Materials");

                ui.add_space(8.0);
                ui.label(egui::RichText::new("Output").strong());
                ui.separator();

                ui.toggle_value(&mut self.windows.paraxial_summary, "Paraxial Summary");
                ui.toggle_value(&mut self.windows.spot_diagram, "Spot Diagram");
                ui.toggle_value(&mut self.windows.cross_section, "Cross Section");
            });

        // Central panel (placeholder for future views)
        egui::CentralPanel::default().show(ctx, |_ui| {});

        // Floating windows
        if self.windows.specs {
            let changed = self.specs_window.show(
                ctx,
                &mut self.windows.specs,
                &mut self.specs,
                #[cfg(feature = "ri-info")]
                &self.material_index,
                #[cfg(feature = "ri-info")]
                &mut self.material_browser,
            );
            if changed {
                self.bump_input_id();
            }
        }

        #[cfg(feature = "ri-info")]
        if self.windows.materials {
            let changed = MaterialsWindow::show(
                ctx,
                &mut self.windows.materials,
                &mut self.specs,
                &self.material_index,
                &mut self.material_browser,
            );
            if changed {
                self.bump_input_id();
            }
        }

        if self.windows.paraxial_summary {
            ParaxialWindow::show(
                ctx,
                &mut self.windows.paraxial_summary,
                self.latest_result.as_ref(),
                self.input_id,
            );
        }

        if self.windows.spot_diagram {
            self.spot_diagram_window.show(
                ctx,
                &mut self.windows.spot_diagram,
                self.latest_result.as_ref(),
                self.input_id,
            );
        }

        if self.windows.cross_section {
            self.cross_section_window.show(
                ctx,
                &mut self.windows.cross_section,
                self.latest_result.as_ref(),
                self.input_id,
            );
        }

        // Keep repainting while a compute is in flight.
        if is_computing {
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
        }
    }
}
