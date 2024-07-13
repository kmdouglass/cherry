use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::core::{sequential_model::SequentialModel, Float};
use crate::specs::{
    aperture::ApertureSpec, fields::FieldSpec, gaps::GapSpec, surfaces::SurfaceSpec,
};
use crate::views::View;

pub struct System {
    sequential_model: SequentialModel,
    views: HashMap<String, View>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemDescription<T: Serialize> {
    pub views: HashMap<String, T>,
}

impl System {
    pub fn new(
        aperture_spec: ApertureSpec,
        field_specs: Vec<FieldSpec>,
        gap_specs: Vec<GapSpec>,
        surface_specs: Vec<SurfaceSpec>,
        wavelengths: Vec<Float>,
        views: Vec<View>,
    ) -> Result<Self> {
        let sequential_model = SequentialModel::new(
            aperture_spec,
            field_specs,
            gap_specs,
            surface_specs,
            wavelengths,
        )?;

        let views: HashMap<String, View> = views
            .into_iter()
            .map(|view| (view.name().to_string(), view))
            .collect();

        Ok(Self {
            sequential_model,
            views: views,
        })
    }

    pub fn describe(&self) -> SystemDescription<impl Serialize> {
        let views = self
            .views
            .iter()
            .map(|(name, view)| (name.clone(), view.describe()))
            .collect();

        SystemDescription { views }
    }

    // This can be removed once the paraxial view tests are moved to the system.
    pub fn sequential_model(&self) -> &SequentialModel {
        &self.sequential_model
    }
}
