use serde::{Deserialize, Serialize};

/// Which surface variant this row represents.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SurfaceVariant {
    Object,
    Conic,
    Stop,
    Probe,
    Image,
}

impl SurfaceVariant {
    /// Variants available for user selection (excludes Object and Image which
    /// are fixed).
    pub const SELECTABLE: &[SurfaceVariant] = &[
        SurfaceVariant::Conic,
        SurfaceVariant::Stop,
        SurfaceVariant::Probe,
    ];
}

impl std::fmt::Display for SurfaceVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SurfaceVariant::Object => write!(f, "Object"),
            SurfaceVariant::Conic => write!(f, "Conic"),
            SurfaceVariant::Stop => write!(f, "Stop"),
            SurfaceVariant::Probe => write!(f, "Probe"),
            SurfaceVariant::Image => write!(f, "Image"),
        }
    }
}

/// Whether a conic surface is refracting or reflecting.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SurfaceKind {
    Refracting,
    Reflecting,
}

impl std::fmt::Display for SurfaceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SurfaceKind::Refracting => write!(f, "Refracting"),
            SurfaceKind::Reflecting => write!(f, "Reflecting"),
        }
    }
}

/// A single row in the surfaces table. All numeric fields are strings for
/// editing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceRow {
    pub variant: SurfaceVariant,
    pub surface_kind: SurfaceKind,
    pub refractive_index: String,
    pub thickness: String,
    pub semi_diameter: String,
    pub radius_of_curvature: String,
    pub conic_constant: String,
}

impl SurfaceRow {
    pub fn new_object(thickness: &str) -> Self {
        Self {
            variant: SurfaceVariant::Object,
            surface_kind: SurfaceKind::Refracting,
            refractive_index: "1.0".into(),
            thickness: thickness.into(),
            semi_diameter: String::new(),
            radius_of_curvature: String::new(),
            conic_constant: String::new(),
        }
    }

    pub fn new_conic(
        semi_diameter: &str,
        radius_of_curvature: &str,
        conic_constant: &str,
        thickness: &str,
        refractive_index: &str,
    ) -> Self {
        Self {
            variant: SurfaceVariant::Conic,
            surface_kind: SurfaceKind::Refracting,
            refractive_index: refractive_index.into(),
            thickness: thickness.into(),
            semi_diameter: semi_diameter.into(),
            radius_of_curvature: radius_of_curvature.into(),
            conic_constant: conic_constant.into(),
        }
    }

    pub fn new_stop(semi_diameter: &str, thickness: &str, refractive_index: &str) -> Self {
        Self {
            variant: SurfaceVariant::Stop,
            surface_kind: SurfaceKind::Refracting,
            refractive_index: refractive_index.into(),
            thickness: thickness.into(),
            semi_diameter: semi_diameter.into(),
            radius_of_curvature: String::new(),
            conic_constant: String::new(),
        }
    }

    pub fn new_image() -> Self {
        Self {
            variant: SurfaceVariant::Image,
            surface_kind: SurfaceKind::Refracting,
            refractive_index: String::new(),
            thickness: String::new(),
            semi_diameter: String::new(),
            radius_of_curvature: String::new(),
            conic_constant: String::new(),
        }
    }

    /// Create a default new surface for insertion.
    pub fn new_default() -> Self {
        Self::new_conic("10.0", "Infinity", "0.0", "1.0", "1.0")
    }
}

/// A single row in the fields table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldRow {
    /// Angle in degrees (Angle mode) or Y position (PointSource mode).
    pub value: String,
    /// X position (PointSource mode only).
    pub x: String,
    /// Pupil sampling grid spacing.
    pub pupil_spacing: String,
}

/// Which field specification mode is active.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FieldMode {
    Angle,
    PointSource,
}

impl std::fmt::Display for FieldMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldMode::Angle => write!(f, "Angle"),
            FieldMode::PointSource => write!(f, "Point Source"),
        }
    }
}

/// Which specs tab is active.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SpecsTab {
    Surfaces,
    Fields,
    Aperture,
    Wavelengths,
}

/// All user-editable input specifications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSpecs {
    pub surfaces: Vec<SurfaceRow>,
    pub fields: Vec<FieldRow>,
    pub aperture_semi_diameter: String,
    pub wavelengths: Vec<String>,
    pub field_mode: FieldMode,
}

impl Default for SystemSpecs {
    /// Default system: f = 50 mm convexplano lens.
    fn default() -> Self {
        Self {
            surfaces: vec![
                SurfaceRow::new_object("Infinity"),
                SurfaceRow::new_conic("12.5", "25.8", "0.0", "5.3", "1.515"),
                SurfaceRow::new_conic("12.5", "Infinity", "0.0", "46.6", "1.0"),
                SurfaceRow::new_image(),
            ],
            fields: vec![FieldRow {
                value: "0.0".into(),
                x: "0.0".into(),
                pupil_spacing: "0.1".into(),
            }],
            aperture_semi_diameter: "12.5".into(),
            wavelengths: vec!["0.567".into()],
            field_mode: FieldMode::Angle,
        }
    }
}
