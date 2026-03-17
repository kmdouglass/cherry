use std::collections::HashMap;

use crate::{
    Axis, core::sequential_model::SubModelID, gui::result_package::ResultPackage,
    views::paraxial::ParaxialSubView,
};

/// Floating paraxial summary output window.
pub struct ParaxialWindow;

impl ParaxialWindow {
    /// Show the paraxial summary window.
    pub fn show(ctx: &egui::Context, open: &mut bool, result: Option<&ResultPackage>) {
        egui::Window::new("Paraxial Summary")
            .open(open)
            .default_width(400.0)
            .show(ctx, |ui| match result {
                None => {
                    ui.label("No data yet.");
                }
                Some(r) if r.paraxial.is_none() => {
                    if let Some(err) = &r.error {
                        ui.colored_label(egui::Color32::RED, format!("System error: {err}"));
                    }
                }
                Some(r) => {
                    render_paraxial_content(ui, r);
                }
            });
    }
}

fn render_paraxial_content(ui: &mut egui::Ui, r: &ResultPackage) {
    let pv = r.paraxial.as_ref().unwrap();
    let subviews = pv.subviews();

    // Collect unique axes, sorted R before U.
    let mut axes: Vec<Axis> = subviews.keys().map(|id| id.1).collect();
    axes.sort_by_key(|a| if *a == Axis::R { 0u8 } else { 1 });
    axes.dedup();
    let n_axes = axes.len();

    for (i, axis) in axes.iter().enumerate() {
        if n_axes > 1 {
            if i > 0 {
                ui.separator();
            }
            let label = if *axis == Axis::R { "R Axis" } else { "U Axis" };
            ui.heading(label);
        }

        // Subview IDs for this axis, sorted by wavelength_id.
        let mut ids: Vec<SubModelID> = subviews
            .keys()
            .filter(|id| id.1 == *axis)
            .copied()
            .collect();
        ids.sort_by_key(|id| id.0);

        render_axis_table(ui, r, &ids, subviews);
        ui.add_space(8.0);
    }

    // Primary Axial Color (only when there are multiple wavelengths).
    if r.wavelengths.len() > 1 {
        let pac = pv.primary_axial_color();
        for axis in &axes {
            if let Some(&color) = pac.get(axis) {
                let axis_suffix = if n_axes > 1 {
                    format!(" ({})", if *axis == Axis::R { "R" } else { "U" })
                } else {
                    String::new()
                };
                ui.label(format!(
                    "Primary Axial Color{}: {}",
                    axis_suffix,
                    format_value(color)
                ));
            }
        }
        ui.add_space(4.0);
    }
}

fn render_axis_table(
    ui: &mut egui::Ui,
    r: &ResultPackage,
    ids: &[SubModelID],
    subviews: &HashMap<SubModelID, ParaxialSubView>,
) {
    let n_wl = ids.len();
    let n_cols = 1 + n_wl;

    egui::Grid::new(format!(
        "paraxial_{}",
        ids.first()
            .map_or("empty", |id| { if id.1 == Axis::R { "r" } else { "u" } })
    ))
    .num_columns(n_cols)
    .spacing([20.0, 4.0])
    .show(ui, |ui| {
        // Wavelength header row — only when there are multiple wavelengths.
        if n_wl > 1 {
            ui.label(""); // empty label cell
            for id in ids {
                let wl_label = r
                    .wavelengths
                    .get(id.0)
                    .map(|wl| format!("{wl:.4} \u{00b5}m"))
                    .unwrap_or_else(|| format!("WL {}", id.0));
                ui.label(wl_label);
            }
            ui.end_row();
            // Separator row.
            for _ in 0..n_cols {
                ui.separator();
            }
            ui.end_row();
        }

        multi_row(ui, "EFL", ids, subviews, |sv| *sv.effective_focal_length());
        multi_row(ui, "BFD", ids, subviews, |sv| *sv.back_focal_distance());
        multi_row(ui, "FFD", ids, subviews, |sv| *sv.front_focal_distance());

        sep_row(ui, n_cols);

        multi_row(
            ui,
            "Entrance pupil dist. from first surface",
            ids,
            subviews,
            |sv| sv.entrance_pupil().location,
        );
        multi_row(ui, "Entrance pupil semi-diameter", ids, subviews, |sv| {
            sv.entrance_pupil().semi_diameter
        });
        multi_row(
            ui,
            "Exit pupil dist. from last surface",
            ids,
            subviews,
            |sv| sv.exit_pupil().location,
        );
        multi_row(ui, "Exit pupil semi-diameter", ids, subviews, |sv| {
            sv.exit_pupil().semi_diameter
        });

        sep_row(ui, n_cols);

        multi_row(
            ui,
            "Front principal plane dist. from first surface",
            ids,
            subviews,
            |sv| *sv.front_principal_plane(),
        );
        multi_row(
            ui,
            "Back principal plane dist. from last surface",
            ids,
            subviews,
            |sv| *sv.back_principal_plane(),
        );

        sep_row(ui, n_cols);

        // Aperture stop is an integer index; format it without decimals.
        ui.label("Aperture stop (surface)");
        for id in ids {
            if let Some(sv) = subviews.get(id) {
                ui.label(sv.aperture_stop().to_string());
            } else {
                ui.label("\u{2014}");
            }
        }
        ui.end_row();
    });
}

fn sep_row(ui: &mut egui::Ui, n_cols: usize) {
    for _ in 0..n_cols {
        ui.separator();
    }
    ui.end_row();
}

fn multi_row<F>(
    ui: &mut egui::Ui,
    label: &str,
    ids: &[SubModelID],
    subviews: &HashMap<SubModelID, ParaxialSubView>,
    get_val: F,
) where
    F: Fn(&ParaxialSubView) -> f64,
{
    ui.label(label);
    for id in ids {
        if let Some(sv) = subviews.get(id) {
            ui.label(format_value(get_val(sv)));
        } else {
            ui.label("\u{2014}");
        }
    }
    ui.end_row();
}

fn format_value(v: f64) -> String {
    if v.is_infinite() {
        if v.is_sign_positive() {
            "\u{221e}".to_string()
        } else {
            "-\u{221e}".to_string()
        }
    } else if v.is_nan() {
        "\u{2014}".to_string()
    } else {
        // 4 significant figures.
        format!("{v:.4}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{Harness, kittest::Queryable};

    use crate::gui::result_package::ResultPackage;

    fn make_result(wavelengths: &[&str]) -> ResultPackage {
        use crate::gui::{convert, model::SystemSpecs};
        use crate::{ParaxialView, SequentialModel};

        let specs = SystemSpecs {
            wavelengths: wavelengths.iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        };
        #[cfg(not(feature = "ri-info"))]
        let parsed = convert::convert_specs(&specs).expect("convert");
        #[cfg(feature = "ri-info")]
        let parsed = convert::convert_specs(&specs, &Default::default()).expect("convert");
        let seq = SequentialModel::new(&parsed.gaps, &parsed.surfaces, &parsed.wavelengths)
            .expect("model");
        let pv = ParaxialView::new(&seq, &parsed.fields, false).expect("paraxial");
        let wls = seq.wavelengths().to_vec();
        ResultPackage {
            id: 1,
            wavelengths: wls,
            surfaces: Vec::new(),
            fields: Vec::new(),
            paraxial: Some(pv),
            ray_trace: None,
            cross_section: None,
            error: None,
        }
    }

    #[test]
    fn no_result_shows_placeholder() {
        let mut harness = Harness::new(|ctx| {
            let mut open = true;
            ParaxialWindow::show(ctx, &mut open, None);
        });
        harness.step();
        harness.get_by_label("No data yet.");
    }

    #[test]
    fn error_result_shows_message() {
        let result = ResultPackage::error(1, "bad specs".to_string());
        let mut harness = Harness::new(|ctx| {
            let mut open = true;
            ParaxialWindow::show(ctx, &mut open, Some(&result));
        });
        harness.step();
        harness.get_by_label_contains("bad specs");
    }

    #[test]
    fn paraxial_data_shows_efl_row() {
        let result = make_result(&["0.567"]);
        let mut harness = Harness::new(|ctx| {
            let mut open = true;
            ParaxialWindow::show(ctx, &mut open, Some(&result));
        });
        harness.step();
        harness.get_by_label("EFL");
        harness.get_by_label("BFD");
        harness.get_by_label("FFD");
    }

    #[test]
    fn single_wavelength_no_wavelength_header() {
        let result = make_result(&["0.567"]);
        let mut harness = Harness::new(|ctx| {
            let mut open = true;
            ParaxialWindow::show(ctx, &mut open, Some(&result));
        });
        harness.step();
        // With one wavelength, no wavelength header row should appear.
        assert!(
            harness.query_by_label_contains("\u{00b5}m").is_none(),
            "no wavelength header should appear for a single wavelength"
        );
    }

    #[test]
    fn multi_wavelength_shows_wavelength_headers() {
        let result = make_result(&["0.486", "0.587", "0.656"]);
        let mut harness = Harness::new(|ctx| {
            let mut open = true;
            ParaxialWindow::show(ctx, &mut open, Some(&result));
        });
        harness.step();
        // Each wavelength should appear as a column header.
        harness.get_by_label_contains("0.4860");
        harness.get_by_label_contains("0.5870");
        harness.get_by_label_contains("0.6560");
    }

    #[test]
    fn multi_wavelength_shows_primary_axial_color() {
        let result = make_result(&["0.486", "0.587", "0.656"]);
        let mut harness = Harness::new(|ctx| {
            let mut open = true;
            ParaxialWindow::show(ctx, &mut open, Some(&result));
        });
        harness.step();
        harness.get_by_label_contains("Primary Axial Color");
    }

    #[test]
    fn single_wavelength_no_primary_axial_color() {
        let result = make_result(&["0.567"]);
        let mut harness = Harness::new(|ctx| {
            let mut open = true;
            ParaxialWindow::show(ctx, &mut open, Some(&result));
        });
        harness.step();
        assert!(
            harness
                .query_by_label_contains("Primary Axial Color")
                .is_none(),
            "Primary Axial Color must not appear for a single wavelength"
        );
    }
}
