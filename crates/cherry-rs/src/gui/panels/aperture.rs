use super::super::model::{SurfaceVariant, SystemSpecs};
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

    ui.add_space(8.0);
    ui.label("Stop Surface");
    ui.separator();

    // Build the list of eligible stop surfaces (Conic and Iris only).
    let eligible: Vec<usize> = specs
        .surfaces
        .iter()
        .enumerate()
        .filter(|(_, row)| {
            matches!(
                row.variant,
                SurfaceVariant::Conic | SurfaceVariant::Sphere | SurfaceVariant::Iris
            )
        })
        .map(|(i, _)| i)
        .collect();

    let selected_label = match specs.stop_surface {
        None => "Auto".to_owned(),
        Some(i) => {
            let variant = &specs.surfaces[i].variant;
            format!("{variant} [{i}]")
        }
    };

    egui::ComboBox::from_id_salt("stop_surface_combo")
        .selected_text(selected_label)
        .show_ui(ui, |ui| {
            if ui
                .selectable_label(specs.stop_surface.is_none(), "Auto")
                .clicked()
            {
                specs.stop_surface = None;
                changed = true;
            }
            for idx in &eligible {
                let variant = &specs.surfaces[*idx].variant;
                let label = format!("{variant} [{idx}]");
                if ui
                    .selectable_label(specs.stop_surface == Some(*idx), label)
                    .clicked()
                {
                    specs.stop_surface = Some(*idx);
                    changed = true;
                }
            }
        });

    changed
}
