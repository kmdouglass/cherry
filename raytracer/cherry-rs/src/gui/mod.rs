pub mod model;
mod panels;

use model::{SpecsTab, SystemSpecs};

fn generate_svg() -> String {
    let svg_data = r#"
    <svg width="100" height="100" xmlns="http://www.w3.org/2000/svg">
        <circle cx="50" cy="50" r="40" stroke="black" stroke-width="3" fill="red" />
    </svg>
    "#;
    svg_data.to_string()
}

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
}

impl Default for CherryApp {
    fn default() -> Self {
        Self {
            specs: SystemSpecs::default(),
            active_specs_tab: SpecsTab::Surfaces,
            show_summary: false,
            specs_dirty: true,
            error_message: None,
        }
    }
}

impl CherryApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }
}

impl eframe::App for CherryApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top menu bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
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
                    ui.selectable_value(
                        &mut self.active_specs_tab,
                        SpecsTab::Fields,
                        "Fields",
                    );
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
                });
                ui.separator();

                // Tab content
                let tab_changed = match self.active_specs_tab {
                    SpecsTab::Surfaces => panels::surfaces_panel(ui, &mut self.specs),
                    SpecsTab::Fields => panels::fields_panel(ui, &mut self.specs),
                    SpecsTab::Aperture => panels::aperture_panel(ui, &mut self.specs),
                    SpecsTab::Wavelengths => panels::wavelengths_panel(ui, &mut self.specs),
                };

                if tab_changed {
                    self.specs_dirty = true;
                }
            });

        // Central panel (placeholder for future analysis views)
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Cherry Ray Tracer");

            if let Some(err) = &self.error_message {
                ui.colored_label(egui::Color32::RED, err);
            }

            ui.label("Analysis views will appear here.");
        });

        // Cross Section window (placeholder SVG)
        egui::Window::new("Cross Section").show(ctx, |ui| {
            let svg_data = generate_svg();
            ui.add(egui::Image::from_bytes(
                "bytes://cross_section.svg",
                svg_data.into_bytes(),
            ));
        });
    }
}
