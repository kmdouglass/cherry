use anyhow::{anyhow, Result};

use crate::core::{seq::Gap, seq::Surface, Float};
use crate::specs::gaps::RefractiveIndexSpec;
use crate::specs::{
    aperture::ApertureSpec, fields::FieldSpec, gaps::GapSpec, gaps::RIDataSpec,
    surfaces::SurfaceSpec,
};

/// A unique identifier for a paraxial model.
///
/// The first element is the index of the wavelength in the system's list of wavelengths.
/// The second element is the axis along which the paraxial model is computed.
type ParaxialModelID = (usize, Axis);

/// The transverse direction along which system properties will be computed.
#[derive(Debug, Clone, Copy)]
enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Debug)]
pub struct SeqSys {
    aperture: ApertureSpec,
    fields: Vec<FieldSpec>,
    gaps: Vec<GapSpec>,
    surfaces: Vec<SurfaceSpec>,
    wavelengths: Vec<Float>,
}

/// Builds a sequential optical system from user specs.
///
/// The builder validates the specs before creating the system. If you want to ensure that the system is valid,
/// use the builder instead of creating the system directly.
#[derive(Debug)]
pub struct SeqSysBuilder {
    aperture: Option<ApertureSpec>,
    fields: Option<Vec<FieldSpec>>,
    gaps: Option<Vec<GapSpec>>,
    surfaces: Option<Vec<SurfaceSpec>>,
    wavelengths: Option<Vec<Float>>,
}

impl SeqSys {
    /// Creates a new sequential optical system.
    ///
    /// There are no validations performed when creating the system directly with this method. If you want to ensure that the system is valid,
    /// then use the `SeqSysBuilder` instead.
    fn new(
        aperture: ApertureSpec,
        fields: Vec<FieldSpec>,
        gaps: Vec<GapSpec>,
        surfaces: Vec<SurfaceSpec>,
        wavelengths: Vec<Float>,
    ) -> Self {
        Self {
            aperture,
            fields,
            gaps,
            surfaces,
            wavelengths,
        }
    }

    /// Composes the optical system from its specs.
    fn compose(&self) {
        !unimplemented!("Compose the optical system from its specs.")
    }

    /// Computes the unique IDs for each paraxial model.
    fn paraxial_model_ids(&self) -> Vec<ParaxialModelID> {
        let mut ids = Vec::new();
        for (idx, _wavelength) in self.wavelengths.iter().enumerate() {
            for axis in [Axis::Horizontal, Axis::Vertical].iter() {
                let id = (idx, *axis);
                ids.push(id);
            }
        }
        ids
    }
}

impl SeqSysBuilder {
    /// Creates a new sequential optical system builder.
    pub fn new() -> Self {
        Self {
            aperture: None,
            fields: None,
            gaps: None,
            surfaces: None,
            wavelengths: None,
        }
    }

    /// Sets the aperture of the optical system.
    pub fn aperture(mut self, aperture: ApertureSpec) -> Self {
        self.aperture = Some(aperture);
        self
    }

    /// Sets the fields of the optical system.
    pub fn fields(mut self, fields: Vec<FieldSpec>) -> Self {
        self.fields = Some(fields);
        self
    }

    /// Sets the gaps of the optical system.
    pub fn gaps(mut self, gaps: Vec<GapSpec>) -> Self {
        self.gaps = Some(gaps);
        self
    }

    /// Sets the surfaces of the optical system.
    pub fn surfaces(mut self, surfaces: Vec<SurfaceSpec>) -> Self {
        self.surfaces = Some(surfaces);
        self
    }

    pub fn wavelengths(mut self, wavelengths: Vec<Float>) -> Self {
        self.wavelengths = Some(wavelengths);
        self
    }

    /// Builds the sequential optical system.
    pub fn build(self) -> Result<SeqSys> {
        let aperture = self
            .aperture
            .ok_or(anyhow!("The system's aperture must be specified."))?;
        let fields = self
            .fields
            .ok_or(anyhow!("The system's fields must be specified."))?;
        let gaps = self
            .gaps
            .ok_or(anyhow!("The system's gaps must be specified."))?;
        let surfaces = self
            .surfaces
            .ok_or(anyhow!("The system's surfaces must be specified."))?;
        let wavelengths = self
            .wavelengths
            .ok_or(anyhow!("The system's wavelengths must be specified."))?;

        Self::validate_specs(&aperture, &fields, &gaps, &surfaces, &wavelengths)?;

        Ok(SeqSys {
            aperture,
            fields,
            gaps,
            surfaces,
            wavelengths,
        })
    }

    fn validate_gaps(gaps: &Vec<GapSpec>, wavelengths: &Vec<Float>) -> Result<()> {
        if gaps.is_empty() {
            return Err(anyhow!("The system must have at least one gap."));
        }

        // If no wavelengths are specified, then the gaps must explicitly specify the refractive index.
        if wavelengths.is_empty() {
            let error = Err(anyhow!(
                "The system's gaps must specify the refractive index if no wavelengths are provided."
            ));

            for gap in gaps.iter() {
                match &gap.refractive_index {
                    RefractiveIndexSpec::N { n } => {
                        if let RIDataSpec::Constant(_) = n {
                            continue;
                        } else {
                            return error;
                        }
                    }
                    _ => {
                        return error;
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_specs(
        aperture: &ApertureSpec,
        fields: &Vec<FieldSpec>,
        gaps: &Vec<GapSpec>,
        surfaces: &Vec<SurfaceSpec>,
        wavelengths: &Vec<Float>,
    ) -> Result<()> {
        Self::validate_gaps(gaps, wavelengths)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::specs::gaps::{RIDataSpec, RefractiveIndexSpec};

    #[test]
    fn gaps_must_specify_ri_when_no_wavelengths_provided() {
        let gaps = vec![
            GapSpec {
                thickness: 1.0,
                refractive_index: RefractiveIndexSpec::N {
                    n: RIDataSpec::Constant(1.0),
                },
            },
            GapSpec {
                thickness: 1.0,
                refractive_index: RefractiveIndexSpec::N {
                    n: RIDataSpec::Formula2 {
                        wavelength_range: [0.3, 0.8],
                        coefficients: vec![1.0, 2.0, 3.0, 4.0],
                    },
                },
            },
        ];
        let wavelengths = Vec::new();

        let result = SeqSysBuilder::validate_gaps(&gaps, &wavelengths);
        assert!(result.is_err());
    }
}
