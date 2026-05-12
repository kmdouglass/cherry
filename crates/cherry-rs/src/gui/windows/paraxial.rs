use egui_extras::{Column, TableBuilder};

use crate::{gui::result_package::ResultPackage, views::paraxial::ParaxialSubView};

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

    // Collect unique v_indices, sorted ascending (ascending phi).
    let mut v_indices: Vec<usize> = pv.iter().map(|sv| sv.tangential_vec_id()).collect();
    v_indices.sort_unstable();
    v_indices.dedup();
    let n_v = v_indices.len();

    for (i, &v_idx) in v_indices.iter().enumerate() {
        if n_v > 1 {
            if i > 0 {
                ui.separator();
            }
            let phi_deg = pv.phi_deg(v_idx);
            ui.heading(format!("\u{03c6} = {phi_deg:.0}\u{00b0}"));
        }

        // Subviews for this tangential_vec_id, sorted by wavelength_id.
        let mut ids: Vec<(usize, usize)> = pv
            .iter()
            .filter(|sv| sv.tangential_vec_id() == v_idx)
            .map(|sv| (sv.wavelength_id(), sv.tangential_vec_id()))
            .collect();
        ids.sort_by_key(|&(wl_id, _)| wl_id);

        render_v_table(ui, r, &ids, pv);
        ui.add_space(8.0);
    }

    // Primary Axial Color (only when there are multiple wavelengths).
    if r.wavelengths.len() > 1 {
        let pac = pv.primary_axial_color();
        for &v_idx in &v_indices {
            if let Some(&color) = pac.get(v_idx) {
                let phi_suffix = if n_v > 1 {
                    let phi_deg = pv.phi_deg(v_idx);
                    format!(" (\u{03c6} = {phi_deg:.0}\u{00b0})")
                } else {
                    String::new()
                };
                ui.label(format!(
                    "Primary Axial Color{}: {}",
                    phi_suffix,
                    format_value(color)
                ));
            }
        }
        ui.add_space(4.0);
    }
}

fn render_v_table(
    ui: &mut egui::Ui,
    r: &ResultPackage,
    ids: &[(usize, usize)],
    pv: &crate::views::paraxial::ParaxialView,
) {
    let n_wl = ids.len();
    let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;

    let builder = TableBuilder::new(ui)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto())
        .columns(Column::auto().at_least(70.0), n_wl);

    if n_wl > 1 {
        builder
            .header(row_h, |mut hdr| {
                hdr.col(|_| {});
                for &(wl_id, _) in ids {
                    hdr.col(|ui| {
                        let wl_label = r
                            .wavelengths
                            .get(wl_id)
                            .map(|wl| format!("{wl:.4} \u{00b5}m"))
                            .unwrap_or_else(|| format!("WL {wl_id}"));
                        ui.label(wl_label);
                    });
                }
            })
            .body(|mut body| {
                render_table_body(&mut body, row_h, ids, pv);
            });
    } else {
        builder.body(|mut body| {
            render_table_body(&mut body, row_h, ids, pv);
        });
    }
}

fn render_table_body(
    body: &mut egui_extras::TableBody<'_>,
    row_h: f32,
    ids: &[(usize, usize)],
    pv: &crate::views::paraxial::ParaxialView,
) {
    let n_cols = 1 + ids.len();

    if ids.len() > 1 {
        tb_sep(body, n_cols);
    }

    tb_row(body, row_h, "EFL", ids, pv, |sv| {
        *sv.effective_focal_length()
    });
    tb_row(body, row_h, "BFD", ids, pv, |sv| *sv.back_focal_distance());
    tb_row(body, row_h, "FFD", ids, pv, |sv| *sv.front_focal_distance());
    tb_row(body, row_h, "Paraxial F/#", ids, pv, |sv| sv.paraxial_fno());
    tb_row(body, row_h, "Image space F/#", ids, pv, |sv| {
        sv.image_space_fno()
    });

    tb_sep(body, n_cols);

    tb_row(
        body,
        row_h,
        "Entrance pupil dist. from first surface",
        ids,
        pv,
        |sv| sv.entrance_pupil().location,
    );
    tb_row(body, row_h, "Entrance pupil semi-diameter", ids, pv, |sv| {
        sv.entrance_pupil().semi_diameter
    });
    tb_row(
        body,
        row_h,
        "Exit pupil dist. from last surface",
        ids,
        pv,
        |sv| sv.exit_pupil().location,
    );
    tb_row(body, row_h, "Exit pupil semi-diameter", ids, pv, |sv| {
        sv.exit_pupil().semi_diameter
    });

    tb_sep(body, n_cols);

    tb_row(
        body,
        row_h,
        "Front principal plane dist. from first surface",
        ids,
        pv,
        |sv| *sv.front_principal_plane(),
    );
    tb_row(
        body,
        row_h,
        "Back principal plane dist. from last surface",
        ids,
        pv,
        |sv| *sv.back_principal_plane(),
    );

    tb_sep(body, n_cols);

    // Aperture stop is an integer index; format it without decimals.
    body.row(row_h, |mut row| {
        row.col(|ui| {
            ui.label("Aperture stop (surface)");
        });
        for &(wl_id, tangential_vec_id) in ids {
            row.col(|ui| {
                if let Some(sv) = pv.get(wl_id, tangential_vec_id) {
                    ui.label(sv.aperture_stop().to_string());
                } else {
                    ui.label("\u{2014}");
                }
            });
        }
    });
}

fn tb_sep(body: &mut egui_extras::TableBody<'_>, n_cols: usize) {
    body.row(6.0, |mut row| {
        for _ in 0..n_cols {
            row.col(|ui| {
                let (rect, _) = ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());
                ui.painter().hline(
                    rect.x_range(),
                    rect.center().y,
                    ui.visuals().widgets.noninteractive.bg_stroke,
                );
            });
        }
    });
}

fn tb_row<F>(
    body: &mut egui_extras::TableBody<'_>,
    row_h: f32,
    label: &str,
    ids: &[(usize, usize)],
    pv: &crate::views::paraxial::ParaxialView,
    get_val: F,
) where
    F: Fn(&ParaxialSubView) -> f64,
{
    body.row(row_h, |mut row| {
        row.col(|ui| {
            ui.label(label);
        });
        for &(wl_id, tangential_vec_id) in ids {
            row.col(|ui| {
                if let Some(sv) = pv.get(wl_id, tangential_vec_id) {
                    ui.label(format_value(get_val(sv)));
                } else {
                    ui.label("\u{2014}");
                }
            });
        }
    });
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
        let seq = SequentialModel::from_surface_specs(
            &parsed.gaps,
            &parsed.surfaces,
            &parsed.wavelengths,
            None,
        )
        .expect("model");
        let pv = ParaxialView::new(&seq, &parsed.fields, false).expect("paraxial");
        let wls = seq.wavelengths().to_vec();
        ResultPackage {
            id: 1,
            wavelengths: wls,
            surfaces: Vec::new(),
            fields: Vec::new(),
            field_specs: parsed.fields.clone(),
            paraxial: Some(pv),
            ray_trace: None,
            cross_section: None,
            error: None,
            solved_values: Default::default(),
            components: Vec::new(),
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
    fn paraxial_data_shows_fno_rows() {
        let result = make_result(&["0.567"]);
        let mut harness = Harness::new(|ctx| {
            let mut open = true;
            ParaxialWindow::show(ctx, &mut open, Some(&result));
        });
        harness.step();
        harness.get_by_label("Paraxial F/#");
        harness.get_by_label("Image space F/#");
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
