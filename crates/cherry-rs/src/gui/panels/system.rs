use super::super::model::SystemSpecs;
use super::{format_display_float, parse_display_float};

/// Draw the system background editor panel. Returns true if any spec was
/// modified.
pub fn system_panel(ui: &mut egui::Ui, specs: &mut SystemSpecs) -> bool {
    let mut changed = false;

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
