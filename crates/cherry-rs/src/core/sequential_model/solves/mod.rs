pub mod marginal_ray;

use anyhow::Result;

use crate::specs::{gaps::GapSpec, surfaces::SurfaceSpec};

use super::SequentialModel;

pub use marginal_ray::MarginalRaySolve;

/// A solve that modifies optical system specs to satisfy a constraint.
///
/// Solves are applied sequentially during model building. Each solve receives
/// a fully-built [`SequentialModel`] (used for downstream calculations such as
/// paraxial ray tracing) and mutable access to the gap and surface specs, which
/// it modifies in place to satisfy its constraint. After each solve the model
/// is rebuilt from the updated specs before the next solve runs.
pub trait Solve {
    fn apply(
        &self,
        model: &SequentialModel,
        gap_specs: &mut Vec<GapSpec>,
        surface_specs: &mut Vec<SurfaceSpec>,
    ) -> Result<()>;

    /// Returns the index of the surface whose parameter this solve modifies.
    ///
    /// The builder sorts solves by this value in ascending order before
    /// applying them, ensuring each solve sees a model that reflects all
    /// earlier solves' changes. The convention matches Zemax: the thickness
    /// of gap k is the "thickness of surface k", so a thickness solve on
    /// gap k returns k here.
    fn surface_index(&self) -> usize;
}
