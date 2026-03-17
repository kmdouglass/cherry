use super::super::model::SystemSpecs;
use super::{format_display_float, parse_display_float};

/// Draw the aperture editor panel. Returns true if any spec was modified.
pub fn aperture_panel(ui: &mut egui::Ui, specs: &mut SystemSpecs) -> bool {
    let mut changed = false;

    ui.label("Entrance Pupil");
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Semi-diameter:");
        let mut val = parse_display_float(&specs.aperture_semi_diameter);
        let response = ui.add(
            egui::DragValue::new(&mut val)
                .range(0.001..=500.0)
                .speed(0.1),
        );
        if response.changed() {
            specs.aperture_semi_diameter = format_display_float(val);
            changed = true;
        }
    });

    changed
}
