use crate::{CrossSectionView, ParaxialView, TraceResultsCollection};

/// Lightweight description of a surface for display in dropdowns.
pub struct SurfaceDesc {
    pub index: usize,
    pub label: String,
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
    pub paraxial: Option<ParaxialView>,
    pub ray_trace: Option<TraceResultsCollection>,
    pub cross_section: Option<CrossSectionView>,
    pub error: Option<String>,
}

impl ResultPackage {
    /// Construct an error-only package (no computed data).
    pub fn error(id: u64, msg: String) -> Self {
        Self {
            id,
            wavelengths: Vec::new(),
            surfaces: Vec::new(),
            fields: Vec::new(),
            paraxial: None,
            ray_trace: None,
            cross_section: None,
            error: Some(msg),
        }
    }
}
