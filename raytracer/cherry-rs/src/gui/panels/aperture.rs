use super::super::model::SystemSpecs;

/// Draw the aperture editor panel. Returns true if any spec was modified.
pub fn aperture_panel(ui: &mut egui::Ui, specs: &mut SystemSpecs) -> bool {
    let mut changed = false;

    ui.label("Entrance Pupil");
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Semi-diameter:");
        let response = ui.add(
            egui::TextEdit::singleline(&mut specs.aperture_semi_diameter)
                .desired_width(100.0)
                .horizontal_align(egui::Align::RIGHT),
        );
        if response.changed() {
            changed = true;
        }
    });

    changed
}
