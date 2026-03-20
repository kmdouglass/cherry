mod aperture;
mod fields;
#[cfg(feature = "ri-info")]
mod materials;
mod surfaces;
mod system;
mod wavelengths;

pub use aperture::aperture_panel;
pub use fields::fields_panel;
#[cfg(feature = "ri-info")]
pub use materials::{MaterialBrowserState, MaterialIndex, materials_panel};
pub use surfaces::surfaces_panel;
pub use system::system_panel;
pub use wavelengths::wavelengths_panel;

/// Parse a string to f64 for display in a DragValue, treating
/// "Infinity"/"inf"/"∞" as `f64::INFINITY` and falling back to `0.0` on
/// error so an invalid string doesn't crash the widget.
pub(super) fn parse_display_float(s: &str) -> f64 {
    match s.trim().to_lowercase().as_str() {
        "infinity" | "inf" | "∞" => f64::INFINITY,
        "-infinity" | "-inf" | "-∞" => f64::NEG_INFINITY,
        other => other.parse().unwrap_or(0.0),
    }
}

/// Write an f64 back to a string, using "Infinity"/"-Infinity" for infinite
/// values to match the save-file format.
pub(super) fn format_display_float(val: f64) -> String {
    if val == f64::INFINITY {
        "Infinity".to_owned()
    } else if val == f64::NEG_INFINITY {
        "-Infinity".to_owned()
    } else {
        val.to_string()
    }
}

/// DragValue formatter: shows "∞"/"-∞" for infinite values.
pub(super) fn inf_formatter(n: f64, _: std::ops::RangeInclusive<usize>) -> String {
    if n == f64::INFINITY {
        "∞".to_owned()
    } else if n == f64::NEG_INFINITY {
        "-∞".to_owned()
    } else {
        format!("{n}")
    }
}

/// DragValue parser: accepts "∞"/"inf"/"infinity" (case-insensitive) as
/// infinity, otherwise defers to the default numeric parser.
pub(super) fn inf_parser(s: &str) -> Option<f64> {
    match s.trim().to_lowercase().as_str() {
        "∞" | "inf" | "infinity" => Some(f64::INFINITY),
        "-∞" | "-inf" | "-infinity" => Some(f64::NEG_INFINITY),
        _ => s.trim().parse().ok(),
    }
}
