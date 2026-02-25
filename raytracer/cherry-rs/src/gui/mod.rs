mod convert;
mod examples;
pub mod model;
mod panels;

#[cfg(feature = "ri-info")]
use std::collections::HashMap;
#[cfg(feature = "ri-info")]
use std::rc::Rc;

use crate::{ParaxialView, SequentialModel, TraceResultsCollection, ray_trace_3d_view};
use model::{SpecsTab, SystemSpecs};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct CherryApp {
    specs: SystemSpecs,

    #[serde(skip)]
    active_specs_tab: SpecsTab,
    #[serde(skip)]
    show_summary: bool,
    #[serde(skip)]
    specs_dirty: bool,
    #[serde(skip)]
    error_message: Option<String>,
    #[serde(skip)]
    sequential_model: Option<SequentialModel>,
    #[serde(skip)]
    paraxial_view: Option<ParaxialView>,
    #[serde(skip)]
    trace_results: Option<TraceResultsCollection>,

    #[cfg(feature = "ri-info")]
    #[serde(skip)]
    materials: HashMap<String, Rc<lib_ria::Material>>,
    #[cfg(feature = "ri-info")]
    #[serde(skip)]
    material_index: panels::MaterialIndex,
    #[cfg(feature = "ri-info")]
    #[serde(skip)]
    material_browser: panels::MaterialBrowserState,
}

impl Default for CherryApp {
    fn default() -> Self {
        Self {
            specs: SystemSpecs::default(),
            active_specs_tab: SpecsTab::Surfaces,
            show_summary: false,
            specs_dirty: true,
            error_message: None,
            sequential_model: None,
            paraxial_view: None,
            trace_results: None,
            #[cfg(feature = "ri-info")]
            materials: HashMap::new(),
            #[cfg(feature = "ri-info")]
            material_index: panels::MaterialIndex::default(),
            #[cfg(feature = "ri-info")]
            material_browser: panels::MaterialBrowserState::default(),
        }
    }
}

#[cfg(feature = "ri-info")]
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

impl CherryApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let mut app: Self = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };

        #[cfg(feature = "ri-info")]
        {
            match load_material_store() {
                Ok(materials) => {
                    let index =
                        panels::MaterialIndex::build_from_keys(materials.keys());
                    app.material_index = index;
                    app.materials = materials;
                }
                Err(e) => {
                    log::error!("Failed to load material database: {e}");
                }
            }
        }

        app
    }

    /// Recompute the sequential model, paraxial view, and ray trace from
    /// current specs.
    fn recompute(&mut self) {
        #[cfg(feature = "ri-info")]
        let parsed = convert::convert_specs(&self.specs, &self.materials);
        #[cfg(not(feature = "ri-info"))]
        let parsed = convert::convert_specs(&self.specs);

        let parsed = match parsed {
            Ok(p) => p,
            Err(e) => {
                self.error_message = Some(format!("Specs error: {e}"));
                self.sequential_model = None;
                self.paraxial_view = None;
                self.trace_results = None;
                return;
            }
        };

        let seq =
            match SequentialModel::new(&parsed.gaps, &parsed.surfaces, &parsed.wavelengths) {
                Ok(s) => s,
                Err(e) => {
                    self.error_message = Some(format!("Model error: {e}"));
                    self.sequential_model = None;
                    self.paraxial_view = None;
                    self.trace_results = None;
                    return;
                }
            };

        let pv = match ParaxialView::new(&seq, &parsed.fields, false) {
            Ok(p) => p,
            Err(e) => {
                self.error_message = Some(format!("Paraxial error: {e}"));
                self.sequential_model = Some(seq);
                self.paraxial_view = None;
                self.trace_results = None;
                return;
            }
        };

        // Ray trace (best-effort; don't block paraxial results on trace
        // failure)
        let trace = match ray_trace_3d_view(
            &parsed.aperture,
            &parsed.fields,
            &seq,
            &pv,
            None,
        ) {
            Ok(t) => Some(t),
            Err(e) => {
                log::warn!("Ray trace failed: {e}");
                None
            }
        };

        self.error_message = None;
        self.sequential_model = Some(seq);
        self.paraxial_view = Some(pv);
        self.trace_results = trace;
    }

    /// Replace the current specs with new ones and trigger recompute.
    fn load_specs(&mut self, specs: SystemSpecs) {
        self.specs = specs;
        self.specs_dirty = true;
    }

    fn save_to_file(&self) {
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

    fn open_from_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Open System")
            .add_filter("JSON", &["json"])
            .pick_file()
        {
            let contents = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    self.error_message = Some(format!("Failed to read file: {e}"));
                    return;
                }
            };
            match serde_json::from_str::<SystemSpecs>(&contents) {
                Ok(specs) => self.load_specs(specs),
                Err(e) => {
                    self.error_message = Some(format!("Failed to parse file: {e}"));
                }
            }
        }
    }
}

impl eframe::App for CherryApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Recompute if specs changed
        if self.specs_dirty {
            self.recompute();
            self.specs_dirty = false;
        }

        // Top menu bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open...").clicked() {
                        ui.close();
                        self.open_from_file();
                    }
                    if ui.button("Save...").clicked() {
                        ui.close();
                        self.save_to_file();
                    }
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

                ui.menu_button("Results", |ui| {
                    if ui.button("Summary").clicked() {
                        self.show_summary = !self.show_summary;
                        ui.close();
                    }
                });

                ui.add_space(16.0);
                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        // Left side panel for specs input
        egui::SidePanel::left("specs_panel")
            .default_width(600.0)
            .min_width(400.0)
            .show(ctx, |ui| {
                // Tab bar
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut self.active_specs_tab,
                        SpecsTab::Surfaces,
                        "Surfaces",
                    );
                    ui.selectable_value(&mut self.active_specs_tab, SpecsTab::Fields, "Fields");
                    ui.selectable_value(
                        &mut self.active_specs_tab,
                        SpecsTab::Aperture,
                        "Aperture",
                    );
                    ui.selectable_value(
                        &mut self.active_specs_tab,
                        SpecsTab::Wavelengths,
                        "Wavelengths",
                    );
                    #[cfg(feature = "ri-info")]
                    ui.selectable_value(
                        &mut self.active_specs_tab,
                        SpecsTab::Materials,
                        "Materials",
                    );
                });
                ui.separator();

                // Tab content
                let tab_changed = match self.active_specs_tab {
                    SpecsTab::Surfaces => panels::surfaces_panel(ui, &mut self.specs),
                    SpecsTab::Fields => panels::fields_panel(ui, &mut self.specs),
                    SpecsTab::Aperture => panels::aperture_panel(ui, &mut self.specs),
                    SpecsTab::Wavelengths => panels::wavelengths_panel(ui, &mut self.specs),
                    #[cfg(feature = "ri-info")]
                    SpecsTab::Materials => panels::materials_panel(
                        ui,
                        &mut self.specs,
                        &self.material_index,
                        &mut self.material_browser,
                    ),
                };

                if tab_changed {
                    self.specs_dirty = true;
                }
            });

        // Central panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Cherry Ray Tracer");

            if let Some(err) = &self.error_message {
                ui.colored_label(egui::Color32::RED, err);
            }

            ui.label("Analysis views will appear here.");
        });

        // Paraxial summary window
        if self.show_summary {
            if let (Some(seq), Some(pv)) = (&self.sequential_model, &self.paraxial_view) {
                let mut open = true;
                panels::summary::summary_window(ctx, &mut open, seq, pv);
                if !open {
                    self.show_summary = false;
                }
            }
        }
    }
}
