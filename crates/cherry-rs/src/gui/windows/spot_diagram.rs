use crate::{Axis, gui::result_package::ResultPackage, views::ray_trace_3d::RayBundle};

const PLOT_SIZE: f32 = 180.0;

/// Floating spot diagram output window.
pub struct SpotDiagramWindow {
    /// Surface index to display. `None` means auto-select the Image surface.
    selected_surface: Option<usize>,
    /// Per-wavelength visibility toggle (indexed by wavelength index in the
    /// result package). Rebuilt whenever the wavelength count changes.
    wavelength_visible: Vec<bool>,
    /// Wavelength count from the last result seen.
    last_n_wavelengths: usize,
}

impl Default for SpotDiagramWindow {
    fn default() -> Self {
        Self {
            selected_surface: None,
            wavelength_visible: Vec::new(),
            last_n_wavelengths: 0,
        }
    }
}

impl SpotDiagramWindow {
    /// Show the spot diagram window.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        result: Option<&ResultPackage>,
        input_id: u64,
    ) {
        egui::Window::new("Spot Diagram")
            .open(open)
            .default_width(640.0)
            .min_width(300.0)
            .show(ctx, |ui| {
                // Sync wavelength visibility when the result package changes.
                if let Some(r) = result {
                    if r.wavelengths.len() != self.last_n_wavelengths {
                        self.wavelength_visible = vec![true; r.wavelengths.len()];
                        self.last_n_wavelengths = r.wavelengths.len();
                    }
                }

                let is_stale = matches!(result, Some(r) if r.id < input_id);
                if is_stale {
                    ui.colored_label(egui::Color32::YELLOW, "\u{26a0} Update in progress\u{2026}");
                    ui.separator();
                }

                match result {
                    None => {
                        ui.label("No data yet.");
                    }
                    Some(r) if r.paraxial.is_none() => {
                        let msg = r.error.as_deref().unwrap_or("Unknown error");
                        ui.colored_label(egui::Color32::RED, format!("System error: {msg}"));
                    }
                    Some(r) if r.ray_trace.is_none() => {
                        let msg = r.error.as_deref().unwrap_or("unknown");
                        ui.colored_label(
                            egui::Color32::from_rgb(255, 165, 0),
                            format!("Ray trace unavailable: {msg}"),
                        );
                    }
                    Some(r) => {
                        self.render_content(ui, r);
                    }
                }
            });
    }

    fn render_content(&mut self, ui: &mut egui::Ui, r: &ResultPackage) {
        let ray_trace = r.ray_trace.as_ref().unwrap();

        // Determine the default surface (Image, last in list).
        let image_surface_idx = r
            .surfaces
            .iter()
            .rev()
            .find(|s| s.label.starts_with("Image"))
            .map(|s| s.index)
            .unwrap_or(r.surfaces.len().saturating_sub(1));
        let selected_idx = self.selected_surface.unwrap_or(image_surface_idx);

        // Surface selector.
        ui.horizontal(|ui| {
            ui.label("Surface:");
            egui::ComboBox::from_id_salt("spot_surface_selector")
                .selected_text(
                    r.surfaces
                        .get(selected_idx)
                        .map(|s| s.label.as_str())
                        .unwrap_or("—"),
                )
                .show_ui(ui, |ui| {
                    for s in &r.surfaces {
                        let is_sel = s.index == selected_idx;
                        if ui.selectable_label(is_sel, &s.label).clicked() {
                            self.selected_surface = Some(s.index);
                        }
                    }
                });
        });

        // Wavelength toggles (only when more than one wavelength).
        if r.wavelengths.len() > 1 {
            ui.horizontal(|ui| {
                for (i, &wl) in r.wavelengths.iter().enumerate() {
                    if let Some(v) = self.wavelength_visible.get_mut(i) {
                        ui.checkbox(v, format!("{wl:.4} \u{00b5}m"));
                    }
                }
            });
        }

        ui.separator();

        let n_fields = r.fields.len();
        if n_fields == 0 {
            ui.label("No fields defined.");
            return;
        }

        // Common axis range across all visible rays.
        let (axis_min, axis_max) =
            compute_axis_range(r, ray_trace, selected_idx, &self.wavelength_visible);

        // Field plots in a row.
        ui.horizontal(|ui| {
            for field_id in 0..n_fields {
                let field_label = r
                    .fields
                    .get(field_id)
                    .map(|f| f.label.as_str())
                    .unwrap_or("—");
                ui.vertical(|ui| {
                    ui.label(field_label);
                    render_field_plot(
                        ui,
                        r,
                        ray_trace,
                        field_id,
                        selected_idx,
                        &self.wavelength_visible,
                        axis_min,
                        axis_max,
                    );
                });
            }
        });
    }
}

/// Compute a common symmetric axis range over all visible fields/wavelengths.
fn compute_axis_range(
    r: &ResultPackage,
    ray_trace: &crate::TraceResultsCollection,
    surface_idx: usize,
    wavelength_visible: &[bool],
) -> (f64, f64) {
    let mut min_v = f64::MAX;
    let mut max_v = f64::MIN;

    for field_id in 0..r.fields.len() {
        for (wl_id, &visible) in wavelength_visible.iter().enumerate() {
            if !visible {
                continue;
            }
            if let Some(tr) = ray_trace.get(field_id, wl_id, Axis::Y) {
                for (x, y) in rays_at_surface(tr.ray_bundle(), surface_idx) {
                    min_v = min_v.min(x).min(y);
                    max_v = max_v.max(x).max(y);
                }
            }
        }
    }

    if min_v > max_v {
        (-1.0, 1.0)
    } else {
        let half = (max_v - min_v).max(1e-6) / 2.0;
        let mid = (min_v + max_v) / 2.0;
        let pad = half * 0.15 + 1e-6;
        (mid - half - pad, mid + half + pad)
    }
}

/// Draw a scatter plot for a single field using egui's painter.
fn render_field_plot(
    ui: &mut egui::Ui,
    r: &ResultPackage,
    ray_trace: &crate::TraceResultsCollection,
    field_id: usize,
    surface_idx: usize,
    wavelength_visible: &[bool],
    axis_min: f64,
    axis_max: f64,
) {
    let size = egui::Vec2::splat(PLOT_SIZE);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter_at(rect);

    // Background and border.
    painter.rect_filled(rect, 2.0, ui.visuals().extreme_bg_color);
    painter.rect_stroke(
        rect,
        2.0,
        ui.visuals().window_stroke(),
        egui::StrokeKind::Outside,
    );

    // Axes through origin (if origin is in range).
    let data_range = (axis_max - axis_min).max(f64::EPSILON);
    let origin_t = (-axis_min / data_range) as f32;
    let axis_color = ui.visuals().weak_text_color();
    if (0.0..=1.0).contains(&origin_t) {
        let cx = rect.left() + origin_t * rect.width();
        painter.vline(cx, rect.y_range(), egui::Stroke::new(1.0, axis_color));
        let cy = rect.bottom() - origin_t * rect.height();
        painter.hline(rect.x_range(), cy, egui::Stroke::new(1.0, axis_color));
    }

    // Helper: map data (x, y) → screen position.
    let to_screen = |dx: f64, dy: f64| -> egui::Pos2 {
        let tx = ((dx - axis_min) / data_range) as f32;
        let ty = ((dy - axis_min) / data_range) as f32;
        egui::pos2(
            rect.left() + tx * rect.width(),
            rect.bottom() - ty * rect.height(),
        )
    };

    for (wl_id, &visible) in wavelength_visible.iter().enumerate() {
        if !visible {
            continue;
        }
        let color = r
            .wavelengths
            .get(wl_id)
            .copied()
            .map(wavelength_to_color)
            .unwrap_or(egui::Color32::WHITE);

        if let Some(tr) = ray_trace.get(field_id, wl_id, Axis::Y) {
            // Ray intersection scatter.
            for (rx, ry) in rays_at_surface(tr.ray_bundle(), surface_idx) {
                let sp = to_screen(rx, ry);
                if rect.contains(sp) {
                    painter.circle_filled(sp, 2.0, color);
                }
            }

            // Chief ray: cross marker.
            for (cx, cy) in rays_at_surface(tr.chief_ray(), surface_idx) {
                let sp = to_screen(cx, cy);
                let arm = 5.0_f32;
                let stroke = egui::Stroke::new(1.5, color);
                painter.line_segment(
                    [sp + egui::vec2(-arm, 0.0), sp + egui::vec2(arm, 0.0)],
                    stroke,
                );
                painter.line_segment(
                    [sp + egui::vec2(0.0, -arm), sp + egui::vec2(0.0, arm)],
                    stroke,
                );
            }
        }
    }

    // Axis labels at edges.
    let label_color = ui.visuals().text_color();
    let font = egui::FontId::proportional(9.0);
    painter.text(
        egui::pos2(rect.left(), rect.bottom()),
        egui::Align2::LEFT_BOTTOM,
        format!("{axis_min:.3}"),
        font.clone(),
        label_color,
    );
    painter.text(
        egui::pos2(rect.right(), rect.bottom()),
        egui::Align2::RIGHT_BOTTOM,
        format!("{axis_max:.3}"),
        font.clone(),
        label_color,
    );
    painter.text(
        egui::pos2(rect.left() + rect.width() / 2.0, rect.bottom()),
        egui::Align2::CENTER_BOTTOM,
        "mm",
        font,
        label_color,
    );
}

/// Extract (x, y) positions of non-terminated rays at `surface_idx`.
///
/// `RayBundle` stores rays as a flat
/// `[surface_0_rays … surface_N_rays]` array of length
/// `num_surfaces × num_rays_per_surface`.
fn rays_at_surface(
    bundle: &RayBundle,
    surface_idx: usize,
) -> impl Iterator<Item = (f64, f64)> + '_ {
    let total = bundle.rays().len();
    let n_surf = bundle.num_surfaces();
    let n_rays = if n_surf > 0 { total / n_surf } else { 0 };

    let (start, end) = if n_surf > 0 && surface_idx < n_surf && n_rays > 0 {
        (surface_idx * n_rays, (surface_idx + 1) * n_rays)
    } else {
        (0, 0)
    };

    let rays = bundle.rays();
    let terminated = bundle.terminated();

    (start..end).filter_map(move |abs_idx| {
        let ray_idx = abs_idx - start;
        if terminated.get(ray_idx).copied().unwrap_or(0) > 0 {
            return None;
        }
        let ray = rays.get(abs_idx)?;
        Some((ray.x() as f64, ray.y() as f64))
    })
}

/// Map a wavelength in μm to an approximate visible-spectrum color.
fn wavelength_to_color(wl_um: f64) -> egui::Color32 {
    let nm = wl_um * 1000.0;
    if nm <= 450.0 {
        egui::Color32::from_rgb(110, 0, 220)
    } else if nm <= 520.0 {
        egui::Color32::from_rgb(0, 100, 255)
    } else if nm <= 565.0 {
        egui::Color32::from_rgb(0, 200, 50)
    } else if nm <= 600.0 {
        egui::Color32::from_rgb(220, 220, 0)
    } else {
        egui::Color32::from_rgb(220, 30, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{Harness, kittest::Queryable};

    use crate::gui::result_package::{BoundingBox3D, FieldDesc, ResultPackage};

    fn show_window(
        window: &mut SpotDiagramWindow,
        result: Option<&ResultPackage>,
        input_id: u64,
        ctx: &egui::Context,
    ) {
        let mut open = true;
        window.show(ctx, &mut open, result, input_id);
    }

    #[test]
    fn no_result_shows_placeholder() {
        let mut window = SpotDiagramWindow::default();
        let mut harness = Harness::new(|ctx| show_window(&mut window, None, 0, ctx));
        harness.step();
        harness.get_by_label("No data yet.");
    }

    #[test]
    fn stale_result_shows_update_in_progress() {
        let mut window = SpotDiagramWindow::default();
        let result = ResultPackage::error(0, String::new());
        let mut harness = Harness::new_state(
            |ctx, (w, r): &mut (SpotDiagramWindow, ResultPackage)| {
                show_window(w, Some(r), 1, ctx);
            },
            (window, result),
        );
        harness.step();
        harness.get_by_label_contains("Update in progress");
    }

    #[test]
    fn no_stale_banner_when_no_result() {
        let mut window = SpotDiagramWindow::default();
        let mut harness = Harness::new(|ctx| show_window(&mut window, None, 5, ctx));
        harness.step();
        assert!(
            harness
                .query_by_label_contains("Update in progress")
                .is_none()
        );
    }

    #[test]
    fn system_error_shown() {
        let mut window = SpotDiagramWindow::default();
        let result = ResultPackage::error(1, "bad specs".to_string());
        let mut harness = Harness::new_state(
            |ctx, (w, r): &mut (SpotDiagramWindow, ResultPackage)| {
                show_window(w, Some(r), 1, ctx);
            },
            (window, result),
        );
        harness.step();
        harness.get_by_label_contains("System error");
        harness.get_by_label_contains("bad specs");
    }

    #[test]
    fn ray_trace_unavailable_shown_when_only_paraxial() {
        use crate::gui::{convert, model::SystemSpecs};
        use crate::{ParaxialView, SequentialModel};

        let specs = SystemSpecs::default();
        #[cfg(not(feature = "ri-info"))]
        let parsed = convert::convert_specs(&specs).expect("convert");
        #[cfg(feature = "ri-info")]
        let parsed = convert::convert_specs(&specs, &Default::default()).expect("convert");
        let seq = SequentialModel::new(&parsed.gaps, &parsed.surfaces, &parsed.wavelengths)
            .expect("model");
        let pv = ParaxialView::new(&seq, &parsed.fields, false).expect("paraxial");

        let result = ResultPackage {
            id: 1,
            wavelengths: seq.wavelengths().to_vec(),
            surfaces: Vec::new(),
            fields: Vec::new(),
            paraxial: Some(pv),
            ray_trace: None,
            bounding_box: BoundingBox3D::default(),
            error: Some("trace failed".to_string()),
        };

        let mut window = SpotDiagramWindow::default();
        let mut harness = Harness::new_state(
            |ctx, (w, r): &mut (SpotDiagramWindow, ResultPackage)| {
                show_window(w, Some(r), 1, ctx);
            },
            (window, result),
        );
        harness.step();
        harness.get_by_label_contains("Ray trace unavailable");
        harness.get_by_label_contains("trace failed");
    }

    #[test]
    fn full_result_shows_field_labels() {
        use crate::gui::{
            convert,
            model::{FieldRow, SystemSpecs},
        };
        use crate::{ParaxialView, SequentialModel, ray_trace_3d_view};

        let mut specs = SystemSpecs::default();
        specs.fields.push(FieldRow {
            value: "5.0".into(),
            x: "0.0".into(),
            pupil_spacing: "0.1".into(),
        });

        #[cfg(not(feature = "ri-info"))]
        let parsed = convert::convert_specs(&specs).expect("convert");
        #[cfg(feature = "ri-info")]
        let parsed = convert::convert_specs(&specs, &Default::default()).expect("convert");
        let seq = SequentialModel::new(&parsed.gaps, &parsed.surfaces, &parsed.wavelengths)
            .expect("model");
        let pv = ParaxialView::new(&seq, &parsed.fields, false).expect("paraxial");
        let trace =
            ray_trace_3d_view(&parsed.aperture, &parsed.fields, &seq, &pv, None).expect("trace");

        let result = ResultPackage {
            id: 1,
            wavelengths: seq.wavelengths().to_vec(),
            surfaces: seq
                .surfaces()
                .iter()
                .enumerate()
                .map(|(i, _)| crate::gui::result_package::SurfaceDesc {
                    index: i,
                    label: format!("S{i}"),
                })
                .collect(),
            fields: vec![
                FieldDesc {
                    label: "0.000\u{00b0}".into(),
                },
                FieldDesc {
                    label: "5.000\u{00b0}".into(),
                },
            ],
            paraxial: Some(pv),
            ray_trace: Some(trace),
            bounding_box: BoundingBox3D::default(),
            error: None,
        };

        let mut window = SpotDiagramWindow::default();
        let mut harness = Harness::new_state(
            |ctx, (w, r): &mut (SpotDiagramWindow, ResultPackage)| {
                show_window(w, Some(r), 1, ctx);
            },
            (window, result),
        );
        harness.step();
        harness.get_by_label_contains("0.000");
        harness.get_by_label_contains("5.000");
    }
}
