/// Data types for modeling sequential ray tracing systems.
pub mod builder;
pub mod solves;

use std::ops::Range;

use anyhow::{Result, anyhow};

#[cfg(feature = "serde")]
use crate::core::surfaces::SurfaceRegistry;
use crate::core::{
    Float,
    math::{
        geometry::reference_frames::Cursor,
        linalg::{mat3x3::Mat3x3, rotations::Rotation3D},
        vec3::Vec3,
    },
    placement::Placement,
    refractive_index::RefractiveIndex,
    surfaces::{Conic, Image, Iris, Object, Probe, Sphere, Surface, SurfaceKind},
};
use crate::specs::{
    gaps::GapSpec,
    surfaces::{BoundaryType, SurfaceSpec},
};

type SurfsPlacementsDirs = (Vec<Box<dyn Surface>>, Vec<Placement>, Vec<Vec3>);

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
    surfaces: Vec<Box<dyn Surface>>,
    placements: Vec<Placement>,
    submodels: Vec<SequentialSubModelBase>,
    wavelengths: Vec<Float>,

    // The cursor forward direction at each surface vertex.
    axis_directions: Vec<Vec3>,

    /// User-specified aperture stop surface index, or `None` for auto-derived.
    stop_surface: Option<usize>,
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
        placements: &'a [Placement],
    ) -> Result<SequentialSubModelIter<'a>>;

    fn slice(&self, idx: Range<usize>) -> SequentialSubModelSlice<'_> {
        SequentialSubModelSlice {
            gaps: &self.gaps()[idx],
        }
    }
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
    gaps: &'a [Gap],
}

/// An iterator over the surfaces and gaps in a submodel.
///
/// Most operations in sequential modeling involve use of this iterator.
pub struct SequentialSubModelIter<'a> {
    surfaces: &'a [Box<dyn Surface>],
    placements: &'a [Placement],
    gaps: &'a [Gap],
    index: usize,
}

/// A reverse iterator over the surfaces and gaps in a submodel.
pub struct SequentialSubModelReverseIter<'a> {
    surfaces: &'a [Box<dyn Surface>],
    placements: &'a [Placement],
    gaps: &'a [Gap],
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
    pub placement: &'a Placement,
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
    placements: &[Placement],
) -> Vec<Vec3> {
    use crate::specs::surfaces::BoundaryType;
    let mut v = v_init;
    surfaces
        .iter()
        .zip(placements.iter())
        .map(|(surf, placement)| {
            let v_incident = v;
            if let BoundaryType::Reflecting = surf.boundary_type() {
                // Normal in global frame: third column of inv_rotation_matrix
                // (maps local Z to global).
                let n = placement.inv_rotation_matrix * Vec3::new(0.0, 0.0, 1.0);
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

/// Returns the index of the first physical surface in the system.
///
/// A physical surface is one that has a finite semi-diameter,
/// i.e., a Conic or Iris. Object, Image, and Probe surfaces are excluded.
pub(crate) fn first_physical_surface(surfaces: &[Box<dyn Surface>]) -> Option<usize> {
    surfaces
        .iter()
        .position(|surf| surf.mask().semi_diameter().is_finite())
}

/// Returns the index of the last physical surface in the system.
///
/// A physical surface is one that limits the has a finite semi-diameter,
/// i.e., a Conic or Iris. Object, Image, and Probe surfaces are excluded.
pub fn last_physical_surface(surfaces: &[Box<dyn Surface>]) -> Option<usize> {
    surfaces
        .iter()
        .rposition(|surf| surf.mask().semi_diameter().is_finite())
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
            let (surfaces, placements, axis_directions) =
                Self::surf_specs_to_surfs(surface_specs, gap_specs)?;
            if let Some(i) = stop_surface {
                Self::validate_stop_surface(&surfaces, i)?;
            }
            let mut models: Vec<SequentialSubModelBase> = Vec::new();
            for &wavelength in wavelengths.iter() {
                let gaps = Self::gap_specs_to_gaps(gap_specs, wavelength)?;
                models.push(SequentialSubModelBase::new(gaps));
            }
            Ok(Self {
                surfaces,
                placements,
                submodels: models,
                wavelengths: wavelengths.to_vec(),
                axis_directions,
                stop_surface,
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
        let (surfaces, placements, axis_directions) =
            Self::surf_specs_to_surfs(surface_specs, gap_specs, registry)?;
        if let Some(i) = stop_surface {
            Self::validate_stop_surface(&surfaces, i)?;
        }
        let mut models: Vec<SequentialSubModelBase> = Vec::new();
        for &wavelength in wavelengths.iter() {
            let gaps = Self::gap_specs_to_gaps(gap_specs, wavelength)?;
            models.push(SequentialSubModelBase::new(gaps));
        }
        Ok(Self {
            surfaces,
            placements,
            submodels: models,
            wavelengths: wavelengths.to_vec(),
            axis_directions,
            stop_surface,
        })
    }

    /// Creates a new sequential model from pre-built surface trait objects.
    ///
    /// Use this when constructing a model programmatically in Rust without
    /// going through the spec/serialization layer. Each element of `surfaces`
    /// pairs a surface implementation with its tilt rotation; pass
    /// [`Rotation3D::None`] for untilted surfaces.
    ///
    /// # Arguments
    /// * `surfaces` - Pre-built surfaces paired with their tilt rotations.
    /// * `gap_specs` - Gaps between surfaces (`surfaces.len() - 1` elements).
    /// * `wavelengths` - Wavelengths at which to model the system.
    pub fn from_surfaces(
        surfaces: Vec<(Box<dyn Surface>, Rotation3D)>,
        gap_specs: &[GapSpec],
        wavelengths: &[Float],
        stop_surface: Option<usize>,
    ) -> Result<Self> {
        if surfaces.len() != gap_specs.len() + 1 {
            return Err(anyhow!(
                "Expected {} gap(s) for {} surface(s), got {}.",
                surfaces.len() - 1,
                surfaces.len(),
                gap_specs.len()
            ));
        }
        Self::validate_specs(gap_specs, wavelengths)?;

        let (surfs, rotations): (Vec<_>, Vec<_>) = surfaces.into_iter().unzip();
        let (placements, axis_directions) =
            Self::build_placements_and_directions(&surfs, &rotations, gap_specs);

        if let Some(i) = stop_surface {
            Self::validate_stop_surface(&surfs, i)?;
        }

        let mut models: Vec<SequentialSubModelBase> = Vec::new();
        for &wavelength in wavelengths.iter() {
            let gaps = Self::gap_specs_to_gaps(gap_specs, wavelength)?;
            models.push(SequentialSubModelBase::new(gaps));
        }

        Ok(Self {
            surfaces: surfs,
            placements,
            submodels: models,
            wavelengths: wavelengths.to_vec(),
            axis_directions,
            stop_surface,
        })
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
            SurfaceKind::Conic | SurfaceKind::Sphere | SurfaceKind::Iris => Ok(()),
            kind => Err(anyhow!(
                "surface {i} ({kind:?}) is not eligible as the aperture stop; \
                 only Conic and Iris surfaces are allowed"
            )),
        }
    }

    /// Returns the user-specified aperture stop surface index, or `None` if the
    /// stop is derived automatically from the paraxial ray trace.
    pub fn stop_surface(&self) -> Option<usize> {
        self.stop_surface
    }

    /// Returns the largest semi-diameter of any surface in the system.
    ///
    /// This ignores surfaces without any size, such as object, probe, and image
    /// surfaces.
    pub fn largest_semi_diameter(&self) -> Float {
        self.surfaces
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
        &self.surfaces
    }

    /// Returns the placements of all surfaces in the system.
    ///
    /// The i-th placement corresponds to the i-th surface returned by
    /// [`surfaces()`](Self::surfaces).
    pub fn placements(&self) -> &[Placement] {
        &self.placements
    }

    /// Returns the submodel for a given wavelength index, or `None` if the
    /// index is out of range.
    ///
    /// Wavelength indices are 0-based and match the order of the wavelengths
    /// slice passed to [`SequentialModel::from_surface_specs`].
    pub fn submodel(&self, wavelength_id: usize) -> Option<&(impl SequentialSubModel + use<'_>)> {
        self.submodels.get(wavelength_id)
    }

    /// Returns all wavelength submodels as a slice.
    ///
    /// The position in the slice is the wavelength index (0-based, matching the
    /// order passed to `new`). Tangential-direction splitting is handled by
    /// `ParaxialView`, which builds one paraxial subview per wavelength ×
    /// tangential-vector combination.
    pub fn submodels(&self) -> &[impl SequentialSubModel + use<'_>] {
        &self.submodels
    }

    /// Returns the wavelengths at which the system is modeled.
    pub fn wavelengths(&self) -> &[Float] {
        &self.wavelengths
    }

    /// Returns the optical axis directions at each surface vertex.
    pub fn axis_directions(&self) -> &[Vec3] {
        &self.axis_directions
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
    /// A system is rotationally symmetric if no physical surface has a tilt
    /// relative to the optical axis, i.e., the surface-tilt rotation equals
    /// the cursor rotation at every physical surface.
    pub fn is_rotationally_symmetric(placements: &[Placement]) -> bool {
        !placements.iter().any(|p| {
            // R_surf = surface_tilt × cursor = global_to_local · cursor_to_global
            let r_surf = p.rotation_matrix * p.cursor_rotation_matrix.transpose();
            !r_surf.approx_eq(&Mat3x3::identity(), 1e-10)
        })
    }

    /// Walks the cursor through the system, building placements and axis
    /// directions from pre-built surfaces and their rotations.
    ///
    /// `surfaces` and `rotations` must have the same length N.
    /// `gap_specs` must have length N - 1.
    fn build_placements_and_directions(
        surfaces: &[Box<dyn Surface>],
        rotations: &[Rotation3D],
        gap_specs: &[GapSpec],
    ) -> (Vec<Placement>, Vec<Vec3>) {
        let mut placements = Vec::new();
        let mut axis_directions = Vec::new();
        let mut cursor = Cursor::new(-gap_specs[0].thickness);

        // Surfaces 0 to N-2 (each paired with a gap that follows it).
        for ((surf, rotation), gap_spec) in
            surfaces.iter().zip(rotations.iter()).zip(gap_specs.iter())
        {
            axis_directions.push(cursor.forward());
            let placement = Placement::from_rotation(rotation, &cursor);

            // Flip the cursor upon reflection. Evaluate the normal at the
            // vertex (local origin = (0,0,0)) and transform to global frame.
            if let BoundaryType::Reflecting = surf.boundary_type() {
                let mut norm = surf.norm(Vec3::new(0.0, 0.0, 0.0));
                norm = (placement.inv_rotation_matrix * norm).normalize();
                cursor.reflect(&norm);
            }

            placements.push(placement);
            cursor.advance(gap_spec.thickness);
        }

        // Last surface — no gap after it.
        axis_directions.push(cursor.forward());
        placements.push(Placement::from_rotation(
            rotations.last().expect("at least one surface"),
            &cursor,
        ));

        (placements, axis_directions)
    }

    #[cfg(feature = "serde")]
    fn surf_specs_to_surfs(
        surf_specs: &[SurfaceSpec],
        gap_specs: &[GapSpec],
        registry: Option<&SurfaceRegistry>,
    ) -> Result<SurfsPlacementsDirs> {
        let surfaces: Vec<Box<dyn Surface>> = surf_specs
            .iter()
            .map(|s| surface_from_spec(s, registry))
            .collect::<Result<Vec<_>>>()?;
        let rotations: Vec<Rotation3D> = surf_specs.iter().map(rotation_from_spec).collect();
        let (placements, axis_directions) =
            Self::build_placements_and_directions(&surfaces, &rotations, gap_specs);
        Ok((surfaces, placements, axis_directions))
    }

    #[cfg(not(feature = "serde"))]
    fn surf_specs_to_surfs(
        surf_specs: &[SurfaceSpec],
        gap_specs: &[GapSpec],
    ) -> Result<SurfsPlacementsDirs> {
        let surfaces: Vec<Box<dyn Surface>> = surf_specs
            .iter()
            .map(surface_from_spec)
            .collect::<Result<Vec<_>>>()?;
        let rotations: Vec<Rotation3D> = surf_specs.iter().map(rotation_from_spec).collect();
        let (placements, axis_directions) =
            Self::build_placements_and_directions(&surfaces, &rotations, gap_specs);
        Ok((surfaces, placements, axis_directions))
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
        placements: &'a [Placement],
    ) -> Result<SequentialSubModelIter<'a>> {
        SequentialSubModelIter::new(surfaces, placements, &self.gaps)
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
        placements: &'b [Placement],
    ) -> Result<SequentialSubModelIter<'b>> {
        SequentialSubModelIter::new(surfaces, placements, self.gaps)
    }
}

impl<'a> SequentialSubModelIter<'a> {
    fn new(
        surfaces: &'a [Box<dyn Surface>],
        placements: &'a [Placement],
        gaps: &'a [Gap],
    ) -> Result<Self> {
        if surfaces.len() != gaps.len() + 1 {
            return Err(anyhow!(
                "The number of surfaces must be one more than the number of gaps in a forward sequential submodel."
            ));
        }

        Ok(Self {
            surfaces,
            placements,
            gaps,
            index: 0,
        })
    }

    pub fn try_reverse(self) -> Result<SequentialSubModelReverseIter<'a>> {
        SequentialSubModelReverseIter::new(self.surfaces, self.placements, self.gaps)
    }
}

impl<'a> Iterator for SequentialSubModelIter<'a> {
    type Item = Step<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let surf_idx = self.index + 1;
        if self.index == self.gaps.len() - 1 {
            // We are at the image space gap
            let result = Some(Step {
                gap_before: &self.gaps[self.index],
                surface: self.surfaces[surf_idx].as_ref(),
                gap_after: None,
                placement: &self.placements[surf_idx],
            });
            self.index += 1;
            result
        } else if self.index < self.gaps.len() {
            let result = Some(Step {
                gap_before: &self.gaps[self.index],
                surface: self.surfaces[surf_idx].as_ref(),
                gap_after: Some(&self.gaps[self.index + 1]),
                placement: &self.placements[surf_idx],
            });
            self.index += 1;
            result
        } else {
            None
        }
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
        placements: &'a [Placement],
        gaps: &'a [Gap],
    ) -> Result<Self> {
        // Note that this requirement is different than the forward iterator.
        if surfaces.len() != gaps.len() + 1 {
            return Err(anyhow!(
                "The number of surfaces must be one more than the number of gaps in a reversed sequential submodel."
            ));
        }

        Ok(Self {
            surfaces,
            placements,
            gaps,
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
            // We are somewhere in the middle of the system or at the object space gap.
            let result = Some(Step {
                gap_before: &self.gaps[forward_index],
                surface: self.surfaces[forward_index].as_ref(),
                gap_after: Some(&self.gaps[forward_index - 1]),
                placement: &self.placements[forward_index],
            });
            self.index += 1;
            result
        } else {
            None
        }
    }
}

/// Extract the tilt [`Rotation3D`] from a surface specification.
fn rotation_from_spec(spec: &SurfaceSpec) -> Rotation3D {
    match spec {
        SurfaceSpec::Conic { rotation, .. }
        | SurfaceSpec::Sphere { rotation, .. }
        | SurfaceSpec::Image { rotation }
        | SurfaceSpec::Probe { rotation }
        | SurfaceSpec::Iris { rotation, .. } => rotation.clone(),
        SurfaceSpec::Object => Rotation3D::None,
        #[cfg(feature = "serde")]
        SurfaceSpec::Custom { rotation, .. } => rotation.clone(),
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
            surf_type,
            ..
        } => Ok(Box::new(Conic::new(
            *semi_diameter,
            *radius_of_curvature,
            *conic_constant,
            *surf_type,
        ))),
        SurfaceSpec::Sphere {
            semi_diameter,
            radius_of_curvature,
            surf_type,
            ..
        } => Ok(Box::new(Sphere::new(
            *semi_diameter,
            *radius_of_curvature,
            *surf_type,
        ))),
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
            surf_type,
            ..
        } => Ok(Box::new(Conic::new(
            *semi_diameter,
            *radius_of_curvature,
            *conic_constant,
            *surf_type,
        ))),
        SurfaceSpec::Sphere {
            semi_diameter,
            radius_of_curvature,
            surf_type,
            ..
        } => Ok(Box::new(Sphere::new(
            *semi_diameter,
            *radius_of_curvature,
            *surf_type,
        ))),
        SurfaceSpec::Image { .. } => Ok(Box::new(Image::new())),
        SurfaceSpec::Object => Ok(Box::new(Object::new())),
        SurfaceSpec::Probe { .. } => Ok(Box::new(Probe::new())),
        SurfaceSpec::Iris { semi_diameter, .. } => Ok(Box::new(Iris::new(*semi_diameter))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        EulerAngles, Rotation3D, core::Float, core::surfaces::Sphere, n,
        specs::surfaces::BoundaryType,
    };

    // Helper: build a Placement for a surface with the given rotation, in an
    // identity cursor frame (cursor aligned with global axes, origin at (0,0,0)).
    fn placement_with_rotation(rotation: Rotation3D) -> Placement {
        let cursor_rotation_matrix = Mat3x3::identity();
        let rotation_matrix = rotation.rotation_matrix() * cursor_rotation_matrix;
        Placement::new(
            Vec3::new(0.0, 0.0, 0.0),
            0.0,
            rotation_matrix,
            cursor_rotation_matrix,
        )
    }

    #[test]
    fn projected_sd_untilted_surface() {
        let r = 10.0;
        let placement = placement_with_rotation(Rotation3D::None);
        let tol = 1e-12;
        let v_u = Vec3::new(0.0, 1.0, 0.0);
        let v_r = Vec3::new(1.0, 0.0, 0.0);
        assert!(
            (placement.projected_semi_diameter(r, v_u) - r).abs() < tol,
            "U axis: expected {r}, got {}",
            placement.projected_semi_diameter(r, v_u)
        );
        assert!(
            (placement.projected_semi_diameter(r, v_r) - r).abs() < tol,
            "R axis: expected {r}, got {}",
            placement.projected_semi_diameter(r, v_r)
        );
    }

    #[test]
    fn projected_sd_theta_tilted_mirror() {
        // 45° rotation about cursor-R; foreshortens only the U axis.
        let r = 10.0;
        let theta = 45.0_f64.to_radians();
        let placement = placement_with_rotation(Rotation3D::IntrinsicPassiveRUF(EulerAngles(
            theta, 0.0, 0.0,
        )));
        let tol = 1e-10;
        let v_u = Vec3::new(0.0, 1.0, 0.0);
        let v_r = Vec3::new(1.0, 0.0, 0.0);
        assert!(
            (placement.projected_semi_diameter(r, v_u) - r * theta.cos()).abs() < tol,
            "U axis: expected {}, got {}",
            r * theta.cos(),
            placement.projected_semi_diameter(r, v_u)
        );
        assert!(
            (placement.projected_semi_diameter(r, v_r) - r).abs() < tol,
            "R axis: expected {r}, got {}",
            placement.projected_semi_diameter(r, v_r)
        );
    }

    #[test]
    fn projected_sd_psi_tilted_surface() {
        // 30° rotation about cursor-U; foreshortens only the R axis.
        let r = 10.0;
        let psi = 30.0_f64.to_radians();
        let placement =
            placement_with_rotation(Rotation3D::IntrinsicPassiveRUF(EulerAngles(0.0, psi, 0.0)));
        let tol = 1e-10;
        let v_u = Vec3::new(0.0, 1.0, 0.0);
        let v_r = Vec3::new(1.0, 0.0, 0.0);
        assert!(
            (placement.projected_semi_diameter(r, v_r) - r * psi.cos()).abs() < tol,
            "R axis: expected {}, got {}",
            r * psi.cos(),
            placement.projected_semi_diameter(r, v_r)
        );
        assert!(
            (placement.projected_semi_diameter(r, v_u) - r).abs() < tol,
            "U axis: expected {r}, got {}",
            placement.projected_semi_diameter(r, v_u)
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
        let r = 12.7_f64;
        let expected_u = r * (30.0_f64.to_radians()).cos();
        let tol = 1e-10;
        let v_u = Vec3::new(0.0, 1.0, 0.0);
        let v_r = Vec3::new(1.0, 0.0, 0.0);

        // Surface indices: 0 = Object, 1 = Mirror 1, 2 = Mirror 2, 3 = Image
        for &mirror_idx in &[1usize, 2usize] {
            let sd = surfaces[mirror_idx].mask().semi_diameter();
            let placement = &placements[mirror_idx];
            assert!(
                (placement.projected_semi_diameter(sd, v_u) - expected_u).abs() < tol,
                "Mirror {mirror_idx} U: expected {expected_u}, got {}",
                placement.projected_semi_diameter(sd, v_u)
            );
            assert!(
                (placement.projected_semi_diameter(sd, v_r) - r).abs() < tol,
                "Mirror {mirror_idx} R: expected {r}, got {}",
                placement.projected_semi_diameter(sd, v_r)
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

        let vecs = propagate_tangential_vec(v_init, model.surfaces(), model.placements());

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
        // A system with identity rotations is rotationally symmetric.
        let id = Mat3x3::identity();
        let placements = vec![
            Placement::new(Vec3::new(0.0, 0.0, 0.0), 0.0, id, id),
            Placement::new(Vec3::new(0.0, 0.0, 0.0), 0.0, id, id),
        ];
        assert!(SequentialModel::is_rotationally_symmetric(&placements));

        // A system with tilted surfaces is not rotationally symmetric.
        use crate::examples::mirrors_figure_z;
        let air = n!(1.0);
        let wavelengths = [0.5876];
        let figure_z = mirrors_figure_z::sequential_model(air, &wavelengths);
        assert!(!SequentialModel::is_rotationally_symmetric(
            figure_z.placements()
        ));
    }

    #[test]
    fn test_first_physical_surface() {
        // Object(0), Probe(1), Sphere(2), Sphere(3), Image(4) — first physical is index
        // 2.
        let surfaces: Vec<Box<dyn Surface>> = vec![
            Box::new(Object::new()),
            Box::new(Probe::new()),
            Box::new(Sphere::new(1.0, 1.0, BoundaryType::Refracting)),
            Box::new(Sphere::new(1.0, 1.0, BoundaryType::Refracting)),
            Box::new(Image::new()),
        ];

        let result = first_physical_surface(&surfaces);
        assert_eq!(result, Some(2));
    }

    #[test]
    fn test_last_physical_surface() {
        // Object(0), Sphere(1), Sphere(2), Probe(3), Image(4) — last physical is index
        // 2.
        let surfaces: Vec<Box<dyn Surface>> = vec![
            Box::new(Object::new()),
            Box::new(Sphere::new(1.0, 1.0, BoundaryType::Refracting)),
            Box::new(Sphere::new(1.0, 1.0, BoundaryType::Refracting)),
            Box::new(Probe::new()),
            Box::new(Image::new()),
        ];

        let result = last_physical_surface(&surfaces);
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
        let p = Placement::new(Vec3::new(0.0, 0.0, Float::INFINITY), 0.0, id, id);
        assert!(p.is_infinite());

        // y-coordinate infinite
        let p = Placement::new(Vec3::new(0.0, Float::INFINITY, 0.0), 0.0, id, id);
        assert!(p.is_infinite());

        // x-coordinate infinite
        let p = Placement::new(Vec3::new(Float::INFINITY, 0.0, 0.0), 0.0, id, id);
        assert!(p.is_infinite());

        // finite
        let p = Placement::new(Vec3::new(0.0, 0.0, 0.0), 0.0, id, id);
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
    fn placement_axis_direction_straight_system() {
        use crate::examples::convexplano_lens;
        use approx::assert_abs_diff_eq;
        let model = convexplano_lens::sequential_model(n!(1.0), n!(1.515), &[0.5876]);
        for placement in model.placements() {
            let axis = placement.axis_direction();
            assert_abs_diff_eq!(axis.x(), 0.0, epsilon = 1e-12);
            assert_abs_diff_eq!(axis.y(), 0.0, epsilon = 1e-12);
            assert_abs_diff_eq!(axis.z(), 1.0, epsilon = 1e-12);
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
                surf_type: BoundaryType::Refracting,
                rotation: Rotation3D::None,
            },
            SurfaceSpec::Probe {
                rotation: Rotation3D::None,
            },
            SurfaceSpec::Iris {
                semi_diameter: 5.0,
                rotation: Rotation3D::None,
            },
            SurfaceSpec::Image {
                rotation: Rotation3D::None,
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
}
