use crate::gui::{
    model::{SpecsTab, SystemSpecs},
    panels,
};

/// Floating specs input window with tabbed panels.
pub struct SpecsWindow {
    pub active_tab: SpecsTab,
}

impl Default for SpecsWindow {
    fn default() -> Self {
        Self {
            active_tab: SpecsTab::Surfaces,
        }
    }
}

impl SpecsWindow {
    /// Show the specs window. Returns true if any spec was modified.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        specs: &mut SystemSpecs,
        #[cfg(feature = "ri-info")] material_index: &panels::MaterialIndex,
        #[cfg(feature = "ri-info")] material_browser: &mut panels::MaterialBrowserState,
    ) -> bool {
        let response = egui::Window::new("Specs")
            .open(open)
            .default_width(640.0)
            .min_width(400.0)
            .show(ctx, |ui| {
                // Tab bar
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.active_tab, SpecsTab::Surfaces, "Surfaces");
                    ui.selectable_value(&mut self.active_tab, SpecsTab::Fields, "Fields");
                    ui.selectable_value(&mut self.active_tab, SpecsTab::Aperture, "Aperture");
                    ui.selectable_value(&mut self.active_tab, SpecsTab::Wavelengths, "Wavelengths");
                    #[cfg(feature = "ri-info")]
                    ui.selectable_value(&mut self.active_tab, SpecsTab::Materials, "Materials");
                });
                ui.separator();

                match self.active_tab {
                    SpecsTab::Surfaces => panels::surfaces_panel(ui, specs),
                    SpecsTab::Fields => panels::fields_panel(ui, specs),
                    SpecsTab::Aperture => panels::aperture_panel(ui, specs),
                    SpecsTab::Wavelengths => panels::wavelengths_panel(ui, specs),
                    #[cfg(feature = "ri-info")]
                    SpecsTab::Materials => {
                        panels::materials_panel(ui, specs, material_index, material_browser)
                    }
                }
            });

        response.and_then(|r| r.inner).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{Harness, kittest::Queryable};

    use crate::gui::model::SystemSpecs;

    fn show_specs_window(window: &mut SpecsWindow, specs: &mut SystemSpecs, ctx: &egui::Context) {
        let mut open = true;
        window.show(
            ctx,
            &mut open,
            specs,
            #[cfg(feature = "ri-info")]
            &panels::MaterialIndex::default(),
            #[cfg(feature = "ri-info")]
            &mut panels::MaterialBrowserState::default(),
        );
    }

    #[test]
    fn default_tab_is_surfaces() {
        let mut window = SpecsWindow::default();
        assert_eq!(window.active_tab, SpecsTab::Surfaces);
    }

    #[test]
    fn tab_bar_shows_all_tabs() {
        let mut window = SpecsWindow::default();
        let mut specs = SystemSpecs::default();
        let mut harness = Harness::new(|ctx| {
            show_specs_window(&mut window, &mut specs, ctx);
        });
        harness.step();
        harness.get_by_label("Surfaces");
        harness.get_by_label("Fields");
        harness.get_by_label("Aperture");
        harness.get_by_label("Wavelengths");
    }

    #[test]
    fn clicking_fields_tab_switches_content() {
        let mut window = SpecsWindow::default();
        let mut specs = SystemSpecs::default();
        let mut harness = Harness::new_state(
            |ctx, (window, specs): &mut (SpecsWindow, SystemSpecs)| {
                show_specs_window(window, specs, ctx);
            },
            (window, specs),
        );
        harness.step();

        // Click the Fields tab.
        harness.get_by_label("Fields").click();
        harness.step();

        assert_eq!(harness.state().0.active_tab, SpecsTab::Fields);
    }
}
