use crate::{
    core::math::{linalg::mat3x3::Mat3x3, vec3::Vec3},
    gui::{
        colors::wavelength_to_color,
        result_package::{ResultPackage, SurfaceDesc},
    },
    views::ray_trace_3d::RayBundle,
};

const PLOT_SIZE: f32 = 180.0;

/// Parameters describing which field/surface/wavelength data to render.
struct FieldPlotQuery<'a> {
    ray_trace: &'a crate::TraceResultsCollection,
    wavelengths: &'a [f64],
    field_id: usize,
    surface_idx: usize,
    wavelength_visible: &'a [bool],
    surf_desc: Option<&'a SurfaceDesc>,
}

/// Floating spot diagram output window.
#[derive(Default)]
pub struct SpotDiagramWindow {
    /// Surface index to display. `None` means auto-select the Image surface.
    selected_surface: Option<usize>,
    /// Per-wavelength visibility toggle (indexed by wavelength index in the
    /// result package). Rebuilt whenever the wavelength count changes.
    wavelength_visible: Vec<bool>,
    /// Wavelength count from the last result seen.
    last_n_wavelengths: usize,
}

impl SpotDiagramWindow {
    /// Show the spot diagram window.
    pub fn show(&mut self, ctx: &egui::Context, open: &mut bool, result: Option<&ResultPackage>) {
        egui::Window::new("Spot Diagram")
            .open(open)
            .default_width(640.0)
            .min_width(300.0)
            .show(ctx, |ui| {
                // Sync wavelength visibility when the result package changes.
                if let Some(r) = result
                    && r.wavelengths.len() != self.last_n_wavelengths
                {
                    self.wavelength_visible = vec![true; r.wavelengths.len()];
                    self.last_n_wavelengths = r.wavelengths.len();
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

        // Field plots in a row, each with its own bounding box.
        ui.horizontal(|ui| {
            for field_id in 0..n_fields {
                let field_label = r
                    .fields
                    .get(field_id)
                    .map(|f| f.label.as_str())
                    .unwrap_or("—");
                let query = FieldPlotQuery {
                    ray_trace,
                    wavelengths: &r.wavelengths,
                    field_id,
                    surface_idx: selected_idx,
                    wavelength_visible: &self.wavelength_visible,
                    surf_desc: r.surfaces.get(selected_idx),
                };
                let ranges = compute_field_axis_range(&query);
                ui.vertical(|ui| {
                    ui.label(field_label);
                    render_field_plot(ui, &query, ranges);
                });
            }
        });
    }
}

/// Compute independent X and Y axis ranges enclosing all visible ray
/// intersections for a single field. Returns `(x_range, y_range)` as a square
/// viewport centered on the spot centroid.
fn compute_field_axis_range(query: &FieldPlotQuery) -> ((f64, f64), (f64, f64)) {
    let mut x_min = f64::MAX;
    let mut x_max = f64::MIN;
    let mut y_min = f64::MAX;
    let mut y_max = f64::MIN;

    for (wl_id, &visible) in query.wavelength_visible.iter().enumerate() {
        if !visible {
            continue;
        }
        if let Some(tr) = query.ray_trace.get(query.field_id, wl_id) {
            for (x, y) in rays_at_surface(tr.full_pupil(), query.surface_idx, query.surf_desc) {
                x_min = x_min.min(x);
                x_max = x_max.max(x);
                y_min = y_min.min(y);
                y_max = y_max.max(y);
            }
        }
    }

    if x_min > x_max || y_min > y_max {
        return ((-1.0, 1.0), (-1.0, 1.0));
    }

    // Square viewport: same span for both axes, centered on the spot centroid.
    let x_center = (x_min + x_max) / 2.0;
    let y_center = (y_min + y_max) / 2.0;
    let half = ((x_max - x_min).max(y_max - y_min)).max(1e-6) / 2.0;
    let pad = half * 0.15 + 1e-6;
    (
        (x_center - half - pad, x_center + half + pad),
        (y_center - half - pad, y_center + half + pad),
    )
}

/// Draw a scatter plot for a single field using egui's painter.
fn render_field_plot(ui: &mut egui::Ui, query: &FieldPlotQuery, ranges: ((f64, f64), (f64, f64))) {
    let (x_range, y_range) = ranges;
    let (x_min, x_max) = x_range;
    let (y_min, y_max) = y_range;
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

    // Axes through origin (if origin is in range), independently for X and Y.
    let x_span = (x_max - x_min).max(f64::EPSILON);
    let y_span = (y_max - y_min).max(f64::EPSILON);
    let axis_color = ui.visuals().weak_text_color();
    let origin_tx = (-x_min / x_span) as f32;
    let origin_ty = (-y_min / y_span) as f32;
    if (0.0..=1.0).contains(&origin_tx) {
        let cx = rect.left() + origin_tx * rect.width();
        painter.vline(cx, rect.y_range(), egui::Stroke::new(1.0, axis_color));
    }
    if (0.0..=1.0).contains(&origin_ty) {
        let cy = rect.bottom() - origin_ty * rect.height();
        painter.hline(rect.x_range(), cy, egui::Stroke::new(1.0, axis_color));
    }

    // Helper: map data (x, y) → screen position.
    let to_screen = |dx: f64, dy: f64| -> egui::Pos2 {
        let tx = ((dx - x_min) / x_span) as f32;
        let ty = ((dy - y_min) / y_span) as f32;
        egui::pos2(
            rect.left() + tx * rect.width(),
            rect.bottom() - ty * rect.height(),
        )
    };

    for (wl_id, &visible) in query.wavelength_visible.iter().enumerate() {
        if !visible {
            continue;
        }
        let color = query
            .wavelengths
            .get(wl_id)
            .copied()
            .map(wavelength_to_color)
            .unwrap_or(egui::Color32::WHITE);

        if let Some(tr) = query.ray_trace.get(query.field_id, wl_id) {
            // Ray intersection scatter.
            for (rx, ry) in rays_at_surface(tr.full_pupil(), query.surface_idx, query.surf_desc) {
                let sp = to_screen(rx, ry);
                if rect.contains(sp) {
                    painter.circle_filled(sp, 2.0, color);
                }
            }

            // Chief ray: cross marker.
            for (cx, cy) in rays_at_surface(tr.chief_ray(), query.surface_idx, query.surf_desc) {
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
        format!("{x_min:.3}"),
        font.clone(),
        label_color,
    );
    painter.text(
        egui::pos2(rect.right(), rect.bottom()),
        egui::Align2::RIGHT_BOTTOM,
        format!("{x_max:.3}"),
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

/// Extract (x, y) positions of non-terminated rays at `surface_idx`,
/// projected into the surface's local coordinate frame.
///
/// `RayBundle` stores ray positions in global coordinates. For a tilted
/// surface (folded system), we rotate into the surface's local frame so that
/// x/y represent positions in the plane of the surface.
///
/// `RayBundle` stores rays as a flat
/// `[surface_0_rays … surface_N_rays]` array of length
/// `num_surfaces × num_rays_per_surface`.
fn rays_at_surface<'a>(
    bundle: &'a RayBundle,
    surface_idx: usize,
    surf_desc: Option<&'a SurfaceDesc>,
) -> impl Iterator<Item = (f64, f64)> + 'a {
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

    // Extract surface transform; default to identity (no rotation, origin).
    let (surf_pos, rot_mat) = surf_desc
        .map(|s| (s.pos, s.rot_mat))
        .unwrap_or_else(|| (Vec3::new(0.0, 0.0, 0.0), Mat3x3::identity()));

    (start..end).filter_map(move |abs_idx| {
        let ray_idx = abs_idx - start;
        if terminated.get(ray_idx).copied().unwrap_or(0) > 0 {
            return None;
        }
        let ray = rays.get(abs_idx)?;
        // Transform the global position into the surface's local frame.
        let global = Vec3::new(ray.x(), ray.y(), ray.z());
        let local = rot_mat * (global - surf_pos);
        Some((local.x(), local.y()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{Harness, kittest::Queryable};

    use crate::gui::result_package::{FieldDesc, ResultPackage};

    fn show_window(
        window: &mut SpotDiagramWindow,
        result: Option<&ResultPackage>,
        ctx: &egui::Context,
    ) {
        let mut open = true;
        window.show(ctx, &mut open, result);
    }

    #[test]
    fn no_result_shows_placeholder() {
        let mut window = SpotDiagramWindow::default();
        let mut harness = Harness::new(|ctx| show_window(&mut window, None, ctx));
        harness.step();
        harness.get_by_label("No data yet.");
    }

    #[test]
    fn system_error_shown() {
        let window = SpotDiagramWindow::default();
        let result = ResultPackage::error(1, "bad specs".to_string());
        let mut harness = Harness::new_state(
            |ctx, (w, r): &mut (SpotDiagramWindow, ResultPackage)| {
                show_window(w, Some(r), ctx);
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
            field_specs: Vec::new(),
            paraxial: Some(pv),
            ray_trace: None,
            cross_section: None,
            error: Some("trace failed".to_string()),
        };

        let window = SpotDiagramWindow::default();
        let mut harness = Harness::new_state(
            |ctx, (w, r): &mut (SpotDiagramWindow, ResultPackage)| {
                show_window(w, Some(r), ctx);
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
            chi: "5.0".into(),
            phi: "90.0".into(),
            x: "0.0".into(),
        });

        #[cfg(not(feature = "ri-info"))]
        let parsed = convert::convert_specs(&specs).expect("convert");
        #[cfg(feature = "ri-info")]
        let parsed = convert::convert_specs(&specs, &Default::default()).expect("convert");
        let seq = SequentialModel::new(&parsed.gaps, &parsed.surfaces, &parsed.wavelengths)
            .expect("model");
        let pv = ParaxialView::new(&seq, &parsed.fields, false).expect("paraxial");
        let trace = ray_trace_3d_view(
            &parsed.aperture,
            &parsed.fields,
            &seq,
            &pv,
            crate::views::ray_trace_3d::SamplingConfig {
                n_fan_rays: 3,
                cross_section_n_fan_rays: 3,
                full_pupil_spacing: 0.1,
            },
        )
        .expect("trace");

        let result = ResultPackage {
            id: 1,
            wavelengths: seq.wavelengths().to_vec(),
            surfaces: seq
                .surfaces()
                .iter()
                .enumerate()
                .map(|(i, s)| crate::gui::result_package::SurfaceDesc {
                    index: i,
                    label: format!("S{i}"),
                    pos: s.pos(),
                    rot_mat: s.rot_mat(),
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
            field_specs: parsed.fields.clone(),
            paraxial: Some(pv),
            ray_trace: Some(trace),
            cross_section: None,
            error: None,
        };

        let window = SpotDiagramWindow::default();
        let mut harness = Harness::new_state(
            |ctx, (w, r): &mut (SpotDiagramWindow, ResultPackage)| {
                show_window(w, Some(r), ctx);
            },
            (window, result),
        );
        harness.step();
        harness.get_by_label_contains("0.000");
        harness.get_by_label_contains("5.000");
    }
}
