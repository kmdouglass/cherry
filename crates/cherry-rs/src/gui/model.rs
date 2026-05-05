use std::fmt;

use serde::{Deserialize, Serialize};

/// Which table parameter a solve controls.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SolveParameter {
    Thickness,
    RadiusOfCurvature,
}

impl fmt::Display for SolveParameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Thickness => write!(f, "Thickness"),
            Self::RadiusOfCurvature => write!(f, "Curvature"),
        }
    }
}

/// Serializable description of one active solve on the system.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SolveSpec {
    MarginalRayHeight {
        gap_index: usize,
        target_height: f64,
        wavelength_id: usize,
    },
    FNumber {
        surface_index: usize,
        target_fno: f64,
        wavelength_id: usize,
    },
}

impl SolveSpec {
    pub(crate) fn surface_index(&self) -> usize {
        match self {
            Self::MarginalRayHeight { gap_index, .. } => *gap_index,
            Self::FNumber { surface_index, .. } => *surface_index,
        }
    }

    pub(crate) fn set_surface_index(&mut self, idx: usize) {
        match self {
            Self::MarginalRayHeight { gap_index, .. } => *gap_index = idx,
            Self::FNumber { surface_index, .. } => *surface_index = idx,
        }
    }

    pub(crate) fn parameter(&self) -> SolveParameter {
        match self {
            Self::MarginalRayHeight { .. } => SolveParameter::Thickness,
            Self::FNumber { .. } => SolveParameter::RadiusOfCurvature,
        }
    }

    pub(crate) fn is_paraxial(&self) -> bool {
        match self {
            Self::MarginalRayHeight { .. } | Self::FNumber { .. } => true,
        }
    }
}

/// Transient UI state for the solve-configuration popup. Lives in `model.rs`
/// so both the window and panel can reference it without a circular dependency.
#[derive(Clone, Debug)]
pub struct SolvePopupState {
    pub surface_index: usize,
    pub parameter: SolveParameter,
    pub type_str: String,
    pub target: String,
    pub wavelength_id: usize,
    pub error: Option<String>,
}

impl SolvePopupState {
    pub fn open(
        surface_index: usize,
        parameter: SolveParameter,
        existing: Option<&SolveSpec>,
    ) -> Self {
        match existing {
            None => Self {
                surface_index,
                parameter,
                type_str: "None".to_owned(),
                target: String::new(),
                wavelength_id: 0,
                error: None,
            },
            Some(SolveSpec::MarginalRayHeight {
                target_height,
                wavelength_id,
                ..
            }) => Self {
                surface_index,
                parameter,
                type_str: "Marginal ray height".to_owned(),
                target: target_height.to_string(),
                wavelength_id: *wavelength_id,
                error: None,
            },
            Some(SolveSpec::FNumber {
                target_fno,
                wavelength_id,
                ..
            }) => Self {
                surface_index,
                parameter,
                type_str: "F/#".to_owned(),
                target: target_fno.to_string(),
                wavelength_id: *wavelength_id,
                error: None,
            },
        }
    }

    /// Returns true if the currently selected solve type is a paraxial solve.
    pub fn is_paraxial(&self) -> bool {
        match self.type_str.as_str() {
            "Marginal ray height" => SolveSpec::MarginalRayHeight {
                gap_index: 0,
                target_height: 0.0,
                wavelength_id: 0,
            }
            .is_paraxial(),
            "F/#" => SolveSpec::FNumber {
                surface_index: 0,
                target_fno: 1.0,
                wavelength_id: 0,
            }
            .is_paraxial(),
            _ => false,
        }
    }
}

/// Which surface variant this row represents.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SurfaceVariant {
    Object,
    Sphere,
    Conic,
    Iris,
    Probe,
    Image,
}

impl SurfaceVariant {
    /// Variants available for user selection (excludes Object and Image which
    /// are fixed).
    pub const SELECTABLE: &[SurfaceVariant] = &[
        SurfaceVariant::Sphere,
        SurfaceVariant::Conic,
        SurfaceVariant::Iris,
        SurfaceVariant::Probe,
    ];
}

impl std::fmt::Display for SurfaceVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SurfaceVariant::Object => write!(f, "Object"),
            SurfaceVariant::Sphere => write!(f, "Sphere"),
            SurfaceVariant::Conic => write!(f, "Conic"),
            SurfaceVariant::Iris => write!(f, "Iris"),
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

fn default_zero() -> String {
    "0".to_string()
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
    /// Tilt in UF plane (about cursor-R axis), degrees. Only meaningful for
    /// reflecting Conic surfaces.
    #[serde(default = "default_zero")]
    pub theta: String,
    /// Tilt in RF plane (about cursor-U axis), degrees. Only meaningful for
    /// reflecting Conic surfaces.
    #[serde(default = "default_zero")]
    pub psi: String,
    /// Material key from rii.db (e.g. "glass:BK7:SCHOTT"). Used when
    /// `SystemSpecs::use_materials` is true.
    #[serde(default)]
    pub material_key: Option<String>,
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
            theta: "0".into(),
            psi: "0".into(),
            material_key: None,
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
            theta: "0".into(),
            psi: "0".into(),
            material_key: None,
        }
    }

    pub fn new_sphere(
        semi_diameter: &str,
        radius_of_curvature: &str,
        thickness: &str,
        refractive_index: &str,
    ) -> Self {
        Self {
            variant: SurfaceVariant::Sphere,
            surface_kind: SurfaceKind::Refracting,
            refractive_index: refractive_index.into(),
            thickness: thickness.into(),
            semi_diameter: semi_diameter.into(),
            radius_of_curvature: radius_of_curvature.into(),
            conic_constant: String::new(),
            theta: "0".into(),
            psi: "0".into(),
            material_key: None,
        }
    }

    pub fn new_iris(semi_diameter: &str, thickness: &str, refractive_index: &str) -> Self {
        Self {
            variant: SurfaceVariant::Iris,
            surface_kind: SurfaceKind::Refracting,
            refractive_index: refractive_index.into(),
            thickness: thickness.into(),
            semi_diameter: semi_diameter.into(),
            radius_of_curvature: String::new(),
            conic_constant: String::new(),
            theta: "0".into(),
            psi: "0".into(),
            material_key: None,
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
            theta: "0".into(),
            psi: "0".into(),
            material_key: None,
        }
    }

    /// Create a default new surface for insertion.
    pub fn new_default() -> Self {
        Self::new_sphere("10.0", "Infinity", "1.0", "1.0")
    }
}

fn default_phi() -> String {
    "90.0".into()
}

/// A single row in the fields table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldRow {
    /// χ (chi): zenith angle in degrees (Angle mode) or Y position (PointSource
    /// mode).
    pub chi: String,
    /// φ (phi): azimuthal angle in degrees (Angle mode only). Defaults to 90.0.
    #[serde(default = "default_phi")]
    pub phi: String,
    /// X position (PointSource mode only).
    pub x: String,
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
    /// When true, surfaces use material keys instead of constant n.
    #[serde(default)]
    pub use_materials: bool,
    /// Material keys the user has selected for use in the surfaces table.
    #[serde(default)]
    pub selected_materials: Vec<String>,
    /// Number of rays in the tangential fan for the cross-section view.
    #[serde(default = "default_cross_section_n_rays")]
    pub cross_section_n_rays: u32,
    /// Grid spacing for full-pupil sampling, in normalised pupil coordinates
    /// [0, 1].
    #[serde(default = "default_full_pupil_spacing")]
    pub full_pupil_spacing: String,
    /// Number of rays in each tangential/sagittal fan bundle. Must be odd;
    /// range 3–501. Controls TA curve resolution in the Ray Fan Plot window.
    #[serde(default = "default_n_fan_rays")]
    pub n_fan_rays: u32,
    /// Refractive index of the background medium (used in constant-n mode).
    #[serde(default = "default_background_n")]
    pub background_n: String,
    /// Material key for the background medium (used in materials mode).
    #[serde(default)]
    pub background_material_key: Option<String>,
    /// User-designated aperture stop surface index. `None` = auto-derived.
    #[serde(default)]
    pub stop_surface: Option<usize>,
    /// Active solves on the system, serialized as part of system state.
    #[serde(default)]
    pub solves: Vec<SolveSpec>,
}

impl SystemSpecs {
    /// Insert a default surface after index `idx` and adjust `stop_surface` and
    /// solves.
    pub fn insert_surface_after(&mut self, idx: usize) {
        self.surfaces.insert(idx + 1, SurfaceRow::new_default());
        if let Some(stop) = self.stop_surface
            && idx < stop
        {
            self.stop_surface = Some(stop + 1);
        }
        for solve in &mut self.solves {
            if solve.surface_index() > idx {
                solve.set_surface_index(solve.surface_index() + 1);
            }
        }
    }

    /// Remove the surface at index `idx` and adjust `stop_surface` and solves.
    ///
    /// Does nothing if fewer than 3 surfaces remain (must keep object + image).
    pub fn delete_surface(&mut self, idx: usize) {
        if self.surfaces.len() <= 2 {
            return;
        }
        self.surfaces.remove(idx);
        self.stop_surface = match self.stop_surface {
            Some(stop) if stop == idx => None,
            Some(stop) if idx < stop => Some(stop - 1),
            other => other,
        };
        self.solves.retain(|s| s.surface_index() != idx);
        for solve in &mut self.solves {
            if solve.surface_index() > idx {
                solve.set_surface_index(solve.surface_index() - 1);
            }
        }
    }

    /// Return the active solve for a given cell, if any.
    pub fn solve_for(&self, surface_index: usize, parameter: SolveParameter) -> Option<&SolveSpec> {
        self.solves
            .iter()
            .find(|s| s.surface_index() == surface_index && s.parameter() == parameter)
    }
}

fn default_cross_section_n_rays() -> u32 {
    3
}

fn default_n_fan_rays() -> u32 {
    65
}

fn default_full_pupil_spacing() -> String {
    "0.1".to_owned()
}

fn default_background_n() -> String {
    "1.0".to_owned()
}

impl Default for SystemSpecs {
    /// Default system: f = 50 mm convexplano lens.
    fn default() -> Self {
        Self {
            surfaces: vec![
                SurfaceRow::new_object("Infinity"),
                SurfaceRow::new_sphere("12.5", "25.8", "5.3", "1.515"),
                SurfaceRow::new_sphere("12.5", "Infinity", "46.6", "1.0"),
                SurfaceRow::new_image(),
            ],
            fields: vec![FieldRow {
                chi: "0.0".into(),
                phi: "90.0".into(),
                x: "0.0".into(),
            }],
            aperture_semi_diameter: "12.5".into(),
            wavelengths: vec!["0.567".into()],
            field_mode: FieldMode::Angle,
            use_materials: false,
            selected_materials: Vec::new(),
            cross_section_n_rays: 3,
            full_pupil_spacing: "0.1".into(),
            n_fan_rays: 65,
            background_n: "1.0".into(),
            background_material_key: None,
            stop_surface: None,
            solves: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Layout used by mutation tests:
    //   0: Object, 1: Sphere, 2: Sphere, 3: Iris, 4: Image
    fn five_surface_specs() -> SystemSpecs {
        SystemSpecs {
            surfaces: vec![
                SurfaceRow::new_object("Infinity"),
                SurfaceRow::new_sphere("10.0", "50.0", "5.0", "1.5"),
                SurfaceRow::new_sphere("10.0", "Infinity", "5.0", "1.0"),
                SurfaceRow::new_iris("5.0", "1.0", "1.0"),
                SurfaceRow::new_image(),
            ],
            ..Default::default()
        }
    }

    // --- insert_surface_after ---

    #[test]
    fn insert_before_stop_increments_stop() {
        let mut specs = five_surface_specs();
        specs.stop_surface = Some(3);
        specs.insert_surface_after(1); // inserts at index 2, stop was 3 → becomes 4
        assert_eq!(specs.stop_surface, Some(4));
    }

    #[test]
    fn insert_at_stop_index_increments_stop() {
        let mut specs = five_surface_specs();
        specs.stop_surface = Some(2);
        specs.insert_surface_after(2); // inserts at index 3, stop was 2 → unchanged
        assert_eq!(specs.stop_surface, Some(2));
    }

    #[test]
    fn insert_after_stop_leaves_stop_unchanged() {
        let mut specs = five_surface_specs();
        specs.stop_surface = Some(2);
        specs.insert_surface_after(3); // inserts after stop → unchanged
        assert_eq!(specs.stop_surface, Some(2));
    }

    #[test]
    fn insert_with_no_stop_stays_none() {
        let mut specs = five_surface_specs();
        specs.stop_surface = None;
        specs.insert_surface_after(1);
        assert_eq!(specs.stop_surface, None);
    }

    // --- delete_surface ---

    #[test]
    fn delete_at_stop_clears_stop() {
        let mut specs = five_surface_specs();
        specs.stop_surface = Some(2);
        specs.delete_surface(2);
        assert_eq!(specs.stop_surface, None);
    }

    #[test]
    fn delete_before_stop_decrements_stop() {
        let mut specs = five_surface_specs();
        specs.stop_surface = Some(3);
        specs.delete_surface(1); // delete before stop → stop was 3, becomes 2
        assert_eq!(specs.stop_surface, Some(2));
    }

    #[test]
    fn delete_after_stop_leaves_stop_unchanged() {
        let mut specs = five_surface_specs();
        specs.stop_surface = Some(2);
        specs.delete_surface(3);
        assert_eq!(specs.stop_surface, Some(2));
    }

    #[test]
    fn delete_with_no_stop_stays_none() {
        let mut specs = five_surface_specs();
        specs.stop_surface = None;
        specs.delete_surface(2);
        assert_eq!(specs.stop_surface, None);
    }

    // --- SolveSpec serde ---

    #[test]
    fn solve_spec_serde_roundtrip() {
        let specs = vec![
            SolveSpec::MarginalRayHeight {
                gap_index: 2,
                target_height: 0.0,
                wavelength_id: 0,
            },
            SolveSpec::FNumber {
                surface_index: 1,
                target_fno: 4.0,
                wavelength_id: 1,
            },
        ];
        let json = serde_json::to_string(&specs).unwrap();
        let roundtripped: Vec<SolveSpec> = serde_json::from_str(&json).unwrap();
        assert_eq!(specs, roundtripped);
    }

    // --- insert_surface_after renumbers solves ---

    #[test]
    fn insert_surface_after_renumbers_solves() {
        let mut specs = five_surface_specs();
        specs.solves = vec![SolveSpec::MarginalRayHeight {
            gap_index: 2,
            target_height: 0.0,
            wavelength_id: 0,
        }];
        specs.insert_surface_after(1); // insert at idx 2, solve was at 2 → becomes 3
        assert_eq!(specs.solves[0].surface_index(), 3);
    }

    #[test]
    fn insert_surface_after_does_not_renumber_solves_at_or_before_insert() {
        let mut specs = five_surface_specs();
        specs.solves = vec![SolveSpec::MarginalRayHeight {
            gap_index: 1,
            target_height: 0.0,
            wavelength_id: 0,
        }];
        specs.insert_surface_after(1); // solve at idx 1, insert after 1 → unchanged
        assert_eq!(specs.solves[0].surface_index(), 1);
    }

    // --- delete_surface removes and renumbers solves ---

    #[test]
    fn delete_surface_removes_matching_solve() {
        let mut specs = five_surface_specs();
        specs.solves = vec![SolveSpec::MarginalRayHeight {
            gap_index: 2,
            target_height: 0.0,
            wavelength_id: 0,
        }];
        specs.delete_surface(2);
        assert!(specs.solves.is_empty());
    }

    #[test]
    fn delete_surface_renumbers_higher_solves() {
        let mut specs = five_surface_specs();
        specs.solves = vec![SolveSpec::MarginalRayHeight {
            gap_index: 3,
            target_height: 0.0,
            wavelength_id: 0,
        }];
        specs.delete_surface(2); // delete surface 2 → solve at 3 becomes 2
        assert_eq!(specs.solves[0].surface_index(), 2);
    }

    // --- solve_for ---

    #[test]
    fn solve_for_returns_matching_solve() {
        let mut specs = five_surface_specs();
        specs.solves = vec![SolveSpec::MarginalRayHeight {
            gap_index: 2,
            target_height: 5.0,
            wavelength_id: 0,
        }];
        let found = specs.solve_for(2, SolveParameter::Thickness);
        assert!(found.is_some());
        assert_eq!(found.unwrap().surface_index(), 2);
    }

    #[test]
    fn solve_for_returns_none_for_unmatched() {
        let mut specs = five_surface_specs();
        specs.solves = vec![SolveSpec::MarginalRayHeight {
            gap_index: 2,
            target_height: 5.0,
            wavelength_id: 0,
        }];
        // Wrong parameter type
        assert!(
            specs
                .solve_for(2, SolveParameter::RadiusOfCurvature)
                .is_none()
        );
        // Wrong index
        assert!(specs.solve_for(3, SolveParameter::Thickness).is_none());
    }
}
