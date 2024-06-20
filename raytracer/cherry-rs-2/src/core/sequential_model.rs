/// Data types for modeling sequential ray tracing systems.
use anyhow::Result;

use crate::core::{math::vec3::Vec3, Float, RefractiveIndex};
use crate::specs::{
    gaps::GapSpec,
    surfaces::{SurfaceSpec, SurfaceType},
};

#[derive(Debug)]
pub(crate) struct Gap {
    thickness: Float,
    refractive_index: RefractiveIndex,
}

#[derive(Debug)]
pub(crate) struct SequentialSubModel {
    gaps: Vec<Gap>,
}

pub(crate) struct SequentialModelIter<'a> {
    surfaces: &'a Vec<Surface>,
    gaps: &'a Vec<Gap>,
    index: usize,
}

/// A single ray tracing step in a sequential system.
pub(crate) type Step<'a> = (&'a Gap, &'a Surface, Option<&'a Gap>);

#[derive(Debug)]
pub(crate) enum Surface {
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
    euler_angles: Vec3,
    semi_diameter: Float,
    radius_of_curvature: Float,
    conic_constant: Float,
    surf_type: SurfaceType,
}

#[derive(Debug)]
pub(crate) struct Image {
    pos: Vec3,
    euler_angles: Vec3,
}

#[derive(Debug)]
pub(crate) struct Object {
    pos: Vec3,
    euler_angles: Vec3,
}

/// A surface without any effect on rays that is used to measure intersections.
#[derive(Debug)]
pub(crate) struct Probe {
    pos: Vec3,
    euler_angles: Vec3,
}

#[derive(Debug)]
pub(crate) struct Stop {
    pos: Vec3,
    euler_angles: Vec3,
    semi_diameter: Float,
}

#[derive(Debug)]
pub(crate) struct Toric {
    pos: Vec3,
    euler_angles: Vec3,
    semi_diameter: Float,
    radius_of_curvature_vert: Float,
    radius_of_curvature_horz: Float,
    conic_constant: Float,
    surf_type: SurfaceType,
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

impl SequentialSubModel {
    pub(crate) fn new(gaps: Vec<Gap>) -> Self {
        Self { gaps }
    }

    pub(crate) fn iter<'a>(&'a self, surfaces: &'a Vec<Surface>) -> SequentialModelIter<'a> {
        SequentialModelIter::new(surfaces, &self.gaps)
    }
}

impl<'a> SequentialModelIter<'a> {
    fn new(surfaces: &'a Vec<Surface>, gaps: &'a Vec<Gap>) -> Self {
        Self {
            surfaces,
            gaps,
            index: 0,
        }
    }
}

impl<'a> Iterator for SequentialModelIter<'a> {
    type Item = Step<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.gaps.len() - 1 {
            // We are at the last gap
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

impl Surface {
    pub(crate) fn from_spec(spec: &SurfaceSpec, pos: Vec3) -> Self {
        // No rotation for the moment
        let euler_angles = Vec3::new(0.0, 0.0, 0.0);

        match spec {
            SurfaceSpec::Conic {
                semi_diameter,
                radius_of_curvature,
                conic_constant,
                surf_type,
            } => Self::Conic(Conic {
                pos,
                euler_angles,
                semi_diameter: *semi_diameter,
                radius_of_curvature: *radius_of_curvature,
                conic_constant: *conic_constant,
                surf_type: *surf_type,
            }),
            SurfaceSpec::Image => Self::Image(Image { pos, euler_angles }),
            SurfaceSpec::Object => Self::Object(Object { pos, euler_angles }),
            SurfaceSpec::Probe => Self::Probe(Probe { pos, euler_angles }),
            SurfaceSpec::Stop { semi_diameter } => Self::Stop(Stop {
                pos,
                euler_angles,
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
                euler_angles,
                semi_diameter: *semi_diameter,
                radius_of_curvature_vert: *radius_of_curvature_vert,
                radius_of_curvature_horz: *radius_of_curvature_horz,
                conic_constant: *conic_constant,
                surf_type: *surf_type,
            }),
        }
    }

    /// The radius of curvature in the horizontal direction.
    pub(crate) fn rocx(&self) -> Float {
        match self {
            Self::Conic(conic) => conic.radius_of_curvature,
            Self::Toric(toric) => toric.radius_of_curvature_horz,
            _ => Float::INFINITY,
        }
    }

    /// The radius of curvature in the vertical direction.
    pub(crate) fn rocy(&self) -> Float {
        match self {
            Self::Conic(conic) => conic.radius_of_curvature,
            Self::Toric(toric) => toric.radius_of_curvature_vert,
            _ => Float::INFINITY,
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
            Self::Conic(conic) => conic.surf_type,
            Self::Toric(toric) => toric.surf_type,
            _ => SurfaceType::NoOp,
        }
    }
}
