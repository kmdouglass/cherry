use egui_extras::{Column, TableBuilder};

use super::super::model::{FieldMode, FieldRow, SystemSpecs};
use super::{format_display_float, parse_display_float};

/// Draw the fields editor panel. Returns true if any spec was modified.
pub fn fields_panel(ui: &mut egui::Ui, specs: &mut SystemSpecs) -> bool {
    let mut changed = false;

    // Field mode toggle
    ui.horizontal(|ui| {
        ui.label("Mode:");
        if ui
            .selectable_label(specs.field_mode == FieldMode::Angle, "Angle")
            .clicked()
        {
            specs.field_mode = FieldMode::Angle;
            changed = true;
        }
        if ui
            .selectable_label(specs.field_mode == FieldMode::PointSource, "Point Source")
            .clicked()
        {
            specs.field_mode = FieldMode::PointSource;
            changed = true;
        }
    });

    ui.separator();

    let is_angle = specs.field_mode == FieldMode::Angle;

    let table = TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto().at_least(30.0)) // #
        .column(Column::initial(100.0).resizable(true)) // Value (angle or y)
        .columns(
            Column::initial(100.0).resizable(true),
            if is_angle { 0 } else { 1 }, // X (only for PointSource)
        )
        .column(Column::initial(100.0).resizable(true)) // Pupil spacing
        .column(Column::auto().at_least(50.0)); // Actions

    table
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.strong("#");
            });
            header.col(|ui| {
                if is_angle {
                    ui.strong("Angle (deg)");
                } else {
                    ui.strong("Y");
                }
            });
            if !is_angle {
                header.col(|ui| {
                    ui.strong("X");
                });
            }
            header.col(|ui| {
                ui.strong("Pupil Spacing");
            });
            header.col(|ui| {
                ui.strong("");
            });
        })
        .body(|mut body| {
            let num_fields = specs.fields.len();
            let mut insert_after: Option<usize> = None;
            let mut delete_at: Option<usize> = None;

            for row_idx in 0..num_fields {
                body.row(22.0, |mut row| {
                    let field = &mut specs.fields[row_idx];

                    row.col(|ui| {
                        ui.label(row_idx.to_string());
                    });

                    // Value column (angle or y position)
                    row.col(|ui| {
                        if is_angle {
                            changed |=
                                drag_value(ui, &mut field.value, row_idx, "val", -90.0..=90.0, 0.1);
                        } else {
                            changed |= drag_value(
                                ui,
                                &mut field.value,
                                row_idx,
                                "val",
                                f64::NEG_INFINITY..=f64::INFINITY,
                                1.0,
                            );
                        }
                    });

                    // X column (PointSource only)
                    if !is_angle {
                        row.col(|ui| {
                            changed |= drag_value(
                                ui,
                                &mut field.x,
                                row_idx,
                                "x",
                                f64::NEG_INFINITY..=f64::INFINITY,
                                1.0,
                            );
                        });
                    }

                    // Pupil spacing
                    row.col(|ui| {
                        changed |= drag_value(
                            ui,
                            &mut field.pupil_spacing,
                            row_idx,
                            "ps",
                            0.001..=1.0,
                            0.001,
                        );
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
                specs.fields.insert(
                    idx + 1,
                    FieldRow {
                        value: "0.0".into(),
                        x: "0.0".into(),
                        pupil_spacing: "0.1".into(),
                    },
                );
                changed = true;
            }
            if let Some(idx) = delete_at {
                specs.fields.remove(idx);
                changed = true;
            }
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
