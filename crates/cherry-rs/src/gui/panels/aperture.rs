use super::super::model::{SurfaceRefRow, SurfaceVariant, SystemSpecs};
use super::{format_display_float, parse_display_float};

/// Describe a step of `specs.paths[active_path]` for display in the
/// stop-surface picker: variant + originating-path label for `Shared` rows,
/// or the row's own variant for `New`/`ObjectLinkedTo` rows.
fn describe_step(specs: &SystemSpecs, active_path: usize, step: usize) -> Option<String> {
    let r = specs.paths.get(active_path)?.surface_refs.get(step)?;
    match r {
        SurfaceRefRow::New(row) => Some(row.variant.to_string()),
        SurfaceRefRow::ObjectLinkedTo { .. } => Some(SurfaceVariant::Object.to_string()),
        SurfaceRefRow::Shared { target, .. } => {
            let table = specs.store_index_table();
            let owner = table.owner_of(*target)?;
            let variant = table.owning_variant(*target)?;
            Some(format!(
                "{variant} \u{2014} shared from {}",
                specs.path_label(owner)
            ))
        }
    }
}

/// Whether the step at `step` is eligible to be a stop surface (Conic,
/// Sphere, Iris, ThinLens, or BeamSplitter — matches the library's own
/// aperture-stop eligibility).
fn is_eligible(specs: &SystemSpecs, active_path: usize, step: usize) -> bool {
    let Some(r) = specs
        .paths
        .get(active_path)
        .and_then(|p| p.surface_refs.get(step))
    else {
        return false;
    };
    let variant = match r {
        SurfaceRefRow::New(row) => Some(row.variant),
        SurfaceRefRow::ObjectLinkedTo { .. } => None,
        SurfaceRefRow::Shared { target, .. } => specs.store_index_table().owning_variant(*target),
    };
    matches!(
        variant,
        Some(
            SurfaceVariant::Conic
                | SurfaceVariant::Sphere
                | SurfaceVariant::Iris
                | SurfaceVariant::ThinLens
                | SurfaceVariant::BeamSplitter
        )
    )
}

/// Draw the aperture editor panel for `specs.paths[active_path]`. Returns
/// true if any spec was modified.
pub fn aperture_panel(ui: &mut egui::Ui, specs: &mut SystemSpecs, active_path: usize) -> bool {
    let mut changed = false;
    if active_path >= specs.paths.len() {
        return false;
    }

    ui.label("Entrance Pupil");
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Semi-diameter:");
        let mut val = parse_display_float(&specs.paths[active_path].aperture_semi_diameter);
        let response = ui.add(
            egui::DragValue::new(&mut val)
                .range(0.001..=500.0)
                .speed(0.1),
        );
        if response.changed() {
            specs.paths[active_path].aperture_semi_diameter = format_display_float(val);
            changed = true;
        }
    });

    ui.add_space(8.0);
    ui.label("Stop Surface");
    ui.separator();

    let n_steps = specs.paths[active_path].surface_refs.len();
    let eligible: Vec<usize> = (0..n_steps)
        .filter(|&step| is_eligible(specs, active_path, step))
        .collect();

    let table = specs.store_index_table();
    let current_stop = specs.paths[active_path].stop_surface;
    let selected_label = match current_stop {
        None => "Auto".to_owned(),
        Some(store_idx) => {
            // Find the step in this path resolving to store_idx, for its label.
            (0..n_steps)
                .find(|&step| {
                    table.resolve(&specs.paths[active_path].surface_refs[step]) == Some(store_idx)
                })
                .and_then(|step| describe_step(specs, active_path, step))
                .unwrap_or_else(|| format!("Surface [{store_idx}]"))
        }
    };

    egui::ComboBox::from_id_salt("stop_surface_combo")
        .selected_text(selected_label)
        .show_ui(ui, |ui| {
            if ui
                .selectable_label(current_stop.is_none(), "Auto")
                .clicked()
            {
                specs.paths[active_path].stop_surface = None;
                changed = true;
            }
            for &step in &eligible {
                let Some(label) = describe_step(specs, active_path, step) else {
                    continue;
                };
                let store_idx = table.resolve(&specs.paths[active_path].surface_refs[step]);
                let is_selected = store_idx.is_some() && store_idx == current_stop;
                if ui.selectable_label(is_selected, label).clicked()
                    && let Some(store_idx) = store_idx
                {
                    specs.paths[active_path].stop_surface = Some(store_idx);
                    changed = true;
                }
            }
        });

    changed
}
