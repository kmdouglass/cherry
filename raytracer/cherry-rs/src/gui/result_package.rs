use crate::{ParaxialView, TraceResultsCollection};

/// Axis-aligned bounding box in 3D space.
#[derive(Default)]
pub struct BoundingBox3D {
    pub x: (f64, f64),
    pub y: (f64, f64),
    pub z: (f64, f64),
}

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
    pub bounding_box: BoundingBox3D,
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
            bounding_box: BoundingBox3D::default(),
            error: Some(msg),
        }
    }
}
