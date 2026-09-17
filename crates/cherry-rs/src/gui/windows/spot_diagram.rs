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
    active_path: usize,
    wavelengths: &'a [f64],
    field_id: usize,
    /// The selected surface's step position within `active_path`'s own
    /// traversal — `None` if no surface is reachable from this path at
    /// all. A `RayBundle` is indexed by this step position, not by
    /// `surf_desc`'s global/store index.
    path_step: Option<usize>,
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
    /// Show the spot diagram window, filtered to `active_path` (FR-OUT-1).
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        result: Option<&ResultPackage>,
        active_path: usize,
    ) {
        egui::Window::new("Spot Diagram")
            .open(open)
            .default_width(640.0)
            .min_width(300.0)
            .show(ctx, |ui| {
                // Sync wavelength visibility when the result package changes.
                let n_wl = result
                    .and_then(|r| r.wavelengths_by_path.get(active_path))
                    .map(|w| w.len())
                    .unwrap_or(0);
                if n_wl != self.last_n_wavelengths {
                    self.wavelength_visible = vec![true; n_wl];
                    self.last_n_wavelengths = n_wl;
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
                        self.render_content(ui, r, active_path);
                    }
                }
            });
    }

    fn render_content(&mut self, ui: &mut egui::Ui, r: &ResultPackage, active_path: usize) {
        let ray_trace = r.ray_trace.as_ref().unwrap();
        let empty_wls = Vec::new();
        let empty_fields = Vec::new();
        let wavelengths = r.wavelengths_by_path.get(active_path).unwrap_or(&empty_wls);
        let fields = r.fields_by_path.get(active_path).unwrap_or(&empty_fields);

        // Surfaces reachable from the active path only (FR-OUT-1): a
        // surface that belongs only to some other path has no step
        // position in this path's own ray bundles at all.
        let path_surfaces: Vec<&SurfaceDesc> = r
            .surfaces
            .iter()
            .filter(|s| s.path_step.is_some())
            .collect();

        let selected_idx = resolve_selected_surface(&r.surfaces, self.selected_surface);
        let selected_desc = selected_idx.and_then(|idx| r.surfaces.get(idx));

        // Surface selector.
        ui.horizontal(|ui| {
            ui.label("Surface:");
            egui::ComboBox::from_id_salt("spot_surface_selector")
                .selected_text(selected_desc.map(|s| s.label.as_str()).unwrap_or("—"))
                .show_ui(ui, |ui| {
                    for s in &path_surfaces {
                        let is_sel = Some(s.index) == selected_idx;
                        if ui.selectable_label(is_sel, &s.label).clicked() {
                            self.selected_surface = Some(s.index);
                        }
                    }
                });
        });

        // Wavelength toggles (only when more than one wavelength).
        if wavelengths.len() > 1 {
            ui.horizontal(|ui| {
                for (i, &wl) in wavelengths.iter().enumerate() {
                    if let Some(v) = self.wavelength_visible.get_mut(i) {
                        ui.checkbox(v, format!("{wl:.4} \u{00b5}m"));
                    }
                }
            });
        }

        ui.separator();

        let n_fields = fields.len();
        if n_fields == 0 {
            ui.label("No fields defined.");
            return;
        }

        // Field plots in a row, each with its own bounding box.
        ui.horizontal(|ui| {
            for field_id in 0..n_fields {
                let field_label = fields
                    .get(field_id)
                    .map(|f| f.label.as_str())
                    .unwrap_or("—");
                let query = FieldPlotQuery {
                    ray_trace,
                    active_path,
                    wavelengths,
                    field_id,
                    path_step: selected_desc.and_then(|s| s.path_step),
                    wavelength_visible: &self.wavelength_visible,
                    surf_desc: selected_desc,
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

/// Resolves which surface to display: the previously-selected one if it's
/// still reachable from the active path (`path_step.is_some()`), else that
/// path's own Image surface (last in its own traversal), else its own last
/// reachable surface, else `None` if the active path reaches no surface at
/// all.
///
/// `surfaces` is the full, global surface list — not pre-filtered to the
/// active path — since `selected` (and the returned index) are global/store
/// indices into it; only reachability is path-scoped.
fn resolve_selected_surface(surfaces: &[SurfaceDesc], selected: Option<usize>) -> Option<usize> {
    let path_surfaces: Vec<&SurfaceDesc> =
        surfaces.iter().filter(|s| s.path_step.is_some()).collect();

    let image_surface_idx = path_surfaces
        .iter()
        .rev()
        .find(|s| s.label.starts_with("Image"))
        .or_else(|| path_surfaces.last())
        .map(|s| s.index);

    selected
        .filter(|idx| path_surfaces.iter().any(|s| s.index == *idx))
        .or(image_surface_idx)
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
        if let Some(tr) = query
            .ray_trace
            .get_for_path(query.active_path, query.field_id, wl_id)
        {
            for (x, y) in rays_at_surface(tr.full_pupil(), query.path_step, query.surf_desc) {
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

        if let Some(tr) = query
            .ray_trace
            .get_for_path(query.active_path, query.field_id, wl_id)
        {
            // Ray intersection scatter.
            for (rx, ry) in rays_at_surface(tr.full_pupil(), query.path_step, query.surf_desc) {
                let sp = to_screen(rx, ry);
                if rect.contains(sp) {
                    painter.circle_filled(sp, 2.0, color);
                }
            }

            // Chief ray: cross marker.
            for (cx, cy) in rays_at_surface(tr.chief_ray(), query.path_step, query.surf_desc) {
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

/// Extract (x, y) positions of non-terminated rays at `path_step` (this
/// path's own step position — see [`FieldPlotQuery::path_step`]),
/// projected into the surface's local coordinate frame.
///
/// `RayBundle` stores ray positions in global coordinates. For a tilted
/// surface (folded system), we rotate into the surface's local frame so that
/// x/y represent positions in the plane of the surface.
///
/// `RayBundle` stores rays as a flat
/// `[surface_0_rays … surface_N_rays]` array of length
/// `num_surfaces × num_rays_per_surface`, indexed by step position within
/// *this* bundle's own path — never by a global/store surface index.
fn rays_at_surface<'a>(
    bundle: &'a RayBundle,
    path_step: Option<usize>,
    surf_desc: Option<&'a SurfaceDesc>,
) -> impl Iterator<Item = (f64, f64)> + 'a {
    let total = bundle.rays().len();
    let n_surf = bundle.num_surfaces();
    let n_rays = if n_surf > 0 { total / n_surf } else { 0 };

    let (start, end) = match path_step {
        Some(idx) if n_surf > 0 && idx < n_surf && n_rays > 0 => (idx * n_rays, (idx + 1) * n_rays),
        _ => (0, 0),
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
        window.show(ctx, &mut open, result, 0);
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
        use crate::{ParaxialView, SequentialModelBuilder};

        let specs = SystemSpecs::default();
        #[cfg(not(feature = "ri-info"))]
        let parsed = convert::convert_specs(&specs).expect("convert");
        #[cfg(feature = "ri-info")]
        let parsed = convert::convert_specs(&specs, &Default::default()).expect("convert");
        let seq = SequentialModelBuilder::new()
            .paths(parsed.path_specs)
            .build()
            .expect("model")
            .model;
        let pv = ParaxialView::new(&seq, &parsed.field_specs_by_path, false).expect("paraxial");

        let wls = seq.wavelengths().to_vec();
        let result = ResultPackage {
            id: 1,
            wavelengths: wls.clone(),
            wavelengths_by_path: vec![wls],
            surfaces: Vec::new(),
            fields_by_path: vec![Vec::new()],
            field_specs_by_path: parsed.field_specs_by_path,
            paraxial: Some(pv),
            ray_trace: None,
            cross_section: None,
            error: Some("trace failed".to_string()),
            solved_values: Default::default(),
            components: Vec::new(),
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
        use crate::{ParaxialView, SequentialModelBuilder, ray_trace_3d_view};

        let mut specs = SystemSpecs::default();
        specs.paths[0].fields.push(FieldRow {
            chi: "5.0".into(),
            phi: "90.0".into(),
            x: "0.0".into(),
        });

        #[cfg(not(feature = "ri-info"))]
        let parsed = convert::convert_specs(&specs).expect("convert");
        #[cfg(feature = "ri-info")]
        let parsed = convert::convert_specs(&specs, &Default::default()).expect("convert");
        let seq = SequentialModelBuilder::new()
            .paths(parsed.path_specs)
            .build()
            .expect("model")
            .model;
        let pv = ParaxialView::new(&seq, &parsed.field_specs_by_path, false).expect("paraxial");
        let trace = ray_trace_3d_view(
            &parsed.aperture_specs_by_path,
            &parsed.field_specs_by_path,
            &seq,
            &pv,
            crate::views::ray_trace_3d::SamplingConfig {
                n_fan_rays: 3,
                full_pupil_spacing: 0.1,
            },
        )
        .expect("trace");

        let wls = seq.wavelengths().to_vec();
        let result = ResultPackage {
            id: 1,
            wavelengths: wls.clone(),
            wavelengths_by_path: vec![wls],
            surfaces: {
                seq.surfaces()
                    .iter()
                    .zip(seq.placements().iter())
                    .enumerate()
                    .map(|(i, (_s, p))| crate::gui::result_package::SurfaceDesc {
                        index: i,
                        label: format!("S{i}"),
                        pos: p.position,
                        rot_mat: p.rotation_matrix,
                        path_step: Some(i),
                    })
                    .collect()
            },
            fields_by_path: vec![vec![
                FieldDesc {
                    label: "0.000\u{00b0}".into(),
                },
                FieldDesc {
                    label: "5.000\u{00b0}".into(),
                },
            ]],
            field_specs_by_path: parsed.field_specs_by_path,
            paraxial: Some(pv),
            ray_trace: Some(trace),
            cross_section: None,
            error: None,
            solved_values: Default::default(),
            components: Vec::new(),
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

    /// Regression: the Spot Diagram window must resolve to the *active
    /// path's own* Image surface, not whichever surface happens to be last
    /// in the model's global/store-index order — a surface only reachable
    /// from a *different* path (`path_step: None`) must never be
    /// auto-selected or retained across an `active_path` switch.
    #[test]
    fn resolve_selected_surface_uses_active_paths_own_image() {
        use crate::core::math::{linalg::mat3x3::Mat3x3, vec3::Vec3};

        let dummy = |index: usize, label: &str, path_step: Option<usize>| SurfaceDesc {
            index,
            label: label.to_string(),
            pos: Vec3::new(0.0, 0.0, 0.0),
            rot_mat: Mat3x3::identity(),
            path_step,
        };

        // Store indices: 0=Object, 1=BeamSplitter (shared by both paths),
        // 2=Image_T (path 0's own), 3=Image_R (path 1's own — the global
        // last surface, which a naive "last Image in the list" heuristic
        // would wrongly pick even when path 0 is active).
        let surfaces_path0_active = vec![
            dummy(0, "Object [0]", Some(0)),
            dummy(1, "Beam Splitter [1]", Some(1)),
            dummy(2, "Image [2]", Some(2)),
            dummy(3, "Image [3]", None),
        ];
        assert_eq!(
            resolve_selected_surface(&surfaces_path0_active, None),
            Some(2),
            "path 0 active: must default to its own image, not path 1's"
        );

        let surfaces_path1_active = vec![
            dummy(0, "Object [0]", Some(0)),
            dummy(1, "Beam Splitter [1]", Some(1)),
            dummy(2, "Image [2]", None),
            dummy(3, "Image [3]", Some(2)),
        ];
        assert_eq!(
            resolve_selected_surface(&surfaces_path1_active, None),
            Some(3),
            "path 1 active: must default to its own image, not path 0's"
        );

        // A previous selection belonging to the now-inactive path must be
        // discarded, falling back to the active path's own image, rather
        // than silently returning an index the active path can't resolve
        // into its own RayBundle.
        assert_eq!(
            resolve_selected_surface(&surfaces_path0_active, Some(3)),
            Some(2),
            "a selection belonging to the inactive path must fall back to the active path's own image"
        );

        // An explicit selection that IS reachable from the active path is
        // honored as-is.
        assert_eq!(
            resolve_selected_surface(&surfaces_path0_active, Some(1)),
            Some(1),
            "an explicit, still-reachable selection should be kept"
        );
    }
}
