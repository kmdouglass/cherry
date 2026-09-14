use egui_extras::{Column, TableBuilder};

use super::super::model::{
    BeamSplitterPathKindRow, BoundaryVariant, GapRow, LinkedObjectOrientationRow, RowId,
    SolveParameter, SolvePopupState, SurfaceRefRow, SurfaceRow, SurfaceVariant, SystemSpecs,
};
use super::{format_display_float, inf_formatter, inf_parser, parse_display_float};
use crate::gui::result_package::SolvedValues;

/// Which reference mode a row uses — mirrors `SurfaceRefRow`'s variants,
/// used purely for the row-mode selector widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RefMode {
    New,
    Shared,
    ObjectLinkedTo,
}

impl std::fmt::Display for RefMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RefMode::New => write!(f, "New"),
            RefMode::Shared => write!(f, "Shared"),
            RefMode::ObjectLinkedTo => write!(f, "Linked"),
        }
    }
}

fn ref_mode(r: &SurfaceRefRow) -> RefMode {
    match r {
        SurfaceRefRow::New(_) => RefMode::New,
        SurfaceRefRow::Shared { .. } => RefMode::Shared,
        SurfaceRefRow::ObjectLinkedTo { .. } => RefMode::ObjectLinkedTo,
    }
}

/// Candidates for a `Shared` row in `paths[active_path]`: every `New`/
/// `ObjectLinkedTo` row introduced by a strictly earlier path (FR-SURF-2),
/// excluding same-path rows (FR-SURF-3).
fn shared_candidates(specs: &SystemSpecs, active_path: usize) -> Vec<(RowId, String)> {
    if active_path == 0 {
        return Vec::new();
    }
    specs.paths[..active_path]
        .iter()
        .enumerate()
        .flat_map(|(path_idx, path)| {
            path.surface_refs.iter().filter_map(move |r| match r {
                SurfaceRefRow::New(row) => {
                    Some((row.id, format!("{} \u{2014} {}", row.variant, path_idx)))
                }
                SurfaceRefRow::ObjectLinkedTo { id, .. } => {
                    Some((*id, format!("Object \u{2014} Path {path_idx}")))
                }
                SurfaceRefRow::Shared { .. } => None,
            })
        })
        .collect()
}

/// Draw the surfaces editor panel for `specs.paths[active_path]`. Returns
/// true if any spec was modified.
pub fn surfaces_panel(
    ui: &mut egui::Ui,
    specs: &mut SystemSpecs,
    active_path: usize,
    solved_values: Option<&SolvedValues>,
    solve_popup: &mut Option<SolvePopupState>,
) -> bool {
    let mut changed = false;
    if active_path >= specs.paths.len() {
        return false;
    }

    // Snapshot material state before mutable table iteration.
    let use_materials = specs.use_materials;
    let selected_materials: Vec<String> = if use_materials {
        specs.selected_materials.clone()
    } else {
        Vec::new()
    };
    let n_col_width = if use_materials { 140.0 } else { 80.0 };

    let table_idx = specs.store_index_table();
    let path_len = specs.paths[active_path].surface_refs.len();

    let variant_of = |r: &SurfaceRefRow| -> Option<SurfaceVariant> {
        match r {
            SurfaceRefRow::New(row) => Some(row.variant),
            SurfaceRefRow::Shared { target, .. } => table_idx.owning_variant(*target),
            SurfaceRefRow::ObjectLinkedTo { .. } => None,
        }
    };
    let is_curved = |r: &SurfaceRefRow| {
        matches!(
            variant_of(r),
            Some(SurfaceVariant::Conic | SurfaceVariant::Sphere)
        )
    };
    let is_tiltable = |r: &SurfaceRefRow| -> bool {
        matches!(r, SurfaceRefRow::New(row) if row.variant == SurfaceVariant::BeamSplitter)
            || (is_curved(r)
                && matches!(
                    r,
                    SurfaceRefRow::New(row) if row.boundary_variant == BoundaryVariant::Reflecting
                ))
    };

    let refs = &specs.paths[active_path].surface_refs;
    let has_reflecting = refs.iter().any(is_tiltable);
    let has_conic = refs
        .iter()
        .any(|r| matches!(r, SurfaceRefRow::New(row) if row.variant == SurfaceVariant::Conic));
    let has_thin_lens = refs
        .iter()
        .any(|r| matches!(r, SurfaceRefRow::New(row) if row.variant == SurfaceVariant::ThinLens));
    let has_beam_splitter = refs
        .iter()
        .any(|r| variant_of(r) == Some(SurfaceVariant::BeamSplitter));
    let show_ref_mode = active_path > 0;

    egui::ScrollArea::horizontal().show(ui, |ui| {
        let ctx = ui.ctx().clone();
        let mut table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .sense(egui::Sense::click())
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto().at_least(30.0)); // #

        if show_ref_mode {
            table = table.column(Column::initial(70.0).resizable(true)); // Ref
        }
        table = table
            .column(Column::auto().at_least(90.0)) // Variant
            .column(Column::auto().at_least(60.0)) // Kind
            .column(Column::initial(80.0).resizable(true)) // Semi-Diam
            .column(Column::initial(80.0).resizable(true)); // RoC

        if has_conic {
            table = table.column(Column::initial(80.0).resizable(true)); // Conic
        }
        if has_thin_lens {
            table = table.column(Column::initial(90.0).resizable(true)); // Focal Length
        }

        table = table
            .column(Column::initial(80.0).resizable(true)) // Thickness
            .column(Column::initial(n_col_width).resizable(true)); // n / Material

        if has_reflecting {
            table = table
                .column(Column::initial(120.0).resizable(true)) // Nom. Rotation θ
                .column(Column::initial(65.0).resizable(true)); // Nom. Rotation ψ
        }
        if has_beam_splitter {
            table = table.column(Column::initial(110.0).resizable(true)); // Arm
        }

        table = table.column(Column::auto().at_least(50.0)); // Actions

        table
            .header(34.0, |mut header| {
                header.col(|ui| header_cell(ui, None, "#"));
                if show_ref_mode {
                    header.col(|ui| header_cell(ui, None, "Ref"));
                }
                header.col(|ui| header_cell(ui, None, "Variant"));
                header.col(|ui| header_cell(ui, None, "Kind"));
                header.col(|ui| header_cell(ui, None, "Semi-Diam"));
                header.col(|ui| header_cell(ui, None, "RoC"));
                if has_conic {
                    header.col(|ui| header_cell(ui, None, "Conic"));
                }
                if has_thin_lens {
                    header.col(|ui| header_cell(ui, None, "Focal Length"));
                }
                header.col(|ui| header_cell(ui, None, "Thickness"));
                header.col(|ui| header_cell(ui, None, "n"));
                if has_reflecting {
                    header.col(|ui| header_cell(ui, Some("Nominal Rotation"), "\u{03b8} (deg)"));
                    header.col(|ui| header_cell(ui, None, "\u{03c8} (deg)"));
                }
                if has_beam_splitter {
                    header.col(|ui| header_cell(ui, None, "Arm"));
                }
                header.col(|ui| header_cell(ui, None, ""));
            })
            .body(|mut body| {
                let mut insert_after: Option<usize> = None;
                let mut delete_at: Option<usize> = None;
                let mut switch_mode: Option<(usize, RefMode)> = None;

                for row_idx in 0..path_len {
                    let r = &specs.paths[active_path].surface_refs[row_idx];
                    let mode = ref_mode(r);
                    let variant = variant_of(r);
                    let is_object = row_idx == 0
                        && mode == RefMode::New
                        && matches!(variant, Some(SurfaceVariant::Object));
                    let is_object_linked = mode == RefMode::ObjectLinkedTo;
                    let is_image = variant == Some(SurfaceVariant::Image);
                    let curved = is_curved(r);
                    let tiltable = is_tiltable(r);
                    let is_locked_object_or_image =
                        (row_idx == 0 && mode == RefMode::New) || is_image;

                    let store_idx = table_idx.resolve(r);
                    let auto_gap = specs.paths[active_path].gap_is_auto_inferred(row_idx);

                    let roc_solve_spec = store_idx
                        .and_then(|si| specs.curvature_solve_for(si))
                        .cloned();
                    let thick_solve_spec = specs.thickness_solve_for(active_path, row_idx).cloned();
                    let has_roc_solve = roc_solve_spec.is_some();
                    let has_thick_solve = thick_solve_spec.is_some();

                    body.row(22.0, |mut row| {
                        if specs.paths[active_path].stop_surface.is_some()
                            && specs.paths[active_path].stop_surface == store_idx
                        {
                            row.set_selected(true);
                        }

                        // # column
                        row.col(|ui| {
                            ui.label(row_idx.to_string());
                        });

                        // Ref-mode selector column
                        if show_ref_mode {
                            row.col(|ui| {
                                if row_idx == 0 || is_image {
                                    // Object (step 0) and Image rows: mode
                                    // selector offered per FR-SURF-5 — New/
                                    // Shared always; ObjectLinkedTo only at
                                    // step 0.
                                }
                                let id = ui
                                    .make_persistent_id(format!("refmode_{active_path}_{row_idx}"));
                                let options: &[RefMode] = if row_idx == 0 {
                                    &[RefMode::New, RefMode::Shared, RefMode::ObjectLinkedTo]
                                } else {
                                    &[RefMode::New, RefMode::Shared]
                                };
                                egui::ComboBox::from_id_salt(id)
                                    .selected_text(mode.to_string())
                                    .width(65.0)
                                    .show_ui(ui, |ui| {
                                        for &m in options {
                                            if ui
                                                .selectable_label(mode == m, m.to_string())
                                                .clicked()
                                                && m != mode
                                            {
                                                switch_mode = Some((row_idx, m));
                                            }
                                        }
                                    });
                            });
                        }

                        // Variant column
                        row.col(|ui| match mode {
                            RefMode::New => {
                                let SurfaceRefRow::New(surf) =
                                    &mut specs.paths[active_path].surface_refs[row_idx]
                                else {
                                    unreachable!()
                                };
                                if is_locked_object_or_image {
                                    ui.label(surf.variant.to_string());
                                } else {
                                    let id = ui.make_persistent_id(format!(
                                        "variant_{active_path}_{row_idx}"
                                    ));
                                    egui::ComboBox::from_id_salt(id)
                                        .selected_text(surf.variant.to_string())
                                        .width(80.0)
                                        .show_ui(ui, |ui| {
                                            for &v in SurfaceVariant::SELECTABLE {
                                                if ui
                                                    .selectable_value(
                                                        &mut surf.variant,
                                                        v,
                                                        v.to_string(),
                                                    )
                                                    .changed()
                                                {
                                                    changed = true;
                                                }
                                            }
                                        });
                                }
                            }
                            RefMode::Shared => {
                                let candidates = shared_candidates(specs, active_path);
                                let SurfaceRefRow::Shared { target, .. } =
                                    &mut specs.paths[active_path].surface_refs[row_idx]
                                else {
                                    unreachable!()
                                };
                                let current_label = candidates
                                    .iter()
                                    .find(|(id, _)| id == target)
                                    .map(|(_, l)| l.clone())
                                    .unwrap_or_else(|| "(unresolved)".to_owned());
                                let id = ui
                                    .make_persistent_id(format!("shared_{active_path}_{row_idx}"));
                                egui::ComboBox::from_id_salt(id)
                                    .selected_text(current_label)
                                    .width(140.0)
                                    .show_ui(ui, |ui| {
                                        for (cand_id, label) in &candidates {
                                            if ui
                                                .selectable_label(target == cand_id, label)
                                                .clicked()
                                                && *target != *cand_id
                                            {
                                                *target = *cand_id;
                                                changed = true;
                                            }
                                        }
                                    });
                            }
                            RefMode::ObjectLinkedTo => {
                                let SurfaceRefRow::ObjectLinkedTo { path, .. } =
                                    &mut specs.paths[active_path].surface_refs[row_idx]
                                else {
                                    unreachable!()
                                };
                                let id = ui
                                    .make_persistent_id(format!("linked_{active_path}_{row_idx}"));
                                let current = *path;
                                egui::ComboBox::from_id_salt(id)
                                    .selected_text(format!("Path {current}"))
                                    .width(90.0)
                                    .show_ui(ui, |ui| {
                                        for p in 0..active_path {
                                            if ui
                                                .selectable_label(*path == p, format!("Path {p}"))
                                                .clicked()
                                                && *path != p
                                            {
                                                *path = p;
                                                changed = true;
                                            }
                                        }
                                    });
                            }
                        });

                        // Kind column: boundary kind for curved New rows;
                        // orientation toggle for ObjectLinkedTo; blank
                        // otherwise.
                        row.col(|ui| match mode {
                            RefMode::New if curved => {
                                let SurfaceRefRow::New(surf) =
                                    &mut specs.paths[active_path].surface_refs[row_idx]
                                else {
                                    unreachable!()
                                };
                                let id =
                                    ui.make_persistent_id(format!("kind_{active_path}_{row_idx}"));
                                egui::ComboBox::from_id_salt(id)
                                    .selected_text(surf.boundary_variant.to_string())
                                    .width(80.0)
                                    .show_ui(ui, |ui| {
                                        for kind in [
                                            BoundaryVariant::Refracting,
                                            BoundaryVariant::Reflecting,
                                        ] {
                                            if ui
                                                .selectable_value(
                                                    &mut surf.boundary_variant,
                                                    kind,
                                                    kind.to_string(),
                                                )
                                                .changed()
                                            {
                                                changed = true;
                                            }
                                        }
                                    });
                            }
                            RefMode::ObjectLinkedTo => {
                                let SurfaceRefRow::ObjectLinkedTo { orientation, .. } =
                                    &mut specs.paths[active_path].surface_refs[row_idx]
                                else {
                                    unreachable!()
                                };
                                let id = ui
                                    .make_persistent_id(format!("orient_{active_path}_{row_idx}"));
                                let label = match orientation {
                                    LinkedObjectOrientationRow::SameDirection => "Same",
                                    LinkedObjectOrientationRow::Reversed => "Reversed",
                                };
                                egui::ComboBox::from_id_salt(id)
                                    .selected_text(label)
                                    .width(80.0)
                                    .show_ui(ui, |ui| {
                                        for (opt, text) in [
                                            (LinkedObjectOrientationRow::SameDirection, "Same"),
                                            (LinkedObjectOrientationRow::Reversed, "Reversed"),
                                        ] {
                                            if ui.selectable_value(orientation, opt, text).changed()
                                            {
                                                changed = true;
                                            }
                                        }
                                    });
                            }
                            _ => {}
                        });

                        // Semi-Diameter (New rows only — Shared/ObjectLinkedTo
                        // rows have no geometry of their own).
                        row.col(|ui| {
                            if let SurfaceRefRow::New(surf) =
                                &mut specs.paths[active_path].surface_refs[row_idx]
                                && !is_object
                                && !is_image
                            {
                                changed |= drag_value(
                                    ui,
                                    &mut surf.semi_diameter,
                                    active_path,
                                    row_idx,
                                    "sd",
                                    0.0..=500.0,
                                    0.1,
                                );
                            }
                        });

                        // Radius of Curvature
                        row.col(|ui| {
                            if curved && matches!(mode, RefMode::New) {
                                if has_roc_solve {
                                    match solved_values.and_then(|sv| {
                                        store_idx.and_then(|si| sv.surface_rocs.get(&si).copied())
                                    }) {
                                        Some(val) => solved_cell(ui, val, "F"),
                                        None => {
                                            ui.weak("\u{2014}");
                                        }
                                    }
                                    if solve_button(ui, active_path, row_idx, "roc_solve", true)
                                        .clicked()
                                        && let Some(si) = store_idx
                                    {
                                        *solve_popup = Some(SolvePopupState::open(
                                            si,
                                            active_path,
                                            SolveParameter::RadiusOfCurvature,
                                            roc_solve_spec.as_ref(),
                                        ));
                                    }
                                } else {
                                    let SurfaceRefRow::New(surf) =
                                        &mut specs.paths[active_path].surface_refs[row_idx]
                                    else {
                                        unreachable!()
                                    };
                                    changed |= drag_inf(
                                        ui,
                                        &mut surf.radius_of_curvature,
                                        active_path,
                                        row_idx,
                                        "roc",
                                        f64::NEG_INFINITY..=f64::INFINITY,
                                        1.0,
                                    );
                                    if solve_button(ui, active_path, row_idx, "roc_solve", false)
                                        .clicked()
                                        && let Some(si) = store_idx
                                    {
                                        *solve_popup = Some(SolvePopupState::open(
                                            si,
                                            active_path,
                                            SolveParameter::RadiusOfCurvature,
                                            None,
                                        ));
                                    }
                                }
                            } else if curved && has_roc_solve {
                                // Shared row referencing a curved surface with
                                // an active solve: show the same read-only
                                // solved value, with a button that opens the
                                // *same* underlying solve (FR-SOLVE-4).
                                match solved_values.and_then(|sv| {
                                    store_idx.and_then(|si| sv.surface_rocs.get(&si).copied())
                                }) {
                                    Some(val) => solved_cell(ui, val, "F"),
                                    None => {
                                        ui.weak("\u{2014}");
                                    }
                                }
                                if solve_button(ui, active_path, row_idx, "roc_solve", true)
                                    .clicked()
                                    && let Some(si) = store_idx
                                {
                                    *solve_popup = Some(SolvePopupState::open(
                                        si,
                                        active_path,
                                        SolveParameter::RadiusOfCurvature,
                                        roc_solve_spec.as_ref(),
                                    ));
                                }
                            }
                        });

                        // Conic Constant (only when the path has Conic surfaces)
                        if has_conic {
                            row.col(|ui| {
                                if let SurfaceRefRow::New(surf) =
                                    &mut specs.paths[active_path].surface_refs[row_idx]
                                    && surf.variant == SurfaceVariant::Conic
                                {
                                    if surf.conic_constant.is_empty() {
                                        surf.conic_constant = "0".into();
                                        changed = true;
                                    }
                                    changed |= drag_value(
                                        ui,
                                        &mut surf.conic_constant,
                                        active_path,
                                        row_idx,
                                        "cc",
                                        -10.0..=10.0,
                                        0.01,
                                    );
                                }
                            });
                        }

                        // Focal Length (only when the path has Thin Lens surfaces)
                        if has_thin_lens {
                            row.col(|ui| {
                                if let SurfaceRefRow::New(surf) =
                                    &mut specs.paths[active_path].surface_refs[row_idx]
                                    && surf.variant == SurfaceVariant::ThinLens
                                {
                                    if surf.focal_length.is_empty() {
                                        surf.focal_length = "100".into();
                                        changed = true;
                                    }
                                    changed |= drag_inf(
                                        ui,
                                        &mut surf.focal_length,
                                        active_path,
                                        row_idx,
                                        "fl",
                                        f64::NEG_INFINITY..=f64::INFINITY,
                                        1.0,
                                    );
                                }
                            });
                        }

                        // Thickness
                        row.col(|ui| {
                            if is_image {
                                return;
                            }
                            if auto_gap {
                                ui.weak("auto");
                                return;
                            }
                            if has_thick_solve {
                                match solved_values
                                    .and_then(|sv| sv.gap_thicknesses.get(&row_idx).copied())
                                {
                                    Some(val) => solved_cell(ui, val, "M"),
                                    None => {
                                        ui.weak("\u{2014}");
                                    }
                                }
                                if solve_button(ui, active_path, row_idx, "thick_solve", true)
                                    .clicked()
                                {
                                    *solve_popup = Some(SolvePopupState::open(
                                        row_idx,
                                        active_path,
                                        SolveParameter::Thickness,
                                        thick_solve_spec.as_ref(),
                                    ));
                                }
                                return;
                            }
                            match &mut specs.paths[active_path].surface_refs[row_idx] {
                                SurfaceRefRow::New(surf) => {
                                    changed |= drag_inf(
                                        ui,
                                        &mut surf.thickness,
                                        active_path,
                                        row_idx,
                                        "thick",
                                        0.0..=f64::INFINITY,
                                        0.5,
                                    );
                                }
                                r => {
                                    if let Some(gap) = r.gap_after_mut() {
                                        changed |= drag_inf(
                                            ui,
                                            &mut gap.thickness,
                                            active_path,
                                            row_idx,
                                            "thick",
                                            0.0..=f64::INFINITY,
                                            0.5,
                                        );
                                    }
                                }
                            }
                            if solve_button(ui, active_path, row_idx, "thick_solve", false)
                                .clicked()
                            {
                                *solve_popup = Some(SolvePopupState::open(
                                    row_idx,
                                    active_path,
                                    SolveParameter::Thickness,
                                    None,
                                ));
                            }
                        });

                        // Refractive Index / Material
                        row.col(|ui| {
                            if is_image || auto_gap {
                                return;
                            }
                            let (ri, mat_key): (&mut String, &mut Option<String>) =
                                match &mut specs.paths[active_path].surface_refs[row_idx] {
                                    SurfaceRefRow::New(surf) => {
                                        (&mut surf.refractive_index, &mut surf.material_key)
                                    }
                                    r => {
                                        let Some(gap) = r.gap_after_mut() else {
                                            return;
                                        };
                                        (&mut gap.refractive_index, &mut gap.material_key)
                                    }
                                };
                            if use_materials {
                                let display = mat_key.as_deref().unwrap_or("(none)");
                                let id =
                                    ui.make_persistent_id(format!("mat_{active_path}_{row_idx}"));
                                egui::ComboBox::from_id_salt(id)
                                    .selected_text(display)
                                    .width(130.0)
                                    .show_ui(ui, |ui| {
                                        if ui
                                            .selectable_label(mat_key.is_none(), "(none)")
                                            .clicked()
                                        {
                                            *mat_key = None;
                                            changed = true;
                                        }
                                        for mat in &selected_materials {
                                            let selected = mat_key.as_deref() == Some(mat.as_str());
                                            if ui.selectable_label(selected, mat).clicked() {
                                                *mat_key = Some(mat.clone());
                                                changed = true;
                                            }
                                        }
                                    });
                            } else {
                                changed |=
                                    drag_value(ui, ri, active_path, row_idx, "n", 1.0..=4.0, 0.01);
                            }
                        });

                        // θ / ψ columns (only shown when the path has tiltable surfaces)
                        if has_reflecting {
                            row.col(|ui| {
                                if tiltable
                                    && let SurfaceRefRow::New(surf) =
                                        &mut specs.paths[active_path].surface_refs[row_idx]
                                {
                                    changed |= drag_value(
                                        ui,
                                        &mut surf.theta,
                                        active_path,
                                        row_idx,
                                        "theta",
                                        -90.0..=90.0,
                                        0.5,
                                    );
                                }
                            });
                            row.col(|ui| {
                                if tiltable
                                    && let SurfaceRefRow::New(surf) =
                                        &mut specs.paths[active_path].surface_refs[row_idx]
                                {
                                    changed |= drag_value(
                                        ui,
                                        &mut surf.psi,
                                        active_path,
                                        row_idx,
                                        "psi",
                                        -90.0..=90.0,
                                        0.5,
                                    );
                                }
                            });
                        }

                        // Arm column (BeamSplitter rows only)
                        if has_beam_splitter {
                            row.col(|ui| {
                                if variant == Some(SurfaceVariant::BeamSplitter) {
                                    let path_row = &mut specs.paths[active_path];
                                    if let Some(arm) =
                                        path_row.beam_splitter_arm_mut(&table_idx, row_idx)
                                    {
                                        let id = ui.make_persistent_id(format!(
                                            "arm_{active_path}_{row_idx}"
                                        ));
                                        egui::ComboBox::from_id_salt(id)
                                            .selected_text(arm.to_string())
                                            .width(100.0)
                                            .show_ui(ui, |ui| {
                                                for opt in [
                                                    BeamSplitterPathKindRow::Transmitting,
                                                    BeamSplitterPathKindRow::Reflecting,
                                                ] {
                                                    if ui
                                                        .selectable_value(arm, opt, opt.to_string())
                                                        .changed()
                                                    {
                                                        changed = true;
                                                    }
                                                }
                                            });
                                    }
                                }
                            });
                        }

                        // Actions column (always last).
                        row.col(|ui| {
                            if !is_image {
                                ui.horizontal(|ui| {
                                    if ui.small_button("+").clicked() {
                                        insert_after = Some(row_idx);
                                    }
                                    if !is_object && !is_object_linked {
                                        let can_delete =
                                            specs.can_delete_surface(active_path, row_idx);
                                        let resp = ui.add_enabled(
                                            can_delete,
                                            egui::Button::new("-").small(),
                                        );
                                        if !can_delete {
                                            resp.clone().on_disabled_hover_text(
                                                "Cannot delete: referenced by a later path",
                                            );
                                        }
                                        if resp.clicked() {
                                            delete_at = Some(row_idx);
                                        }
                                    }
                                });
                            }
                        });

                        if row.response().hovered() && !is_object && !is_image {
                            ctx.data_mut(|d| {
                                d.insert_temp(
                                    egui::Id::new("annotation_hover_surface_idx"),
                                    row_idx,
                                );
                            });
                        }
                    });
                }

                // Apply deferred mutations
                if let Some(idx) = insert_after {
                    specs.insert_surface_after(active_path, idx);
                    changed = true;
                }
                if let Some(idx) = delete_at {
                    changed |= specs.delete_surface(active_path, idx);
                }
                if let Some((idx, new_mode)) = switch_mode {
                    switch_row_mode(specs, active_path, idx, new_mode);
                    changed = true;
                }
            });
    });

    changed
}

/// Switch the row at `step_idx` of `active_path` to a new `RefMode`,
/// replacing it with a freshly-defaulted row of that kind. FR-SURF-5:
/// `ObjectLinkedTo` is only ever offered by the selector at step 0 of a
/// path with index > 0, so this never needs to validate that separately.
fn switch_row_mode(
    specs: &mut SystemSpecs,
    active_path: usize,
    step_idx: usize,
    new_mode: RefMode,
) {
    let new_ref = match new_mode {
        RefMode::New => {
            let id = specs.mint_row_id();
            SurfaceRefRow::New(SurfaceRow::new_default().with_id(id))
        }
        RefMode::Shared => {
            let candidates = shared_candidates(specs, active_path);
            let target = candidates.first().map(|(id, _)| *id).unwrap_or(RowId(0));
            SurfaceRefRow::Shared {
                target,
                gap_after: GapRow::default(),
            }
        }
        RefMode::ObjectLinkedTo => {
            let id = specs.mint_row_id();
            SurfaceRefRow::ObjectLinkedTo {
                id,
                path: 0,
                orientation: LinkedObjectOrientationRow::Reversed,
                gap_after: GapRow::default(),
            }
        }
    };
    if let Some(path) = specs.paths.get_mut(active_path)
        && let Some(slot) = path.surface_refs.get_mut(step_idx)
    {
        *slot = new_ref;
    }
}

/// Renders a header cell with an optional group label above the column name.
fn header_cell(ui: &mut egui::Ui, group: Option<&str>, name: &str) {
    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
        ui.strong(name);
        if let Some(g) = group {
            ui.weak(g);
        }
    });
}

/// Renders a read-only label for a solved cell with a type badge.
fn solved_cell(ui: &mut egui::Ui, val: f64, badge: &str) {
    ui.horizontal(|ui| {
        ui.label(format!("{val:.4}"));
        ui.weak(format!("[{badge}]"));
    });
}

/// Renders a small solve indicator button. Returns the egui Response.
/// `active` = a solve is currently set on this cell.
fn solve_button(
    ui: &mut egui::Ui,
    path: usize,
    row: usize,
    col: &str,
    active: bool,
) -> egui::Response {
    let label = if active { "\u{25cf}" } else { "\u{25cb}" };
    let id = egui::Id::new(format!("solve_btn_{path}_{row}_{col}"));
    ui.push_id(id, |ui| {
        let btn = ui.small_button(label);
        btn.on_hover_text(if active {
            "Solve active"
        } else {
            "Configure solve"
        })
    })
    .inner
}

/// DragValue cell without special infinity handling.
fn drag_value(
    ui: &mut egui::Ui,
    field: &mut String,
    path: usize,
    row: usize,
    col: &str,
    range: std::ops::RangeInclusive<f64>,
    speed: f64,
) -> bool {
    let mut val = parse_display_float(field);
    let response = ui.push_id(format!("cell_{path}_{row}_{col}"), |ui| {
        ui.add(egui::DragValue::new(&mut val).range(range).speed(speed))
    });
    if response.inner.changed() {
        *field = format_display_float(val);
        true
    } else {
        false
    }
}

/// DragValue cell with infinity-aware formatting.
fn drag_inf(
    ui: &mut egui::Ui,
    field: &mut String,
    path: usize,
    row: usize,
    col: &str,
    range: std::ops::RangeInclusive<f64>,
    speed: f64,
) -> bool {
    let mut val = parse_display_float(field);
    let response = ui.push_id(format!("cell_{path}_{row}_{col}"), |ui| {
        ui.add(
            egui::DragValue::new(&mut val)
                .range(range)
                .speed(speed)
                .custom_formatter(inf_formatter)
                .custom_parser(inf_parser),
        )
    });
    if response.inner.changed() {
        *field = format_display_float(val);
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{Harness, kittest::Queryable};

    use crate::gui::model::{
        BoundaryVariant, PathRow, SolveSpec, SurfaceRow, SurfaceVariant, SystemSpecs,
    };

    fn specs_with_reflecting_surface() -> SystemSpecs {
        let mut mirror = SurfaceRow::new_sphere("12.7", "Infinity", "100.0", "1.0");
        mirror.boundary_variant = BoundaryVariant::Reflecting;
        SystemSpecs::new_single(PathRow::from_surface_rows(vec![
            SurfaceRow::new_object("Infinity"),
            mirror,
            SurfaceRow::new_image(),
        ]))
    }

    fn minimal_specs() -> SystemSpecs {
        SystemSpecs::new_single(PathRow::from_surface_rows(vec![
            SurfaceRow::new_object("Infinity"),
            SurfaceRow::new_image(),
        ]))
    }

    fn default_panel(ui: &mut egui::Ui, specs: &mut SystemSpecs) -> bool {
        surfaces_panel(ui, specs, 0, None, &mut None)
    }

    /// The object row must have a + button so users can insert surfaces after
    /// it.
    #[test]
    fn object_row_has_add_button() {
        let mut specs = minimal_specs();
        let mut harness = Harness::builder()
            .with_size(egui::vec2(2000.0, 600.0))
            .build_ui(|ui| {
                default_panel(ui, &mut specs);
            });
        harness.run();
        harness.get_by_label("+");
    }

    /// Clicking + on the object row inserts a new surface after it.
    #[test]
    fn clicking_add_on_object_row_inserts_surface() {
        let mut specs = minimal_specs();
        {
            let mut harness = Harness::builder()
                .with_size(egui::vec2(2000.0, 600.0))
                .build_ui(|ui| {
                    default_panel(ui, &mut specs);
                });
            harness.run();
            harness.get_by_label("+").click();
            harness.run();
        }
        assert_eq!(
            specs.paths[0].surface_refs.len(),
            3,
            "clicking + on the object row should insert a surface"
        );
        assert!(matches!(
            &specs.paths[0].surface_refs[1],
            SurfaceRefRow::New(row) if row.variant == SurfaceVariant::Sphere
        ));
    }

    /// The "Nominal Rotation" group label is absent when no reflecting
    /// surface exists.
    #[test]
    fn nominal_rotation_group_absent_without_reflecting() {
        let mut specs = minimal_specs();
        let mut harness = Harness::builder()
            .with_size(egui::vec2(2000.0, 600.0))
            .build_ui(|ui| {
                default_panel(ui, &mut specs);
            });
        harness.run();
        assert!(
            harness
                .query_all_by_label("Nominal Rotation")
                .next()
                .is_none(),
            "Nominal Rotation label should not appear without a reflecting surface"
        );
    }

    /// The "Nominal Rotation" group label is present when a reflecting
    /// surface exists.
    #[test]
    fn nominal_rotation_group_present_with_reflecting() {
        let mut specs = specs_with_reflecting_surface();
        let mut harness = Harness::builder()
            .with_size(egui::vec2(2000.0, 600.0))
            .build_ui(|ui| {
                default_panel(ui, &mut specs);
            });
        harness.run();
        harness.get_by_label("Nominal Rotation");
    }

    /// The "Focal Length" column appears, and is editable, when a thin lens
    /// surface exists.
    #[test]
    fn focal_length_column_present_with_thin_lens() {
        let mut specs = SystemSpecs::new_single(PathRow::from_surface_rows(vec![
            SurfaceRow::new_object("Infinity"),
            SurfaceRow::new_thin_lens("12.5", "100.0", "100.0", "1.0"),
            SurfaceRow::new_image(),
        ]));
        let mut harness = Harness::builder()
            .with_size(egui::vec2(2000.0, 600.0))
            .build_ui(|ui| {
                default_panel(ui, &mut specs);
            });
        harness.run();
        harness.get_by_label("Focal Length");
    }

    fn lens_specs() -> SystemSpecs {
        SystemSpecs::new_single(PathRow::from_surface_rows(vec![
            SurfaceRow::new_object("Infinity"),
            SurfaceRow::new_sphere("12.5", "25.8", "5.3", "1.515"),
            SurfaceRow::new_sphere("12.5", "Infinity", "46.6", "1.0"),
            SurfaceRow::new_image(),
        ]))
    }

    #[test]
    fn solve_button_present_on_roc_cell_for_sphere() {
        let mut specs = lens_specs();
        let mut harness = Harness::builder()
            .with_size(egui::vec2(2000.0, 600.0))
            .build_ui(|ui| {
                default_panel(ui, &mut specs);
            });
        harness.run();
        harness
            .get_all_by_label("\u{25cb}")
            .next()
            .expect("solve button should exist on RoC cell");
    }

    #[test]
    fn solved_roc_cell_shows_badge() {
        let mut specs = lens_specs();
        specs.solves = vec![SolveSpec::FNumber {
            surface_index: 1,
            target_fno: 4.0,
            wavelength_id: 0,
            path_id: 0,
        }];
        let mut sv = SolvedValues::default();
        sv.surface_rocs.insert(1, 12.345);

        let mut harness = Harness::builder()
            .with_size(egui::vec2(2000.0, 600.0))
            .build_ui(|ui| {
                surfaces_panel(ui, &mut specs, 0, Some(&sv), &mut None);
            });
        harness.run();
        harness.get_by_label("[F]");
    }

    #[test]
    fn unsolved_cell_shows_drag_value_not_badge() {
        let mut specs = lens_specs();
        let mut harness = Harness::builder()
            .with_size(egui::vec2(2000.0, 600.0))
            .build_ui(|ui| {
                default_panel(ui, &mut specs);
            });
        harness.run();
        let active_buttons: Vec<_> = harness.query_all_by_label("\u{25cf}").collect();
        assert!(
            active_buttons.is_empty(),
            "should not show active solve buttons when no solve is active"
        );
    }

    /// Two-path system: path 1's Ref-mode selector column is visible, path
    /// 0's is not (mirrors NFR-5 for the common single-path case).
    #[test]
    fn ref_mode_selector_shown_only_for_active_path_gt_zero() {
        let mut specs = minimal_specs();
        specs.add_path();

        let mut harness0 = Harness::builder()
            .with_size(egui::vec2(2000.0, 600.0))
            .build_ui(|ui| {
                surfaces_panel(ui, &mut specs, 0, None, &mut None);
            });
        harness0.run();
        assert!(
            harness0.query_all_by_label("Ref").next().is_none(),
            "path 0 should not show the Ref-mode column"
        );
    }

    #[test]
    fn ref_mode_selector_shown_for_second_path() {
        let mut specs = minimal_specs();
        specs.add_path();

        let mut harness1 = Harness::builder()
            .with_size(egui::vec2(2000.0, 600.0))
            .build_ui(|ui| {
                surfaces_panel(ui, &mut specs, 1, None, &mut None);
            });
        harness1.run();
        harness1.get_by_label("Ref");
    }
}
