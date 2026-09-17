use std::collections::HashMap;

use crate::{
    FieldSpec, TraceResults,
    core::math::vec3::Vec3,
    gui::{
        colors::wavelength_to_color,
        result_package::{ResultPackage, SurfaceDesc},
    },
    views::ray_trace_3d::RayBundle,
};
use egui_plot::{HLine, Line, Plot, PlotPoints};

const PLOT_SIZE: f32 = 180.0;

/// `[pupil_coord, ta]` pairs for one (field, wavelength, fan-type) combination.
struct TaCurve {
    points: Vec<[f64; 2]>,
}

/// Precomputed TA data for one field column.
struct FieldTaData {
    /// True when at least one wavelength uses the paraxial chief-ray fallback.
    paraxial_fallback: bool,
    /// Tangential TA curves, keyed by wavelength_id.
    tangential: HashMap<usize, TaCurve>,
    /// Sagittal TA curves, keyed by wavelength_id.
    sagittal: HashMap<usize, TaCurve>,
}

/// Floating Ray Fan Plot window.
#[derive(Default)]
pub struct RayFanWindow {
    wavelength_visible: Vec<bool>,
    last_n_wavelengths: usize,
}

impl RayFanWindow {
    /// Show the Ray Fan Plot window, filtered to `active_path` (FR-OUT-1).
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        result: Option<&ResultPackage>,
        active_path: usize,
    ) {
        egui::Window::new("Ray Fan Plot")
            .open(open)
            .default_width(640.0)
            .show(ctx, |ui| {
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
                    Some(r) if r.ray_trace.is_none() => {
                        let msg = r.error.as_deref().unwrap_or("unknown");
                        ui.colored_label(
                            egui::Color32::RED,
                            format!("Ray trace unavailable: {msg}"),
                        );
                    }
                    Some(r) => self.render_content(ui, r, active_path),
                }
            });
    }

    fn render_content(&mut self, ui: &mut egui::Ui, r: &ResultPackage, active_path: usize) {
        let empty_fields = Vec::new();
        let empty_wls = Vec::new();
        let fields = r.fields_by_path.get(active_path).unwrap_or(&empty_fields);
        let wavelengths = r.wavelengths_by_path.get(active_path).unwrap_or(&empty_wls);
        let n_fields = fields.len();
        let n_wl = wavelengths.len();

        if n_fields == 0 {
            ui.label("No fields defined.");
            return;
        }

        // Wavelength toggles — outside the scroll area, always visible.
        if n_wl > 1 {
            ui.horizontal(|ui| {
                for (i, &wl) in wavelengths.iter().enumerate() {
                    if let Some(v) = self.wavelength_visible.get_mut(i) {
                        let color = wavelength_to_color(wl);
                        ui.checkbox(
                            v,
                            egui::RichText::new(format!("{wl:.4} \u{00b5}m")).color(color),
                        );
                    }
                }
            });
        }

        // Compute TA curves for all fields.
        let image_surf = active_path_image_surface(&r.surfaces);
        let ta_data: Vec<FieldTaData> = (0..n_fields)
            .map(|fid| compute_field_ta(r, active_path, fid, image_surf))
            .collect();

        // Shared Y scales: tangential and sagittal computed independently.
        let (tan_min, tan_max) =
            shared_y_scale(&ta_data, &self.wavelength_visible, |fd| &fd.tangential);
        let (sag_min, sag_max) =
            shared_y_scale(&ta_data, &self.wavelength_visible, |fd| &fd.sagittal);

        // Plot grid: row headers on left (fixed), scrollable field columns on right.
        ui.horizontal_top(|ui| {
            // Row headers — not inside the scroll area so they don't scroll away.
            ui.vertical(|ui| {
                // Spacer to align with the column-header row (~24 px).
                ui.add_space(24.0);
                for label in &["Tangential", "Sagittal"] {
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(70.0, PLOT_SIZE),
                        egui::Sense::hover(),
                    );
                    ui.painter_at(rect).text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        *label,
                        egui::FontId::proportional(12.0),
                        ui.visuals().text_color(),
                    );
                }
            });

            // Scrollable field columns.
            egui::ScrollArea::horizontal()
                .id_salt("ray_fan_scroll")
                .show(ui, |ui| {
                    ui.horizontal_top(|ui| {
                        for (fid, field_data) in ta_data.iter().enumerate() {
                            let field_label =
                                fields.get(fid).map(|f| f.label.as_str()).unwrap_or("\u{2014}");
                            ui.vertical(|ui| {
                                // Column header.
                                ui.horizontal(|ui| {
                                    ui.label(field_label);
                                    if field_data.paraxial_fallback {
                                        ui.label("\u{26a0}").on_hover_text(
                                            "Chief ray vignetted \u{2014} TA reference from paraxial model",
                                        );
                                    }
                                });

                                let plot_ctx = FanPlotCtx {
                                    wavelengths,
                                    wavelength_visible: &self.wavelength_visible,
                                };
                                draw_fan_plot(
                                    ui,
                                    format!("rf_tan_{fid}"),
                                    "Tangential TA (mm)",
                                    &field_data.tangential,
                                    &plot_ctx,
                                    tan_min,
                                    tan_max,
                                );
                                draw_fan_plot(
                                    ui,
                                    format!("rf_sag_{fid}"),
                                    "Sagittal TA (mm)",
                                    &field_data.sagittal,
                                    &plot_ctx,
                                    sag_min,
                                    sag_max,
                                );
                            });
                        }
                    });
                });
        });
    }
}

// ── Plot drawing
// ──────────────────────────────────────────────────────────────

struct FanPlotCtx<'a> {
    wavelengths: &'a [f64],
    wavelength_visible: &'a [bool],
}

fn draw_fan_plot(
    ui: &mut egui::Ui,
    id: String,
    y_label: &str,
    curves: &HashMap<usize, TaCurve>,
    ctx: &FanPlotCtx<'_>,
    y_min: f64,
    y_max: f64,
) {
    let FanPlotCtx {
        wavelengths,
        wavelength_visible,
    } = ctx;
    Plot::new(id)
        .width(PLOT_SIZE)
        .height(PLOT_SIZE)
        .x_axis_label("p")
        .y_axis_label(y_label)
        .include_x(-1.0)
        .include_x(1.0)
        .include_y(y_min)
        .include_y(y_max)
        .allow_zoom(false)
        .allow_drag(false)
        .allow_scroll(false)
        .allow_boxed_zoom(false)
        .allow_double_click_reset(false)
        .show(ui, |plot_ui| {
            // Zero reference line.
            plot_ui.hline(
                HLine::new("zero", 0.0)
                    .color(egui::Color32::from_gray(140))
                    .width(1.0),
            );

            for (wl_id, &visible) in wavelength_visible.iter().enumerate() {
                if !visible {
                    continue;
                }
                let Some(curve) = curves.get(&wl_id) else {
                    continue;
                };
                if curve.points.is_empty() {
                    continue;
                }
                let color = wavelengths
                    .get(wl_id)
                    .copied()
                    .map(wavelength_to_color)
                    .unwrap_or(egui::Color32::WHITE);
                let name = wavelengths
                    .get(wl_id)
                    .map(|w| format!("{w:.4} \u{00b5}m"))
                    .unwrap_or_default();
                plot_ui.line(
                    Line::new(name, PlotPoints::new(curve.points.clone()))
                        .color(color)
                        .width(1.5),
                );
            }
        });
}

// ── TA computation
// ────────────────────────────────────────────────────────────

/// Returns the active path's own Image surface — the one with the highest
/// `path_step` among surfaces reachable from this path (every path ends in
/// an Image surface, so this is always that path's own) — rather than
/// whichever surface happens to be last in the model's global/store-index
/// order, which may belong to a different path entirely.
fn active_path_image_surface(surfaces: &[SurfaceDesc]) -> Option<&SurfaceDesc> {
    surfaces
        .iter()
        .filter(|s| s.path_step.is_some())
        .max_by_key(|s| s.path_step)
}

/// Compute TA data for one field column across all wavelengths.
fn compute_field_ta(
    r: &ResultPackage,
    active_path: usize,
    field_id: usize,
    image_surf: Option<&SurfaceDesc>,
) -> FieldTaData {
    let empty = FieldTaData {
        paraxial_fallback: false,
        tangential: HashMap::new(),
        sagittal: HashMap::new(),
    };
    let Some(ray_trace) = &r.ray_trace else {
        return empty;
    };
    let Some(image_surf) = image_surf else {
        return empty;
    };
    let Some(field_specs) = r.field_specs_by_path.get(active_path) else {
        return empty;
    };

    let phi = field_specs
        .get(field_id)
        .map(|fs| fs.tangential_fan_phi())
        .unwrap_or(0.0);

    // Direction vectors in global frame: R_cursor_to_global = rot_mat^T.
    let rot_t = image_surf.rot_mat.transpose();
    let tan_dir = rot_t * Vec3::new(phi.cos(), phi.sin(), 0.0);
    let sag_dir = rot_t * Vec3::new(-phi.sin(), phi.cos(), 0.0);

    let mut tangential: HashMap<usize, TaCurve> = HashMap::new();
    let mut sagittal: HashMap<usize, TaCurve> = HashMap::new();
    let mut paraxial_fallback = false;

    let n_wl = r
        .wavelengths_by_path
        .get(active_path)
        .map(|w| w.len())
        .unwrap_or(0);
    for wl_id in 0..n_wl {
        let Some(tr) = ray_trace.get_for_path(active_path, field_id, wl_id) else {
            continue;
        };

        let (chief_pos, used_fallback) =
            chief_ray_image_pos(tr, r, active_path, field_id, wl_id, image_surf, phi);
        let Some(chief_pos) = chief_pos else {
            continue;
        };
        if used_fallback {
            paraxial_fallback = true;
        }

        tangential.insert(
            wl_id,
            fan_ta_curve(tr.tangential_fan(), &chief_pos, &tan_dir),
        );
        sagittal.insert(wl_id, fan_ta_curve(tr.sagittal_fan(), &chief_pos, &sag_dir));
    }

    FieldTaData {
        paraxial_fallback,
        tangential,
        sagittal,
    }
}

/// Return the chief ray 3D position at the image surface and whether the
/// paraxial fallback was used.
fn chief_ray_image_pos(
    tr: &TraceResults,
    r: &ResultPackage,
    active_path: usize,
    field_id: usize,
    wl_id: usize,
    image_surf: &SurfaceDesc,
    phi: f64,
) -> (Option<Vec3>, bool) {
    if tr.chief_ray_reached_image() {
        let bundle = tr.chief_ray();
        let n_surf = bundle.num_surfaces();
        if n_surf == 0 {
            return (None, false);
        }
        let rays = bundle.rays();
        // Chief ray bundle has exactly 1 ray; rays are stored flat with 1 per surface.
        let Some(ray) = rays.get(n_surf - 1) else {
            return (None, false);
        };
        (Some(Vec3::new(ray.x(), ray.y(), ray.z())), false)
    } else {
        // Paraxial fallback.
        let Some(pv) = &r.paraxial else {
            return (None, false);
        };
        let tangential_vec_id = pv.tangential_vec_id_for_phi(active_path, phi);
        let Some(sv) = pv.get_for_path(active_path, wl_id, tangential_vec_id) else {
            return (None, false);
        };

        let Some(last) = sv.chief_ray().last_surface() else {
            return (None, false);
        };
        let y_max = last.first().map(|pr| pr.height).unwrap_or(0.0);

        let field_specs = r.field_specs_by_path.get(active_path);
        let ratio = field_specs
            .and_then(|fs| fs.get(field_id).map(|f| (f, fs)))
            .map(|(fs, all)| field_height_ratio(fs, all, phi))
            .unwrap_or(0.0);
        let y_fallback = ratio * y_max;

        // Lift scalar paraxial height to a 3D position along the tangential direction.
        let tv = pv.tangential_vec(tangential_vec_id);
        let rot_t = image_surf.rot_mat.transpose();
        let tan_global = rot_t * tv;
        let pos = image_surf.pos + tan_global * y_fallback;
        (Some(pos), true)
    }
}

/// Ratio `height_i / height_max` for the paraxial chief-ray fallback scaling.
fn field_height_ratio(fs: &FieldSpec, all_fields: &[FieldSpec], phi: f64) -> f64 {
    const EPS: f64 = 1e-9;
    match fs {
        FieldSpec::Angle { chi, .. } => {
            let chi_max = all_fields
                .iter()
                .filter(|f| (f.tangential_fan_phi() - phi).abs() < EPS)
                .filter_map(|f| match f {
                    FieldSpec::Angle { chi, .. } => Some(chi.abs()),
                    _ => None,
                })
                .fold(0.0_f64, f64::max);
            if chi_max.abs() < EPS {
                return 0.0;
            }
            chi.to_radians().tan() / chi_max.to_radians().tan()
        }
        FieldSpec::PointSource { x, y } => {
            let r_i = (x.powi(2) + y.powi(2)).sqrt();
            let r_max = all_fields
                .iter()
                .filter_map(|f| match f {
                    FieldSpec::PointSource { x, y } => Some((x.powi(2) + y.powi(2)).sqrt()),
                    _ => None,
                })
                .fold(0.0_f64, f64::max);
            if r_max < EPS {
                return 0.0;
            }
            r_i / r_max
        }
    }
}

/// Extract TA values from a fan bundle at the image surface (last surface).
fn fan_ta_curve(bundle: &RayBundle, chief_pos: &Vec3, dir: &Vec3) -> TaCurve {
    let total = bundle.rays().len();
    let n_surf = bundle.num_surfaces();
    if n_surf == 0 {
        return TaCurve { points: Vec::new() };
    }
    let n_rays = total / n_surf;
    if n_rays == 0 {
        return TaCurve { points: Vec::new() };
    }
    let image_surf_idx = n_surf - 1;
    let start = image_surf_idx * n_rays;
    let rays = bundle.rays();
    let terminated = bundle.terminated();

    let mut points = Vec::with_capacity(n_rays);
    for i in 0..n_rays {
        // Skip terminated rays.
        if terminated.get(i).copied().unwrap_or(0) > 0 {
            continue;
        }
        let Some(ray) = rays.get(start + i) else {
            continue;
        };
        let p = if n_rays > 1 {
            -1.0 + 2.0 * i as f64 / (n_rays - 1) as f64
        } else {
            0.0
        };
        let ray_pos = Vec3::new(ray.x(), ray.y(), ray.z());
        let delta = ray_pos - *chief_pos;
        let ta = delta.dot(dir);
        points.push([p, ta]);
    }
    TaCurve { points }
}

/// Shared [min, max] Y range for all visible wavelengths across all field
/// columns. Adds ~15% padding. Falls back to a tiny default range when no data.
fn shared_y_scale<F>(ta_data: &[FieldTaData], wavelength_visible: &[bool], get_map: F) -> (f64, f64)
where
    F: Fn(&FieldTaData) -> &HashMap<usize, TaCurve>,
{
    let mut y_min = f64::MAX;
    let mut y_max = f64::MIN;
    for fd in ta_data {
        for (wl_id, &visible) in wavelength_visible.iter().enumerate() {
            if !visible {
                continue;
            }
            if let Some(curve) = get_map(fd).get(&wl_id) {
                for &[_, ta] in &curve.points {
                    y_min = y_min.min(ta);
                    y_max = y_max.max(ta);
                }
            }
        }
    }
    if y_min > y_max {
        return (-1e-6, 1e-6);
    }
    let span = (y_max - y_min).max(1e-9);
    let pad = span * 0.15;
    (y_min - pad, y_max + pad)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{Harness, kittest::Queryable};

    use crate::gui::result_package::{FieldDesc, ResultPackage};

    /// Build a minimal ResultPackage with the given wavelengths and a single
    /// on-axis angle field. Surfaces and ray trace are omitted so the window
    /// falls through to the "ray trace unavailable" branch — sufficient for
    /// label / toggle tests.
    fn make_minimal(wavelengths: &[f64]) -> ResultPackage {
        ResultPackage {
            id: 1,
            wavelengths: wavelengths.to_vec(),
            wavelengths_by_path: vec![wavelengths.to_vec()],
            surfaces: Vec::new(),
            fields_by_path: vec![vec![FieldDesc {
                label: "\u{03c7}=0.000\u{00b0}, \u{03c6}=90.000\u{00b0}".to_string(),
            }]],
            field_specs_by_path: vec![vec![crate::FieldSpec::Angle {
                chi: 0.0,
                phi: 90.0,
            }]],
            paraxial: None,
            ray_trace: None,
            cross_section: None,
            error: None,
            solved_values: Default::default(),
            components: Vec::new(),
        }
    }

    /// Build a full ResultPackage (with ray trace) for the given wavelengths
    /// using the default SystemSpecs (convexplano-like default lens).
    fn make_result(wavelengths: &[&str]) -> ResultPackage {
        use crate::gui::{convert, model::SystemSpecs};
        use crate::{
            ParaxialView, SequentialModelBuilder, ray_trace_3d_view,
            views::ray_trace_3d::SamplingConfig,
        };

        let mut specs = SystemSpecs::default();
        specs.paths[0].wavelengths = wavelengths.iter().map(|s| s.to_string()).collect();
        specs.n_fan_rays = 11;
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
        let config = SamplingConfig {
            n_fan_rays: 11,
            full_pupil_spacing: 0.1,
        };
        let trace = ray_trace_3d_view(
            &parsed.aperture_specs_by_path,
            &parsed.field_specs_by_path,
            &seq,
            &pv,
            config,
        )
        .ok();
        let wls = seq.wavelengths().to_vec();

        // Build surface descs manually (mirrors compute.rs logic).
        use crate::{SurfaceKind, gui::result_package::SurfaceDesc};
        let surfaces: Vec<SurfaceDesc> = seq
            .surfaces()
            .iter()
            .zip(seq.placements().iter())
            .enumerate()
            .map(|(i, (s, p))| {
                let name = match s.surface_kind() {
                    SurfaceKind::BeamSplitter => "Beam Splitter",
                    SurfaceKind::Conic => "Conic",
                    SurfaceKind::Image => "Image",
                    SurfaceKind::Object => "Object",
                    SurfaceKind::Probe => "Probe",
                    SurfaceKind::Iris => "Iris",
                    SurfaceKind::Sphere => "Sphere",
                    SurfaceKind::ThinLens => "Thin Lens",
                    SurfaceKind::Custom => "Custom",
                };
                SurfaceDesc {
                    index: i,
                    label: format!("{name} [{i}]"),
                    pos: p.position,
                    rot_mat: p.rotation_matrix,
                    path_step: Some(i),
                }
            })
            .collect();

        let fields: Vec<FieldDesc> = parsed.field_specs_by_path[0]
            .iter()
            .map(|f| {
                let label = match f {
                    crate::FieldSpec::Angle { chi, phi } => {
                        format!("\u{03c7}={chi:.3}\u{00b0}, \u{03c6}={phi:.3}\u{00b0}")
                    }
                    crate::FieldSpec::PointSource { x, y } => format!("({x}, {y}) mm"),
                };
                FieldDesc { label }
            })
            .collect();

        ResultPackage {
            id: 1,
            wavelengths: wls.clone(),
            wavelengths_by_path: vec![wls],
            surfaces,
            fields_by_path: vec![fields],
            field_specs_by_path: parsed.field_specs_by_path,
            paraxial: Some(pv),
            ray_trace: trace,
            cross_section: None,
            error: None,
            solved_values: Default::default(),
            components: Vec::new(),
        }
    }

    #[test]
    fn no_result_shows_placeholder() {
        let mut window = RayFanWindow::default();
        let mut harness = Harness::new(|ctx| {
            let mut open = true;
            window.show(ctx, &mut open, None, 0);
        });
        harness.step();
        harness.get_by_label("No data yet.");
    }

    #[test]
    fn with_result_shows_field_label() {
        let result = make_result(&["0.567"]);
        let mut harness = Harness::new_state(
            |ctx, (w, r): &mut (RayFanWindow, ResultPackage)| {
                let mut open = true;
                w.show(ctx, &mut open, Some(r), 0);
            },
            (RayFanWindow::default(), result),
        );
        harness.step();
        harness.get_by_label_contains("\u{03c7}=");
    }

    #[test]
    fn single_wavelength_no_toggle_row() {
        let result = make_minimal(&[0.567]);
        let mut harness = Harness::new_state(
            |ctx, (w, r): &mut (RayFanWindow, ResultPackage)| {
                let mut open = true;
                w.show(ctx, &mut open, Some(r), 0);
            },
            (RayFanWindow::default(), result),
        );
        harness.step();
        assert!(
            harness.query_by_label_contains("\u{00b5}m").is_none(),
            "wavelength toggle row must not appear for a single wavelength"
        );
    }

    #[test]
    fn multi_wavelength_shows_toggle_row() {
        let result = make_result(&["0.486", "0.587", "0.656"]);
        let mut harness = Harness::new_state(
            |ctx, (w, r): &mut (RayFanWindow, ResultPackage)| {
                let mut open = true;
                w.show(ctx, &mut open, Some(r), 0);
            },
            (RayFanWindow::default(), result),
        );
        harness.step();
        // The toggle row contains wavelength labels; assert at least one per
        // wavelength.
        assert!(
            harness
                .query_all_by_label_contains("0.4860")
                .next()
                .is_some()
        );
        assert!(
            harness
                .query_all_by_label_contains("0.5870")
                .next()
                .is_some()
        );
        assert!(
            harness
                .query_all_by_label_contains("0.6560")
                .next()
                .is_some()
        );
    }

    /// Regression: TA computation must use the *active path's own* Image
    /// surface, not whichever surface happens to be last in the model's
    /// global/store-index order — a surface only reachable from a
    /// *different* path (`path_step: None`) must never be picked.
    #[test]
    fn active_path_image_surface_uses_active_paths_own_image() {
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
        // last surface, which `.last()` would wrongly pick even when path 0
        // is active).
        let surfaces_path0_active = vec![
            dummy(0, "Object [0]", Some(0)),
            dummy(1, "Beam Splitter [1]", Some(1)),
            dummy(2, "Image [2]", Some(2)),
            dummy(3, "Image [3]", None),
        ];
        assert_eq!(
            active_path_image_surface(&surfaces_path0_active).map(|s| s.index),
            Some(2),
            "path 0 active: must use its own image, not path 1's"
        );

        let surfaces_path1_active = vec![
            dummy(0, "Object [0]", Some(0)),
            dummy(1, "Beam Splitter [1]", Some(1)),
            dummy(2, "Image [2]", None),
            dummy(3, "Image [3]", Some(2)),
        ];
        assert_eq!(
            active_path_image_surface(&surfaces_path1_active).map(|s| s.index),
            Some(3),
            "path 1 active: must use its own image, not path 0's"
        );
    }
}
