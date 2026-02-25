use crate::{Axis, ParaxialView, SequentialModel, SubModelID};

/// Show the paraxial summary window.
pub fn summary_window(
    ctx: &egui::Context,
    open: &mut bool,
    sequential_model: &SequentialModel,
    paraxial_view: &ParaxialView,
) {
    egui::Window::new("Paraxial Summary")
        .open(open)
        .default_width(350.0)
        .show(ctx, |ui| {
            let wavelengths = sequential_model.wavelengths();
            let axes = sequential_model.axes();

            for (i, wl) in wavelengths.iter().enumerate() {
                for axis in &axes {
                    let id = SubModelID(i, *axis);
                    let Some(sv) = paraxial_view.subviews().get(&id) else {
                        continue;
                    };

                    // Only show header if there are multiple subviews.
                    if wavelengths.len() > 1 || axes.len() > 1 {
                        let axis_label = match axis {
                            Axis::X => "X",
                            Axis::Y => "Y",
                        };
                        ui.heading(format!("{:.4} \u{00b5}m — {axis_label}", wl));
                        ui.separator();
                    }

                    egui::Grid::new(format!("summary_{i}_{axis:?}"))
                        .num_columns(2)
                        .spacing([20.0, 4.0])
                        .show(ui, |ui| {
                            row(ui, "EFL", *sv.effective_focal_length());
                            row(ui, "BFD", *sv.back_focal_distance());
                            row(ui, "FFD", *sv.front_focal_distance());

                            ui.separator();
                            ui.separator();
                            ui.end_row();

                            let ep = sv.entrance_pupil();
                            row(ui, "Entrance pupil location", ep.location);
                            row(ui, "Entrance pupil semi-dia", ep.semi_diameter);

                            let xp = sv.exit_pupil();
                            row(ui, "Exit pupil location", xp.location);
                            row(ui, "Exit pupil semi-dia", xp.semi_diameter);

                            ui.separator();
                            ui.separator();
                            ui.end_row();

                            row(ui, "Front principal plane", *sv.front_principal_plane());
                            row(ui, "Back principal plane", *sv.back_principal_plane());

                            ui.separator();
                            ui.separator();
                            ui.end_row();

                            row(ui, "Aperture stop (surface)", *sv.aperture_stop() as f64);
                        });

                    ui.add_space(8.0);
                }
            }
        });
}

fn row(ui: &mut egui::Ui, label: &str, value: f64) {
    ui.label(label);
    ui.label(format_value(value));
    ui.end_row();
}

fn format_value(v: f64) -> String {
    if v.is_infinite() {
        if v.is_sign_positive() {
            "\u{221e}".to_string()
        } else {
            "-\u{221e}".to_string()
        }
    } else if v == v.floor() && v.abs() < 1e12 {
        format!("{v:.1}")
    } else {
        format!("{v:.6}")
    }
}
