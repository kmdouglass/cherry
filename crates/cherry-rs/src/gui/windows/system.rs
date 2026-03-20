use crate::gui::{model::SystemSpecs, panels};

pub struct SystemWindow;

impl SystemWindow {
    pub fn show(ctx: &egui::Context, open: &mut bool, specs: &mut SystemSpecs) -> bool {
        let response = egui::Window::new("System")
            .open(open)
            .default_width(250.0)
            .show(ctx, |ui| panels::system_panel(ui, specs));
        response.and_then(|r| r.inner).unwrap_or(false)
    }
}
