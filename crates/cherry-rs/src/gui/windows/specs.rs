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
    /// `active_path` is a reborrow of `CherryApp`'s own field (FR-NAV-5 — it
    /// is not part of `SystemSpecs`).
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        specs: &mut SystemSpecs,
        result: Option<&ResultPackage>,
        active_path: &mut usize,
    ) -> bool {
        let response = egui::Window::new("Specs")
            .open(open)
            .default_width(640.0)
            .show(ctx, |ui| {
                // "+ Add Path" is always visible (it's how a second path gets
                // created); the switcher arrows/Remove Path are gated on
                // paths.len() > 1 inside show_path_switcher (FR-NAV-1).
                self.show_path_switcher(ui, specs, active_path);
                ui.separator();

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
                        let panel_changed = panels::surfaces_panel(
                            ui,
                            specs,
                            *active_path,
                            solved_values,
                            &mut self.solve_popup,
                        );
                        let popup_committed =
                            self.show_solve_popup(ctx, specs, *active_path, solved_values);
                        panel_changed || popup_committed
                    }
                    SpecsTab::Fields => panels::fields_panel(ui, specs, *active_path),
                    SpecsTab::Aperture => panels::aperture_panel(ui, specs, *active_path),
                    SpecsTab::Wavelengths => panels::wavelengths_panel(ui, specs, *active_path),
                }
            });

        response.and_then(|r| r.inner).unwrap_or(false)
    }

    /// Path switcher: ◀ Path N: name ▶, + Add Path, Remove Path (FR-NAV-1..7).
    fn show_path_switcher(
        &mut self,
        ui: &mut egui::Ui,
        specs: &mut SystemSpecs,
        active_path: &mut usize,
    ) {
        ui.horizontal(|ui| {
            if specs.paths.len() > 1 {
                if ui.button("\u{25c0}").clicked() && *active_path > 0 {
                    *active_path -= 1;
                }
                ui.label(specs.path_label(*active_path));
                if ui.button("\u{25b6}").clicked() && *active_path + 1 < specs.paths.len() {
                    *active_path += 1;
                }
            } else {
                ui.label(specs.path_label(*active_path));
            }
            if ui.button("+ Add Path").clicked() {
                specs.add_path();
                *active_path = specs.paths.len() - 1;
            }
            if specs.paths.len() > 1 {
                let removable = specs.path_is_removable(*active_path);
                let resp = ui.add_enabled(removable, egui::Button::new("Remove Path"));
                if !removable {
                    let tooltip = if *active_path == 0 {
                        "Path 0 cannot be removed".to_owned()
                    } else if let Some(blocker) = specs.path_removal_blocker(*active_path) {
                        format!("Path {blocker} references this path's surfaces")
                    } else {
                        "This path cannot be removed".to_owned()
                    };
                    resp.on_disabled_hover_text(tooltip);
                } else if resp.clicked() {
                    specs.remove_path(*active_path);
                    let max = specs.paths.len().saturating_sub(1);
                    *active_path = (*active_path).min(max);
                }
            }
        });
    }

    fn show_solve_popup(
        &mut self,
        ctx: &egui::Context,
        specs: &mut SystemSpecs,
        active_path: usize,
        solved_values: Option<&SolvedValues>,
    ) -> bool {
        let mut mutated = false;
        let Some(state) = &mut self.solve_popup else {
            return false;
        };

        let wavelength_count = specs
            .paths
            .get(state.path_id)
            .map(|p| p.wavelengths.len())
            .unwrap_or(0);
        let surface_index = state.surface_index;
        let path_id = state.path_id;
        let parameter = state.parameter;

        let type_options: &[&str] = match parameter {
            SolveParameter::Thickness => &["None", "Marginal ray height"],
            SolveParameter::RadiusOfCurvature => &["None", "F/#"],
        };

        let mut open = true;
        let mut commit = false;
        let mut remove = false;

        egui::Window::new(format!(
            "Solve \u{2014} {parameter} \u{2014} {surface_index}"
        ))
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

                if wavelength_count > 1
                    && let Some(path) = specs.paths.get(path_id)
                {
                    ui.label("Wavelength:");
                    let selected_wl = path
                        .wavelengths
                        .get(state.wavelength_id)
                        .map(|wl| format!("{wl} \u{3bc}m ({})", state.wavelength_id))
                        .unwrap_or_else(|| format!("{}", state.wavelength_id));
                    egui::ComboBox::from_id_salt("solve_wl")
                        .selected_text(selected_wl)
                        .show_ui(ui, |ui| {
                            for (i, wl) in path.wavelengths.iter().enumerate() {
                                ui.selectable_value(
                                    &mut state.wavelength_id,
                                    i,
                                    format!("{wl} \u{3bc}m ({i})"),
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
                let has_active = match parameter {
                    SolveParameter::Thickness => {
                        specs.thickness_solve_for(path_id, surface_index).is_some()
                    }
                    SolveParameter::RadiusOfCurvature => {
                        specs.curvature_solve_for(surface_index).is_some()
                    }
                };
                if ui
                    .add_enabled(has_active, egui::Button::new("Remove"))
                    .clicked()
                {
                    remove = true;
                }
            });
        });

        if remove {
            Self::write_back_solved_value(
                specs,
                active_path,
                surface_index,
                parameter,
                solved_values,
            );
            Self::retain_other_solves(specs, path_id, surface_index, parameter);
            self.solve_popup = None;
            return true;
        }

        if commit {
            if state.type_str == "None" {
                Self::write_back_solved_value(
                    specs,
                    active_path,
                    surface_index,
                    parameter,
                    solved_values,
                );
                Self::retain_other_solves(specs, path_id, surface_index, parameter);
                self.solve_popup = None;
                return true;
            }
            let type_str = state.type_str.clone();
            let target = state.target.clone();
            let wavelength_id = state.wavelength_id;
            if let Some(spec) = Self::build_solve_spec_from(
                &type_str,
                &target,
                wavelength_id,
                surface_index,
                path_id,
                state,
            ) {
                Self::retain_other_solves(specs, path_id, surface_index, parameter);
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

    /// Remove any existing solve matching this (parameter, key) identity —
    /// `Curvature`-kind solves are keyed purely by store index (cross-path,
    /// FR-SOLVE-5); `Thickness`-kind solves are keyed by (path_id,
    /// gap_index).
    fn retain_other_solves(
        specs: &mut SystemSpecs,
        path_id: usize,
        surface_index: usize,
        parameter: SolveParameter,
    ) {
        specs.solves.retain(|s| {
            if s.parameter() != parameter {
                return true;
            }
            match parameter {
                SolveParameter::RadiusOfCurvature => s.surface_index() != surface_index,
                SolveParameter::Thickness => {
                    !(s.path_id() == path_id && s.surface_index() == surface_index)
                }
            }
        });
    }

    fn write_back_solved_value(
        specs: &mut SystemSpecs,
        active_path: usize,
        surface_index: usize,
        parameter: SolveParameter,
        solved_values: Option<&SolvedValues>,
    ) {
        let Some(sv) = solved_values else { return };
        match parameter {
            SolveParameter::Thickness => {
                if let Some(&val) = sv.gap_thicknesses.get(&surface_index)
                    && let Some(path) = specs.paths.get_mut(active_path)
                    && let Some(r) = path.surface_refs.get_mut(surface_index)
                {
                    match r {
                        crate::gui::model::SurfaceRefRow::New(row) => {
                            row.thickness = format!("{val:.4}");
                        }
                        _ => {
                            if let Some(gap) = r.gap_after_mut() {
                                gap.thickness = format!("{val:.4}");
                            }
                        }
                    }
                }
            }
            SolveParameter::RadiusOfCurvature => {
                if let Some(&val) = sv.surface_rocs.get(&surface_index) {
                    let table = specs.store_index_table();
                    for path in &mut specs.paths {
                        for r in &mut path.surface_refs {
                            if let crate::gui::model::SurfaceRefRow::New(row) = r
                                && table.store_index_of(row.id) == Some(surface_index)
                            {
                                row.radius_of_curvature = format!("{val:.4}");
                            }
                        }
                    }
                }
            }
        }
    }

    fn build_solve_spec_from(
        type_str: &str,
        target: &str,
        wavelength_id: usize,
        surface_index: usize,
        path_id: usize,
        state: &mut SolvePopupState,
    ) -> Option<SolveSpec> {
        match type_str {
            "Marginal ray height" => match target.trim().parse::<f64>() {
                Ok(h) if h.is_finite() => Some(SolveSpec::MarginalRayHeight {
                    gap_index: surface_index,
                    target_height: h,
                    wavelength_id,
                    path_id,
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
                    path_id,
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

    fn show_specs_window(
        window: &mut SpecsWindow,
        specs: &mut SystemSpecs,
        active_path: &mut usize,
        ctx: &egui::Context,
    ) {
        let mut open = true;
        window.show(ctx, &mut open, specs, None, active_path);
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
        let mut active_path = 0usize;
        let mut harness = Harness::new(|ctx| {
            show_specs_window(&mut window, &mut specs, &mut active_path, ctx);
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
        let active_path = 0usize;
        let mut harness = Harness::new_state(
            |ctx, (window, specs, active_path): &mut (SpecsWindow, SystemSpecs, usize)| {
                show_specs_window(window, specs, active_path, ctx);
            },
            (window, specs, active_path),
        );
        harness.step();

        harness.get_by_label("Fields").click();
        harness.step();

        assert_eq!(harness.state().0.active_tab, SpecsTab::Fields);
    }

    #[test]
    fn add_path_button_creates_second_path() {
        let window = SpecsWindow::default();
        let specs = SystemSpecs::default();
        let active_path = 0usize;
        let mut harness = Harness::new_state(
            |ctx, (window, specs, active_path): &mut (SpecsWindow, SystemSpecs, usize)| {
                show_specs_window(window, specs, active_path, ctx);
            },
            (window, specs, active_path),
        );
        harness.step();
        harness.get_by_label("+ Add Path").click();
        harness.step();
        let (_, specs, active_path) = harness.state();
        assert_eq!(specs.paths.len(), 2);
        assert_eq!(*active_path, 1);
    }
}
