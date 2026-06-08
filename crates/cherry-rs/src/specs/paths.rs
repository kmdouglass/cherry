use crate::specs::{
    gaps::GapSpec,
    surfaces::{BeamSplitterPathKind, SurfaceSpec},
};

/// Specifies one optical path for [`SequentialModelBuilder::paths`].
///
/// Each path is an ordered sequence of surface references and the gaps between
/// them. Beam splitter steps must declare which arm (transmitting or
/// reflecting) this path traverses.
///
/// [`SequentialModelBuilder::paths`]:
///     crate::core::sequential_model::builder::SequentialModelBuilder::paths
pub struct PathSpec {
    /// Ordered sequence of surfaces for this path.
    pub surface_refs: Vec<PathSurfaceRef>,
    /// Step-indexed gaps. `gaps.len()` must equal `surface_refs.len() - 1`.
    pub gaps: Vec<GapSpec>,
    /// Arm declaration for each beam splitter surface in this path, in step
    /// order. Provide one entry per beam splitter step; the builder matches
    /// them to beam splitter surfaces in order of appearance.
    pub beam_splitter_arms: Vec<BeamSplitterPathKind>,
}

/// One element of a [`PathSpec`]'s surface sequence.
pub enum PathSurfaceRef {
    /// Introduce a new surface into the store.
    New(SurfaceSpec),
    /// Reference the surface already at this store index, introduced by an
    /// earlier [`PathSpec`].
    Shared(usize),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GapSpec, Rotation3D, Vec3, n, specs::surfaces::SurfaceSpec};

    #[test]
    fn path_spec_construction() {
        let ps = PathSpec {
            surface_refs: vec![
                PathSurfaceRef::New(SurfaceSpec::Object),
                PathSurfaceRef::New(SurfaceSpec::Image {
                    rotation: Rotation3D::None,
                    decenter: Vec3::new(0.0, 0.0, 0.0),
                    rotation_offset: Rotation3D::None,
                }),
            ],
            gaps: vec![GapSpec {
                thickness: f64::INFINITY,
                refractive_index: n!(1.0),
            }],
            beam_splitter_arms: vec![],
        };
        assert_eq!(ps.surface_refs.len(), 2);
    }
}
