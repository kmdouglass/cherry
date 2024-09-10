/// Data types for modeling sequential ray tracing systems.
use std::collections::HashMap;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::core::{
    math::{mat3::Mat3, vec3::Vec3},
    Cursor, Float, RefractiveIndex,
};
use crate::specs::{
    gaps::GapSpec,
    surfaces::{SurfaceSpec, SurfaceType},
};

/// The transverse direction along which system properties will be computed with
/// respect to the current reference frame of the cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Axis {
    X,
    Y,
}

#[derive(Debug)]
pub(crate) struct Gap {
    pub thickness: Float,
    pub refractive_index: RefractiveIndex,
}

/// A collection of submodels for sequential ray tracing.
#[derive(Debug)]
pub struct SequentialModel {
    surfaces: Vec<Surface>,
    submodels: HashMap<SubModelID, SequentialSubModel>,
}

#[derive(Debug)]
pub struct SequentialSubModel {
    gaps: Vec<Gap>,
}

/// A unique identifier for a submodel.
///
/// The first element is the index of the wavelength in the system's list of
/// wavelengths. The second element is the transverse axis along which the model
/// is computed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubModelID(pub Option<usize>, pub Axis);

/// An iterator over the surfaces and gaps in a submodel.
///
/// Most operations in sequential modeling involve use of this iterator.
pub(crate) struct SequentialSubModelIter<'a> {
    surfaces: &'a [Surface],
    gaps: &'a [Gap],
    index: usize,
}

/// A reverse iterator over the surfaces and gaps in a submodel.
pub(crate) struct SequentialSubModelReverseIter<'a> {
    surfaces: &'a [Surface],
    gaps: &'a [Gap],
    index: usize,
}

/// A single ray tracing step in a sequential system.
pub(crate) type Step<'a> = (&'a Gap, &'a Surface, Option<&'a Gap>);

#[derive(Debug)]
pub enum Surface {
    Conic(Conic),
    Image(Image),
    Object(Object),
    Probe(Probe),
    Stop(Stop),
    Toric(Toric),
}

#[derive(Debug)]
pub(crate) struct Conic {
    pos: Vec3,
    rotation_matrix: Mat3,
    semi_diameter: Float,
    radius_of_curvature: Float,
    conic_constant: Float,
    surface_type: SurfaceType,
}

#[derive(Debug)]
pub(crate) struct Image {
    pos: Vec3,
    rotation_matrix: Mat3,
}

#[derive(Debug)]
pub(crate) struct Object {
    pos: Vec3,
    rotation_matrix: Mat3,
}

/// A surface without any effect on rays that is used to measure intersections.
#[derive(Debug)]
pub(crate) struct Probe {
    pos: Vec3,
    rotation_matrix: Mat3,
}

#[derive(Debug)]
pub(crate) struct Stop {
    pos: Vec3,
    rotation_matrix: Mat3,
    semi_diameter: Float,
}

#[derive(Debug)]
pub(crate) struct Toric {
    pos: Vec3,
    rotation_matrix: Mat3,
    semi_diameter: Float,
    radius_of_curvature_y: Float,
    radius_of_curvature_x: Float,
    conic_constant: Float,
    surface_type: SurfaceType,
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
    pub(crate) fn try_from_spec(spec: &GapSpec, wavelength: Option<Float>) -> Result<Self> {
        let thickness = spec.thickness;
        let refractive_index = RefractiveIndex::try_from_spec(&spec.refractive_index, wavelength)?;
        Ok(Self {
            thickness,
            refractive_index,
        })
    }
}

impl SequentialModel {
    /// Creates a new sequential model of an optical system.
    pub fn new(
        gap_specs: &[GapSpec],
        surface_specs: &[SurfaceSpec],
        wavelengths: &[Float],
    ) -> Result<Self> {
        Self::validate_specs(gap_specs, surface_specs, wavelengths)?;

        let surfaces = Self::surf_specs_to_surfs(&surface_specs, &gap_specs);

        let model_ids: Vec<SubModelID> = Self::calc_model_ids(&surfaces, &wavelengths);
        let mut models: HashMap<SubModelID, SequentialSubModel> = HashMap::new();
        for model_id in model_ids.iter() {
            let wavelength = match model_id.0 {
                Some(idx) => Some(wavelengths[idx]),
                None => None,
            };
            let gaps = Self::gap_specs_to_gaps(&gap_specs, wavelength)?;
            let model = SequentialSubModel::new(gaps);
            models.insert(*model_id, model);
        }

        Ok(Self {
            surfaces,
            submodels: models,
        })
    }

    pub fn surfaces(&self) -> &[Surface] {
        &self.surfaces
    }

    pub fn submodels(&self) -> &HashMap<SubModelID, SequentialSubModel> {
        &self.submodels
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
                Surface::Toric(toric) => Some(toric.semi_diameter),
                Surface::Stop(stop) => Some(stop.semi_diameter),
                _ => None,
            })
            .fold(0.0, |acc, x| acc.max(x))
    }

    /// Computes the unique IDs for each paraxial model.
    fn calc_model_ids(surfaces: &[Surface], wavelengths: &[Float]) -> Vec<SubModelID> {
        let mut ids = Vec::new();
        if wavelengths.is_empty() && Self::is_rotationally_symmetric(surfaces) {
            ids.push(SubModelID(None, Axis::Y));
            return ids;
        } else if wavelengths.is_empty() {
            ids.push(SubModelID(None, Axis::X));
            ids.push(SubModelID(None, Axis::Y));
            return ids;
        }

        let axes: Vec<Axis> = if Self::is_rotationally_symmetric(surfaces) {
            vec![Axis::Y]
        } else {
            vec![Axis::X, Axis::Y]
        };

        for (idx, _wavelength) in wavelengths.iter().enumerate() {
            for axis in axes.iter() {
                let id = SubModelID(Some(idx), *axis);
                ids.push(id);
            }
        }
        ids
    }

    fn gap_specs_to_gaps(gap_specs: &[GapSpec], wavelength: Option<Float>) -> Result<Vec<Gap>> {
        let mut gaps = Vec::new();
        for gap_spec in gap_specs.iter() {
            let gap = Gap::try_from_spec(gap_spec, wavelength)?;
            gaps.push(gap);
        }
        Ok(gaps)
    }

    /// Returns true if the system is rotationally symmetric about the optical
    /// axis.
    fn is_rotationally_symmetric(surfaces: &[Surface]) -> bool {
        // Return false if any toric surface is present in the system.
        !surfaces
            .iter()
            .any(|surf| matches!(surf, Surface::Toric(_)))
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
            &surf_specs
                .last()
                .expect("There should always be one last surface."),
            cursor.pos(),
        ));
        surfaces
    }

    fn validate_gaps(gaps: &[GapSpec], wavelengths: &[Float]) -> Result<()> {
        if gaps.is_empty() {
            return Err(anyhow!("The system must have at least one gap."));
        }

        // If no wavelengths are specified, then the gaps must explicitly specify the
        // refractive index.
        if wavelengths.is_empty() {
            for gap in gaps.iter() {
                if gap.refractive_index.depends_on_wavelength() {
                    return Err(anyhow!(
                        "The refractive index of the gap must be a constant when no wavelengths are provided."
                    ));
                }
            }
        }
        Ok(())
    }

    fn validate_specs(
        gaps: &[GapSpec],
        surfaces: &[SurfaceSpec],
        wavelengths: &[Float],
    ) -> Result<()> {
        Self::validate_gaps(gaps, wavelengths)?;
        Ok(())
    }
}

impl SequentialSubModel {
    pub(crate) fn new(gaps: Vec<Gap>) -> Self {
        Self { gaps }
    }

    pub(crate) fn gaps(&self) -> &[Gap] {
        &self.gaps
    }

    pub(crate) fn is_obj_at_inf(&self) -> bool {
        self.gaps
            .first()
            .expect("There must be at least one gap in a sequential submodel.")
            .thickness
            .is_infinite()
    }

    pub(crate) fn iter<'a>(&'a self, surfaces: &'a [Surface]) -> SequentialSubModelIter<'a> {
        SequentialSubModelIter::new(surfaces, &self.gaps)
    }
}

impl<'a> SequentialSubModelIter<'a> {
    fn new(surfaces: &'a [Surface], gaps: &'a [Gap]) -> Self {
        Self {
            surfaces,
            gaps,
            index: 0,
        }
    }

    pub fn reverse(self) -> SequentialSubModelReverseIter<'a> {
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

impl<'a> ExactSizeIterator for SequentialSubModelIter<'a> {
    fn len(&self) -> usize {
        self.gaps.len()
    }
}

impl<'a> SequentialSubModelReverseIter<'a> {
    fn new(surfaces: &'a [Surface], gaps: &'a [Gap]) -> Self {
        Self {
            surfaces,
            gaps,
            // We will never iterate from the image space surface in reverse.
            index: 1,
        }
    }
}

impl<'a> Iterator for SequentialSubModelReverseIter<'a> {
    type Item = Step<'a>;

    fn next(&mut self) -> Option<Self::Item> {
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
            } => Self::Conic(Conic {
                pos,
                rotation_matrix,
                semi_diameter: *semi_diameter,
                radius_of_curvature: *radius_of_curvature,
                conic_constant: *conic_constant,
                surface_type: *surf_type,
            }),
            SurfaceSpec::Image => Self::Image(Image {
                pos,
                rotation_matrix,
            }),
            SurfaceSpec::Object => Self::Object(Object {
                pos,
                rotation_matrix,
            }),
            SurfaceSpec::Probe => Self::Probe(Probe {
                pos,
                rotation_matrix,
            }),
            SurfaceSpec::Stop { semi_diameter } => Self::Stop(Stop {
                pos,
                rotation_matrix,
                semi_diameter: *semi_diameter,
            }),
            SurfaceSpec::Toric {
                semi_diameter,
                radius_of_curvature_vert,
                radius_of_curvature_horz,
                conic_constant,
                surf_type,
            } => Self::Toric(Toric {
                pos,
                rotation_matrix,
                semi_diameter: *semi_diameter,
                radius_of_curvature_y: *radius_of_curvature_vert,
                radius_of_curvature_x: *radius_of_curvature_horz,
                conic_constant: *conic_constant,
                surface_type: *surf_type,
            }),
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

    pub(crate) fn roc(&self, axis: Axis) -> Float {
        match axis {
            Axis::X => self.rocx(),
            Axis::Y => self.rocy(),
        }
    }

    /// The radius of curvature in the horizontal direction.
    fn rocx(&self) -> Float {
        match self {
            Self::Conic(conic) => conic.radius_of_curvature,
            Self::Toric(toric) => toric.radius_of_curvature_x,
            _ => Float::INFINITY,
        }
    }

    /// The radius of curvature in the vertical direction.
    fn rocy(&self) -> Float {
        match self {
            Self::Conic(conic) => conic.radius_of_curvature,
            Self::Toric(toric) => toric.radius_of_curvature_y,
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
            Self::Toric(toric) => toric.rotation_matrix,
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
            Self::Toric(toric) => toric.pos,
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
            }

            Self::Toric(_) => unimplemented!(),
        }
    }

    pub(crate) fn semi_diameter(&self) -> Float {
        match self {
            Self::Conic(conic) => conic.semi_diameter,
            Self::Toric(toric) => toric.semi_diameter,
            Self::Stop(stop) => stop.semi_diameter,
            _ => Float::INFINITY,
        }
    }

    pub(crate) fn surface_type(&self) -> SurfaceType {
        match self {
            Self::Conic(conic) => conic.surface_type,
            Self::Toric(toric) => toric.surface_type,
            _ => SurfaceType::NoOp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        core::math::mat3::mat3,
        examples::convexplano_lens::sequential_model,
        specs::gaps::{RealSpec, RefractiveIndexSpec},
    };

    #[test]
    fn gaps_must_specify_ri_when_no_wavelengths_provided() {
        let gaps = vec![
            GapSpec {
                thickness: 1.0,
                refractive_index: RefractiveIndexSpec {
                    real: RealSpec::Constant(1.0),
                    imag: None,
                },
            },
            GapSpec {
                thickness: 1.0,
                refractive_index: RefractiveIndexSpec {
                    real: RealSpec::Formula2 {
                        wavelength_range: [0.3, 0.8],
                        coefficients: vec![1.0, 2.0, 3.0, 4.0],
                    },
                    imag: None,
                },
            },
        ];
        let wavelengths = Vec::new();

        let result = SequentialModel::validate_gaps(&gaps, &wavelengths);
        assert!(result.is_err());
    }

    #[test]
    fn is_rotationally_symmetric() {
        let surfaces = vec![
            Surface::Conic(Conic {
                pos: Vec3::new(0.0, 0.0, 0.0),
                rotation_matrix: mat3!(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
                semi_diameter: 1.0,
                radius_of_curvature: 1.0,
                conic_constant: 0.0,
                surface_type: SurfaceType::Refracting,
            }),
            Surface::Conic(Conic {
                pos: Vec3::new(0.0, 0.0, 0.0),
                rotation_matrix: mat3!(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
                semi_diameter: 1.0,
                radius_of_curvature: 1.0,
                conic_constant: 0.0,
                surface_type: SurfaceType::Refracting,
            }),
        ];
        assert!(SequentialModel::is_rotationally_symmetric(&surfaces));

        let surfaces = vec![
            Surface::Conic(Conic {
                pos: Vec3::new(0.0, 0.0, 0.0),
                rotation_matrix: mat3!(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
                semi_diameter: 1.0,
                radius_of_curvature: 1.0,
                conic_constant: 0.0,
                surface_type: SurfaceType::Refracting,
            }),
            Surface::Toric(Toric {
                pos: Vec3::new(0.0, 0.0, 0.0),
                rotation_matrix: mat3!(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
                semi_diameter: 1.0,
                radius_of_curvature_y: 1.0,
                radius_of_curvature_x: 1.0,
                conic_constant: 0.0,
                surface_type: SurfaceType::Refracting,
            }),
        ];
        assert!(!SequentialModel::is_rotationally_symmetric(&surfaces));
    }

    #[test]
    fn test_calc_model_ids() {
        let sequential_model = sequential_model();
        let surfaces = sequential_model.surfaces();
        let wavelengths = vec![0.4, 0.6];

        let model_ids = SequentialModel::calc_model_ids(surfaces, &wavelengths);

        assert_eq!(model_ids.len(), 2); // Two wavelengths, rotationally
                                        // symmetric
    }

    #[test]
    fn test_calc_model_ids_no_wavelength() {
        let sequential_model = sequential_model();
        let surfaces = sequential_model.surfaces();
        let wavelengths = Vec::new();

        let model_ids = SequentialModel::calc_model_ids(surfaces, &wavelengths);

        assert_eq!(model_ids.len(), 1); // Circularly symmetric, no wavelengths
    }
}
