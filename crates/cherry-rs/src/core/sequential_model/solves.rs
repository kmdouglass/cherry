use anyhow::Result;

use crate::specs::{gaps::GapSpec, surfaces::SurfaceSpec};

use super::SequentialModel;

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
}
