use anyhow::Result;

use crate::core::{sequential_model::SequentialModel, Float};
use crate::specs::{
    aperture::ApertureSpec, fields::FieldSpec, gaps::GapSpec, surfaces::SurfaceSpec,
};

pub struct System {
    sequential_model: SequentialModel,
}

impl System {
    pub fn new(
        aperture_spec: ApertureSpec,
        field_specs: Vec<FieldSpec>,
        gap_specs: Vec<GapSpec>,
        surface_specs: Vec<SurfaceSpec>,
        wavelengths: Vec<Float>,
    ) -> Result<Self> {
        let sequential_model = SequentialModel::new(
            aperture_spec,
            field_specs,
            gap_specs,
            surface_specs,
            wavelengths,
        )?;

        Ok(Self { sequential_model })
    }

    pub fn sequential_model(&self) -> &SequentialModel {
        &self.sequential_model
    }
}
