use crate::gui::{model::SystemSpecs, panels};

/// Floating materials browser window (ri-info only).
pub struct MaterialsWindow;

impl MaterialsWindow {
    /// Show the materials browser window. Returns true if specs were modified.
    pub fn show(
        ctx: &egui::Context,
        open: &mut bool,
        specs: &mut SystemSpecs,
        material_index: &panels::MaterialIndex,
        material_browser: &mut panels::MaterialBrowserState,
    ) -> bool {
        let response = egui::Window::new("Materials")
            .open(open)
            .default_width(400.0)
            .show(ctx, |ui| {
                panels::materials_panel(ui, specs, material_index, material_browser)
            });

        response.and_then(|r| r.inner).unwrap_or(false)
    }
}
