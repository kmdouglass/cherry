/// Map a wavelength in μm to an approximate visible-spectrum color.
pub fn wavelength_to_color(wl_um: f64) -> egui::Color32 {
    let nm = wl_um * 1000.0;
    if nm <= 450.0 {
        egui::Color32::from_rgb(110, 0, 220)
    } else if nm <= 520.0 {
        egui::Color32::from_rgb(0, 100, 255)
    } else if nm <= 565.0 {
        egui::Color32::from_rgb(0, 200, 50)
    } else if nm <= 600.0 {
        egui::Color32::from_rgb(220, 220, 0)
    } else {
        egui::Color32::from_rgb(220, 30, 0)
    }
}
