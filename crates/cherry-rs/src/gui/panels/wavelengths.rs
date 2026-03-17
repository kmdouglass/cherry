use super::super::model::SystemSpecs;
use super::{format_display_float, parse_display_float};

/// Draw the wavelengths editor panel. Returns true if any spec was modified.
pub fn wavelengths_panel(ui: &mut egui::Ui, specs: &mut SystemSpecs) -> bool {
    let mut changed = false;

    ui.label("Wavelengths (micrometers)");
    ui.separator();

    let mut delete_at: Option<usize> = None;
    let num_wavelengths = specs.wavelengths.len();

    for idx in 0..num_wavelengths {
        ui.horizontal(|ui| {
            ui.label(format!("{}:", idx));
            let mut val = parse_display_float(&specs.wavelengths[idx]);
            let response = ui.push_id(format!("wl_{idx}"), |ui| {
                ui.add(
                    egui::DragValue::new(&mut val)
                        .range(0.1..=15.0)
                        .speed(0.001),
                )
            });
            if response.inner.changed() {
                specs.wavelengths[idx] = format_display_float(val);
                changed = true;
            }
            if num_wavelengths > 1 && ui.small_button("-").clicked() {
                delete_at = Some(idx);
            }
        });
    }

    if let Some(idx) = delete_at {
        specs.wavelengths.remove(idx);
        changed = true;
    }

    if ui.button("Add wavelength").clicked() {
        specs.wavelengths.push("0.567".into());
        changed = true;
    }

    changed
}
