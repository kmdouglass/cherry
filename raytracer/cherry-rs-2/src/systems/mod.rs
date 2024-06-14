use std::collections::HashMap;

use anyhow::{anyhow, Result};

use crate::core::{models::sequential_model::SequentialModel, seq::Gap, Cursor, Float};
use crate::specs::{gaps, surfaces};
use crate::specs::{
    aperture::ApertureSpec, fields::FieldSpec, gaps::GapSpec, surfaces::SurfaceSpec,
};

/// A unique identifier for a model.
///
/// The first element is the index of the wavelength in the system's list of
/// wavelengths. The second element is the transverse axis along which the model
/// is computed.
type ModelID = (Option<usize>, Axis);

/// The transverse direction along which system properties will be computed.
#[derive(Debug, Clone, Copy)]
enum Axis {
    Horizontal,
    Vertical,
}

/// An optical system for sequential ray tracing.
///
/// The surfaces are wrapped in a Rc<RefCell<...>> to allow for sharing mutable
/// references to the surfaces across multiple models and any optimizers.
#[derive(Debug)]
pub struct SeqSys {
    aperture: ApertureSpec,
    fields: Vec<FieldSpec>,
    gaps: Vec<GapSpec>,
    surface_specs: Vec<SurfaceSpec>,
    wavelengths: Vec<Float>,

    cursor: Cursor,
    model_ids: Vec<ModelID>,
}

impl SeqSys {
    /// Creates a new sequential optical system.
    pub fn new(
        aperture: ApertureSpec,
        fields: Vec<FieldSpec>,
        gaps: Vec<GapSpec>,
        surface_specs: Vec<SurfaceSpec>,
        wavelengths: Vec<Float>,
    ) -> Result<Self> {
        Self::validate_specs(&aperture, &fields, &gaps, &surface_specs, &wavelengths)?;

        let cursor = Cursor::new();

        let model_ids = Self::model_ids(&wavelengths);
        let models: HashMap<ModelID, SequentialModel>;
        for model_id in model_ids.iter() {
            let wavelength = match model_id.0 {
                Some(idx) => Some(wavelengths[idx]),
                None => None,
            };
            let gaps = Self::gap_specs_to_gaps(&gaps, wavelength)?;
            
            !unimplemented!("TODO Create the surfaces from the specs")
        }

        Ok(Self {
            aperture,
            fields,
            gaps,
            surface_specs: surface_specs,
            wavelengths,
            cursor,
            model_ids,
        })
    }

    fn gap_specs_to_gaps(gap_specs: &Vec<GapSpec>, wavelength: Option<Float>) -> Result<Vec<Gap>> {
        let mut gaps = Vec::new();
        for gap_spec in gap_specs.iter() {
            let gap = Gap::try_from_spec(gap_spec, wavelength)?;
            gaps.push(gap);
        }
        Ok(gaps)
    }

    /// Computes the unique IDs for each paraxial model.
    fn model_ids(wavelengths: &Vec<Float>) -> Vec<ModelID> {
        let mut ids = Vec::new();
        if wavelengths.is_empty() {
            ids.push((None, Axis::Horizontal));
            ids.push((None, Axis::Vertical));
            return ids;
        }

        for (idx, _wavelength) in wavelengths.iter().enumerate() {
            for axis in [Axis::Horizontal, Axis::Vertical].iter() {
                let id = (Some(idx), *axis);
                ids.push(id);
            }
        }
        ids
    }

    fn validate_gaps(gaps: &Vec<GapSpec>, wavelengths: &Vec<Float>) -> Result<()> {
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
    use crate::specs::gaps::{RealSpec, RefractiveIndexSpec};

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

        let result = SeqSys::validate_gaps(&gaps, &wavelengths);
        assert!(result.is_err());
    }
}
