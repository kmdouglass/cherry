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
    /// Step-indexed gaps. `gaps.len()` must equal `surface_refs.len() - 1`,
    /// except for a `Reversed` [`PathSurfaceRef::ObjectLinkedTo`] path, where
    /// leading `Shared` steps have their gaps inferred automatically (see
    /// [`PathSurfaceRef::ObjectLinkedTo`]).
    pub gaps: Vec<GapSpec>,
    /// Arm declaration for each beam splitter surface in this path, in step
    /// order. Provide one entry per beam splitter step; the builder matches
    /// them to beam splitter surfaces in order of appearance.
    pub beam_splitter_arms: Vec<BeamSplitterPathKind>,
    /// User-specified aperture stop as a store index for this path, or `None`
    /// to fall back to heuristic aperture-stop selection.
    pub stop_surface: Option<usize>,
}

/// One element of a [`PathSpec`]'s surface sequence.
pub enum PathSurfaceRef {
    /// Introduce a new surface into the store.
    New(SurfaceSpec),
    /// Reference the surface already at this store index, introduced by an
    /// earlier [`PathSpec`].
    Shared(usize),
    /// Place a new `Object` surface at the same 3D position as the `Image`
    /// surface of a previously processed path, deriving the cursor frame from
    /// that path's terminal cursor. Must be the first `surface_ref` of a
    /// `PathSpec`.
    ObjectLinkedTo {
        /// Zero-based index of a previously processed [`PathSpec`].
        path: usize,
        /// How the new cursor frame is derived from the linked path's
        /// terminal (Image) cursor frame.
        orientation: LinkedObjectOrientation,
    },
}

/// Controls how a secondary path's cursor frame is derived from a primary
/// path's Image cursor frame when using
/// [`PathSurfaceRef::ObjectLinkedTo`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkedObjectOrientation {
    /// Use the primary Image cursor's R, U, and F unchanged; the secondary
    /// path departs forward past the Image into new territory.
    SameDirection,
    /// Keep R, negate U and F (a 180° rotation about R); the secondary path
    /// travels back through the shared surfaces traversed by the primary
    /// path.
    Reversed,
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
            stop_surface: None,
        };
        assert_eq!(ps.surface_refs.len(), 2);
    }
}
