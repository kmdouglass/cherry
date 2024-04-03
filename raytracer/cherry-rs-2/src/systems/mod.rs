use anyhow::{anyhow, Result};

use crate::core::Float;
use crate::specs::{fields::FieldSpec, gaps::GapSpec};

#[derive(Debug)]
pub struct SeqSystem {
    fields: Vec<FieldSpec>,
    gaps: Vec<GapSpec>,
    wavelengths: Vec<Float>,
}

/// Builds a sequential optical system from user specs.
#[derive(Debug)]
pub struct SeqSysBuilder {
    fields: Option<Vec<FieldSpec>>,
    gaps: Option<Vec<GapSpec>>,
    wavelengths: Option<Vec<Float>>,
}

impl SeqSysBuilder {
    /// Creates a new sequential optical system builder.
    pub fn new() -> Self {
        Self {
            fields: None,
            gaps: None,
            wavelengths: None,
        }
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
        let fields = self
            .fields
            .ok_or(anyhow!("The system's fields must be specified."))?;
        let gaps = self
            .gaps
            .ok_or(anyhow!("The system's gaps must be specified."))?;
        let wavelengths = self
            .wavelengths
            .ok_or(anyhow!("The system's wavelenghts must be specified."))?;

        Ok(SeqSystem {
            fields,
            gaps,
            wavelengths,
        })
    }
}
