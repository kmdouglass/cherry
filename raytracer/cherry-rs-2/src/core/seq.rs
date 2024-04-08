/// Data types for modeling sequential ray tracing systems.
use anyhow::Result;

use crate::core::{Float, RefractiveIndex};
use crate::specs::{gaps::GapSpec, surfaces::SurfaceType};

pub(crate) struct Gap {
    thickness: Float,
    refractive_index: RefractiveIndex,
}

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
    semi_diameter: Float,
    radius_of_curvature: Float,
    conic_constant: Float,
    surf_type: SurfaceType,
}

#[derive(Debug)]
pub(crate) struct Image {}

#[derive(Debug)]
pub(crate) struct Object {}

/// A surface without any effect on rays that is used to measure intersections.
#[derive(Debug)]
pub(crate) struct Probe {}

#[derive(Debug)]
pub(crate) struct Stop {
    semi_diameter: Float,
}

#[derive(Debug)]
pub(crate) struct Toric {
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

impl Surface {
    /// The radius of curvature in the horizontal direction.
    pub(crate) fn roch(&self) -> Float {
        match self {
            Self::Conic(conic) => conic.radius_of_curvature,
            Self::Toric(toric) => toric.radius_of_curvature_horz,
            _ => Float::INFINITY,
        }
    }

    /// The radius of curvature in the vertical direction.
    pub(crate) fn rocv(&self) -> Float {
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
