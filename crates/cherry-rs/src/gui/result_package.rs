use std::collections::HashMap;

use crate::{
    CrossSectionView, FieldSpec, ParaxialView, TraceResultsCollection,
    core::math::{linalg::mat3x3::Mat3x3, vec3::Vec3},
    views::components::PathComponent,
};

/// Post-solve parameter values keyed by their index in the surfaces table.
/// Only cells with an active solve are populated.
#[derive(Default)]
pub struct SolvedValues {
    /// gap_index → solved thickness (mm).
    pub gap_thicknesses: HashMap<usize, f64>,
    /// surface_index → solved radius of curvature (mm).
    pub surface_rocs: HashMap<usize, f64>,
}

/// Lightweight description of a surface for display in dropdowns.
pub struct SurfaceDesc {
    pub index: usize,
    pub label: String,
    /// Position of the surface in the global coordinate system.
    pub pos: Vec3,
    /// Rotation matrix from global into the surface's local coordinate system.
    pub rot_mat: Mat3x3,
}

/// Lightweight description of a field point for display.
pub struct FieldDesc {
    pub label: String,
}

/// The complete computed output for one version of the system specs.
pub struct ResultPackage {
    /// Matches the `input_id` of the request that produced this result.
    pub id: u64,
    /// Path-0 shorthand, kept for callers that haven't been scoped to
    /// `active_path` yet; prefer `wavelengths_by_path` for anything
    /// path-aware.
    pub wavelengths: Vec<f64>,
    /// Wavelengths for each path, indexed by `path_id`.
    pub wavelengths_by_path: Vec<Vec<f64>>,
    pub surfaces: Vec<SurfaceDesc>,
    /// Field descriptions for each path, indexed by `path_id`.
    pub fields_by_path: Vec<Vec<FieldDesc>>,
    /// Parsed field specs for each path, indexed by `path_id`, in the same
    /// order as `fields_by_path`. Used by the Ray Fan Plot window for TA
    /// computation and the paraxial chief-ray fallback.
    pub field_specs_by_path: Vec<Vec<FieldSpec>>,
    pub paraxial: Option<ParaxialView>,
    pub ray_trace: Option<TraceResultsCollection>,
    pub cross_section: Option<CrossSectionView>,
    pub error: Option<String>,
    pub solved_values: SolvedValues,
    /// Auto-detected optical components from the sequential model.
    pub components: Vec<PathComponent>,
}

impl ResultPackage {
    /// Construct an error-only package (no computed data).
    pub fn error(id: u64, msg: String) -> Self {
        Self {
            id,
            wavelengths: Vec::new(),
            wavelengths_by_path: Vec::new(),
            surfaces: Vec::new(),
            fields_by_path: Vec::new(),
            field_specs_by_path: Vec::new(),
            paraxial: None,
            ray_trace: None,
            cross_section: None,
            error: Some(msg),
            solved_values: SolvedValues::default(),
            components: Vec::new(),
        }
    }
}
