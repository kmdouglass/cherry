use anyhow::{anyhow, Result};

use crate::core::Float;
use crate::specs::aperture;
use crate::specs::{aperture::ApertureSpec, fields::FieldSpec, gaps::GapSpec};

type ParaxialModelID = (usize, Axis);

/// The transverse direction along which system properties will be computed.
#[derive(Debug, Clone, Copy)]
enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Debug)]
pub struct SeqSystem {
    aperture: ApertureSpec,
    fields: Vec<FieldSpec>,
    gaps: Vec<GapSpec>,
    wavelengths: Vec<Float>,
}

/// Builds a sequential optical system from user specs.
#[derive(Debug)]
pub struct SeqSysBuilder {
    aperture: Option<ApertureSpec>,
    fields: Option<Vec<FieldSpec>>,
    gaps: Option<Vec<GapSpec>>,
    wavelengths: Option<Vec<Float>>,
}

impl SeqSystem {
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

    pub fn wavelengths(mut self, wavelengths: Vec<Float>) -> Self {
        self.wavelengths = Some(wavelengths);
        self
    }

    /// Builds the sequential optical system.
    pub fn build(self) -> Result<SeqSystem> {
        let aperture = self
            .aperture
            .ok_or(anyhow!("The system's aperture must be specified."))?;
        let fields = self
            .fields
            .ok_or(anyhow!("The system's fields must be specified."))?;
        let gaps = self
            .gaps
            .ok_or(anyhow!("The system's gaps must be specified."))?;
        let wavelengths = self
            .wavelengths
            .ok_or(anyhow!("The system's wavelengths must be specified."))?;

        Ok(SeqSystem {
            aperture,
            fields,
            gaps,
            wavelengths,
        })
    }
}
