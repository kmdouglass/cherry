use std::collections::HashMap;

use anyhow::{anyhow, Result};

use crate::core::{
    sequential_model::{Axis, Gap, Surface},
    sequential_model::{SequentialSubModel, SequentialSubModelIter},
    Cursor, Float,
};
use crate::specs::{
    aperture::ApertureSpec,
    fields::FieldSpec,
    gaps::GapSpec,
    surfaces::{SurfaceSpec, SurfaceType},
};

/// A unique identifier for a model.
///
/// The first element is the index of the wavelength in the system's list of
/// wavelengths. The second element is the transverse axis along which the model
/// is computed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubModelID(pub Option<usize>, pub Axis);

/// An optical system for sequential ray tracing.
#[derive(Debug)]
pub struct SequentialModel {
    aperture: ApertureSpec,
    fields: Vec<FieldSpec>,
    surfaces: Vec<Surface>,
    wavelengths: Vec<Float>,

    submodels: HashMap<SubModelID, SequentialSubModel>,
}

impl SequentialModel {
    /// Creates a new sequential optical system.
    pub fn new(
        aperture: ApertureSpec,
        fields: Vec<FieldSpec>,
        gap_specs: Vec<GapSpec>,
        surface_specs: Vec<SurfaceSpec>,
        wavelengths: Vec<Float>,
    ) -> Result<Self> {
        Self::validate_specs(&aperture, &fields, &gap_specs, &surface_specs, &wavelengths)?;

        let surfaces = Self::surf_specs_to_surfs(&surface_specs, &gap_specs);

        let model_ids: Vec<SubModelID> = Self::calc_model_ids(&wavelengths);
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
            aperture,
            fields,
            surfaces,
            wavelengths,
            submodels: models,
        })
    }

    pub fn surfaces(&self) -> &[Surface] {
        &self.surfaces
    }

    pub fn submodels(&self) -> &HashMap<SubModelID, SequentialSubModel> {
        &self.submodels
    }

    /// Computes the unique IDs for each paraxial model.
    fn calc_model_ids(wavelengths: &Vec<Float>) -> Vec<SubModelID> {
        let mut ids = Vec::new();
        if wavelengths.is_empty() {
            ids.push(SubModelID(None, Axis::X));
            ids.push(SubModelID(None, Axis::Y));
            return ids;
        }

        for (idx, _wavelength) in wavelengths.iter().enumerate() {
            for axis in [Axis::X, Axis::Y].iter() {
                let id = SubModelID(Some(idx), *axis);
                ids.push(id);
            }
        }
        ids
    }

    fn gap_specs_to_gaps(gap_specs: &Vec<GapSpec>, wavelength: Option<Float>) -> Result<Vec<Gap>> {
        let mut gaps = Vec::new();
        for gap_spec in gap_specs.iter() {
            let gap = Gap::try_from_spec(gap_spec, wavelength)?;
            gaps.push(gap);
        }
        Ok(gaps)
    }

    fn surf_specs_to_surfs(
        surf_specs: &Vec<SurfaceSpec>,
        gap_specs: &Vec<GapSpec>,
    ) -> Vec<Surface> {
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

        let result = SequentialModel::validate_gaps(&gaps, &wavelengths);
        assert!(result.is_err());
    }
}
