use egui_extras::{Column, TableBuilder};

use super::super::model::{SurfaceKind, SurfaceRow, SurfaceVariant, SystemSpecs};
use super::{format_display_float, inf_formatter, inf_parser, parse_display_float};

/// Draw the surfaces editor panel. Returns true if any spec was modified.
pub fn surfaces_panel(ui: &mut egui::Ui, specs: &mut SystemSpecs) -> bool {
    let mut changed = false;

    // Snapshot material state before mutable table iteration.
    let use_materials = specs.use_materials;
    let selected_materials: Vec<String> = if use_materials {
        specs.selected_materials.clone()
    } else {
        Vec::new()
    };

    let n_col_width = if use_materials { 140.0 } else { 80.0 };

    let has_reflecting = specs
        .surfaces
        .iter()
        .any(|s| s.variant == SurfaceVariant::Conic && s.surface_kind == SurfaceKind::Reflecting);

    let table = TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto().at_least(30.0)) // #
        .column(Column::auto().at_least(70.0)) // Variant
        .column(Column::auto().at_least(60.0)) // Kind
        .column(Column::initial(80.0).resizable(true)) // Semi-Diam
        .column(Column::initial(80.0).resizable(true)) // RoC
        .column(Column::initial(80.0).resizable(true)) // Conic
        .column(Column::initial(80.0).resizable(true)) // Thickness
        .column(Column::initial(n_col_width).resizable(true)); // n / Material

    let table = if has_reflecting {
        table
            .column(Column::initial(65.0).resizable(true)) // θ (deg)
            .column(Column::initial(65.0).resizable(true)) // ψ (deg)
    } else {
        table
    };

    let table = table.column(Column::auto().at_least(50.0)); // Actions

    table
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.strong("#");
            });
            header.col(|ui| {
                ui.strong("Variant");
            });
            header.col(|ui| {
                ui.strong("Kind");
            });
            header.col(|ui| {
                ui.strong("Semi-Diam");
            });
            header.col(|ui| {
                ui.strong("RoC");
            });
            header.col(|ui| {
                ui.strong("Conic");
            });
            header.col(|ui| {
                ui.strong("Thickness");
            });
            header.col(|ui| {
                ui.strong("n");
            });
            if has_reflecting {
                header.col(|ui| {
                    ui.strong("\u{03b8} (deg)");
                });
                header.col(|ui| {
                    ui.strong("\u{03c8} (deg)");
                });
            }
            header.col(|ui| {
                ui.strong("");
            });
        })
        .body(|mut body| {
            let num_surfaces = specs.surfaces.len();
            // We need to collect actions to apply after iteration to avoid
            // borrow conflicts.
            let mut insert_after: Option<usize> = None;
            let mut delete_at: Option<usize> = None;

            for row_idx in 0..num_surfaces {
                body.row(22.0, |mut row| {
                    let surf = &mut specs.surfaces[row_idx];
                    let is_object = surf.variant == SurfaceVariant::Object;
                    let is_image = surf.variant == SurfaceVariant::Image;
                    let is_conic = surf.variant == SurfaceVariant::Conic;
                    let is_locked = is_object || is_image;

                    // # column
                    row.col(|ui| {
                        ui.label(row_idx.to_string());
                    });

                    // Variant column
                    row.col(|ui| {
                        if is_locked {
                            ui.label(surf.variant.to_string());
                        } else {
                            let id = ui.make_persistent_id(format!("variant_{row_idx}"));
                            egui::ComboBox::from_id_salt(id)
                                .selected_text(surf.variant.to_string())
                                .width(60.0)
                                .show_ui(ui, |ui| {
                                    for &v in SurfaceVariant::SELECTABLE {
                                        if ui
                                            .selectable_value(&mut surf.variant, v, v.to_string())
                                            .changed()
                                        {
                                            changed = true;
                                        }
                                    }
                                });
                        }
                    });

                    // Kind column (only for Conic)
                    row.col(|ui| {
                        if is_conic {
                            let id = ui.make_persistent_id(format!("kind_{row_idx}"));
                            egui::ComboBox::from_id_salt(id)
                                .selected_text(surf.surface_kind.to_string())
                                .width(80.0)
                                .show_ui(ui, |ui| {
                                    for kind in [SurfaceKind::Refracting, SurfaceKind::Reflecting] {
                                        if ui
                                            .selectable_value(
                                                &mut surf.surface_kind,
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
                    });

                    // Semi-Diameter
                    row.col(|ui| {
                        if !is_object && !is_image {
                            changed |= drag_value(
                                ui,
                                &mut surf.semi_diameter,
                                row_idx,
                                "sd",
                                0.0..=500.0,
                                0.1,
                            );
                        }
                    });

                    // Radius of Curvature
                    row.col(|ui| {
                        if is_conic {
                            changed |= drag_inf(
                                ui,
                                &mut surf.radius_of_curvature,
                                row_idx,
                                "roc",
                                f64::NEG_INFINITY..=f64::INFINITY,
                                1.0,
                            );
                        }
                    });

                    // Conic Constant
                    row.col(|ui| {
                        if is_conic {
                            changed |= drag_value(
                                ui,
                                &mut surf.conic_constant,
                                row_idx,
                                "cc",
                                -10.0..=10.0,
                                0.01,
                            );
                        }
                    });

                    // Thickness
                    row.col(|ui| {
                        if !is_image {
                            changed |= drag_inf(
                                ui,
                                &mut surf.thickness,
                                row_idx,
                                "thick",
                                0.0..=f64::INFINITY,
                                0.5,
                            );
                        }
                    });

                    // Refractive Index / Material
                    row.col(|ui| {
                        if !is_image {
                            if use_materials {
                                let display = surf.material_key.as_deref().unwrap_or("(none)");
                                let id = ui.make_persistent_id(format!("mat_{row_idx}"));
                                egui::ComboBox::from_id_salt(id)
                                    .selected_text(display)
                                    .width(130.0)
                                    .show_ui(ui, |ui| {
                                        if ui
                                            .selectable_label(surf.material_key.is_none(), "(none)")
                                            .clicked()
                                        {
                                            surf.material_key = None;
                                            changed = true;
                                        }
                                        for mat_key in &selected_materials {
                                            let selected = surf.material_key.as_deref()
                                                == Some(mat_key.as_str());
                                            if ui.selectable_label(selected, mat_key).clicked() {
                                                surf.material_key = Some(mat_key.clone());
                                                changed = true;
                                            }
                                        }
                                    });
                            } else {
                                changed |= drag_value(
                                    ui,
                                    &mut surf.refractive_index,
                                    row_idx,
                                    "n",
                                    1.0..=4.0,
                                    0.01,
                                );
                            }
                        }
                    });

                    // θ / ψ columns (only shown when system has reflecting surfaces)
                    if has_reflecting {
                        let is_reflecting_conic =
                            is_conic && surf.surface_kind == SurfaceKind::Reflecting;
                        row.col(|ui| {
                            if is_reflecting_conic {
                                changed |= drag_value(
                                    ui,
                                    &mut surf.theta,
                                    row_idx,
                                    "theta",
                                    -90.0..=90.0,
                                    0.5,
                                );
                            }
                        });
                        row.col(|ui| {
                            if is_reflecting_conic {
                                changed |= drag_value(
                                    ui,
                                    &mut surf.psi,
                                    row_idx,
                                    "psi",
                                    -90.0..=90.0,
                                    0.5,
                                );
                            }
                        });
                    }

                    // Actions column (always last).
                    // Object row: + only (cannot remove object surface).
                    // Image row: no buttons (cannot insert after or remove image).
                    // All other rows: + and -.
                    row.col(|ui| {
                        if !is_image {
                            ui.horizontal(|ui| {
                                if ui.small_button("+").clicked() {
                                    insert_after = Some(row_idx);
                                }
                                if !is_object && ui.small_button("-").clicked() {
                                    delete_at = Some(row_idx);
                                }
                            });
                        }
                    });
                });
            }

            // Apply deferred mutations
            if let Some(idx) = insert_after {
                specs.surfaces.insert(idx + 1, SurfaceRow::new_default());
                changed = true;
            }
            if let Some(idx) = delete_at
                && specs.surfaces.len() > 2
            {
                specs.surfaces.remove(idx);
                changed = true;
            }
        });

    changed
}

/// DragValue cell without special infinity handling.
fn drag_value(
    ui: &mut egui::Ui,
    field: &mut String,
    row: usize,
    col: &str,
    range: std::ops::RangeInclusive<f64>,
    speed: f64,
) -> bool {
    let mut val = parse_display_float(field);
    let response = ui.push_id(format!("cell_{row}_{col}"), |ui| {
        ui.add(egui::DragValue::new(&mut val).range(range).speed(speed))
    });
    if response.inner.changed() {
        *field = format_display_float(val);
        true
    } else {
        false
    }
}

/// DragValue cell with infinity-aware formatting: displays and accepts
/// `"Infinity"` as a string value. Use this for fields that legitimately hold
/// `f64::INFINITY` (e.g. RoC, thickness).
fn drag_inf(
    ui: &mut egui::Ui,
    field: &mut String,
    row: usize,
    col: &str,
    range: std::ops::RangeInclusive<f64>,
    speed: f64,
) -> bool {
    let mut val = parse_display_float(field);
    let response = ui.push_id(format!("cell_{row}_{col}"), |ui| {
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

    use crate::gui::model::{SurfaceKind, SurfaceRow, SurfaceVariant, SystemSpecs};

    fn specs_with_reflecting_surface() -> SystemSpecs {
        let mut mirror = SurfaceRow::new_conic("12.7", "Infinity", "0.0", "100.0", "1.0");
        mirror.surface_kind = SurfaceKind::Reflecting;
        SystemSpecs {
            surfaces: vec![
                SurfaceRow::new_object("Infinity"),
                mirror,
                SurfaceRow::new_image(),
            ],
            ..Default::default()
        }
    }

    fn minimal_specs() -> SystemSpecs {
        SystemSpecs {
            surfaces: vec![SurfaceRow::new_object("Infinity"), SurfaceRow::new_image()],
            ..Default::default()
        }
    }

    /// The object row must have a + button so users can insert surfaces after
    /// it.
    #[test]
    fn object_row_has_add_button() {
        let mut specs = minimal_specs();
        let mut harness = Harness::new_ui(|ui| {
            surfaces_panel(ui, &mut specs);
        });
        harness.run();
        // Panics if no + button is found — that is the red state.
        harness.get_by_label("+");
    }

    /// Clicking + on the object row inserts a new surface after it.
    #[test]
    fn clicking_add_on_object_row_inserts_surface() {
        let mut specs = minimal_specs();
        {
            let mut harness = Harness::new_ui(|ui| {
                surfaces_panel(ui, &mut specs);
            });
            harness.run();
            harness.get_by_label("+").click();
            harness.run();
        }
        assert_eq!(
            specs.surfaces.len(),
            3,
            "clicking + on the object row should insert a surface"
        );
        assert_eq!(specs.surfaces[1].variant, SurfaceVariant::Conic);
    }

    /// The actions (+/-) column must always be the last column, appearing to
    /// the right of the θ/ψ columns when a reflecting surface is present.
    #[test]
    fn actions_column_is_rightmost_with_reflecting_surfaces() {
        let mut specs = specs_with_reflecting_surface();
        let mut harness = Harness::new_ui(|ui| {
            surfaces_panel(ui, &mut specs);
        });
        harness.run();

        let theta_header = harness.get_by_label("\u{03b8} (deg)");
        let add_button = harness
            .get_all_by_label("+")
            .next()
            .expect("at least one + button should be present");

        assert!(
            add_button.rect().center().x > theta_header.rect().center().x,
            "actions column must be to the right of θ column: \
             + button at x={:.1}, θ header at x={:.1}",
            add_button.rect().center().x,
            theta_header.rect().center().x,
        );
    }
}
