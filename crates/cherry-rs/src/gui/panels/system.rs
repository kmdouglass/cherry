use super::super::model::SystemSpecs;
use super::{format_display_float, parse_display_float};

/// Draw the system background editor panel. Returns true if any spec was
/// modified.
pub fn system_panel(ui: &mut egui::Ui, specs: &mut SystemSpecs) -> bool {
    let mut changed = false;

    ui.label("Sampling");
    ui.separator();

    let mut spacing_val = parse_display_float(&specs.full_pupil_spacing);
    let response = ui.add(
        egui::DragValue::new(&mut spacing_val)
            .range(0.01..=0.2)
            .speed(0.001)
            .prefix("Pupil grid spacing: "),
    );
    if response.changed() {
        specs.full_pupil_spacing = format_display_float(spacing_val);
        changed = true;
    }

    // Fan ray count (must be odd, range 3–501).
    ui.horizontal(|ui| {
        ui.label("Fan rays:");
        let mut n = specs.n_fan_rays;
        let response = ui.add(
            egui::DragValue::new(&mut n)
                .range(33u32..=501u32)
                .speed(2.0),
        );
        if response.changed() || n.is_multiple_of(2) {
            // Snap to nearest odd, staying in range.
            if n.is_multiple_of(2) {
                n = n.saturating_add(1).min(501);
            }
            n = n.clamp(33, 501);
            if n != specs.n_fan_rays {
                specs.n_fan_rays = n;
                changed = true;
            }
        }
    });

    ui.add_space(8.0);
    ui.label("Background");
    ui.separator();

    if specs.use_materials {
        let current = specs.background_material_key.clone().unwrap_or_default();
        egui::ComboBox::from_label("Medium")
            .selected_text(&current)
            .show_ui(ui, |ui| {
                for key in &specs.selected_materials {
                    if ui
                        .selectable_value(
                            &mut specs.background_material_key,
                            Some(key.clone()),
                            key,
                        )
                        .changed()
                    {
                        changed = true;
                    }
                }
            });
    } else {
        let mut val = parse_display_float(&specs.background_n);
        let response = ui.add(
            egui::DragValue::new(&mut val)
                .range(1.0..=f64::MAX)
                .speed(0.001)
                .prefix("n = "),
        );
        if response.changed() {
            specs.background_n = format_display_float(val);
            changed = true;
        }
    }

    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::model::SystemSpecs;
    use egui_kittest::{Harness, kittest::Queryable};

    #[test]
    fn fan_rays_label_is_present() {
        let mut specs = SystemSpecs::default();
        let mut harness = Harness::new(|ctx| {
            egui::Window::new("System").show(ctx, |ui| {
                system_panel(ui, &mut specs);
            });
        });
        harness.step();
        harness.get_by_label_contains("Fan rays");
    }

    #[test]
    fn fan_rays_snaps_to_odd() {
        // Manually set an even value to test snapping.
        let specs = SystemSpecs {
            n_fan_rays: 64,
            ..Default::default()
        };
        let mut harness = Harness::new_state(
            |ctx, s: &mut SystemSpecs| {
                egui::Window::new("System").show(ctx, |ui| {
                    system_panel(ui, s);
                });
            },
            specs,
        );
        harness.step();
        // After one frame with an even value, it should have been snapped to odd
        let snapped = harness.state().n_fan_rays;
        assert!(
            !snapped.is_multiple_of(2),
            "n_fan_rays should be odd after snapping, got {snapped}"
        );
    }
}
