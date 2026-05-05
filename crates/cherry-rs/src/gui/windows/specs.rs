use crate::gui::{
    model::{SolveParameter, SolvePopupState, SolveSpec, SpecsTab, SystemSpecs},
    panels,
    result_package::{ResultPackage, SolvedValues},
};

/// Floating specs input window with tabbed panels.
pub struct SpecsWindow {
    pub active_tab: SpecsTab,
    solve_popup: Option<SolvePopupState>,
}

impl Default for SpecsWindow {
    fn default() -> Self {
        Self {
            active_tab: SpecsTab::Surfaces,
            solve_popup: None,
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
        result: Option<&ResultPackage>,
    ) -> bool {
        let response = egui::Window::new("Specs")
            .open(open)
            .default_width(640.0)
            .show(ctx, |ui| {
                // Tab bar
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.active_tab, SpecsTab::Surfaces, "Surfaces");
                    ui.selectable_value(&mut self.active_tab, SpecsTab::Fields, "Fields");
                    ui.selectable_value(&mut self.active_tab, SpecsTab::Aperture, "Aperture");
                    ui.selectable_value(&mut self.active_tab, SpecsTab::Wavelengths, "Wavelengths");
                });
                ui.separator();

                match self.active_tab {
                    SpecsTab::Surfaces => {
                        let solved_values = result.map(|r| &r.solved_values);
                        let panel_changed =
                            panels::surfaces_panel(ui, specs, solved_values, &mut self.solve_popup);
                        let popup_committed = self.show_solve_popup(ctx, specs, solved_values);
                        panel_changed || popup_committed
                    }
                    SpecsTab::Fields => panels::fields_panel(ui, specs),
                    SpecsTab::Aperture => panels::aperture_panel(ui, specs),
                    SpecsTab::Wavelengths => panels::wavelengths_panel(ui, specs),
                }
            });

        response.and_then(|r| r.inner).unwrap_or(false)
    }

    fn show_solve_popup(
        &mut self,
        ctx: &egui::Context,
        specs: &mut SystemSpecs,
        solved_values: Option<&SolvedValues>,
    ) -> bool {
        let mut mutated = false;
        let Some(state) = &mut self.solve_popup else {
            return false;
        };

        let wavelength_count = specs.wavelengths.len();
        let surface_index = state.surface_index;
        let parameter = state.parameter;

        let type_options: &[&str] = match parameter {
            SolveParameter::Thickness => &["None", "Marginal ray height"],
            SolveParameter::RadiusOfCurvature => &["None", "F/#"],
        };

        let mut open = true;
        let mut commit = false;
        let mut remove = false;

        egui::Window::new(format!("Solve — surface {surface_index} — {parameter}"))
            .id(egui::Id::new("solve_popup"))
            .resizable(false)
            .collapsible(false)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label("Solve type:");
                egui::ComboBox::from_id_salt("solve_type")
                    .selected_text(state.type_str.as_str())
                    .show_ui(ui, |ui| {
                        for &opt in type_options {
                            ui.selectable_value(&mut state.type_str, opt.to_owned(), opt);
                        }
                    });

                if state.type_str != "None" {
                    if state.is_paraxial() {
                        ui.weak("Paraxial solve");
                    }

                    let target_label = match state.type_str.as_str() {
                        "Marginal ray height" => "Target height (mm):",
                        "F/#" => "Target F/#:",
                        _ => "Target:",
                    };
                    ui.label(target_label);
                    ui.text_edit_singleline(&mut state.target);
                    if let Some(err) = &state.error {
                        ui.colored_label(egui::Color32::from_rgb(200, 80, 80), err);
                    }

                    if wavelength_count > 1 {
                        ui.label("Wavelength:");
                        let selected_wl = specs
                            .wavelengths
                            .get(state.wavelength_id)
                            .map(|wl| format!("{wl} μm ({})", state.wavelength_id))
                            .unwrap_or_else(|| format!("{}", state.wavelength_id));
                        egui::ComboBox::from_id_salt("solve_wl")
                            .selected_text(selected_wl)
                            .show_ui(ui, |ui| {
                                for (i, wl) in specs.wavelengths.iter().enumerate() {
                                    ui.selectable_value(
                                        &mut state.wavelength_id,
                                        i,
                                        format!("{wl} μm ({i})"),
                                    );
                                }
                            });
                    }
                }

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("OK").clicked() {
                        commit = true;
                    }
                    let has_active = specs.solve_for(surface_index, parameter).is_some();
                    if ui
                        .add_enabled(has_active, egui::Button::new("Remove"))
                        .clicked()
                    {
                        remove = true;
                    }
                });
            });

        if remove {
            Self::write_back_solved_value(specs, surface_index, parameter, solved_values);
            specs
                .solves
                .retain(|s| !(s.surface_index() == surface_index && s.parameter() == parameter));
            self.solve_popup = None;
            return true;
        }

        if commit {
            if state.type_str == "None" {
                Self::write_back_solved_value(specs, surface_index, parameter, solved_values);
                specs.solves.retain(|s| {
                    !(s.surface_index() == surface_index && s.parameter() == parameter)
                });
                self.solve_popup = None;
                return true;
            }
            // Borrow state mutably for validation — clone what we need first.
            let type_str = state.type_str.clone();
            let target = state.target.clone();
            let wavelength_id = state.wavelength_id;
            if let Some(spec) =
                Self::build_solve_spec_from(&type_str, &target, wavelength_id, surface_index, state)
            {
                specs.solves.retain(|s| {
                    !(s.surface_index() == surface_index && s.parameter() == parameter)
                });
                specs.solves.push(spec);
                self.solve_popup = None;
                mutated = true;
            }
        }

        if !open {
            self.solve_popup = None;
        }

        mutated
    }

    fn write_back_solved_value(
        specs: &mut SystemSpecs,
        surface_index: usize,
        parameter: SolveParameter,
        solved_values: Option<&SolvedValues>,
    ) {
        let Some(sv) = solved_values else { return };
        match parameter {
            SolveParameter::Thickness => {
                if let Some(&val) = sv.gap_thicknesses.get(&surface_index)
                    && surface_index < specs.surfaces.len()
                {
                    specs.surfaces[surface_index].thickness = format!("{val:.4}");
                }
            }
            SolveParameter::RadiusOfCurvature => {
                if let Some(&val) = sv.surface_rocs.get(&surface_index)
                    && surface_index < specs.surfaces.len()
                {
                    specs.surfaces[surface_index].radius_of_curvature = format!("{val:.4}");
                }
            }
        }
    }

    fn build_solve_spec_from(
        type_str: &str,
        target: &str,
        wavelength_id: usize,
        surface_index: usize,
        state: &mut SolvePopupState,
    ) -> Option<SolveSpec> {
        match type_str {
            "Marginal ray height" => match target.trim().parse::<f64>() {
                Ok(h) if h.is_finite() => Some(SolveSpec::MarginalRayHeight {
                    gap_index: surface_index,
                    target_height: h,
                    wavelength_id,
                }),
                _ => {
                    state.error = Some("Target height must be a finite number.".to_owned());
                    None
                }
            },
            "F/#" => match target.trim().parse::<f64>() {
                Ok(f) if f.is_finite() && f > 0.0 => Some(SolveSpec::FNumber {
                    surface_index,
                    target_fno: f,
                    wavelength_id,
                }),
                _ => {
                    state.error = Some("Target F/# must be a positive number.".to_owned());
                    None
                }
            },
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{Harness, kittest::Queryable};

    use crate::gui::model::SystemSpecs;

    fn show_specs_window(window: &mut SpecsWindow, specs: &mut SystemSpecs, ctx: &egui::Context) {
        let mut open = true;
        window.show(ctx, &mut open, specs, None);
    }

    #[test]
    fn default_tab_is_surfaces() {
        let window = SpecsWindow::default();
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
        let window = SpecsWindow::default();
        let specs = SystemSpecs::default();
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
