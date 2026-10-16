use super::super::model::{StoreIndexTable, SurfaceRefRow, SurfaceVariant, SystemSpecs};
use super::{format_display_float, parse_display_float};

/// Describe a step of `specs.paths[active_path]` for display in the
/// stop-surface picker as `(path, #) Variant` of the row that introduced the
/// surface: the active path's own row for `New`/`ObjectLinkedTo` steps, the
/// owning path's row for `Shared` steps.
fn describe_step(specs: &SystemSpecs, active_path: usize, step: usize) -> Option<String> {
    let r = specs.paths.get(active_path)?.surface_refs.get(step)?;
    match r {
        SurfaceRefRow::Shared { target, .. } => specs.surface_label(*target),
        _ => specs.surface_label(r.row_id()?),
    }
}

/// Whether the step at `step` is eligible to be a stop surface (Conic,
/// Sphere, Iris, ThinLens, or BeamSplitter — matches the library's own
/// aperture-stop eligibility). A surface the path visits more than once is
/// ineligible: the paraxial view rejects it as an ambiguous stop (FR-15).
fn is_eligible(
    specs: &SystemSpecs,
    table: &StoreIndexTable,
    active_path: usize,
    step: usize,
) -> bool {
    let Some(path) = specs.paths.get(active_path) else {
        return false;
    };
    let Some(r) = path.surface_refs.get(step) else {
        return false;
    };
    let store_idx = table.resolve(r);
    let visits = path
        .surface_refs
        .iter()
        .filter(|other| table.resolve(other) == store_idx)
        .count();
    if store_idx.is_none() || visits > 1 {
        return false;
    }
    let variant = match r {
        SurfaceRefRow::New(row) => Some(row.variant),
        SurfaceRefRow::ObjectLinkedTo { .. } => None,
        SurfaceRefRow::Shared { target, .. } => table.owning_variant(*target),
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
    let table = specs.store_index_table();
    let eligible: Vec<usize> = (0..n_steps)
        .filter(|&step| is_eligible(specs, &table, active_path, step))
        .collect();

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

#[cfg(test)]
mod tests {
    use super::*;

    /// The stop-surface picker labels a `Shared` step with its owning path
    /// and row, matching the Shared-surface picker.
    #[test]
    fn describe_step_labels_shared_step_with_owning_path_and_row() {
        let specs = crate::gui::examples::wf_epi_microscope();
        // Path 1 step 1 is the shared objective, path 0 row 3.
        assert_eq!(
            describe_step(&specs, 1, 1).as_deref(),
            Some("(0, 3) Thin Lens")
        );
    }

    /// FR-15: a surface the active path visits more than once is not offered
    /// as a stop (the paraxial view rejects it as ambiguous); surfaces
    /// visited once still are.
    #[test]
    fn multiply_visited_surface_is_not_an_eligible_stop() {
        use crate::gui::model::GapRow;
        let mut specs = crate::gui::examples::wf_epi_microscope();
        // Revisit path 0's beam splitter (step 2) after the objective.
        let bs_id = specs.paths[0].surface_refs[2].row_id().unwrap();
        specs.paths[0].surface_refs.insert(
            4,
            SurfaceRefRow::Shared {
                target: bs_id,
                gap_after: GapRow::new("10.0", "1.0"),
            },
        );
        let table = specs.store_index_table();
        assert!(!is_eligible(&specs, &table, 0, 2));
        assert!(!is_eligible(&specs, &table, 0, 4));
        assert!(is_eligible(&specs, &table, 0, 1));
        assert!(is_eligible(&specs, &table, 0, 3));
    }

    /// The active path's own rows are labelled the same way, so two thin
    /// lenses in one path are distinguishable.
    #[test]
    fn describe_step_labels_own_step_with_path_and_row() {
        let specs = crate::gui::examples::wf_epi_microscope();
        assert_eq!(
            describe_step(&specs, 0, 1).as_deref(),
            Some("(0, 1) Thin Lens")
        );
        assert_eq!(
            describe_step(&specs, 1, 4).as_deref(),
            Some("(1, 4) Thin Lens")
        );
    }
}
