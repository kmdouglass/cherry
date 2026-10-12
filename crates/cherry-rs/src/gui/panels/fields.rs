use egui_extras::{Column, TableBuilder};

use super::super::model::{FieldMode, FieldRow, SystemSpecs};
use super::{format_display_float, parse_display_float};

/// Draw the fields editor panel for `specs.paths[active_path]`. Returns true
/// if any spec was modified.
pub fn fields_panel(ui: &mut egui::Ui, specs: &mut SystemSpecs, active_path: usize) -> bool {
    let mut changed = false;
    let Some(path) = specs.paths.get_mut(active_path) else {
        return false;
    };

    // Field mode toggle
    ui.horizontal(|ui| {
        ui.label("Mode:");
        if ui
            .selectable_label(path.field_mode == FieldMode::Angle, "Angle")
            .clicked()
        {
            path.field_mode = FieldMode::Angle;
            changed = true;
        }
        if ui
            .selectable_label(path.field_mode == FieldMode::PointSource, "Point Source")
            .clicked()
        {
            path.field_mode = FieldMode::PointSource;
            changed = true;
        }
    });

    ui.separator();

    let is_angle = path.field_mode == FieldMode::Angle;

    egui::ScrollArea::horizontal().show(ui, |ui| {
        let table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .sense(egui::Sense::click())
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto().at_least(30.0)) // #
            .column(Column::initial(100.0).resizable(true)) // χ (angle) or Y (point source)
            .column(Column::initial(100.0).resizable(true)) // φ (angle mode) or X (point source)
            .column(Column::auto().at_least(50.0)); // Actions

        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("#");
                });
                header.col(|ui| {
                    if is_angle {
                        ui.strong("\u{03c7} (\u{00b0})");
                    } else {
                        ui.strong("Y");
                    }
                });
                header.col(|ui| {
                    if is_angle {
                        ui.strong("\u{03c6} (\u{00b0})");
                    } else {
                        ui.strong("X");
                    }
                });
                header.col(|ui| {
                    ui.strong("");
                });
            })
            .body(|mut body| {
                let num_fields = path.fields.len();
                let mut insert_after: Option<usize> = None;
                let mut delete_at: Option<usize> = None;

                for row_idx in 0..num_fields {
                    body.row(22.0, |mut row| {
                        let field = &mut path.fields[row_idx];

                        row.col(|ui| {
                            ui.label(row_idx.to_string());
                        });

                        // Value column (angle or y position)
                        row.col(|ui| {
                            if is_angle {
                                changed |= drag_value(
                                    ui,
                                    &mut field.chi,
                                    row_idx,
                                    "val",
                                    -90.0..=90.0,
                                    0.1,
                                );
                            } else {
                                changed |= drag_value(
                                    ui,
                                    &mut field.chi,
                                    row_idx,
                                    "val",
                                    f64::NEG_INFINITY..=f64::INFINITY,
                                    1.0,
                                );
                            }
                        });

                        // φ column (Angle mode) or X column (PointSource mode)
                        row.col(|ui| {
                            if is_angle {
                                changed |= drag_value(
                                    ui,
                                    &mut field.phi,
                                    row_idx,
                                    "phi",
                                    -180.0..=180.0,
                                    1.0,
                                );
                            } else {
                                changed |= drag_value(
                                    ui,
                                    &mut field.x,
                                    row_idx,
                                    "x",
                                    f64::NEG_INFINITY..=f64::INFINITY,
                                    1.0,
                                );
                            }
                        });

                        // Actions
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                if ui.small_button("+").clicked() {
                                    insert_after = Some(row_idx);
                                }
                                if num_fields > 1 && ui.small_button("-").clicked() {
                                    delete_at = Some(row_idx);
                                }
                            });
                        });
                    });
                }

                if let Some(idx) = insert_after {
                    path.fields.insert(
                        idx + 1,
                        FieldRow {
                            chi: "0.0".into(),
                            phi: "90.0".into(),
                            x: "0.0".into(),
                        },
                    );
                    changed = true;
                }
                if let Some(idx) = delete_at {
                    path.fields.remove(idx);
                    changed = true;
                }
            });
    });

    changed
}

fn drag_value(
    ui: &mut egui::Ui,
    field: &mut String,
    row: usize,
    col: &str,
    range: std::ops::RangeInclusive<f64>,
    speed: f64,
) -> bool {
    let mut val = parse_display_float(field);
    let response = ui.push_id(format!("field_{row}_{col}"), |ui| {
        ui.add(egui::DragValue::new(&mut val).range(range).speed(speed))
    });
    if response.inner.changed() {
        *field = format_display_float(val);
        true
    } else {
        false
    }
}
