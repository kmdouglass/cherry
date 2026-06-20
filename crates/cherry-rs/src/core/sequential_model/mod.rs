/// Data types for modeling sequential ray tracing systems.
pub mod builder;
pub(crate) mod cursor;
pub mod solves;
pub mod surface_placement;

use std::ops::Range;

use anyhow::{Result, anyhow};

use self::cursor::Cursor;
use self::surface_placement::SurfacePlacement;
#[cfg(feature = "serde")]
use crate::core::surfaces::SurfaceRegistry;
use crate::core::{
    Float,
    math::{linalg::mat3x3::Mat3x3, vec3::Vec3},
    refractive_index::RefractiveIndex,
    surfaces::{
        BeamSplitter, Conic, Image, Iris, Object, Probe, Sphere, Surface, SurfaceKind, ThinLens,
    },
};
use crate::specs::surfaces::PlacementSpec;
use crate::specs::{
    gaps::GapSpec,
    paths::{PathSpec, PathSurfaceRef},
    surfaces::{BeamSplitterPathKind, BoundaryKind, SurfaceSpec},
};

type SurfaceStoreContents = (
    Vec<Box<dyn Surface>>,
    Vec<SurfacePlacement>,
    Vec<CursorPlacement>,
);

/// Owns all surface objects and their computed placements.
#[derive(Debug)]
struct SurfaceStore {
    surfaces: Vec<Box<dyn Surface>>,
    placements: Vec<SurfacePlacement>,
}

/// Per-step cursor state recorded along one optical path.
///
/// Captures the cursor orientation, position, and rotation matrix before any
/// reflection at each surface, for both `New` and `Shared` steps. Carried by
/// each iterator [`Step`] so that views have fully path-specific cursor data
/// without separately querying the model.
#[derive(Debug, Clone, Copy)]
pub struct CursorPlacement {
    /// Cursor forward direction (unit vector) as the beam approaches this
    /// surface.
    pub axis_direction: Vec3,
    /// Nominal on-axis cursor position before any decenter.
    pub cursor_position: Vec3,
    /// Rotation from the global frame into the cursor frame at this step.
    pub cursor_rotation_matrix: Mat3x3,
}

/// One optical path through the system.
#[derive(Debug)]
struct OpticalPath {
    /// Ordered store indices visited by this path, one per step.
    surface_indices: Vec<usize>,
    /// Beam-splitter arm kind per step; parallel to `surface_indices`.
    beam_splitter_arms: Vec<Option<BeamSplitterPathKind>>,
    submodels: Vec<SequentialSubModelBase>,
    /// User-specified aperture stop as a store index, or `None` for auto.
    stop_surface: Option<usize>,
    /// Step-indexed cursor state, parallel to `surface_indices`.
    steps: Vec<CursorPlacement>,
}

/// A gap between two surfaces in a sequential system.
#[derive(Debug)]
pub struct Gap {
    pub thickness: Float,
    pub refractive_index: RefractiveIndex,
}

/// A collection of submodels for sequential ray tracing.
///
/// A sequential model is a collection of surfaces and gaps that define the
/// optical system. The model is divided into submodels, each of which is
/// computed along a specific axis and for a specific wavelength.
///
/// See the documentation for
/// [SequentialSubModel](trait@SequentialSubModel) for more information.
#[derive(Debug)]
pub struct SequentialModel {
    store: SurfaceStore,
    paths: Vec<OpticalPath>,
    wavelengths: Vec<Float>,
}

/// A submodel of a sequential optical system.
///
/// A sequential submodel is the primary unit of computation in a sequential
/// optical system. It is a collection of N + 1 surfaces and N gaps from which
/// an iterator can be created to trace rays through the system.
///
/// Each submodel represents a sequence of surfaces and gaps for a given
/// set of system parameters, such as wavelength and transverse axis. The set of
/// all submodels spans the entire set of parameters of interest.
///
/// The iterator over a submodel yields a series of steps, each of which is a
/// tuple of the form (Gap, Surface, Option\<Gap\>). The first element of a step
/// is the gap before the surface, the second element is the surface itself, and
/// the third element is the gap after the surface. The last Gap is optional
/// because no gap exists after the image plane surface.
///
/// Given a system of N + 1 surfaces and N gaps, the first surface S0 is always
/// an object plane and the last surface S(N) is always an image plane. The
/// length of the iterator is N.
///
/// A forward iterator for such a system looks like the following:
///
/// ```text
/// S0   S1    S2    S3        S(N-1)    S(N)
///  \  /  \  /  \  /  \   ... /    \    /  \
///   G0    G1    G2    G3          G(N-1)   None
///   --------    --------          -------------
///    Step 0      Step 2             Step(N-1)
///         --------
///          Step 1
/// ```
///
/// Step 0 is the tuple (G0, S1, G1), Step 1 is (G1, S2, G2), and so on.
///
/// A reverse iterator for the same system looks like the following:
///
/// ```text
///    S(N)   S(N-1)  S(N-2)  S(N-3)            S1    S0
///   /    \  /    \  /    \  /    \      ...  /  \  /
/// None  G(N-1)  G(N-2)  G(N-3)    G(N-4)    G1    G0
///       --------------  ----------------    ---------
///           Step 0           Step 2         Step(N-2)
///               --------------
///                   Step 1
/// ```
///
/// In the reverse iteration, we  never iterate from the image plane surface.
/// For this reason, the number of steps in the reverse iterator is N - 1.
///
/// If `i` is the index of a surface in the forward iterator and `j` the index
/// of the surface in the reverse iterator, then the two indexes are related by
/// the equation j = N - i as shown above.
///
/// Strictly speaking, the last gap need not be None. Additionally, the first
/// and last surfaces need not be object and image planes, repectively. These
/// constraints are guaranteed at the level of the SequentialModel. However,
/// since a SequentialSubModel is always created by a SequentialModel, we can
/// assume that these constraints are always met. This would not be the case for
/// user-supplied implementations of this trait, where care should be taken to
/// ensure that the implementation conforms to these constraints.
///
/// With all of these constraints, the problem of sequential optical modeling is
/// reduced to the problem of iterating over the surfaces and gaps in a submodel
/// and determining what happens at each step. The same iterator can be used for
/// different modeling approaches, e.g. paraxial ray tracing, 3D ray tracing,
/// paraxial Gaussian beam propagation, etc. without changing the representation
/// of the underlying system.
pub trait SequentialSubModel {
    fn gaps(&self) -> &[Gap];
    fn is_obj_at_inf(&self) -> bool;

    fn is_empty(&self) -> bool {
        self.gaps().is_empty()
    }
    fn len(&self) -> usize {
        self.gaps().len()
    }
    fn try_iter<'a>(
        &'a self,
        surfaces: &'a [Box<dyn Surface>],
        placements: &'a [SurfacePlacement],
        surface_indices: &'a [usize],
        beam_splitter_arms: &'a [Option<BeamSplitterPathKind>],
        path_steps: &'a [CursorPlacement],
    ) -> Result<SequentialSubModelIter<'a>>;

    fn slice<'a>(
        &'a self,
        idx: Range<usize>,
        surface_indices: &'a [usize],
        beam_splitter_arms: &'a [Option<BeamSplitterPathKind>],
    ) -> SequentialSubModelSlice<'a>;
}

#[derive(Debug)]
pub struct SequentialSubModelBase {
    gaps: Vec<Gap>,
}

/// A view of a single submodel in a sequential system.
///
/// This is used to slice the system into smaller parts.
#[derive(Debug)]
pub struct SequentialSubModelSlice<'a> {
    surface_indices: &'a [usize],
    beam_splitter_arms: &'a [Option<BeamSplitterPathKind>],
    gaps: &'a [Gap],
    /// First step index of this slice within the full path's step list.
    /// Used by the iterator to correctly index into the caller-supplied
    /// `path_steps` slice.
    step_offset: usize,
}

/// An iterator over the surfaces and gaps in a submodel.
///
/// Most operations in sequential modeling involve use of this iterator.
pub struct SequentialSubModelIter<'a> {
    surfaces: &'a [Box<dyn Surface>],
    placements: &'a [SurfacePlacement],
    surface_indices: &'a [usize],
    beam_splitter_arms: &'a [Option<BeamSplitterPathKind>],
    gaps: &'a [Gap],
    path_steps: &'a [CursorPlacement],
    step_offset: usize,
    index: usize,
}

/// A reverse iterator over the surfaces and gaps in a submodel.
pub struct SequentialSubModelReverseIter<'a> {
    surfaces: &'a [Box<dyn Surface>],
    placements: &'a [SurfacePlacement],
    surface_indices: &'a [usize],
    beam_splitter_arms: &'a [Option<BeamSplitterPathKind>],
    gaps: &'a [Gap],
    path_steps: &'a [CursorPlacement],
    step_offset: usize,
    index: usize,
}

/// A single ray tracing step in a sequential system.
///
/// See the documentation for
/// [SequentialSubModel](trait@SequentialSubModel) for more information.
pub struct Step<'a> {
    pub gap_before: &'a Gap,
    pub surface: &'a dyn Surface,
    pub gap_after: Option<&'a Gap>,
    pub surface_placement: &'a SurfacePlacement,
    /// The beam-splitter arm traversed at this step, or `None` for non-BS
    /// surfaces. Set from the path's `beam_splitter_arms` declaration.
    pub bs_arm: Option<BeamSplitterPathKind>,
    /// Path-specific cursor state at this step (orientation, position, rotation
    /// matrix of the cursor as it arrives at this surface).
    pub cursor_placement: CursorPlacement,
}

/// Propagates a tangential direction unit vector through the mirror surfaces of
/// a system using the vector law of reflection.
///
/// Returns one `Vec3` per surface (same indexing as `surfaces`). Each entry is
/// the **incident** direction at that surface (before any reflection). At a
/// reflecting surface the returned vector is the direction arriving at the
/// surface; subsequent surfaces receive the post-reflection direction as their
/// incident vector. The vector is expressed in global coordinates throughout.
pub(crate) fn propagate_tangential_vec(
    v_init: Vec3,
    surfaces: &[Box<dyn Surface>],
    placements: &[SurfacePlacement],
    surface_indices: &[usize],
) -> Vec<Vec3> {
    use crate::specs::surfaces::BoundaryKind;
    let mut v = v_init;
    surface_indices
        .iter()
        .map(|&idx| {
            let surf = &surfaces[idx];
            let placement = &placements[idx];
            let v_incident = v;
            if let BoundaryKind::Reflecting = surf.boundary_kind() {
                // Normal in global frame derived from the *nominal* orientation
                // so that rotation_offset never redirects the paraxial axis.
                let n = placement.nominal_inv_rotation_matrix * Vec3::new(0.0, 0.0, 1.0);
                let dot = v.x() * n.x() + v.y() * n.y() + v.z() * n.z();
                v = Vec3::new(
                    v.x() - 2.0 * dot * n.x(),
                    v.y() - 2.0 * dot * n.y(),
                    v.z() - 2.0 * dot * n.z(),
                );
            }
            v_incident
        })
        .collect()
}

/// Returns the step index of the first physical surface visited by
/// `surface_indices`.
///
/// A physical surface has a finite semi-diameter (Conic or Iris). Object,
/// Image, and Probe surfaces are excluded. Returns the position within
/// `surface_indices` (i.e. a step index), not a store index.
pub(crate) fn first_physical_step(
    surface_indices: &[usize],
    surfaces: &[Box<dyn Surface>],
) -> Option<usize> {
    surface_indices
        .iter()
        .position(|&i| surfaces[i].mask().semi_diameter().is_finite())
}

/// Returns the step index of the last physical surface visited by
/// `surface_indices`.
///
/// A physical surface has a finite semi-diameter (Conic or Iris). Object,
/// Image, and Probe surfaces are excluded. Returns the position within
/// `surface_indices` (i.e. a step index), not a store index.
pub(crate) fn last_physical_step(
    surface_indices: &[usize],
    surfaces: &[Box<dyn Surface>],
) -> Option<usize> {
    surface_indices
        .iter()
        .rposition(|&i| surfaces[i].mask().semi_diameter().is_finite())
}

/// Returns the id of a surface in a reversed system.
pub fn reversed_surface_id(num_surfaces: usize, surf_id: usize) -> usize {
    // Reversed IDs are ray starts, then image plane, then surfaces
    num_surfaces - surf_id - 1
}

impl Gap {
    pub(crate) fn try_from_spec(spec: &GapSpec, wavelength: Float) -> Result<Self> {
        let thickness = spec.thickness;
        if thickness < 0.0 {
            return Err(anyhow!(
                "gap thickness must be non-negative, got {thickness}"
            ));
        }
        let refractive_index =
            RefractiveIndex::try_from_spec(spec.refractive_index.as_ref(), wavelength)?;
        Ok(Self {
            thickness,
            refractive_index,
        })
    }
}

impl SequentialModel {
    /// Creates a new sequential model of an optical system.
    ///
    /// # Arguments
    /// * `gap_specs` - The specifications for the gaps between the surfaces.
    /// * `surface_specs` - The specifications for the surfaces in the system.
    /// * `wavelengths` - The wavelengths at which to model the system.
    /// * `stop_surface` - Optional index of the user-designated aperture stop.
    ///   `None` uses the paraxial heuristic. `Some(i)` requires `i` to refer to
    ///   a `Conic` or `Iris` surface that is neither the object nor the image
    ///   surface; otherwise an error is returned.
    pub fn from_surface_specs(
        gap_specs: &[GapSpec],
        surface_specs: &[SurfaceSpec],
        wavelengths: &[Float],
        stop_surface: Option<usize>,
    ) -> Result<Self> {
        #[cfg(feature = "serde")]
        return Self::from_surface_specs_with_registry(
            gap_specs,
            surface_specs,
            wavelengths,
            stop_surface,
            None,
        );
        #[cfg(not(feature = "serde"))]
        {
            Self::validate_specs(gap_specs, wavelengths)?;
            let (surfaces, placements, cursor_placements) =
                Self::surf_specs_to_surfs(surface_specs, gap_specs)?;
            if let Some(i) = stop_surface {
                Self::validate_stop_surface(&surfaces, i)?;
            }
            let surface_indices: Vec<usize> = (0..surfaces.len()).collect();
            let bs_arms = vec![None; surfaces.len()];
            let mut submodels: Vec<SequentialSubModelBase> = Vec::new();
            for &wavelength in wavelengths.iter() {
                let gaps = Self::gap_specs_to_gaps(gap_specs, wavelength)?;
                submodels.push(SequentialSubModelBase::new(gaps));
            }
            let store = SurfaceStore {
                surfaces,
                placements,
            };
            let path = OpticalPath {
                surface_indices,
                beam_splitter_arms: bs_arms,
                submodels,
                stop_surface,
                steps: cursor_placements,
            };
            Ok(Self {
                store,
                paths: vec![path],
                wavelengths: wavelengths.to_vec(),
            })
        }
    }

    /// Like [`from_surface_specs`](Self::from_surface_specs) but also accepts
    /// an optional [`SurfaceRegistry`] for resolving [`SurfaceSpec::Custom`]
    /// variants. Pass `None` to skip registry lookup (equivalent to
    /// `from_surface_specs`).
    #[cfg(feature = "serde")]
    pub(crate) fn from_surface_specs_with_registry(
        gap_specs: &[GapSpec],
        surface_specs: &[SurfaceSpec],
        wavelengths: &[Float],
        stop_surface: Option<usize>,
        registry: Option<&SurfaceRegistry>,
    ) -> Result<Self> {
        Self::validate_specs(gap_specs, wavelengths)?;
        let (surfaces, placements, cursor_placements) =
            Self::surf_specs_to_surfs(surface_specs, gap_specs, registry)?;
        if let Some(i) = stop_surface {
            Self::validate_stop_surface(&surfaces, i)?;
        }
        let surface_indices: Vec<usize> = (0..surfaces.len()).collect();
        let bs_arms = vec![None; surfaces.len()];
        let mut submodels: Vec<SequentialSubModelBase> = Vec::new();
        for &wavelength in wavelengths.iter() {
            let gaps = Self::gap_specs_to_gaps(gap_specs, wavelength)?;
            submodels.push(SequentialSubModelBase::new(gaps));
        }
        let store = SurfaceStore {
            surfaces,
            placements,
        };
        let path = OpticalPath {
            surface_indices,
            beam_splitter_arms: bs_arms,
            submodels,
            stop_surface,
            steps: cursor_placements,
        };
        Ok(Self {
            store,
            paths: vec![path],
            wavelengths: wavelengths.to_vec(),
        })
    }

    /// Creates a new sequential model from pre-built surface trait objects.
    ///
    /// Use this when constructing a model programmatically in Rust without
    /// going through the surface spec/serialization layer. Use
    /// [`PlacementSpec::none()`] for untilted surfaces with no displacement,
    /// or [`PlacementSpec::from_rotation`] to supply only a tilt rotation.
    ///
    /// # Arguments
    /// * `surfaces` - Pre-built surface trait objects.
    /// * `placement_specs` - Tilt/decenter spec for each surface; must be the
    ///   same length as `surfaces`.
    /// * `gap_specs` - Gaps between surfaces (`surfaces.len() - 1` elements).
    /// * `wavelengths` - Wavelengths at which to model the system.
    pub fn from_surfaces(
        surfaces: Vec<Box<dyn Surface>>,
        placement_specs: &[PlacementSpec],
        gap_specs: &[GapSpec],
        wavelengths: &[Float],
        stop_surface: Option<usize>,
    ) -> Result<Self> {
        if surfaces.len() != placement_specs.len() {
            return Err(anyhow!(
                "Expected {} placement spec(s) for {} surface(s), got {}.",
                surfaces.len(),
                surfaces.len(),
                placement_specs.len()
            ));
        }
        if surfaces.len() != gap_specs.len() + 1 {
            return Err(anyhow!(
                "Expected {} gap(s) for {} surface(s), got {}.",
                surfaces.len() - 1,
                surfaces.len(),
                gap_specs.len()
            ));
        }
        Self::validate_specs(gap_specs, wavelengths)?;

        let (placements, cursor_placements) =
            Self::build_placements_and_directions(&surfaces, placement_specs, gap_specs);

        if let Some(i) = stop_surface {
            Self::validate_stop_surface(&surfaces, i)?;
        }

        let surface_indices: Vec<usize> = (0..surfaces.len()).collect();
        let bs_arms = vec![None; surfaces.len()];
        let mut submodels: Vec<SequentialSubModelBase> = Vec::new();
        for &wavelength in wavelengths.iter() {
            let gaps = Self::gap_specs_to_gaps(gap_specs, wavelength)?;
            submodels.push(SequentialSubModelBase::new(gaps));
        }

        let store = SurfaceStore {
            surfaces,
            placements,
        };
        let path = OpticalPath {
            surface_indices,
            beam_splitter_arms: bs_arms,
            submodels,
            stop_surface,
            steps: cursor_placements,
        };
        Ok(Self {
            store,
            paths: vec![path],
            wavelengths: wavelengths.to_vec(),
        })
    }

    /// Builds a multipath `SequentialModel` from a list of [`PathSpec`]s.
    ///
    /// `build_surface` is a closure that constructs a [`Surface`] trait object
    /// from a [`SurfaceSpec`]. It is injected so that the serde and non-serde
    /// versions can supply the appropriate `surface_from_spec` variant.
    fn from_path_specs_with_builder(
        paths: Vec<PathSpec>,
        wavelengths: &[Float],
        stop_surface: Option<usize>,
        mut build_surface: impl FnMut(&SurfaceSpec) -> Result<Box<dyn Surface>>,
    ) -> Result<Self> {
        if wavelengths.is_empty() {
            return Err(anyhow!("At least one wavelength must be specified."));
        }

        let mut store_surfaces: Vec<Box<dyn Surface>> = Vec::new();
        let mut store_placements: Vec<SurfacePlacement> = Vec::new();
        let mut optical_paths: Vec<OpticalPath> = Vec::new();

        for ps in paths {
            let n_refs = ps.surface_refs.len();

            if n_refs == 0 {
                return Err(anyhow!("a PathSpec must have at least one surface_ref"));
            }
            if ps.gaps.len() + 1 != n_refs {
                return Err(anyhow!(
                    "PathSpec has {} surface_ref(s) but {} gap(s); expected {} gap(s)",
                    n_refs,
                    ps.gaps.len(),
                    n_refs - 1,
                ));
            }

            // Build the dense arm vec by consuming beam_splitter_arms in step order.
            let mut bs_arms_iter = ps.beam_splitter_arms.into_iter();
            let mut dense_bs_arms: Vec<Option<BeamSplitterPathKind>> = Vec::with_capacity(n_refs);

            let mut cursor = Cursor::new(-ps.gaps[0].thickness);
            let mut surface_indices: Vec<usize> = Vec::new();
            let mut path_steps: Vec<CursorPlacement> = Vec::new();

            for (step, sref) in ps.surface_refs.iter().enumerate() {
                let is_first = step == 0;
                let is_last = step == n_refs - 1;

                // Determine whether this step is a beam splitter before building/looking
                // up the surface, so we can consume the arm declaration in order.
                let is_bs = match sref {
                    PathSurfaceRef::New(spec) => {
                        matches!(spec, SurfaceSpec::BeamSplitter { .. })
                    }
                    PathSurfaceRef::Shared(i) => {
                        *i < store_surfaces.len()
                            && store_surfaces[*i].surface_kind() == SurfaceKind::BeamSplitter
                    }
                };
                let bs_arm = if is_bs {
                    Some(bs_arms_iter.next().ok_or_else(|| {
                        anyhow!(
                            "step {step} resolves to a BeamSplitter but beam_splitter_arms \
                             has no more entries; provide one BeamSplitterPathKind per \
                             beam splitter step, in order"
                        )
                    })?)
                } else {
                    None
                };
                dense_bs_arms.push(bs_arm);

                // Record cursor state before any reflection at this surface.
                path_steps.push(CursorPlacement {
                    axis_direction: cursor.forward(),
                    cursor_position: cursor.pos(),
                    cursor_rotation_matrix: cursor.rotation_matrix(),
                });

                match sref {
                    PathSurfaceRef::New(spec) => {
                        let surface = build_surface(spec)?;
                        let surf_kind = surface.surface_kind();

                        if is_first && surf_kind != SurfaceKind::Object {
                            return Err(anyhow!(
                                "the first surface_ref of a PathSpec must resolve to an Object \
                                 surface, got {surf_kind:?}"
                            ));
                        }
                        if is_last && surf_kind != SurfaceKind::Image {
                            return Err(anyhow!(
                                "the last surface_ref of a PathSpec must resolve to an Image \
                                 surface, got {surf_kind:?}"
                            ));
                        }

                        let nominal_rot = spec.rotation().rotation_matrix();
                        let actual_rot = spec.rotation_offset().rotation_matrix() * nominal_rot;
                        let placement = SurfacePlacement::from_decenter_and_rotation(
                            spec.decenter(),
                            actual_rot,
                            nominal_rot,
                            &cursor,
                        );

                        let should_reflect = bs_arm == Some(BeamSplitterPathKind::Reflecting)
                            || matches!(surface.boundary_kind(), BoundaryKind::Reflecting);
                        if should_reflect {
                            let norm_local = surface.norm(Vec3::new(0.0, 0.0, 0.0));
                            let nominal_local_to_global =
                                (nominal_rot * cursor.rotation_matrix()).transpose();
                            let norm_global = (nominal_local_to_global * norm_local).normalize();
                            cursor.reflect(&norm_global);
                        }

                        let store_idx = store_surfaces.len();
                        store_placements.push(placement);
                        store_surfaces.push(surface);
                        surface_indices.push(store_idx);
                    }
                    PathSurfaceRef::Shared(i) => {
                        if *i >= store_surfaces.len() {
                            return Err(anyhow!(
                                "Shared({i}) references store index {i} which has not yet \
                                 been committed by any preceding PathSpec"
                            ));
                        }

                        let surf_kind = store_surfaces[*i].surface_kind();

                        if is_first && surf_kind != SurfaceKind::Object {
                            return Err(anyhow!(
                                "the first surface_ref of a PathSpec must resolve to an Object \
                                 surface, got {surf_kind:?}"
                            ));
                        }
                        if is_last && surf_kind != SurfaceKind::Image {
                            return Err(anyhow!(
                                "the last surface_ref of a PathSpec must resolve to an Image \
                                 surface, got {surf_kind:?}"
                            ));
                        }

                        // Adopt the stored placement; only the cursor may change.
                        let should_reflect = bs_arm == Some(BeamSplitterPathKind::Reflecting)
                            || matches!(
                                store_surfaces[*i].boundary_kind(),
                                BoundaryKind::Reflecting
                            );
                        if should_reflect {
                            let norm_local = store_surfaces[*i].norm(Vec3::new(0.0, 0.0, 0.0));
                            let norm_global = (store_placements[*i].nominal_inv_rotation_matrix
                                * norm_local)
                                .normalize();
                            cursor.reflect(&norm_global);
                        }

                        surface_indices.push(*i);
                    }
                }

                if !is_last {
                    cursor.advance(ps.gaps[step].thickness);
                }
            }

            if bs_arms_iter.next().is_some() {
                return Err(anyhow!(
                    "beam_splitter_arms has more entries than there are BeamSplitter \
                     steps in this path"
                ));
            }

            let mut submodels: Vec<SequentialSubModelBase> = Vec::new();
            for &wavelength in wavelengths.iter() {
                let gaps = Self::gap_specs_to_gaps(&ps.gaps, wavelength)?;
                submodels.push(SequentialSubModelBase::new(gaps));
            }
            optical_paths.push(OpticalPath {
                surface_indices,
                beam_splitter_arms: dense_bs_arms,
                submodels,
                stop_surface,
                steps: path_steps,
            });
        }

        if let Some(i) = stop_surface {
            Self::validate_stop_surface(&store_surfaces, i)?;
        }

        let store = SurfaceStore {
            surfaces: store_surfaces,
            placements: store_placements,
        };
        Ok(Self {
            store,
            paths: optical_paths,
            wavelengths: wavelengths.to_vec(),
        })
    }

    /// Builds a multipath model from `PathSpec`s (serde + registry variant).
    #[cfg(feature = "serde")]
    pub(crate) fn from_path_specs(
        paths: Vec<PathSpec>,
        wavelengths: &[Float],
        stop_surface: Option<usize>,
        registry: Option<&SurfaceRegistry>,
    ) -> Result<Self> {
        Self::from_path_specs_with_builder(paths, wavelengths, stop_surface, |spec| {
            surface_from_spec(spec, registry)
        })
    }

    /// Builds a multipath model from `PathSpec`s (non-serde variant).
    #[cfg(not(feature = "serde"))]
    pub(crate) fn from_path_specs(
        paths: Vec<PathSpec>,
        wavelengths: &[Float],
        stop_surface: Option<usize>,
    ) -> Result<Self> {
        Self::from_path_specs_with_builder(paths, wavelengths, stop_surface, surface_from_spec)
    }

    /// Number of optical paths in the model.
    pub fn path_count(&self) -> usize {
        self.paths.len()
    }

    /// Ordered store indices visited by path `path_id`.
    pub fn path_surface_indices(&self, path_id: usize) -> &[usize] {
        &self.paths[path_id].surface_indices
    }

    /// Beam-splitter arm kind per step for path `path_id`.
    pub fn path_beam_splitter_arms(&self, path_id: usize) -> &[Option<BeamSplitterPathKind>] {
        &self.paths[path_id].beam_splitter_arms
    }

    /// Placement of the surface at `step` in path `path_id`, looked up from
    /// the store via that path's `surface_indices`.
    pub fn path_placement(&self, path_id: usize, step: usize) -> &SurfacePlacement {
        let store_idx = self.paths[path_id].surface_indices[step];
        &self.store.placements[store_idx]
    }

    /// Validates that index `i` is an eligible aperture stop surface.
    fn validate_stop_surface(surfaces: &[Box<dyn Surface>], i: usize) -> Result<()> {
        let last = surfaces.len().saturating_sub(1);
        if i == 0 || i >= last {
            return Err(anyhow!(
                "stop surface index {i} is out of range; \
                 must be between 1 and {} (inclusive)",
                last - 1
            ));
        }
        match surfaces[i].surface_kind() {
            SurfaceKind::BeamSplitter
            | SurfaceKind::Conic
            | SurfaceKind::Sphere
            | SurfaceKind::Iris
            | SurfaceKind::ThinLens => Ok(()),
            kind => Err(anyhow!(
                "surface {i} ({kind:?}) is not eligible as the aperture stop; \
                 only Conic and Iris surfaces are allowed"
            )),
        }
    }

    /// Returns the user-specified aperture stop surface index, or `None` if the
    /// stop is derived automatically from the paraxial ray trace.
    ///
    /// Single-path shorthand; delegates to `paths[0]`.
    pub fn stop_surface(&self) -> Option<usize> {
        self.paths[0].stop_surface
    }

    /// Returns the aperture stop for the given path index.
    pub fn stop_surface_for_path(&self, path_id: usize) -> Option<usize> {
        self.paths[path_id].stop_surface
    }

    /// Returns the per-step cursor data for path `path_id`.
    ///
    /// Each entry corresponds to one surface in that path's traversal order,
    /// and records the cursor state as the beam *approaches* that surface
    /// (before any reflection).
    pub fn path_steps(&self, path_id: usize) -> &[CursorPlacement] {
        &self.paths[path_id].steps
    }

    /// Returns all wavelength submodels for path `path_id`.
    pub fn submodels_for_path(&self, path_id: usize) -> &[SequentialSubModelBase] {
        &self.paths[path_id].submodels
    }

    /// Returns the largest semi-diameter of any surface in the system.
    ///
    /// This ignores surfaces without any size, such as object, probe, and image
    /// surfaces.
    pub fn largest_semi_diameter(&self) -> Float {
        self.store
            .surfaces
            .iter()
            .filter_map(|surf| {
                let sd = surf.mask().semi_diameter();
                if sd.is_finite() { Some(sd) } else { None }
            })
            .fold(0.0, |acc, x| acc.max(x))
    }

    /// Returns the surfaces in the system.
    ///
    /// The i-th surface corresponds to the i-th placement returned by
    /// [`placements()`](Self::placements).
    pub fn surfaces(&self) -> &[Box<dyn Surface>] {
        &self.store.surfaces
    }

    /// Returns the placements of all surfaces in the system.
    ///
    /// The i-th placement corresponds to the i-th surface returned by
    /// [`surfaces()`](Self::surfaces).
    pub fn placements(&self) -> &[SurfacePlacement] {
        &self.store.placements
    }

    /// Returns the submodel for a given wavelength index, or `None` if the
    /// index is out of range.
    ///
    /// Wavelength indices are 0-based and match the order of the wavelengths
    /// slice passed to [`SequentialModel::from_surface_specs`].
    pub fn submodel(&self, wavelength_id: usize) -> Option<&(impl SequentialSubModel + use<'_>)> {
        self.paths[0].submodels.get(wavelength_id)
    }

    /// Returns all wavelength submodels as a slice.
    ///
    /// The position in the slice is the wavelength index (0-based, matching the
    /// order passed to `new`). Tangential-direction splitting is handled by
    /// `ParaxialView`, which builds one paraxial subview per wavelength ×
    /// tangential-vector combination.
    pub fn submodels(&self) -> &[impl SequentialSubModel + use<'_>] {
        &self.paths[0].submodels
    }

    /// Returns the wavelengths at which the system is modeled.
    pub fn wavelengths(&self) -> &[Float] {
        &self.wavelengths
    }

    fn gap_specs_to_gaps(gap_specs: &[GapSpec], wavelength: Float) -> Result<Vec<Gap>> {
        let mut gaps = Vec::new();
        for gap_spec in gap_specs.iter() {
            let gap = Gap::try_from_spec(gap_spec, wavelength)?;
            gaps.push(gap);
        }
        Ok(gaps)
    }

    /// Returns true if the system is rotationally symmetric about the optical
    /// axis.
    ///
    /// A system is rotationally symmetric if no surface has a tilt relative to
    /// the cursor approaching it on any path, i.e., the surface-tilt rotation
    /// equals the cursor rotation at every step across all paths.
    pub fn is_rotationally_symmetric(&self) -> bool {
        let placements = &self.store.placements;
        for path_id in 0..self.path_count() {
            let steps = self.path_steps(path_id);
            let indices = self.path_surface_indices(path_id);
            for (step, &idx) in steps.iter().zip(indices.iter()) {
                let p = &placements[idx];
                // R_surf = surface_tilt × cursor = global_to_local · cursor_to_global
                let r_surf = p.rotation_matrix * step.cursor_rotation_matrix.transpose();
                if !r_surf.approx_eq(&Mat3x3::identity(), 1e-10) {
                    return false;
                }
            }
        }
        true
    }

    /// Walks the cursor through the system, building placements and axis
    /// directions from pre-built surfaces and their placement data.
    ///
    /// `surfaces` and `surface_placements` must have the same length N.
    /// `gap_specs` must have length N - 1.
    fn build_placements_and_directions(
        surfaces: &[Box<dyn Surface>],
        surface_placements: &[PlacementSpec],
        gap_specs: &[GapSpec],
    ) -> (Vec<SurfacePlacement>, Vec<CursorPlacement>) {
        let mut placements = Vec::new();
        let mut cursor_placements = Vec::new();
        let mut cursor = Cursor::new(-gap_specs[0].thickness);

        // Surfaces 0 to N-2 (each paired with a gap that follows it).
        for ((surf, sp), gap_spec) in surfaces
            .iter()
            .zip(surface_placements.iter())
            .zip(gap_specs.iter())
        {
            cursor_placements.push(CursorPlacement {
                axis_direction: cursor.forward(),
                cursor_position: cursor.pos(),
                cursor_rotation_matrix: cursor.rotation_matrix(),
            });

            let nominal_rot = sp.rotation.rotation_matrix();
            let actual_rot = sp.rotation_offset.rotation_matrix() * nominal_rot;
            let placement = SurfacePlacement::from_decenter_and_rotation(
                sp.decenter,
                actual_rot,
                nominal_rot,
                &cursor,
            );

            // Flip the cursor upon reflection. Use only the nominal rotation
            // so that rotation_offset never redirects the cursor.
            if let BoundaryKind::Reflecting = surf.boundary_kind() {
                let norm_local = surf.norm(Vec3::new(0.0, 0.0, 0.0));
                let nominal_local_to_global = (nominal_rot * cursor.rotation_matrix()).transpose();
                let norm_global = (nominal_local_to_global * norm_local).normalize();
                cursor.reflect(&norm_global);
            }

            placements.push(placement);
            cursor.advance(gap_spec.thickness);
        }

        // Last surface - no gap after it.
        cursor_placements.push(CursorPlacement {
            axis_direction: cursor.forward(),
            cursor_position: cursor.pos(),
            cursor_rotation_matrix: cursor.rotation_matrix(),
        });
        let sp = surface_placements.last().expect("at least one surface");
        let nominal_rot = sp.rotation.rotation_matrix();
        let actual_rot = sp.rotation_offset.rotation_matrix() * nominal_rot;
        placements.push(SurfacePlacement::from_decenter_and_rotation(
            sp.decenter,
            actual_rot,
            nominal_rot,
            &cursor,
        ));

        (placements, cursor_placements)
    }

    #[cfg(feature = "serde")]
    fn surf_specs_to_surfs(
        surf_specs: &[SurfaceSpec],
        gap_specs: &[GapSpec],
        registry: Option<&SurfaceRegistry>,
    ) -> Result<SurfaceStoreContents> {
        let surfaces: Vec<Box<dyn Surface>> = surf_specs
            .iter()
            .map(|s| surface_from_spec(s, registry))
            .collect::<Result<Vec<_>>>()?;
        let surface_placements: Vec<PlacementSpec> = surf_specs
            .iter()
            .map(|spec| PlacementSpec {
                decenter: spec.decenter(),
                rotation: spec.rotation(),
                rotation_offset: spec.rotation_offset(),
            })
            .collect();
        let (placements, cursor_placements) =
            Self::build_placements_and_directions(&surfaces, &surface_placements, gap_specs);
        Ok((surfaces, placements, cursor_placements))
    }

    #[cfg(not(feature = "serde"))]
    fn surf_specs_to_surfs(
        surf_specs: &[SurfaceSpec],
        gap_specs: &[GapSpec],
    ) -> Result<SurfaceStoreContents> {
        let surfaces: Vec<Box<dyn Surface>> = surf_specs
            .iter()
            .map(surface_from_spec)
            .collect::<Result<Vec<_>>>()?;
        let surface_placements: Vec<PlacementSpec> = surf_specs
            .iter()
            .map(|spec| PlacementSpec {
                decenter: spec.decenter(),
                rotation: spec.rotation(),
                rotation_offset: spec.rotation_offset(),
            })
            .collect();
        let (placements, cursor_placements) =
            Self::build_placements_and_directions(&surfaces, &surface_placements, gap_specs);
        Ok((surfaces, placements, cursor_placements))
    }

    fn validate_gaps(gaps: &[GapSpec]) -> Result<()> {
        if gaps.is_empty() {
            return Err(anyhow!("The system must have at least one gap."));
        }
        Ok(())
    }

    fn validate_specs(gaps: &[GapSpec], wavelengths: &[Float]) -> Result<()> {
        // TODO: Validate surface specs as well!
        Self::validate_gaps(gaps)?;
        Self::validate_wavelegths(wavelengths)?;
        Ok(())
    }

    fn validate_wavelegths(wavelengths: &[Float]) -> Result<()> {
        if wavelengths.is_empty() {
            return Err(anyhow!("The system must have at least one wavelength."));
        }
        Ok(())
    }
}

impl SequentialSubModelBase {
    pub(crate) fn new(gaps: Vec<Gap>) -> Self {
        Self { gaps }
    }
}

impl SequentialSubModel for SequentialSubModelBase {
    fn gaps(&self) -> &[Gap] {
        &self.gaps
    }

    fn is_obj_at_inf(&self) -> bool {
        self.gaps
            .first()
            .expect("There must be at least one gap in a sequential submodel.")
            .thickness
            .is_infinite()
    }

    fn try_iter<'a>(
        &'a self,
        surfaces: &'a [Box<dyn Surface>],
        placements: &'a [SurfacePlacement],
        surface_indices: &'a [usize],
        beam_splitter_arms: &'a [Option<BeamSplitterPathKind>],
        path_steps: &'a [CursorPlacement],
    ) -> Result<SequentialSubModelIter<'a>> {
        SequentialSubModelIter::new(
            surfaces,
            placements,
            surface_indices,
            beam_splitter_arms,
            &self.gaps,
            path_steps,
            0,
        )
    }

    fn slice<'a>(
        &'a self,
        idx: Range<usize>,
        surface_indices: &'a [usize],
        beam_splitter_arms: &'a [Option<BeamSplitterPathKind>],
    ) -> SequentialSubModelSlice<'a> {
        let si_range = idx.start..=idx.end;
        SequentialSubModelSlice {
            surface_indices: &surface_indices[si_range.clone()],
            beam_splitter_arms: &beam_splitter_arms[si_range],
            gaps: &self.gaps[idx.clone()],
            step_offset: idx.start,
        }
    }
}

impl SequentialSubModel for SequentialSubModelSlice<'_> {
    fn gaps(&self) -> &[Gap] {
        self.gaps
    }

    fn is_obj_at_inf(&self) -> bool {
        self.gaps
            .first()
            .expect("There must be at least one gap in a sequential submodel.")
            .thickness
            .is_infinite()
    }

    fn try_iter<'b>(
        &'b self,
        surfaces: &'b [Box<dyn Surface>],
        placements: &'b [SurfacePlacement],
        _surface_indices: &'b [usize],
        _beam_splitter_arms: &'b [Option<BeamSplitterPathKind>],
        path_steps: &'b [CursorPlacement],
    ) -> Result<SequentialSubModelIter<'b>> {
        SequentialSubModelIter::new(
            surfaces,
            placements,
            self.surface_indices,
            self.beam_splitter_arms,
            self.gaps,
            path_steps,
            self.step_offset,
        )
    }

    fn slice<'b>(
        &'b self,
        idx: Range<usize>,
        _surface_indices: &'b [usize],
        _beam_splitter_arms: &'b [Option<BeamSplitterPathKind>],
    ) -> SequentialSubModelSlice<'b> {
        let si_range = idx.start..=idx.end;
        SequentialSubModelSlice {
            surface_indices: &self.surface_indices[si_range.clone()],
            beam_splitter_arms: &self.beam_splitter_arms[si_range],
            gaps: &self.gaps[idx.clone()],
            step_offset: self.step_offset + idx.start,
        }
    }
}

impl<'a> SequentialSubModelIter<'a> {
    fn new(
        surfaces: &'a [Box<dyn Surface>],
        placements: &'a [SurfacePlacement],
        surface_indices: &'a [usize],
        beam_splitter_arms: &'a [Option<BeamSplitterPathKind>],
        gaps: &'a [Gap],
        path_steps: &'a [CursorPlacement],
        step_offset: usize,
    ) -> Result<Self> {
        if surface_indices.len() != gaps.len() + 1 {
            return Err(anyhow!(
                "The number of surface indices must be one more than the number of gaps in a forward sequential submodel."
            ));
        }

        Ok(Self {
            surfaces,
            placements,
            surface_indices,
            beam_splitter_arms,
            gaps,
            path_steps,
            step_offset,
            index: 0,
        })
    }

    pub fn try_reverse(self) -> Result<SequentialSubModelReverseIter<'a>> {
        SequentialSubModelReverseIter::new(
            self.surfaces,
            self.placements,
            self.surface_indices,
            self.beam_splitter_arms,
            self.gaps,
            self.path_steps,
            self.step_offset,
        )
    }
}

impl<'a> Iterator for SequentialSubModelIter<'a> {
    type Item = Step<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.gaps.len() {
            return None;
        }
        let path_pos = self.index + 1;
        let store_idx = self.surface_indices[path_pos];
        let bs_arm = self.beam_splitter_arms[path_pos];
        let cursor_placement = self.path_steps[self.step_offset + path_pos];
        let result = if self.index == self.gaps.len() - 1 {
            // We are at the image space gap
            Step {
                gap_before: &self.gaps[self.index],
                surface: self.surfaces[store_idx].as_ref(),
                gap_after: None,
                surface_placement: &self.placements[store_idx],
                bs_arm,
                cursor_placement,
            }
        } else {
            Step {
                gap_before: &self.gaps[self.index],
                surface: self.surfaces[store_idx].as_ref(),
                gap_after: Some(&self.gaps[self.index + 1]),
                surface_placement: &self.placements[store_idx],
                bs_arm,
                cursor_placement,
            }
        };
        self.index += 1;
        Some(result)
    }
}

impl ExactSizeIterator for SequentialSubModelIter<'_> {
    fn len(&self) -> usize {
        self.gaps.len()
    }
}

impl<'a> SequentialSubModelReverseIter<'a> {
    fn new(
        surfaces: &'a [Box<dyn Surface>],
        placements: &'a [SurfacePlacement],
        surface_indices: &'a [usize],
        beam_splitter_arms: &'a [Option<BeamSplitterPathKind>],
        gaps: &'a [Gap],
        path_steps: &'a [CursorPlacement],
        step_offset: usize,
    ) -> Result<Self> {
        // Note that this requirement is different than the forward iterator.
        if surface_indices.len() != gaps.len() + 1 {
            return Err(anyhow!(
                "The number of surface indices must be one more than the number of gaps in a reversed sequential submodel."
            ));
        }

        Ok(Self {
            surfaces,
            placements,
            surface_indices,
            beam_splitter_arms,
            gaps,
            path_steps,
            step_offset,
            // We will never iterate from the image space surface in reverse.
            index: 1,
        })
    }
}

impl<'a> Iterator for SequentialSubModelReverseIter<'a> {
    type Item = Step<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Verify index's starting value; it's not necessarily 0.
        let n = self.gaps.len();
        let forward_index = n - self.index;
        if self.index < n {
            let store_idx = self.surface_indices[forward_index];
            let bs_arm = self.beam_splitter_arms[forward_index];
            let cursor_placement = self.path_steps[self.step_offset + forward_index];
            // We are somewhere in the middle of the system or at the object space gap.
            let result = Some(Step {
                gap_before: &self.gaps[forward_index],
                surface: self.surfaces[store_idx].as_ref(),
                gap_after: Some(&self.gaps[forward_index - 1]),
                surface_placement: &self.placements[store_idx],
                bs_arm,
                cursor_placement,
            });
            self.index += 1;
            result
        } else {
            None
        }
    }
}

/// Build a [`Surface`] trait object from a surface specification.
#[cfg(feature = "serde")]
pub(crate) fn surface_from_spec(
    spec: &SurfaceSpec,
    registry: Option<&SurfaceRegistry>,
) -> Result<Box<dyn Surface>> {
    match spec {
        SurfaceSpec::Conic {
            semi_diameter,
            radius_of_curvature,
            conic_constant,
            surf_kind,
            ..
        } => Ok(Box::new(Conic::new(
            *semi_diameter,
            *radius_of_curvature,
            *conic_constant,
            *surf_kind,
        ))),
        SurfaceSpec::Sphere {
            semi_diameter,
            radius_of_curvature,
            surf_kind,
            ..
        } => Ok(Box::new(Sphere::new(
            *semi_diameter,
            *radius_of_curvature,
            *surf_kind,
        ))),
        SurfaceSpec::ThinLens {
            semi_diameter,
            focal_length,
            ..
        } => Ok(Box::new(ThinLens::new(*semi_diameter, *focal_length))),
        SurfaceSpec::Custom {
            type_id, params, ..
        } => registry
            .ok_or_else(|| {
                anyhow!(
                    "a SurfaceRegistry is required to build custom surface '{type_id}'; \
                     use SequentialModel::new_with_registry"
                )
            })?
            .build(type_id, params),
        SurfaceSpec::Image { .. } => Ok(Box::new(Image::new())),
        SurfaceSpec::Object => Ok(Box::new(Object::new())),
        SurfaceSpec::Probe { .. } => Ok(Box::new(Probe::new())),
        SurfaceSpec::Iris { semi_diameter, .. } => Ok(Box::new(Iris::new(*semi_diameter))),
        SurfaceSpec::BeamSplitter { semi_diameter, .. } => {
            Ok(Box::new(BeamSplitter::new(*semi_diameter)))
        }
    }
}

/// Build a [`Surface`] trait object from a surface specification.
#[cfg(not(feature = "serde"))]
pub(crate) fn surface_from_spec(spec: &SurfaceSpec) -> Result<Box<dyn Surface>> {
    match spec {
        SurfaceSpec::Conic {
            semi_diameter,
            radius_of_curvature,
            conic_constant,
            surf_kind,
            ..
        } => Ok(Box::new(Conic::new(
            *semi_diameter,
            *radius_of_curvature,
            *conic_constant,
            *surf_kind,
        ))),
        SurfaceSpec::Sphere {
            semi_diameter,
            radius_of_curvature,
            surf_kind,
            ..
        } => Ok(Box::new(Sphere::new(
            *semi_diameter,
            *radius_of_curvature,
            *surf_kind,
        ))),
        SurfaceSpec::ThinLens {
            semi_diameter,
            focal_length,
            ..
        } => Ok(Box::new(ThinLens::new(*semi_diameter, *focal_length))),
        SurfaceSpec::Image { .. } => Ok(Box::new(Image::new())),
        SurfaceSpec::Object => Ok(Box::new(Object::new())),
        SurfaceSpec::Probe { .. } => Ok(Box::new(Probe::new())),
        SurfaceSpec::Iris { semi_diameter, .. } => Ok(Box::new(Iris::new(*semi_diameter))),
        SurfaceSpec::BeamSplitter { semi_diameter, .. } => {
            Ok(Box::new(BeamSplitter::new(*semi_diameter)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        EulerAngles, Rotation3D, core::Float, core::surfaces::Sphere, n,
        specs::surfaces::BoundaryKind,
    };

    // Helper: build a SurfacePlacement for a surface with the given rotation, in
    // an identity cursor frame (cursor aligned with global axes, origin at
    // (0,0,0)).
    fn placement_with_rotation(rotation: Rotation3D) -> (SurfacePlacement, Mat3x3) {
        let cursor_rotation_matrix = Mat3x3::identity();
        let rotation_matrix = rotation.rotation_matrix() * cursor_rotation_matrix;
        let sp = SurfacePlacement::new(
            Vec3::new(0.0, 0.0, 0.0),
            0.0,
            rotation_matrix,
            rotation_matrix,
        );
        (sp, cursor_rotation_matrix)
    }

    #[test]
    fn projected_sd_untilted_surface() {
        let r = 10.0;
        let (placement, crm) = placement_with_rotation(Rotation3D::None);
        let tol = 1e-12;
        let v_u = Vec3::new(0.0, 1.0, 0.0);
        let v_r = Vec3::new(1.0, 0.0, 0.0);
        assert!(
            (placement.projected_semi_diameter(crm, r, v_u) - r).abs() < tol,
            "U axis: expected {r}, got {}",
            placement.projected_semi_diameter(crm, r, v_u)
        );
        assert!(
            (placement.projected_semi_diameter(crm, r, v_r) - r).abs() < tol,
            "R axis: expected {r}, got {}",
            placement.projected_semi_diameter(crm, r, v_r)
        );
    }

    #[test]
    fn projected_sd_theta_tilted_mirror() {
        // 45° rotation about cursor-R; foreshortens only the U axis.
        let r = 10.0;
        let theta = 45.0_f64.to_radians();
        let (placement, crm) = placement_with_rotation(Rotation3D::IntrinsicPassiveRUF(
            EulerAngles(theta, 0.0, 0.0),
        ));
        let tol = 1e-10;
        let v_u = Vec3::new(0.0, 1.0, 0.0);
        let v_r = Vec3::new(1.0, 0.0, 0.0);
        assert!(
            (placement.projected_semi_diameter(crm, r, v_u) - r * theta.cos()).abs() < tol,
            "U axis: expected {}, got {}",
            r * theta.cos(),
            placement.projected_semi_diameter(crm, r, v_u)
        );
        assert!(
            (placement.projected_semi_diameter(crm, r, v_r) - r).abs() < tol,
            "R axis: expected {r}, got {}",
            placement.projected_semi_diameter(crm, r, v_r)
        );
    }

    #[test]
    fn projected_sd_psi_tilted_surface() {
        // 30° rotation about cursor-U; foreshortens only the R axis.
        let r = 10.0;
        let psi = 30.0_f64.to_radians();
        let (placement, crm) =
            placement_with_rotation(Rotation3D::IntrinsicPassiveRUF(EulerAngles(0.0, psi, 0.0)));
        let tol = 1e-10;
        let v_u = Vec3::new(0.0, 1.0, 0.0);
        let v_r = Vec3::new(1.0, 0.0, 0.0);
        assert!(
            (placement.projected_semi_diameter(crm, r, v_r) - r * psi.cos()).abs() < tol,
            "R axis: expected {}, got {}",
            r * psi.cos(),
            placement.projected_semi_diameter(crm, r, v_r)
        );
        assert!(
            (placement.projected_semi_diameter(crm, r, v_u) - r).abs() < tol,
            "U axis: expected {r}, got {}",
            placement.projected_semi_diameter(crm, r, v_u)
        );
    }

    #[test]
    fn projected_sd_after_fold() {
        // Figure-Z system: two flat mirrors each with theta = 30° about cursor-R.
        // The projected SD in the U direction should be r * cos(30°) for both mirrors.
        use crate::examples::mirrors_figure_z;
        let air = n!(1.0);
        let wavelengths = [0.5876];
        let model = mirrors_figure_z::sequential_model(air, &wavelengths);
        let surfaces = model.surfaces();
        let placements = model.placements();
        let path_steps = model.path_steps(0);
        let r = 12.7_f64;
        let expected_u = r * (30.0_f64.to_radians()).cos();
        let tol = 1e-10;
        let v_u = Vec3::new(0.0, 1.0, 0.0);
        let v_r = Vec3::new(1.0, 0.0, 0.0);

        // Surface indices: 0 = Object, 1 = Mirror 1, 2 = Mirror 2, 3 = Image
        for &mirror_idx in &[1usize, 2usize] {
            let sd = surfaces[mirror_idx].mask().semi_diameter();
            let placement = &placements[mirror_idx];
            let crm = path_steps[mirror_idx].cursor_rotation_matrix;
            assert!(
                (placement.projected_semi_diameter(crm, sd, v_u) - expected_u).abs() < tol,
                "Mirror {mirror_idx} U: expected {expected_u}, got {}",
                placement.projected_semi_diameter(crm, sd, v_u)
            );
            assert!(
                (placement.projected_semi_diameter(crm, sd, v_r) - r).abs() < tol,
                "Mirror {mirror_idx} R: expected {r}, got {}",
                placement.projected_semi_diameter(crm, sd, v_r)
            );
        }
    }

    /// Each entry in the result is the *incident* direction at that surface.
    ///
    /// Mirror normal in global frame (30° passive rotation about X):
    ///   n = (0, −sin30°, cos30°) = (0, −0.5, √3/2)
    ///
    /// Incident at Mirror 1 (surface 1): v = v_init = (0, 1, 0)
    /// Reflected by Mirror 1: v' = Y − 2(−0.5)·n = (0, 0.5, √3/2)
    /// Incident at Mirror 2 (surface 2): v = (0, 0.5, √3/2)
    #[test]
    fn propagate_tangential_vec_through_fold() {
        use crate::examples::mirrors_figure_z;
        use approx::assert_abs_diff_eq;

        let model = mirrors_figure_z::sequential_model(n!(1.0), &[0.5876]);
        let v_init = Vec3::new(0.0, 1.0, 0.0); // phi = 90°

        let surface_indices: Vec<usize> = (0..model.surfaces().len()).collect();
        let vecs = propagate_tangential_vec(
            v_init,
            model.surfaces(),
            model.placements(),
            &surface_indices,
        );

        let sqrt3_over_2 = (3.0_f64 / 4.0_f64).sqrt();

        // Incident at Mirror 1 (surface 1): unchanged from v_init
        assert_abs_diff_eq!(vecs[1].x(), 0.0, epsilon = 1e-10);
        assert_abs_diff_eq!(vecs[1].y(), 1.0, epsilon = 1e-10);
        assert_abs_diff_eq!(vecs[1].z(), 0.0, epsilon = 1e-10);

        // Incident at Mirror 2 (surface 2): reflected from Mirror 1 = (0, 0.5, √3/2)
        assert_abs_diff_eq!(vecs[2].x(), 0.0, epsilon = 1e-10);
        assert_abs_diff_eq!(vecs[2].y(), 0.5, epsilon = 1e-10);
        assert_abs_diff_eq!(vecs[2].z(), sqrt3_over_2, epsilon = 1e-10);
    }

    #[test]
    fn is_rotationally_symmetric() {
        // A simple on-axis system (Object → Sphere → Image) is symmetric.
        use crate::{GapSpec, SurfaceSpec, specs::surfaces::BoundaryKind};
        let air = n!(1.0);
        let glass = n!(1.5);
        let gaps = vec![
            GapSpec {
                thickness: f64::INFINITY,
                refractive_index: air.clone(),
            },
            GapSpec {
                thickness: 5.0,
                refractive_index: glass,
            },
            GapSpec {
                thickness: 50.0,
                refractive_index: air,
            },
        ];
        let surfaces = vec![
            SurfaceSpec::Object,
            SurfaceSpec::Sphere {
                semi_diameter: 12.5,
                radius_of_curvature: 25.8,
                surf_kind: BoundaryKind::Refracting,
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            },
            SurfaceSpec::Image {
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            },
        ];
        let simple =
            SequentialModel::from_surface_specs(&gaps, &surfaces, &[0.5876], None).unwrap();
        assert!(simple.is_rotationally_symmetric());

        // A system with tilted surfaces is not rotationally symmetric.
        use crate::examples::mirrors_figure_z;
        let air2 = n!(1.0);
        let figure_z = mirrors_figure_z::sequential_model(air2, &[0.5876]);
        assert!(!figure_z.is_rotationally_symmetric());
    }

    #[test]
    fn test_first_physical_step() {
        // Object(0), Probe(1), Sphere(2), Sphere(3), Image(4) — first physical step is
        // 2.
        let surfaces: Vec<Box<dyn Surface>> = vec![
            Box::new(Object::new()),
            Box::new(Probe::new()),
            Box::new(Sphere::new(1.0, 1.0, BoundaryKind::Refracting)),
            Box::new(Sphere::new(1.0, 1.0, BoundaryKind::Refracting)),
            Box::new(Image::new()),
        ];
        let surface_indices: Vec<usize> = (0..surfaces.len()).collect();

        let result = first_physical_step(&surface_indices, &surfaces);
        assert_eq!(result, Some(2));
    }

    #[test]
    fn test_last_physical_step() {
        // Object(0), Sphere(1), Sphere(2), Probe(3), Image(4) — last physical step is
        // 2.
        let surfaces: Vec<Box<dyn Surface>> = vec![
            Box::new(Object::new()),
            Box::new(Sphere::new(1.0, 1.0, BoundaryKind::Refracting)),
            Box::new(Sphere::new(1.0, 1.0, BoundaryKind::Refracting)),
            Box::new(Probe::new()),
            Box::new(Image::new()),
        ];
        let surface_indices: Vec<usize> = (0..surfaces.len()).collect();

        let result = last_physical_step(&surface_indices, &surfaces);
        assert_eq!(result, Some(2));
    }

    #[test]
    fn test_reversed_surface_id() {
        // 5-surface system (indices 0-4): reversed_surface_id(5, i) = 5 - i - 1 = 4 - i
        let result = reversed_surface_id(5, 2);
        assert_eq!(result, 2);

        let result = reversed_surface_id(5, 1);
        assert_eq!(result, 3);
    }

    #[test]
    fn placement_is_infinite() {
        let id = Mat3x3::identity();

        // z-coordinate infinite
        let p = SurfacePlacement::new(Vec3::new(0.0, 0.0, Float::INFINITY), 0.0, id, id);
        assert!(p.is_infinite());

        // y-coordinate infinite
        let p = SurfacePlacement::new(Vec3::new(0.0, Float::INFINITY, 0.0), 0.0, id, id);
        assert!(p.is_infinite());

        // x-coordinate infinite
        let p = SurfacePlacement::new(Vec3::new(Float::INFINITY, 0.0, 0.0), 0.0, id, id);
        assert!(p.is_infinite());

        // finite
        let p = SurfacePlacement::new(Vec3::new(0.0, 0.0, 0.0), 0.0, id, id);
        assert!(!p.is_infinite());
    }

    #[test]
    fn track_equals_z_for_straight_system() {
        use crate::examples::convexplano_lens;
        let air = n!(1.0);
        let nbk7 = n!(1.515);
        let wavelengths = [0.5876];
        let model = convexplano_lens::sequential_model(air, nbk7, &wavelengths);
        for placement in model.placements() {
            if placement.position.z().is_finite() {
                assert!(
                    (placement.track - placement.position.z()).abs() < 1e-10,
                    "Expected track == z for straight system, got track={}, z={}",
                    placement.track,
                    placement.position.z()
                );
            }
        }
    }

    /// For a straight system, axis_direction should equal (0, 0, 1) everywhere.
    #[test]
    fn cursor_placement_axis_direction_straight_system() {
        use crate::examples::convexplano_lens;
        use approx::assert_abs_diff_eq;
        let model = convexplano_lens::sequential_model(n!(1.0), n!(1.515), &[0.5876]);
        for cp in model.path_steps(0) {
            assert_abs_diff_eq!(cp.axis_direction.x(), 0.0, epsilon = 1e-12);
            assert_abs_diff_eq!(cp.axis_direction.y(), 0.0, epsilon = 1e-12);
            assert_abs_diff_eq!(cp.axis_direction.z(), 1.0, epsilon = 1e-12);
        }
    }

    // --- stop_surface validation tests ---
    //
    // System layout for these tests:
    //   0: Object
    //   1: Sphere  (eligible)
    //   2: Probe   (ineligible)
    //   3: Iris    (eligible)
    //   4: Image
    fn stop_validation_specs() -> (Vec<GapSpec>, Vec<SurfaceSpec>) {
        let air = n!(1.0);
        let glass = n!(1.5);
        let gaps = vec![
            GapSpec {
                thickness: f64::INFINITY,
                refractive_index: air.clone(),
            },
            GapSpec {
                thickness: 5.0,
                refractive_index: glass,
            },
            GapSpec {
                thickness: 1.0,
                refractive_index: air.clone(),
            },
            GapSpec {
                thickness: 5.0,
                refractive_index: air,
            },
        ];
        let surfaces = vec![
            SurfaceSpec::Object,
            SurfaceSpec::Sphere {
                semi_diameter: 10.0,
                radius_of_curvature: 50.0,
                surf_kind: BoundaryKind::Refracting,
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            },
            SurfaceSpec::Probe {
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            },
            SurfaceSpec::Iris {
                semi_diameter: 5.0,
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            },
            SurfaceSpec::Image {
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            },
        ];
        (gaps, surfaces)
    }

    #[test]
    fn stop_surface_sphere_is_accepted() {
        let (gaps, surfaces) = stop_validation_specs();
        assert!(SequentialModel::from_surface_specs(&gaps, &surfaces, &[0.5876], Some(1)).is_ok());
    }

    #[test]
    fn stop_surface_iris_is_accepted() {
        let (gaps, surfaces) = stop_validation_specs();
        assert!(SequentialModel::from_surface_specs(&gaps, &surfaces, &[0.5876], Some(3)).is_ok());
    }

    #[test]
    fn stop_surface_object_is_rejected() {
        let (gaps, surfaces) = stop_validation_specs();
        assert!(SequentialModel::from_surface_specs(&gaps, &surfaces, &[0.5876], Some(0)).is_err());
    }

    #[test]
    fn stop_surface_image_is_rejected() {
        let (gaps, surfaces) = stop_validation_specs();
        assert!(SequentialModel::from_surface_specs(&gaps, &surfaces, &[0.5876], Some(4)).is_err());
    }

    #[test]
    fn stop_surface_out_of_range_is_rejected() {
        let (gaps, surfaces) = stop_validation_specs();
        assert!(
            SequentialModel::from_surface_specs(&gaps, &surfaces, &[0.5876], Some(99)).is_err()
        );
    }

    #[test]
    fn stop_surface_probe_is_rejected() {
        let (gaps, surfaces) = stop_validation_specs();
        assert!(SequentialModel::from_surface_specs(&gaps, &surfaces, &[0.5876], Some(2)).is_err());
    }

    // AT-6: rotation_offset on a reflecting surface never redirects the cursor.
    // A mirror with rotation = 45° about R redirects the cursor regardless of
    // rotation_offset. Adding a rotation_offset should not change the image
    // surface position (it only changes the surface's tilt, not the beam path).
    #[test]
    fn at6_rotation_offset_does_not_redirect_cursor() {
        use crate::specs::surfaces::BoundaryKind;
        use crate::{GapSpec, SurfaceSpec, Vec3, n};

        let theta = 45.0_f64.to_radians();
        let gaps = vec![
            GapSpec {
                thickness: 10.0,
                refractive_index: n!(1.0),
            },
            GapSpec {
                thickness: 10.0,
                refractive_index: n!(1.0),
            },
        ];

        // Mirror with rotation only (no rotation_offset).
        let surfaces_no_offset = vec![
            SurfaceSpec::Object,
            SurfaceSpec::Sphere {
                semi_diameter: 25.4,
                radius_of_curvature: f64::INFINITY,
                surf_kind: BoundaryKind::Reflecting,
                rotation: Rotation3D::IntrinsicPassiveRUF(EulerAngles(theta, 0.0, 0.0)),
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            },
            SurfaceSpec::Image {
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            },
        ];
        let model_no_offset =
            SequentialModel::from_surface_specs(&gaps, &surfaces_no_offset, &[0.5876], None)
                .expect("model_no_offset builds");

        // Mirror with the same rotation plus a non-trivial rotation_offset.
        let phi = 10.0_f64.to_radians();
        let surfaces_with_offset = vec![
            SurfaceSpec::Object,
            SurfaceSpec::Sphere {
                semi_diameter: 25.4,
                radius_of_curvature: f64::INFINITY,
                surf_kind: BoundaryKind::Reflecting,
                rotation: Rotation3D::IntrinsicPassiveRUF(EulerAngles(theta, 0.0, 0.0)),
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::IntrinsicPassiveRUF(EulerAngles(0.0, phi, 0.0)),
            },
            SurfaceSpec::Image {
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            },
        ];
        let model_with_offset =
            SequentialModel::from_surface_specs(&gaps, &surfaces_with_offset, &[0.5876], None)
                .expect("model_with_offset builds");

        // Image surface position must be identical in both models because the
        // cursor path is determined by `rotation` only.
        let tol = 1e-10;
        let pos_no_offset = model_no_offset.placements()[2].position;
        let pos_with_offset = model_with_offset.placements()[2].position;
        assert!(
            (pos_no_offset.x() - pos_with_offset.x()).abs() < tol
                && (pos_no_offset.y() - pos_with_offset.y()).abs() < tol
                && (pos_no_offset.z() - pos_with_offset.z()).abs() < tol,
            "rotation_offset changed cursor path: no_offset={:?}, with_offset={:?}",
            pos_no_offset,
            pos_with_offset
        );
    }

    #[test]
    fn iterator_uses_surface_indices_for_lookup() {
        // Build a 4-surface store: [Object, Sphere_A(sd=10), Sphere_B(sd=20), Image]
        // Build a submodel whose surface_indices = [0, 2, 1, 3]
        // (visits Sphere_B before Sphere_A — reversed order).
        // Verify the iterator yields Sphere_B at step 0 and Sphere_A at step 1.
        let surfaces: Vec<Box<dyn Surface>> = vec![
            Box::new(Object::new()),
            Box::new(Sphere::new(10.0, 50.0, BoundaryKind::Refracting)),
            Box::new(Sphere::new(20.0, 100.0, BoundaryKind::Refracting)),
            Box::new(Image::new()),
        ];
        let id = Mat3x3::identity();
        let placements = vec![
            SurfacePlacement::new(Vec3::new(0., 0., 0.), 0., id, id),
            SurfacePlacement::new(Vec3::new(0., 0., 5.), 5., id, id),
            SurfacePlacement::new(Vec3::new(0., 0., 10.), 10., id, id),
            SurfacePlacement::new(Vec3::new(0., 0., 15.), 15., id, id),
        ];
        let path_steps: Vec<CursorPlacement> = (0..4)
            .map(|_| CursorPlacement {
                axis_direction: Vec3::new(0.0, 0.0, 1.0),
                cursor_position: Vec3::new(0.0, 0.0, 0.0),
                cursor_rotation_matrix: id,
            })
            .collect();
        let gaps = vec![
            Gap {
                thickness: 5.0,
                refractive_index: RefractiveIndex::try_from_spec(n!(1.0).as_ref(), 0.5876).unwrap(),
            },
            Gap {
                thickness: 5.0,
                refractive_index: RefractiveIndex::try_from_spec(n!(1.0).as_ref(), 0.5876).unwrap(),
            },
            Gap {
                thickness: 5.0,
                refractive_index: RefractiveIndex::try_from_spec(n!(1.0).as_ref(), 0.5876).unwrap(),
            },
        ];
        let surface_indices = vec![0, 2, 1, 3];
        let bs_arms = vec![None; 4];
        let submodel = SequentialSubModelBase::new(gaps);
        let mut iter = submodel
            .try_iter(
                &surfaces,
                &placements,
                &surface_indices,
                &bs_arms,
                &path_steps,
            )
            .unwrap();

        let step0 = iter.next().unwrap();
        // step0 surface should be Sphere at index 2 (sd = 20.0)
        assert_eq!(step0.surface.mask().semi_diameter(), 20.0);

        let step1 = iter.next().unwrap();
        // step1 surface should be Sphere at index 1 (sd = 10.0)
        assert_eq!(step1.surface.mask().semi_diameter(), 10.0);
    }

    #[test]
    fn repeated_surface_index_is_permitted() {
        // surface_indices = [0, 1, 1, 2]: index 1 repeated — groundwork for return
        // paths.
        let surfaces: Vec<Box<dyn Surface>> = vec![
            Box::new(Object::new()),
            Box::new(Sphere::new(10.0, 50.0, BoundaryKind::Refracting)),
            Box::new(Image::new()),
        ];
        let id = Mat3x3::identity();
        let placements = vec![
            SurfacePlacement::new(Vec3::new(0., 0., 0.), 0., id, id),
            SurfacePlacement::new(Vec3::new(0., 0., 5.), 5., id, id),
            SurfacePlacement::new(Vec3::new(0., 0., 10.), 10., id, id),
        ];
        let path_steps: Vec<CursorPlacement> = (0..4)
            .map(|_| CursorPlacement {
                axis_direction: Vec3::new(0.0, 0.0, 1.0),
                cursor_position: Vec3::new(0.0, 0.0, 0.0),
                cursor_rotation_matrix: id,
            })
            .collect();
        let gaps = vec![
            Gap {
                thickness: 5.0,
                refractive_index: RefractiveIndex::try_from_spec(n!(1.0).as_ref(), 0.5876).unwrap(),
            },
            Gap {
                thickness: 5.0,
                refractive_index: RefractiveIndex::try_from_spec(n!(1.0).as_ref(), 0.5876).unwrap(),
            },
            Gap {
                thickness: 5.0,
                refractive_index: RefractiveIndex::try_from_spec(n!(1.0).as_ref(), 0.5876).unwrap(),
            },
        ];
        let surface_indices = vec![0, 1, 1, 2];
        let bs_arms = vec![None; 4];
        let submodel = SequentialSubModelBase::new(gaps);
        // Must not panic or error — iteration visits index 1 twice.
        let count = submodel
            .try_iter(
                &surfaces,
                &placements,
                &surface_indices,
                &bs_arms,
                &path_steps,
            )
            .unwrap()
            .count();
        assert_eq!(count, 3); // 3 gaps → 3 steps
    }

    #[test]
    fn iterator_exposes_bs_arm_on_beam_splitter_step() {
        use crate::core::surfaces::BeamSplitter;
        use crate::specs::surfaces::BeamSplitterPathKind;
        // Surfaces: [Object(0), Sphere(1), BeamSplitter(2), Image(3)]
        // beam_splitter_arms = [None, None, Some(Transmitting), None]
        let surfaces: Vec<Box<dyn Surface>> = vec![
            Box::new(Object::new()),
            Box::new(Sphere::new(10.0, 50.0, BoundaryKind::Refracting)),
            Box::new(BeamSplitter::new(10.0)),
            Box::new(Image::new()),
        ];
        let id = Mat3x3::identity();
        let placements = vec![
            SurfacePlacement::new(Vec3::new(0., 0., 0.), 0., id, id),
            SurfacePlacement::new(Vec3::new(0., 0., 10.), 10., id, id),
            SurfacePlacement::new(Vec3::new(0., 0., 20.), 20., id, id),
            SurfacePlacement::new(Vec3::new(0., 0., 30.), 30., id, id),
        ];
        let path_steps: Vec<CursorPlacement> = (0..4)
            .map(|_| CursorPlacement {
                axis_direction: Vec3::new(0.0, 0.0, 1.0),
                cursor_position: Vec3::new(0.0, 0.0, 0.0),
                cursor_rotation_matrix: id,
            })
            .collect();
        let gaps = vec![
            Gap {
                thickness: 10.0,
                refractive_index: RefractiveIndex::try_from_spec(n!(1.0).as_ref(), 0.5876).unwrap(),
            },
            Gap {
                thickness: 10.0,
                refractive_index: RefractiveIndex::try_from_spec(n!(1.0).as_ref(), 0.5876).unwrap(),
            },
            Gap {
                thickness: 10.0,
                refractive_index: RefractiveIndex::try_from_spec(n!(1.0).as_ref(), 0.5876).unwrap(),
            },
        ];
        let surface_indices = vec![0, 1, 2, 3];
        let bs_arms = vec![None, None, Some(BeamSplitterPathKind::Transmitting), None];
        let submodel = SequentialSubModelBase::new(gaps);
        let mut iter = submodel
            .try_iter(
                &surfaces,
                &placements,
                &surface_indices,
                &bs_arms,
                &path_steps,
            )
            .unwrap();

        let step0 = iter.next().unwrap(); // Sphere
        assert_eq!(step0.bs_arm, None);

        let step1 = iter.next().unwrap(); // BeamSplitter
        assert_eq!(step1.bs_arm, Some(BeamSplitterPathKind::Transmitting));

        let step2 = iter.next().unwrap(); // Image
        assert_eq!(step2.bs_arm, None);
    }

    // Step 2 — verify public accessors are unchanged after SurfaceStore/OpticalPath
    // refactor.
    #[test]
    fn single_path_accessors_unchanged_after_store_refactor() {
        use crate::examples::convexplano_lens;
        let model = convexplano_lens::sequential_model(n!(1.0), n!(1.515), &[0.5876]);

        // surfaces() returns all surfaces
        assert_eq!(model.surfaces().len(), 4); // Object, Sphere, Sphere, Image

        // submodel(0) is accessible and has correct gap count
        let sm = model.submodel(0).unwrap();
        assert_eq!(sm.gaps().len(), 3);

        // wavelengths preserved
        assert_eq!(model.wavelengths(), &[0.5876]);
    }

    // AT-7: rotation_offset changes placement.rotation_matrix without changing
    // the cursor direction (verified via the mirror surface itself).
    #[test]
    fn at7_rotation_offset_changes_rotation_matrix() {
        use crate::specs::surfaces::BoundaryKind;
        use crate::{GapSpec, SurfaceSpec, Vec3, n};

        let gaps = vec![
            GapSpec {
                thickness: 10.0,
                refractive_index: n!(1.0),
            },
            GapSpec {
                thickness: 10.0,
                refractive_index: n!(1.0),
            },
        ];
        let phi = 5.0_f64.to_radians();

        let surfaces_no_offset = vec![
            SurfaceSpec::Object,
            SurfaceSpec::Sphere {
                semi_diameter: 25.4,
                radius_of_curvature: f64::INFINITY,
                surf_kind: BoundaryKind::Refracting,
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            },
            SurfaceSpec::Image {
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            },
        ];
        let surfaces_with_offset = vec![
            SurfaceSpec::Object,
            SurfaceSpec::Sphere {
                semi_diameter: 25.4,
                radius_of_curvature: f64::INFINITY,
                surf_kind: BoundaryKind::Refracting,
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::IntrinsicPassiveRUF(EulerAngles(0.0, phi, 0.0)),
            },
            SurfaceSpec::Image {
                rotation: Rotation3D::None,
                decenter: Vec3::new(0.0, 0.0, 0.0),
                rotation_offset: Rotation3D::None,
            },
        ];

        let model_no =
            SequentialModel::from_surface_specs(&gaps, &surfaces_no_offset, &[0.5876], None)
                .expect("model_no builds");
        let model_with =
            SequentialModel::from_surface_specs(&gaps, &surfaces_with_offset, &[0.5876], None)
                .expect("model_with builds");

        let rot_no = model_no.placements()[1].rotation_matrix;
        let rot_with = model_with.placements()[1].rotation_matrix;

        // rotation_matrix must differ when rotation_offset != None.
        assert!(
            !rot_no.approx_eq(&rot_with, 1e-10),
            "rotation_matrix should change when rotation_offset is set"
        );

        // nominal_inv_rotation_matrix must be identical (cursor sees same nominal).
        let nom_no = model_no.placements()[1].nominal_inv_rotation_matrix;
        let nom_with = model_with.placements()[1].nominal_inv_rotation_matrix;
        assert!(
            nom_no.approx_eq(&nom_with, 1e-10),
            "nominal_inv_rotation_matrix should be identical: no={:?}, with={:?}",
            nom_no,
            nom_with
        );

        // Image surface position must be identical (cursor path unchanged).
        let tol = 1e-10;
        let img_no = model_no.placements()[2].position;
        let img_with = model_with.placements()[2].position;
        assert!(
            (img_no.z() - img_with.z()).abs() < tol,
            "image z changed: no={}, with={}",
            img_no.z(),
            img_with.z()
        );
    }

    // AT: path 0 cursor positions lie along +Z; path 1 cursor positions
    // diverge to +Y after the beam splitter.
    #[test]
    fn path_steps_cursor_positions_two_path_model() {
        use crate::examples::beam_splitter;
        use crate::specs::gaps::ConstantRefractiveIndex;
        use std::rc::Rc;

        let n_air: Rc<dyn crate::RefractiveIndexSpec> =
            Rc::new(ConstantRefractiveIndex::new(1.0, 0.0));
        let model = beam_splitter::two_path_model(n_air, &[0.5876], 10.0, 10.0);

        let tol = 1e-10;

        // Path 0: Object(z=−inf), BS(z=0), Image_T(z=10)
        let steps0 = model.path_steps(0);
        assert_eq!(steps0.len(), 3);
        // Object cursor is at −∞ along Z.
        assert!(steps0[0].cursor_position.z().is_infinite());
        // BS cursor at z=0.
        assert!((steps0[1].cursor_position.z()).abs() < tol);
        // Image_T cursor at z=10.
        assert!((steps0[2].cursor_position.z() - 10.0).abs() < tol);
        // All path-0 cursor positions have zero x and y.
        for s in steps0.iter().filter(|s| s.cursor_position.z().is_finite()) {
            assert!(s.cursor_position.x().abs() < tol);
            assert!(s.cursor_position.y().abs() < tol);
        }

        // Path 1: Object(shared), BS(shared, reflected to −Y), Image_R
        // The BS uses a −45° rotation about R, so the reflected arm travels in −Y.
        let steps1 = model.path_steps(1);
        assert_eq!(steps1.len(), 3);
        // Image_R cursor position diverges to −Y: x=0, y=−10, z=0.
        assert!(
            steps1[2].cursor_position.x().abs() < tol,
            "x={}",
            steps1[2].cursor_position.x()
        );
        assert!(
            (steps1[2].cursor_position.y() + 10.0).abs() < tol,
            "y={}",
            steps1[2].cursor_position.y()
        );
        assert!(
            steps1[2].cursor_position.z().abs() < tol,
            "z={}",
            steps1[2].cursor_position.z()
        );
    }

    // AT: path 1's axis direction at the Image_R step points along −Y after a
    // −45° BS rotation about R.
    #[test]
    fn path_steps_axis_direction_reflected_arm() {
        use crate::examples::beam_splitter;
        use crate::specs::gaps::ConstantRefractiveIndex;
        use std::rc::Rc;

        let n_air: Rc<dyn crate::RefractiveIndexSpec> =
            Rc::new(ConstantRefractiveIndex::new(1.0, 0.0));
        let model = beam_splitter::two_path_model(n_air, &[0.5876], 10.0, 10.0);

        let tol = 1e-10;
        let steps1 = model.path_steps(1);

        // The BS example uses a −45° rotation about R, so the reflected arm
        // travels in the −Y direction.
        let d = steps1[2].axis_direction;
        assert!(d.x().abs() < tol, "x={}", d.x());
        assert!((d.y() + 1.0).abs() < tol, "y={}", d.y());
        assert!(d.z().abs() < tol, "z={}", d.z());
    }
}
