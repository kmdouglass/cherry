/// Data types for modeling ray tracing systems.
pub(super) mod math;
pub(crate) mod placement;
pub(crate) mod ray;
pub(crate) mod refractive_index;
pub(crate) mod sequential_model;
pub(crate) mod surfaces;

pub(crate) type Float = f64;

pub(crate) const EPSILON: Float = Float::EPSILON;
pub(crate) const PI: Float = std::f64::consts::PI;
