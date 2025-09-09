use super::super::model::SystemSpecs;

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
            let response = ui.add(
                egui::TextEdit::singleline(&mut specs.wavelengths[idx])
                    .desired_width(100.0)
                    .horizontal_align(egui::Align::RIGHT)
                    .id(egui::Id::new(format!("wl_{idx}"))),
            );
            if response.changed() {
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
