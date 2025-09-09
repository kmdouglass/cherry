use egui_extras::{Column, TableBuilder};

use super::super::model::{SurfaceKind, SurfaceRow, SurfaceVariant, SystemSpecs};

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
        .column(Column::initial(n_col_width).resizable(true)) // n / Material
        .column(Column::auto().at_least(50.0)); // Actions

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
                            changed |= editable_cell(ui, &mut surf.semi_diameter, row_idx, "sd");
                        }
                    });

                    // Radius of Curvature
                    row.col(|ui| {
                        if is_conic {
                            changed |=
                                editable_cell(ui, &mut surf.radius_of_curvature, row_idx, "roc");
                        }
                    });

                    // Conic Constant
                    row.col(|ui| {
                        if is_conic {
                            changed |= editable_cell(ui, &mut surf.conic_constant, row_idx, "cc");
                        }
                    });

                    // Thickness
                    row.col(|ui| {
                        if !is_image {
                            changed |= editable_cell(ui, &mut surf.thickness, row_idx, "thick");
                        }
                    });

                    // Refractive Index / Material
                    row.col(|ui| {
                        if !is_image {
                            if use_materials {
                                let display =
                                    surf.material_key.as_deref().unwrap_or("(none)");
                                let id =
                                    ui.make_persistent_id(format!("mat_{row_idx}"));
                                egui::ComboBox::from_id_salt(id)
                                    .selected_text(display)
                                    .width(130.0)
                                    .show_ui(ui, |ui| {
                                        if ui
                                            .selectable_label(
                                                surf.material_key.is_none(),
                                                "(none)",
                                            )
                                            .clicked()
                                        {
                                            surf.material_key = None;
                                            changed = true;
                                        }
                                        for mat_key in &selected_materials {
                                            let selected = surf
                                                .material_key
                                                .as_deref()
                                                == Some(mat_key.as_str());
                                            if ui
                                                .selectable_label(selected, mat_key)
                                                .clicked()
                                            {
                                                surf.material_key =
                                                    Some(mat_key.clone());
                                                changed = true;
                                            }
                                        }
                                    });
                            } else {
                                changed |= editable_cell(
                                    ui,
                                    &mut surf.refractive_index,
                                    row_idx,
                                    "n",
                                );
                            }
                        }
                    });

                    // Actions column
                    row.col(|ui| {
                        if !is_locked {
                            ui.horizontal(|ui| {
                                if ui.small_button("+").clicked() {
                                    insert_after = Some(row_idx);
                                }
                                if ui.small_button("-").clicked() {
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
            if let Some(idx) = delete_at {
                if specs.surfaces.len() > 2 {
                    specs.surfaces.remove(idx);
                    changed = true;
                }
            }
        });

    changed
}

fn editable_cell(ui: &mut egui::Ui, value: &mut String, row: usize, col: &str) -> bool {
    let response = ui.add(
        egui::TextEdit::singleline(value)
            .desired_width(70.0)
            .horizontal_align(egui::Align::RIGHT)
            .id(egui::Id::new(format!("cell_{row}_{col}"))),
    );
    response.changed()
}
