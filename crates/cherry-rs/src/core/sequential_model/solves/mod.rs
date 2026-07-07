pub mod fno;
pub mod marginal_ray;

use anyhow::{Result, anyhow};

use crate::specs::{gaps::GapSpec, paths::PathSpec, surfaces::SurfaceSpec};

use super::SequentialModel;

pub use fno::FNumberSolve;
pub use marginal_ray::MarginalRaySolve;

/// Which optical parameter a solve modifies.
///
/// Declaration order defines execution order within the same surface index:
/// earlier variants run first. Shape parameters (`Curvature`) precede
/// positional parameters (`Thickness`), matching the convention used by Zemax
/// OpticStudio and CODE V.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SolveKind {
    Curvature,
    Thickness,
}

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
    /// gap k returns k here. For curvature solves (e.g. [`FNumberSolve`]),
    /// this is the index of the surface whose radius of curvature is modified.
    fn surface_index(&self) -> usize;

    /// Secondary sort key used when two solves share the same `surface_index`.
    ///
    /// The builder sorts by `(surface_index, parameter_kind)`. Since
    /// [`SolveKind`] derives `Ord` in declaration order,
    /// [`SolveKind::Curvature`] runs before
    /// [`SolveKind::Thickness`] at the same surface index. Implementations
    /// that modify a parameter not yet represented here should add a new
    /// variant to [`SolveKind`] in the appropriate position.
    fn parameter_kind(&self) -> SolveKind {
        SolveKind::Curvature
    }

    /// Which path this solve applies to in a multipath model. Ignored by
    /// single-path builds (`.gap_specs()`/`.surface_specs()`) — there is
    /// only ever path 0. Defaults to 0 so every existing `Solve`
    /// implementation keeps compiling and behaving identically without
    /// change.
    fn path_id(&self) -> usize {
        0
    }

    /// Applies this solve within a multipath build
    /// (`SequentialModelBuilder::paths`).
    ///
    /// A multipath model has no flat `gap_specs`/`surface_specs` pair —
    /// `paths` is the full, mutable set of `PathSpec`s instead. A
    /// `Thickness`-kind solve mutates `paths[path_id()].gaps` directly
    /// (gaps are never shared between paths, so no ownership resolution is
    /// needed). A `Curvature`-kind solve's [`surface_index`] is a store
    /// index; the implementation must resolve two independent things from
    /// it: which path's own ray trace evaluates the constraint (`path_id()`),
    /// and which path actually owns the mutable `SurfaceSpec` for that store
    /// index (which may be a *different* path, when the surface is
    /// `Shared`) — see `locate_surface_owner`.
    ///
    /// The default implementation errors; override to support multipath
    /// models.
    ///
    /// [`surface_index`]: Solve::surface_index
    fn apply_multipath(&self, _model: &SequentialModel, _paths: &mut [PathSpec]) -> Result<()> {
        Err(anyhow!("this solve does not support multipath models"))
    }
}
