use crate::{
    gui::{colors::wavelength_to_color, result_package::ResultPackage},
    views::cross_section::{Bounds2D, CrossSectionView, DrawElement, FlatPlaneKind, PlaneGeometry},
};

const VIEWPORT_HEIGHT_RATIO: f32 = 0.5;
const MAX_CROSS_SECTION_N_RAYS: u32 = 32;
const MIN_VIEWPORT_HEIGHT: f32 = 200.0;
const SCALEBAR_MARGIN: f32 = 8.0;
const SCALEBAR_HEIGHT: f32 = 4.0;

/// SVG canvas dimensions in logical pixels (for file export only).
const SVG_W: f64 = 700.0;
const SVG_H: f64 = 350.0;
/// Padding on each side (pixels). 5 % per side ≈ 10 % total margin.
const SVG_PAD: f64 = 35.0;

// ── Cutting plane
// ─────────────────────────────────────────────────────────────

/// Which 2D cutting plane to display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CuttingPlane {
    #[default]
    YZ,
    XZ,
}

// ── Window struct
// ─────────────────────────────────────────────────────────────

/// Cross-section output window.
pub struct CrossSectionWindow {
    cutting_plane: CuttingPlane,
}

impl Default for CrossSectionWindow {
    fn default() -> Self {
        Self {
            cutting_plane: CuttingPlane::YZ,
        }
    }
}

impl CrossSectionWindow {
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        result: Option<&ResultPackage>,
        n_rays: &mut u32,
    ) -> bool {
        let mut changed = false;
        egui::Window::new("Cross Section")
            .open(open)
            .default_width(700.0)
            .min_width(300.0)
            .resizable(true)
            .show(ctx, |ui| {
                changed = self.show_content(ui, result, n_rays);
            });
        changed
    }

    fn show_content(
        &mut self,
        ui: &mut egui::Ui,
        result: Option<&ResultPackage>,
        n_rays: &mut u32,
    ) -> bool {
        let Some(result) = result else {
            self.show_empty_viewport(ui);
            return false;
        };

        let cs_data = result.cross_section.as_ref();

        // Controls.
        let changed = self.show_controls(ui, cs_data, n_rays);
        ui.separator();

        // Viewport.
        match cs_data {
            Some(cs) if cs.yz_valid || cs.xz_valid => {
                self.show_viewport(ui, cs, result.wavelengths.as_slice());
            }
            Some(_) => {
                self.show_invalid_plane_message(ui);
            }
            None => {
                self.show_empty_viewport(ui);
            }
        }

        changed
    }

    fn show_controls(
        &mut self,
        ui: &mut egui::Ui,
        cs_data: Option<&CrossSectionView>,
        n_rays: &mut u32,
    ) -> bool {
        // Auto-select the valid plane.
        if let Some(cs) = cs_data {
            if self.cutting_plane == CuttingPlane::YZ && !cs.yz_valid && cs.xz_valid {
                self.cutting_plane = CuttingPlane::XZ;
            } else if self.cutting_plane == CuttingPlane::XZ && !cs.xz_valid && cs.yz_valid {
                self.cutting_plane = CuttingPlane::YZ;
            }
        }

        let yz_valid = cs_data.is_none_or(|cs| cs.yz_valid);
        let xz_valid = cs_data.is_none_or(|cs| cs.xz_valid);

        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Plane:");
            ui.add_enabled_ui(yz_valid, |ui| {
                ui.radio_value(&mut self.cutting_plane, CuttingPlane::YZ, "YZ");
            });
            ui.add_enabled_ui(xz_valid, |ui| {
                ui.radio_value(&mut self.cutting_plane, CuttingPlane::XZ, "XZ");
            });
            ui.separator();
            ui.label("Rays:");
            let resp = ui.add(
                egui::DragValue::new(n_rays)
                    .range(0_u32..=MAX_CROSS_SECTION_N_RAYS)
                    .speed(1.0),
            );
            if resp.changed() {
                changed = true;
            }
        });
        changed
    }

    fn show_viewport(&self, ui: &mut egui::Ui, cs: &CrossSectionView, wavelengths: &[f64]) {
        let geom = match self.cutting_plane {
            CuttingPlane::YZ => &cs.yz,
            CuttingPlane::XZ => &cs.xz,
        };

        let available = ui.available_size();
        let height = (available.x * VIEWPORT_HEIGHT_RATIO).max(MIN_VIEWPORT_HEIGHT);
        let viewport_size = egui::vec2(available.x, height);
        let (rect, _) = ui.allocate_exact_size(viewport_size, egui::Sense::hover());

        // Background.
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, ui.visuals().extreme_bg_color);
        painter.rect_stroke(
            rect,
            0.0,
            ui.visuals().window_stroke(),
            egui::StrokeKind::Outside,
        );

        let w2s = WorldToScreen::new(&geom.bounding_box, rect);

        // Draw elements.
        for elem in &geom.elements {
            draw_element(&painter, elem, &w2s, ui.visuals());
        }

        // Draw rays.
        for (wl_idx, paths) in geom.ray_paths.iter().enumerate() {
            let color = wavelengths
                .get(wl_idx)
                .copied()
                .map(wavelength_to_color)
                .unwrap_or(egui::Color32::WHITE);
            draw_rays(&painter, paths, &w2s, color);
        }

        // Draw scale bar.
        draw_scalebar(&painter, rect, &geom.bounding_box);
    }

    fn show_invalid_plane_message(&self, ui: &mut egui::Ui) {
        let available = ui.available_size();
        let height = (available.x * VIEWPORT_HEIGHT_RATIO).max(MIN_VIEWPORT_HEIGHT);
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(available.x, height), egui::Sense::hover());
        ui.painter_at(rect).rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.0, egui::Color32::from_gray(180)),
            egui::StrokeKind::Outside,
        );
        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            ui.centered_and_justified(|ui| {
                ui.label(
                    "The optical axis leaves both coordinate planes.\n\
                     The cross-section view only supports systems\n\
                     whose axis lies in the YZ or XZ plane.",
                );
            });
        });
    }

    fn show_empty_viewport(&self, ui: &mut egui::Ui) {
        let available = ui.available_size();
        let height = (available.x * VIEWPORT_HEIGHT_RATIO).max(MIN_VIEWPORT_HEIGHT);
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(available.x, height), egui::Sense::hover());
        ui.painter_at(rect).rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.0, egui::Color32::from_gray(100)),
            egui::StrokeKind::Outside,
        );
        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            ui.centered_and_justified(|ui| {
                ui.label("No data yet.");
            });
        });
    }

    /// Generate an SVG string of the current cross-section view for file
    /// export.
    ///
    /// Returns `None` if the selected cutting plane is not valid.
    pub fn export_svg_string(&self, cs: &CrossSectionView, dark_mode: bool) -> Option<String> {
        let geom = match self.cutting_plane {
            CuttingPlane::YZ if cs.yz_valid => &cs.yz,
            CuttingPlane::XZ if cs.xz_valid => &cs.xz,
            _ => return None,
        };
        Some(render_svg(geom, &cs.wavelengths, dark_mode))
    }
}

// ── WorldToScreen (egui Painter coordinate transform)
// ────────────────────────────────────

/// Maps world (z, transverse) coordinates to egui screen positions.
///
/// World: z increases right, transverse increases upward.
/// Screen: x increases right, y increases downward (transverse axis negated).
struct WorldToScreen {
    z_min: f32,
    z_range: f32,
    t_min: f32,
    t_range: f32,
    /// Inner rect after centering and padding (in screen pixels).
    inner: egui::Rect,
}

impl WorldToScreen {
    fn new(bb: &Bounds2D, rect: egui::Rect) -> Self {
        let padding = 8.0_f32;
        let z_min = bb.z.0 as f32;
        let z_max = bb.z.1 as f32;
        let t_min = bb.transverse.0 as f32;
        let t_max = bb.transverse.1 as f32;
        let z_range = (z_max - z_min).max(f32::EPSILON);
        let t_range = (t_max - t_min).max(f32::EPSILON);

        // Preserve aspect ratio by using the smaller scale factor.
        let scale_z = (rect.width() - 2.0 * padding) / z_range;
        let scale_t = (rect.height() - 2.0 * padding) / t_range;
        let scale = scale_z.min(scale_t);

        // Center within the padded rect.
        let z_screen = z_range * scale;
        let t_screen = t_range * scale;
        let z_offset = rect.left() + padding + (rect.width() - 2.0 * padding - z_screen) / 2.0;
        let t_offset = rect.top() + padding + (rect.height() - 2.0 * padding - t_screen) / 2.0;

        Self {
            z_min,
            z_range,
            t_min,
            t_range,
            inner: egui::Rect::from_min_size(
                egui::pos2(z_offset, t_offset),
                egui::vec2(z_screen, t_screen),
            ),
        }
    }

    #[inline]
    fn map(&self, z: f32, transverse: f32) -> egui::Pos2 {
        let tx = (z - self.z_min) / self.z_range;
        let ty = (transverse - self.t_min) / self.t_range;
        egui::pos2(
            self.inner.left() + tx * self.inner.width(),
            self.inner.bottom() - ty * self.inner.height(), // y-flip
        )
    }
}

// ── egui Painter drawing functions
// ───────────────────────────────────────────

fn draw_element(
    painter: &egui::Painter,
    elem: &DrawElement,
    w2s: &WorldToScreen,
    visuals: &egui::Visuals,
) {
    match elem {
        DrawElement::LensGroup {
            front_pts,
            back_pts,
        } => draw_lens_group(painter, front_pts, back_pts, w2s, visuals),
        DrawElement::SurfaceProfile { points } => {
            draw_surface_profile(painter, points, w2s);
        }
        DrawElement::Iris {
            z,
            half_gap,
            extent,
        } => {
            draw_stop(painter, *z as f32, *half_gap as f32, *extent as f32, w2s);
        }
        DrawElement::FlatPlane { p1, p2, kind } => {
            draw_flat_plane(painter, *p1, *p2, *kind, w2s);
        }
    }
}

fn draw_lens_group(
    painter: &egui::Painter,
    front_pts: &[[f64; 2]],
    back_pts: &[[f64; 2]],
    w2s: &WorldToScreen,
    visuals: &egui::Visuals,
) {
    let n = front_pts.len().min(back_pts.len());
    if n < 2 {
        return;
    }

    let fill = if visuals.dark_mode {
        egui::Color32::from_rgba_premultiplied(50, 100, 160, 80)
    } else {
        egui::Color32::from_rgba_premultiplied(100, 160, 220, 80)
    };

    // Zipper-triangulate the band between the two surface curves.
    // Front vertices at indices 0..n-1; back vertices at indices n..2n-1.
    // Both curves are sampled bottom-to-top, so front[i] and back[i] share
    // the same transverse parameter — quads connect them correctly regardless
    // of concavity.
    let mut mesh = egui::Mesh::default();
    for &[z, t] in front_pts.iter().take(n) {
        mesh.colored_vertex(w2s.map(z as f32, t as f32), fill);
    }
    for &[z, t] in back_pts.iter().take(n) {
        mesh.colored_vertex(w2s.map(z as f32, t as f32), fill);
    }
    for i in 0..(n as u32 - 1) {
        let fi = i;
        let fi1 = i + 1;
        let bi = n as u32 + i;
        let bi1 = n as u32 + i + 1;
        mesh.indices.extend_from_slice(&[fi, fi1, bi]);
        mesh.indices.extend_from_slice(&[fi1, bi1, bi]);
    }
    painter.add(egui::Shape::mesh(mesh));

    let stroke_color = if visuals.dark_mode {
        egui::Color32::from_rgb(100, 149, 220)
    } else {
        egui::Color32::from_rgb(30, 80, 160)
    };
    let stroke = egui::Stroke::new(1.5, stroke_color);

    let front_screen: Vec<egui::Pos2> = front_pts
        .iter()
        .map(|&[z, t]| w2s.map(z as f32, t as f32))
        .collect();
    let back_screen: Vec<egui::Pos2> = back_pts
        .iter()
        .map(|&[z, t]| w2s.map(z as f32, t as f32))
        .collect();

    for pair in front_screen.windows(2) {
        painter.line_segment([pair[0], pair[1]], stroke);
    }
    for pair in back_screen.windows(2) {
        painter.line_segment([pair[0], pair[1]], stroke);
    }
    // Top and bottom rims.
    if let (Some(&ft), Some(&bt)) = (front_screen.last(), back_screen.last()) {
        painter.line_segment([ft, bt], stroke);
    }
    if let (Some(&fb), Some(&bb)) = (front_screen.first(), back_screen.first()) {
        painter.line_segment([fb, bb], stroke);
    }
}

fn draw_surface_profile(painter: &egui::Painter, points: &[[f64; 2]], w2s: &WorldToScreen) {
    if points.len() < 2 {
        return;
    }
    let stroke = egui::Stroke::new(1.5, egui::Color32::from_rgb(200, 120, 50));
    let screen_pts: Vec<egui::Pos2> = points
        .iter()
        .map(|&[z, t]| w2s.map(z as f32, t as f32))
        .collect();
    for pair in screen_pts.windows(2) {
        painter.line_segment([pair[0], pair[1]], stroke);
    }
}

fn draw_stop(painter: &egui::Painter, z: f32, half_gap: f32, extent: f32, w2s: &WorldToScreen) {
    let stroke = egui::Stroke::new(2.0, egui::Color32::from_gray(160));
    painter.line_segment([w2s.map(z, extent), w2s.map(z, half_gap)], stroke);
    painter.line_segment([w2s.map(z, -half_gap), w2s.map(z, -extent)], stroke);
}

fn draw_flat_plane(
    painter: &egui::Painter,
    p1: [f64; 2],
    p2: [f64; 2],
    kind: FlatPlaneKind,
    w2s: &WorldToScreen,
) {
    let (color, width) = match kind {
        FlatPlaneKind::Image => (egui::Color32::from_rgb(0, 200, 100), 2.0),
        FlatPlaneKind::Probe => (egui::Color32::from_rgb(200, 200, 0), 1.0),
        FlatPlaneKind::Object => (egui::Color32::from_gray(150), 1.0),
    };
    painter.line_segment(
        [
            w2s.map(p1[0] as f32, p1[1] as f32),
            w2s.map(p2[0] as f32, p2[1] as f32),
        ],
        egui::Stroke::new(width, color),
    );
}

fn draw_rays(
    painter: &egui::Painter,
    paths: &[Vec<[f64; 2]>],
    w2s: &WorldToScreen,
    color: egui::Color32,
) {
    let stroke = egui::Stroke::new(1.0, color);
    for path in paths {
        if path.len() < 2 {
            continue;
        }
        let screen: Vec<egui::Pos2> = path
            .iter()
            .map(|&[z, t]| w2s.map(z as f32, t as f32))
            .collect();
        painter.add(egui::Shape::line(screen, stroke));
    }
}

fn draw_scalebar(painter: &egui::Painter, rect: egui::Rect, bb: &Bounds2D) {
    let world_width = (bb.z.1 - bb.z.0) as f32;
    if world_width <= 0.0 {
        return;
    }
    let target = world_width * 0.15;
    let magnitude = 10_f32.powf(target.log10().floor());
    let normalized = target / magnitude;
    let nice = if normalized >= 5.0 {
        5.0
    } else if normalized >= 2.0 {
        2.0
    } else {
        1.0
    };
    let nice_len = nice * magnitude;
    let bar_px = nice_len * rect.width() / world_width;

    let bar_right = rect.right() - SCALEBAR_MARGIN;
    let bar_left = bar_right - bar_px;
    let bar_y = rect.bottom() - SCALEBAR_MARGIN;

    let color = egui::Color32::from_gray(200);
    let stroke = egui::Stroke::new(2.0, color);

    painter.line_segment(
        [egui::pos2(bar_left, bar_y), egui::pos2(bar_right, bar_y)],
        stroke,
    );
    painter.line_segment(
        [
            egui::pos2(bar_left, bar_y - SCALEBAR_HEIGHT),
            egui::pos2(bar_left, bar_y),
        ],
        stroke,
    );
    painter.line_segment(
        [
            egui::pos2(bar_right, bar_y - SCALEBAR_HEIGHT),
            egui::pos2(bar_right, bar_y),
        ],
        stroke,
    );

    let label = if nice_len >= 1.0 {
        format!("{:.0} mm", nice_len)
    } else {
        format!("{:.2} mm", nice_len)
    };
    painter.text(
        egui::pos2((bar_left + bar_right) / 2.0, bar_y - SCALEBAR_HEIGHT - 2.0),
        egui::Align2::CENTER_BOTTOM,
        label,
        egui::FontId::proportional(10.0),
        color,
    );
}

// ── SVG export
// ──────────────────────────────────────────────────────────────────
//
// The functions below generate an SVG string for file export only.
// On-screen rendering uses egui::Painter (see above).

/// Maps world (z, transverse) to SVG pixel (x, y) for the fixed export canvas.
struct WorldToSvg {
    z_center: f64,
    t_center: f64,
    scale: f64, // SVG px per world unit
    cx: f64,    // SVG x at world z = 0
    cy: f64,    // SVG y at world t = 0
}

impl WorldToSvg {
    fn new(bb: &Bounds2D) -> Self {
        let z_range = (bb.z.1 - bb.z.0).max(f64::EPSILON);
        let t_range = (bb.transverse.1 - bb.transverse.0).max(f64::EPSILON);
        let scale = ((SVG_W - 2.0 * SVG_PAD) / z_range).min((SVG_H - 2.0 * SVG_PAD) / t_range);
        Self {
            z_center: (bb.z.0 + bb.z.1) / 2.0,
            t_center: (bb.transverse.0 + bb.transverse.1) / 2.0,
            scale,
            cx: SVG_W / 2.0,
            cy: SVG_H / 2.0,
        }
    }

    #[inline]
    fn map(&self, z: f64, t: f64) -> (f64, f64) {
        (
            self.cx + (z - self.z_center) * self.scale,
            self.cy - (t - self.t_center) * self.scale, // y-flip
        )
    }

    #[inline]
    fn len(&self, world_len: f64) -> f64 {
        world_len.abs() * self.scale
    }
}

fn render_svg(geom: &PlaneGeometry, wavelengths: &[f64], dark_mode: bool) -> String {
    let w2s = WorldToSvg::new(&geom.bounding_box);

    let bg = if dark_mode { "#1e1e2e" } else { "#f5f5f5" };
    let border = if dark_mode { "#4a4a5a" } else { "#b4b4c8" };
    let lens_fill = if dark_mode {
        "rgba(50,100,160,0.31)"
    } else {
        "rgba(100,160,220,0.31)"
    };
    let lens_stroke = if dark_mode { "#6495dc" } else { "#1e50a0" };
    let profile_color = "#c87832";
    let stop_color = if dark_mode { "#a0a0a0" } else { "#606060" };
    let scalebar_color = if dark_mode { "#c8c8c8" } else { "#505050" };

    let w = SVG_W as u32;
    let h = SVG_H as u32;

    let mut s = format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}">"#);
    s.push_str(&format!(r#"<rect width="{w}" height="{h}" fill="{bg}"/>"#));
    s.push_str(&format!(
        r#"<rect width="{w}" height="{h}" fill="none" stroke="{border}" stroke-width="1"/>"#
    ));

    for elem in &geom.elements {
        match elem {
            DrawElement::LensGroup {
                front_pts,
                back_pts,
            } => {
                svg_lens_group(&mut s, front_pts, back_pts, &w2s, lens_fill, lens_stroke);
            }
            DrawElement::SurfaceProfile { points } => {
                svg_polyline(&mut s, points, &w2s, profile_color, 1.5);
            }
            DrawElement::Iris {
                z,
                half_gap,
                extent,
            } => {
                svg_stop(&mut s, *z, *half_gap, *extent, &w2s, stop_color);
            }
            DrawElement::FlatPlane { p1, p2, kind } => {
                svg_flat_plane(&mut s, *p1, *p2, *kind, &w2s);
            }
        }
    }

    for (wl_idx, paths) in geom.ray_paths.iter().enumerate() {
        let color = wavelengths
            .get(wl_idx)
            .copied()
            .map(|wl| color_to_hex(wavelength_to_color(wl)))
            .unwrap_or_else(|| {
                if dark_mode {
                    "#ffffff".to_owned()
                } else {
                    "#000000".to_owned()
                }
            });
        for path in paths {
            svg_polyline(&mut s, path, &w2s, &color, 1.0);
        }
    }

    svg_scalebar(&mut s, &geom.bounding_box, &w2s, scalebar_color);

    s.push_str("</svg>");
    s
}

fn svg_lens_group(
    s: &mut String,
    front_pts: &[[f64; 2]],
    back_pts: &[[f64; 2]],
    w2s: &WorldToSvg,
    fill: &str,
    stroke: &str,
) {
    if front_pts.is_empty() || back_pts.is_empty() {
        return;
    }
    // Outline: front bottom→top, then back top→bottom. SVG's nonzero fill rule
    // handles non-convex polygons correctly.
    let pts: String = front_pts
        .iter()
        .chain(back_pts.iter().rev())
        .map(|&[z, t]| {
            let (x, y) = w2s.map(z, t);
            format!("{x:.2},{y:.2}")
        })
        .collect::<Vec<_>>()
        .join(" ");
    s.push_str(&format!(
        r#"<polygon points="{pts}" fill="{fill}" stroke="{stroke}" stroke-width="1.5" stroke-linejoin="round"/>"#
    ));
}

fn svg_polyline(s: &mut String, points: &[[f64; 2]], w2s: &WorldToSvg, stroke: &str, width: f64) {
    if points.len() < 2 {
        return;
    }
    let pts: String = points
        .iter()
        .map(|&[z, t]| {
            let (x, y) = w2s.map(z, t);
            format!("{x:.2},{y:.2}")
        })
        .collect::<Vec<_>>()
        .join(" ");
    s.push_str(&format!(
        r#"<polyline points="{pts}" fill="none" stroke="{stroke}" stroke-width="{width}"/>"#
    ));
}

fn svg_stop(s: &mut String, z: f64, half_gap: f64, extent: f64, w2s: &WorldToSvg, color: &str) {
    let (x, y_top) = w2s.map(z, extent);
    let (_, y_gap_top) = w2s.map(z, half_gap);
    s.push_str(&format!(
        r#"<line x1="{x:.2}" y1="{y_top:.2}" x2="{x:.2}" y2="{y_gap_top:.2}" stroke="{color}" stroke-width="2"/>"#
    ));
    let (_, y_gap_bot) = w2s.map(z, -half_gap);
    let (_, y_bot) = w2s.map(z, -extent);
    s.push_str(&format!(
        r#"<line x1="{x:.2}" y1="{y_gap_bot:.2}" x2="{x:.2}" y2="{y_bot:.2}" stroke="{color}" stroke-width="2"/>"#
    ));
}

fn svg_flat_plane(
    s: &mut String,
    p1: [f64; 2],
    p2: [f64; 2],
    kind: FlatPlaneKind,
    w2s: &WorldToSvg,
) {
    let (color, width) = match kind {
        FlatPlaneKind::Image => ("#00c864", 2.0f64),
        FlatPlaneKind::Probe => ("#c8c800", 1.0),
        FlatPlaneKind::Object => ("#969696", 1.0),
    };
    let (x1, y1) = w2s.map(p1[0], p1[1]);
    let (x2, y2) = w2s.map(p2[0], p2[1]);
    s.push_str(&format!(
        r#"<line x1="{x1:.2}" y1="{y1:.2}" x2="{x2:.2}" y2="{y2:.2}" stroke="{color}" stroke-width="{width}"/>"#
    ));
}

fn svg_scalebar(s: &mut String, bb: &Bounds2D, w2s: &WorldToSvg, color: &str) {
    let world_width = (bb.z.1 - bb.z.0).max(f64::EPSILON);
    let target = world_width * 0.15;
    let magnitude = 10f64.powf(target.log10().floor());
    let normalized = target / magnitude;
    let nice = if normalized >= 5.0 {
        5.0
    } else if normalized >= 2.0 {
        2.0
    } else {
        1.0
    };
    let nice_len = nice * magnitude;
    let bar_px = w2s.len(nice_len);

    let margin = 8.0_f64;
    let serif_h = 4.0_f64;
    let bar_right = SVG_W - margin;
    let bar_left = bar_right - bar_px;
    let bar_y = SVG_H - margin;

    s.push_str(&format!(
        r#"<line x1="{bar_left:.2}" y1="{bar_y:.2}" x2="{bar_right:.2}" y2="{bar_y:.2}" stroke="{color}" stroke-width="2"/>"#
    ));
    s.push_str(&format!(
        r#"<line x1="{bar_left:.2}" y1="{:.2}" x2="{bar_left:.2}" y2="{bar_y:.2}" stroke="{color}" stroke-width="2"/>"#,
        bar_y - serif_h
    ));
    s.push_str(&format!(
        r#"<line x1="{bar_right:.2}" y1="{:.2}" x2="{bar_right:.2}" y2="{bar_y:.2}" stroke="{color}" stroke-width="2"/>"#,
        bar_y - serif_h
    ));

    let label = if nice_len >= 1.0 {
        format!("{:.0} mm", nice_len)
    } else {
        format!("{:.2} mm", nice_len)
    };
    let label_x = (bar_left + bar_right) / 2.0;
    let label_y = bar_y - serif_h - 2.0;
    s.push_str(&format!(
        r#"<text x="{label_x:.2}" y="{label_y:.2}" text-anchor="middle" font-family="sans-serif" font-size="11" fill="{color}">{label}</text>"#
    ));
}

fn color_to_hex(c: egui::Color32) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::result_package::ResultPackage;
    use egui_kittest::{Harness, kittest::Queryable};

    fn show_window(
        window: &mut CrossSectionWindow,
        result: Option<&ResultPackage>,
        ctx: &egui::Context,
    ) {
        let mut open = true;
        let mut n_rays = 11u32;
        window.show(ctx, &mut open, result, &mut n_rays);
    }

    #[test]
    fn no_result_shows_empty_viewport() {
        let mut window = CrossSectionWindow::default();
        let mut harness = Harness::new(|ctx| show_window(&mut window, None, ctx));
        harness.step();
        harness.get_by_label("No data yet.");
    }

    #[test]
    fn n_rays_change_does_not_signal_recompute_without_result() {
        // show() must return false when there is no result, even if n_rays
        // changes, because there is nothing to recompute.
        struct State {
            window: CrossSectionWindow,
            n_rays: u32,
            changed: bool,
        }
        let state = State {
            window: CrossSectionWindow::default(),
            n_rays: 3,
            changed: false,
        };
        let mut harness = Harness::new_state(
            |ctx, s: &mut State| {
                let mut open = true;
                s.changed = s.window.show(ctx, &mut open, None, &mut s.n_rays);
            },
            state,
        );
        harness.step();
        harness.step();
        assert!(
            !harness.state().changed,
            "show() must not signal recompute when result is None"
        );
    }

    #[test]
    fn neither_valid_shows_message() {
        let window = CrossSectionWindow::default();
        use crate::views::cross_section::{Bounds2D, CrossSectionView, PlaneGeometry};
        let cs = CrossSectionView {
            wavelengths: vec![0.5876],
            yz_valid: false,
            xz_valid: false,
            yz: PlaneGeometry {
                bounding_box: Bounds2D {
                    z: (-1.0, 1.0),
                    transverse: (-1.0, 1.0),
                },
                elements: Vec::new(),
                ray_paths: Vec::new(),
            },
            xz: PlaneGeometry {
                bounding_box: Bounds2D {
                    z: (-1.0, 1.0),
                    transverse: (-1.0, 1.0),
                },
                elements: Vec::new(),
                ray_paths: Vec::new(),
            },
        };
        let result = ResultPackage {
            id: 1,
            wavelengths: vec![0.5876],
            surfaces: Vec::new(),
            fields: Vec::new(),
            field_specs: Vec::new(),
            paraxial: None,
            ray_trace: None,
            cross_section: Some(cs),
            error: None,
        };
        let mut harness = Harness::new_state(
            |ctx, (w, r): &mut (CrossSectionWindow, ResultPackage)| {
                show_window(w, Some(r), ctx);
            },
            (window, result),
        );
        harness.step();
        harness.get_by_label_contains("The optical axis leaves both coordinate planes");
    }
}
