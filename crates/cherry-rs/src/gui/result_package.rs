use std::collections::HashMap;

use crate::{
    CrossSectionView, FieldSpec, ParaxialView, TraceResultsCollection,
    core::math::{linalg::mat3x3::Mat3x3, vec3::Vec3},
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
    pub wavelengths: Vec<f64>,
    pub surfaces: Vec<SurfaceDesc>,
    pub fields: Vec<FieldDesc>,
    /// Parsed field specs in the same order as `fields`. Used by the Ray Fan
    /// Plot window for TA computation and the paraxial chief-ray fallback.
    pub field_specs: Vec<FieldSpec>,
    pub paraxial: Option<ParaxialView>,
    pub ray_trace: Option<TraceResultsCollection>,
    pub cross_section: Option<CrossSectionView>,
    pub error: Option<String>,
    pub solved_values: SolvedValues,
}

impl ResultPackage {
    /// Construct an error-only package (no computed data).
    pub fn error(id: u64, msg: String) -> Self {
        Self {
            id,
            wavelengths: Vec::new(),
            surfaces: Vec::new(),
            fields: Vec::new(),
            field_specs: Vec::new(),
            paraxial: None,
            ray_trace: None,
            cross_section: None,
            error: Some(msg),
            solved_values: SolvedValues::default(),
        }
    }
}
