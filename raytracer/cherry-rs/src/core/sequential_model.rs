/// Data types for modeling sequential ray tracing systems.
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::Range;

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize, Serializer};

use crate::core::{
    Float,
    math::{mat3::Mat3, vec3::Vec3},
    reference_frames::Cursor,
    refractive_index::RefractiveIndex,
};
use crate::specs::{
    gaps::GapSpec,
    surfaces::{SurfaceSpec, SurfaceType},
};

/// The transverse direction along which system properties will be computed with
/// respect to the reference frame of the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Axis {
    X,
    Y,
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
    surfaces: Vec<Surface>,
    submodels: HashMap<SubModelID, SequentialSubModelBase>,
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
    fn try_iter<'a>(&'a self, surfaces: &'a [Surface]) -> Result<SequentialSubModelIter<'a>>;

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

/// A unique identifier for a submodel.
///
/// The first element is the index of the wavelength in the system's list of
/// wavelengths. The second element is the transverse axis along which the model
/// is computed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubModelID(pub usize, pub Axis);

/// An iterator over the surfaces and gaps in a submodel.
///
/// Most operations in sequential modeling involve use of this iterator.
pub struct SequentialSubModelIter<'a> {
    surfaces: &'a [Surface],
    gaps: &'a [Gap],
    index: usize,
}

/// A reverse iterator over the surfaces and gaps in a submodel.
pub struct SequentialSubModelReverseIter<'a> {
    surfaces: &'a [Surface],
    gaps: &'a [Gap],
    index: usize,
}

/// A single ray tracing step in a sequential system.
///
/// See the documentation for
/// [SequentialSubModel](trait@SequentialSubModel) for more information.
pub type Step<'a> = (&'a Gap, &'a Surface, Option<&'a Gap>);

#[derive(Debug)]
pub enum Surface {
    Conic(Conic),
    Image(Image),
    Object(Object),
    Probe(Probe),
    Stop(Stop),
    //Toric(Toric),
}

#[derive(Debug)]
pub struct Conic {
    pos: Vec3,
    rotation_matrix: Mat3,
    semi_diameter: Float,
    radius_of_curvature: Float,
    conic_constant: Float,
    surface_type: SurfaceType,
}

#[derive(Debug)]
pub struct Image {
    pos: Vec3,
    rotation_matrix: Mat3,
}

#[derive(Debug)]
pub struct Object {
    pos: Vec3,
    rotation_matrix: Mat3,
}

/// A surface without any effect on rays that is used to measure intersections.
#[derive(Debug)]
pub struct Probe {
    pos: Vec3,
    rotation_matrix: Mat3,
}

#[derive(Debug)]
pub struct Stop {
    pos: Vec3,
    rotation_matrix: Mat3,
    semi_diameter: Float,
}

// TODO: Implement Toric surfaces
//#[derive(Debug)]
//pub struct Toric {
//    pos: Vec3,
//    rotation_matrix: Mat3,
//    semi_diameter: Float,
//    radius_of_curvature_y: Float,
//    radius_of_curvature_x: Float,
//    conic_constant: Float,
//    surface_type: SurfaceType,
//}

/// Returns the index of the first physical surface in the system.
/// This is the first surface that is not an object, image, or probe surface.
/// If no such surface exists, then the function returns None.
pub(crate) fn first_physical_surface(surfaces: &[Surface]) -> Option<usize> {
    surfaces
        .iter()
        .position(|surf| matches!(surf, Surface::Conic(_) | Surface::Stop(_)))
}

/// Returns the index of the last physical surface in the system.
/// This is the last surface that is not an object, image, or probe surface.
/// If no such surface exists, then the function returns None.
pub fn last_physical_surface(surfaces: &[Surface]) -> Option<usize> {
    surfaces
        .iter()
        .rposition(|surf| matches!(surf, Surface::Conic(_) | Surface::Stop(_)))
}

/// Returns the id of a surface in a reversed system.
pub fn reversed_surface_id(surfaces: &[Surface], surf_id: usize) -> usize {
    // Reversed IDs are ray starts, then image plane, then surfaces
    surfaces.len() - surf_id - 1
}

impl Conic {
    pub fn sag_norm(&self, pos: Vec3) -> (Float, Vec3) {
        if self.radius_of_curvature.is_infinite() {
            return (0.0, Vec3::new(0.0, 0.0, 1.0));
        }

        // Convert to polar coordinates in x, y plane
        let r = (pos.x().powi(2) + pos.y().powi(2)).sqrt();
        let theta = pos.y().atan2(pos.x());

        // Compute surface sag
        let a = r.powi(2) / self.radius_of_curvature;
        let sag =
            a / (1.0 + (1.0 - (1.0 + self.conic_constant) * a / self.radius_of_curvature).sqrt());

        // Compute surface normal
        let denom = (self.radius_of_curvature.powi(4)
            - (1.0 + self.conic_constant) * (r * self.radius_of_curvature).powi(2))
        .sqrt();
        let dfdx = -r * self.radius_of_curvature * theta.cos() / denom;
        let dfdy = -r * self.radius_of_curvature * theta.sin() / denom;
        let dfdz = 1.0 as Float;
        let norm = Vec3::new(dfdx, dfdy, dfdz).normalize();

        (sag, norm)
    }
}

impl Gap {
    pub(crate) fn try_from_spec(spec: &GapSpec, wavelength: Float) -> Result<Self> {
        let thickness = spec.thickness;
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
    ///
    /// # Returns
    /// A new sequential model.
    pub fn new(
        gap_specs: &[GapSpec],
        surface_specs: &[SurfaceSpec],
        wavelengths: &[Float],
    ) -> Result<Self> {
        Self::validate_specs(gap_specs, wavelengths)?;

        let surfaces = Self::surf_specs_to_surfs(surface_specs, gap_specs);

        let model_ids: Vec<SubModelID> = Self::calc_model_ids(&surfaces, wavelengths);
        let mut models: HashMap<SubModelID, SequentialSubModelBase> = HashMap::new();
        for model_id in model_ids.iter() {
            let wavelength = wavelengths[model_id.0];
            let gaps = Self::gap_specs_to_gaps(gap_specs, wavelength)?;
            let model = SequentialSubModelBase::new(gaps);
            models.insert(*model_id, model);
        }

        Ok(Self {
            surfaces,
            submodels: models,
            wavelengths: wavelengths.to_vec(),
        })
    }

    /// Returns the axes along which the system is modeled.
    pub fn axes(&self) -> Vec<Axis> {
        // Loop over submodel IDs and extract all axes
        let mut axes = Vec::new();
        for id in self.submodels.keys() {
            // Avoid duplicates just in case
            if axes.contains(&id.1) {
                continue;
            }

            axes.push(id.1);
        }

        axes
    }

    /// Returns the largest semi-diameter of any surface in the system.
    ///
    /// This ignores surfaces without any size, such as object, probe, and image
    /// surfaces.
    pub fn largest_semi_diameter(&self) -> Float {
        self.surfaces
            .iter()
            .filter_map(|surf| match surf {
                Surface::Conic(conic) => Some(conic.semi_diameter),
                //Surface::Toric(toric) => Some(toric.semi_diameter),
                Surface::Stop(stop) => Some(stop.semi_diameter),
                _ => None,
            })
            .fold(0.0, |acc, x| acc.max(x))
    }

    /// Returns the surfaces in the system.
    pub fn surfaces(&self) -> &[Surface] {
        &self.surfaces
    }

    /// Returns the submodels in the system.
    pub fn submodels(&self) -> &HashMap<SubModelID, impl SequentialSubModel + use<>> {
        &self.submodels
    }

    /// Returns the wavelengths at which the system is modeled.
    pub fn wavelengths(&self) -> &[Float] {
        &self.wavelengths
    }

    /// Computes the unique IDs for each paraxial model.
    fn calc_model_ids(surfaces: &[Surface], wavelengths: &[Float]) -> Vec<SubModelID> {
        let mut ids = Vec::new();

        let axes: Vec<Axis> = if Self::is_rotationally_symmetric(surfaces) {
            vec![Axis::Y]
        } else {
            vec![Axis::X, Axis::Y]
        };

        for (idx, _wavelength) in wavelengths.iter().enumerate() {
            for axis in axes.iter() {
                let id = SubModelID(idx, *axis);
                ids.push(id);
            }
        }
        ids
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
    fn is_rotationally_symmetric(_surfaces: &[Surface]) -> bool {
        // Return false if any toric surface is present in the system.
        //!surfaces
        //    .iter()
        //    .any(|surf| matches!(surf, Surface::Toric(_)))
        true
    }

    fn surf_specs_to_surfs(surf_specs: &[SurfaceSpec], gap_specs: &[GapSpec]) -> Vec<Surface> {
        let mut surfaces = Vec::new();

        // The first surface is an object surface.
        // The second surface is at z=0 by convention.
        let mut cursor = Cursor::new(-gap_specs[0].thickness);

        // Create surfaces 0 to n-1
        for (surf_spec, gap_spec) in surf_specs.iter().zip(gap_specs.iter()) {
            let surf = Surface::from_spec(surf_spec, cursor.pos());

            // Flip the cursor upon reflection
            if let SurfaceType::Reflecting = surf.surface_type() {
                cursor.invert();
            }

            // Add the surface to the list and advance the cursor
            surfaces.push(surf);
            cursor.advance(gap_spec.thickness);
        }

        // Add the last surface
        surfaces.push(Surface::from_spec(
            surf_specs
                .last()
                .expect("There should always be one last surface."),
            cursor.pos(),
        ));
        surfaces
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

    fn try_iter<'a>(&'a self, surfaces: &'a [Surface]) -> Result<SequentialSubModelIter<'a>> {
        SequentialSubModelIter::new(surfaces, &self.gaps)
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

    fn try_iter<'b>(&'b self, surfaces: &'b [Surface]) -> Result<SequentialSubModelIter<'b>> {
        SequentialSubModelIter::new(surfaces, self.gaps)
    }
}

impl Serialize for SubModelID {
    // Serialize as a string like "0:Y" because tuples as map keys are difficult to
    // work with in languages like Javascript.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as a string like "0:Y"
        let key = format!(
            "{}:{}",
            self.0,
            match self.1 {
                Axis::X => "X",
                Axis::Y => "Y",
            }
        );
        serializer.serialize_str(&key)
    }
}

impl<'a> SequentialSubModelIter<'a> {
    fn new(surfaces: &'a [Surface], gaps: &'a [Gap]) -> Result<Self> {
        if surfaces.len() != gaps.len() + 1 {
            return Err(anyhow!(
                "The number of surfaces must be one more than the number of gaps in a forward sequential submodel."
            ));
        }

        Ok(Self {
            surfaces,
            gaps,
            index: 0,
        })
    }

    pub fn try_reverse(self) -> Result<SequentialSubModelReverseIter<'a>> {
        SequentialSubModelReverseIter::new(self.surfaces, self.gaps)
    }
}

impl<'a> Iterator for SequentialSubModelIter<'a> {
    type Item = Step<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.gaps.len() - 1 {
            // We are at the image space gap
            let result = Some((&self.gaps[self.index], &self.surfaces[self.index + 1], None));
            self.index += 1;
            result
        } else if self.index < self.gaps.len() {
            let result = Some((
                &self.gaps[self.index],
                &self.surfaces[self.index + 1],
                Some(&self.gaps[self.index + 1]),
            ));
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
    fn new(surfaces: &'a [Surface], gaps: &'a [Gap]) -> Result<Self> {
        // Note that this requirement is different than the forward iterator.
        if surfaces.len() != gaps.len() + 1 {
            return Err(anyhow!(
                "The number of surfaces must be one more than the number of gaps in a reversed sequential submodel."
            ));
        }

        Ok(Self {
            surfaces,
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
            let result = Some((
                &self.gaps[forward_index],
                &self.surfaces[forward_index],
                Some(&self.gaps[forward_index - 1]),
            ));
            self.index += 1;
            result
        } else {
            None
        }
    }
}

impl Surface {
    /// Returns the z-coordinate of the surface's vertex.
    pub fn z(&self) -> Float {
        self.pos().z()
    }

    pub(crate) fn from_spec(spec: &SurfaceSpec, pos: Vec3) -> Self {
        // No rotation for the moment
        let euler_angles = Vec3::new(0.0, 0.0, 0.0);
        let rotation_matrix =
            Mat3::from_euler_angles(euler_angles.x(), euler_angles.y(), euler_angles.z());

        match spec {
            SurfaceSpec::Conic {
                semi_diameter,
                radius_of_curvature,
                conic_constant,
                surf_type,
                rotation: _,
            } => Self::Conic(Conic {
                pos,
                rotation_matrix,
                semi_diameter: *semi_diameter,
                radius_of_curvature: *radius_of_curvature,
                conic_constant: *conic_constant,
                surface_type: *surf_type,
            }),
            SurfaceSpec::Image { rotation: _ } => Self::Image(Image {
                pos,
                rotation_matrix,
            }),
            SurfaceSpec::Object => Self::Object(Object {
                pos,
                rotation_matrix,
            }),
            SurfaceSpec::Probe { rotation: _ } => Self::Probe(Probe {
                pos,
                rotation_matrix,
            }),
            SurfaceSpec::Stop {
                semi_diameter,
                rotation: _,
            } => Self::Stop(Stop {
                pos,
                rotation_matrix,
                semi_diameter: *semi_diameter,
            }),
            // SurfaceSpec::Toric {
            //     semi_diameter,
            //     radius_of_curvature_vert,
            //     radius_of_curvature_horz,
            //     conic_constant,
            //     surf_type,
            // } => Self::Toric(Toric {
            //     pos,
            //     rotation_matrix,
            //     semi_diameter: *semi_diameter,
            //     radius_of_curvature_y: *radius_of_curvature_vert,
            //     radius_of_curvature_x: *radius_of_curvature_horz,
            //     conic_constant: *conic_constant,
            //     surface_type: *surf_type,
            // }),
        }
    }

    /// Determines whether a transverse point is outside the clear aperture of
    /// the surface.
    ///
    /// The axial z-position is ignored.
    pub(crate) fn outside_clear_aperture(&self, pos: Vec3) -> bool {
        let r_transv = pos.x() * pos.x() + pos.y() * pos.y();
        let r_max = self.semi_diameter();

        r_transv > r_max * r_max
    }

    pub(crate) fn roc(&self, axis: &Axis) -> Float {
        match axis {
            Axis::X => self.rocx(),
            Axis::Y => self.rocy(),
        }
    }

    /// The radius of curvature in the horizontal direction.
    fn rocx(&self) -> Float {
        match self {
            Self::Conic(conic) => conic.radius_of_curvature,
            //Self::Toric(toric) => toric.radius_of_curvature_x,
            _ => Float::INFINITY,
        }
    }

    /// The radius of curvature in the vertical direction.
    fn rocy(&self) -> Float {
        match self {
            Self::Conic(conic) => conic.radius_of_curvature,
            //Self::Toric(toric) => toric.radius_of_curvature_y,
            _ => Float::INFINITY,
        }
    }

    /// Returns the rotation matrix of the surface into the local coordinate
    /// system.
    pub(crate) fn rot_mat(&self) -> Mat3 {
        match self {
            Self::Conic(conic) => conic.rotation_matrix,
            Self::Image(image) => image.rotation_matrix,
            Self::Object(object) => object.rotation_matrix,
            Self::Probe(probe) => probe.rotation_matrix,
            Self::Stop(stop) => stop.rotation_matrix,
            //Self::Toric(toric) => toric.rotation_matrix,
        }
    }

    /// Returns the position of the surface in the global coordinate system.
    pub(crate) fn pos(&self) -> Vec3 {
        match self {
            Self::Conic(conic) => conic.pos,
            Self::Image(image) => image.pos,
            Self::Object(object) => object.pos,
            Self::Probe(probe) => probe.pos,
            Self::Stop(stop) => stop.pos,
            //Self::Toric(toric) => toric.pos,
        }
    }

    /// Returns the surface sag and normal vector on the surface at a given
    /// position.
    ///
    /// The position is given in the local coordinate system of the surface.
    pub(crate) fn sag_norm(&self, pos: Vec3) -> (Float, Vec3) {
        match self {
            Self::Conic(conic) => conic.sag_norm(pos),
            // Flat surfaces
            Self::Image(_) | Self::Object(_) | Self::Probe(_) | Self::Stop(_) => {
                (0.0, Vec3::new(0.0, 0.0, 1.0))
            } //Self::Toric(_) => unimplemented!(),
        }
    }

    pub(crate) fn semi_diameter(&self) -> Float {
        match self {
            Self::Conic(conic) => conic.semi_diameter,
            //Self::Toric(toric) => toric.semi_diameter,
            Self::Stop(stop) => stop.semi_diameter,
            _ => Float::INFINITY,
        }
    }

    pub(crate) fn surface_type(&self) -> SurfaceType {
        match self {
            Self::Conic(conic) => conic.surface_type,
            //Self::Toric(toric) => toric.surface_type,
            _ => SurfaceType::NoOp,
        }
    }
}

impl Display for Surface {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::Conic(_) => write!(f, "Conic"),
            Self::Image(_) => write!(f, "Image"),
            Self::Object(_) => write!(f, "Object"),
            Self::Probe(_) => write!(f, "Probe"),
            Self::Stop(_) => write!(f, "Stop"),
            //Self::Toric(toric) => write!(f, "Toric surface at z = {}", toric.pos.z()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        core::{Float, math::mat3::Mat3},
        examples::convexplano_lens::sequential_model,
        n,
    };

    #[test]
    fn is_rotationally_symmetric() {
        let surfaces = vec![
            Surface::Conic(Conic {
                pos: Vec3::new(0.0, 0.0, 0.0),
                rotation_matrix: Mat3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
                semi_diameter: 1.0,
                radius_of_curvature: 1.0,
                conic_constant: 0.0,
                surface_type: SurfaceType::Refracting,
            }),
            Surface::Conic(Conic {
                pos: Vec3::new(0.0, 0.0, 0.0),
                rotation_matrix: Mat3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
                semi_diameter: 1.0,
                radius_of_curvature: 1.0,
                conic_constant: 0.0,
                surface_type: SurfaceType::Refracting,
            }),
        ];
        assert!(SequentialModel::is_rotationally_symmetric(&surfaces));

        // let surfaces = vec![
        //     Surface::Conic(Conic {
        //         pos: Vec3::new(0.0, 0.0, 0.0),
        //         rotation_matrix: Mat3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0,
        // 0.0, 1.0),         semi_diameter: 1.0,
        //         radius_of_curvature: 1.0,
        //         conic_constant: 0.0,
        //         surface_type: SurfaceType::Refracting,
        //     }),
        //     Surface::Toric(Toric {
        //         pos: Vec3::new(0.0, 0.0, 0.0),
        //         rotation_matrix: Mat3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0,
        // 0.0, 1.0),         semi_diameter: 1.0,
        //         radius_of_curvature_y: 1.0,
        //         radius_of_curvature_x: 1.0,
        //         conic_constant: 0.0,
        //         surface_type: SurfaceType::Refracting,
        //     }),
        // ];
        // assert!(!SequentialModel::is_rotationally_symmetric(&surfaces));
    }

    #[test]
    fn test_calc_model_ids() {
        let air = n!(1.0);
        let nbk7 = n!(1.515);
        let wavelengths: [Float; 2] = [0.4, 0.6];
        let sequential_model = sequential_model(air, nbk7, &wavelengths);
        let surfaces = sequential_model.surfaces();

        let model_ids = SequentialModel::calc_model_ids(surfaces, &wavelengths);

        assert_eq!(model_ids.len(), 2); // Two wavelengths, rotationally
        // symmetric
    }

    #[test]
    fn test_axes() {
        let air = n!(1.0);
        let nbk7 = n!(1.515);
        let wavelengths: [Float; 1] = [0.5876];
        let sequential_model = sequential_model(air, nbk7, &wavelengths);
        let axes = sequential_model.axes();

        assert_eq!(axes.len(), 1); // Rotationally symmetric
        assert_eq!(axes[0], Axis::Y);
    }

    #[test]
    fn test_first_physical_surface() {
        let surfaces = vec![
            Surface::Object(Object {
                pos: Vec3::new(0.0, 0.0, 0.0),
                rotation_matrix: Mat3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
            }),
            Surface::Probe(Probe {
                pos: Vec3::new(0.0, 0.0, 0.0),
                rotation_matrix: Mat3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
            }),
            Surface::Conic(Conic {
                pos: Vec3::new(0.0, 0.0, 0.0),
                rotation_matrix: Mat3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
                semi_diameter: 1.0,
                radius_of_curvature: 1.0,
                conic_constant: 0.0,
                surface_type: SurfaceType::Refracting,
            }),
            Surface::Conic(Conic {
                pos: Vec3::new(0.0, 0.0, 0.0),
                rotation_matrix: Mat3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
                semi_diameter: 1.0,
                radius_of_curvature: 1.0,
                conic_constant: 0.0,
                surface_type: SurfaceType::Refracting,
            }),
            Surface::Image(Image {
                pos: Vec3::new(0.0, 0.0, 0.0),
                rotation_matrix: Mat3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
            }),
        ];

        let result = first_physical_surface(&surfaces);
        assert_eq!(result, Some(2));
    }

    #[test]
    fn test_last_physical_surface() {
        let surfaces = vec![
            Surface::Object(Object {
                pos: Vec3::new(0.0, 0.0, 0.0),
                rotation_matrix: Mat3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
            }),
            Surface::Conic(Conic {
                pos: Vec3::new(0.0, 0.0, 0.0),
                rotation_matrix: Mat3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
                semi_diameter: 1.0,
                radius_of_curvature: 1.0,
                conic_constant: 0.0,
                surface_type: SurfaceType::Refracting,
            }),
            Surface::Conic(Conic {
                pos: Vec3::new(0.0, 0.0, 0.0),
                rotation_matrix: Mat3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
                semi_diameter: 1.0,
                radius_of_curvature: 1.0,
                conic_constant: 0.0,
                surface_type: SurfaceType::Refracting,
            }),
            Surface::Probe(Probe {
                pos: Vec3::new(0.0, 0.0, 0.0),
                rotation_matrix: Mat3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
            }),
            Surface::Image(Image {
                pos: Vec3::new(0.0, 0.0, 0.0),
                rotation_matrix: Mat3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
            }),
        ];

        let result = last_physical_surface(&surfaces);
        assert_eq!(result, Some(2));
    }

    #[test]
    fn test_reversed_surface_id() {
        let surfaces = vec![
            Surface::Object(Object {
                pos: Vec3::new(0.0, 0.0, 0.0),
                rotation_matrix: Mat3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
            }),
            Surface::Conic(Conic {
                pos: Vec3::new(0.0, 0.0, 0.0),
                rotation_matrix: Mat3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
                semi_diameter: 1.0,
                radius_of_curvature: 1.0,
                conic_constant: 0.0,
                surface_type: SurfaceType::Refracting,
            }),
            Surface::Conic(Conic {
                pos: Vec3::new(0.0, 0.0, 0.0),
                rotation_matrix: Mat3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
                semi_diameter: 1.0,
                radius_of_curvature: 1.0,
                conic_constant: 0.0,
                surface_type: SurfaceType::Refracting,
            }),
            Surface::Probe(Probe {
                pos: Vec3::new(0.0, 0.0, 0.0),
                rotation_matrix: Mat3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
            }),
            Surface::Image(Image {
                pos: Vec3::new(0.0, 0.0, 0.0),
                rotation_matrix: Mat3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
            }),
        ];

        let result = reversed_surface_id(&surfaces, 2);
        assert_eq!(result, 2);

        let result = reversed_surface_id(&surfaces, 1);
        assert_eq!(result, 3);
    }
}
